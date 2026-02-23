//! CLI tool to bundle CEF framework into macOS `.app` bundles.
//!
//! Embeds the Chromium Embedded Framework, creates helper app bundles,
//! merges CEF-required keys into the main Info.plist, and codesigns
//! everything in the correct inside-out order.

use std::collections::HashSet;
#[allow(deprecated)]
use std::env::home_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};
use clap::Parser;
use plist::Value;

/// Helper app name suffixes (base, GPU, Renderer, Plugin).
const HELPER_SUFFIXES: &[&str] = &["", " (GPU)", " (Renderer)", " (Plugin)"];

/// Bundle CEF framework into an existing macOS `.app` bundle.
#[derive(Parser, Debug)]
#[command(name = "bevy_cef_bundle_app", version, about)]
struct Args {
    /// Path to the existing `.app` bundle.
    #[arg(long)]
    app: PathBuf,

    /// Base bundle identifier for helper apps (e.g. `com.example.myapp`).
    #[arg(long)]
    bundle_id_base: String,

    /// Executable name used as the base for helper app names.
    /// If omitted, read from the app's `Info.plist` `CFBundleExecutable`.
    #[arg(long)]
    bin_name: Option<String>,

    /// Path to the CEF framework.
    #[arg(long, default_value_os_t = default_cef_framework_path())]
    cef_framework: PathBuf,

    /// Path to the CEF render-process helper binary.
    #[arg(long, default_value_os_t = default_helper_bin_path())]
    helper_bin: PathBuf,

    /// Codesign identity. Use `-` for ad-hoc signing.
    #[arg(long, default_value = "-")]
    sign_identity: String,

    /// Skip codesigning entirely.
    #[arg(long, default_value_t = false)]
    no_sign: bool,
}

#[allow(deprecated)]
fn default_cef_framework_path() -> PathBuf {
    home_dir()
        .unwrap_or_default()
        .join(".local/share/Chromium Embedded Framework.framework")
}

#[allow(deprecated)]
fn default_helper_bin_path() -> PathBuf {
    home_dir()
        .unwrap_or_default()
        .join(".cargo/bin/bevy_cef_render_process")
}

fn main() -> Result<()> {
    let args = Args::parse();

    verify_prerequisites(&args)?;

    let bin_name = resolve_bin_name(&args)?;
    println!("Binary name: {bin_name}");

    check_architectures(&args, &bin_name)?;
    merge_main_plist(&args)?;
    clean_old_cef_files(&args, &bin_name)?;
    copy_cef_framework(&args)?;
    create_helper_apps(&args, &bin_name)?;
    strip_xattrs(&args)?;

    if !args.no_sign {
        codesign_bundle(&args, &bin_name)?;
    } else {
        println!("Skipping codesign (--no-sign).");
    }

    println!("CEF bundling complete: {}", args.app.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Prerequisites
// ---------------------------------------------------------------------------

/// Verify that the `.app` bundle, CEF framework, and helper binary exist.
fn verify_prerequisites(args: &Args) -> Result<()> {
    ensure!(
        args.app.is_dir(),
        "App bundle not found: {}",
        args.app.display()
    );
    ensure!(
        args.app.join("Contents/Info.plist").is_file(),
        "Info.plist not found in {}",
        args.app.display()
    );
    ensure!(
        args.cef_framework.is_dir(),
        "CEF framework not found: {}",
        args.cef_framework.display()
    );
    ensure!(
        args.helper_bin.is_file(),
        "Helper binary not found: {}",
        args.helper_bin.display()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Architecture check
// ---------------------------------------------------------------------------

/// Run `lipo -archs` and return the set of architectures.
fn lipo_archs(path: &Path) -> Result<HashSet<String>> {
    let output = Command::new("lipo")
        .args(["-archs"])
        .arg(path)
        .output()
        .with_context(|| format!("failed to run lipo -archs on {}", path.display()))?;

    ensure!(
        output.status.success(),
        "lipo -archs failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(String::from)
        .collect())
}

/// Verify that the main binary, helper binary, and CEF framework share at
/// least one common architecture.
fn check_architectures(args: &Args, bin_name: &str) -> Result<()> {
    let main_bin = args.app.join(format!("Contents/MacOS/{bin_name}"));
    let cef_bin = args.cef_framework.join("Chromium Embedded Framework");

    let main_archs =
        lipo_archs(&main_bin).with_context(|| format!("main binary: {}", main_bin.display()))?;
    let helper_archs = lipo_archs(&args.helper_bin)
        .with_context(|| format!("helper binary: {}", args.helper_bin.display()))?;
    let cef_archs =
        lipo_archs(&cef_bin).with_context(|| format!("CEF framework: {}", cef_bin.display()))?;

    let common: HashSet<_> = main_archs
        .intersection(&helper_archs)
        .cloned()
        .collect::<HashSet<_>>()
        .intersection(&cef_archs)
        .cloned()
        .collect();

    ensure!(
        !common.is_empty(),
        "Architecture mismatch — no common architecture found.\n  \
         Main binary ({main_bin}): {main_archs:?}\n  \
         Helper binary ({}): {helper_archs:?}\n  \
         CEF framework ({}): {cef_archs:?}",
        args.helper_bin.display(),
        cef_bin.display(),
        main_bin = main_bin.display(),
    );

    println!(
        "Architecture check passed (common: {})",
        common.into_iter().collect::<Vec<_>>().join(", ")
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// bin-name resolution
// ---------------------------------------------------------------------------

/// Resolve the binary name: use `--bin-name` if provided, otherwise read
/// `CFBundleExecutable` from the app's `Info.plist`.
fn resolve_bin_name(args: &Args) -> Result<String> {
    if let Some(name) = &args.bin_name {
        return Ok(name.clone());
    }

    let plist_path = args.app.join("Contents/Info.plist");
    let dict = read_plist_dict(&plist_path)?;

    dict.get("CFBundleExecutable")
        .and_then(|v| v.as_string())
        .map(String::from)
        .context("CFBundleExecutable not found in Info.plist; use --bin-name")
}

// ---------------------------------------------------------------------------
// Info.plist merge
// ---------------------------------------------------------------------------

/// Read a plist file as a `plist::Dictionary`.
fn read_plist_dict(path: &Path) -> Result<plist::Dictionary> {
    let value = Value::from_file(path)
        .with_context(|| format!("failed to read plist: {}", path.display()))?;
    value
        .into_dictionary()
        .context("Info.plist root is not a dictionary")
}

/// Merge CEF-required keys into the main app's `Info.plist`.
///
/// Merge rules:
/// - `LSEnvironment.MallocNanoZone`: always set to `"0"`.
/// - `LSMinimumSystemVersion`: set to `"11.0"` if absent or less than `11.0`.
/// - `NSSupportsAutomaticGraphicsSwitching`: add `true` if absent, otherwise
///   keep the existing value.
fn merge_main_plist(args: &Args) -> Result<()> {
    let plist_path = args.app.join("Contents/Info.plist");
    let mut dict = read_plist_dict(&plist_path)?;

    // LSEnvironment.MallocNanoZone = "0"
    merge_ls_environment(&mut dict)?;

    // LSMinimumSystemVersion >= "11.0"
    merge_minimum_system_version(&mut dict);

    // NSSupportsAutomaticGraphicsSwitching (add if absent)
    if !dict.contains_key("NSSupportsAutomaticGraphicsSwitching") {
        dict.insert(
            "NSSupportsAutomaticGraphicsSwitching".into(),
            Value::Boolean(true),
        );
    }

    Value::Dictionary(dict)
        .to_file_xml(&plist_path)
        .with_context(|| format!("failed to write plist: {}", plist_path.display()))?;

    println!("Merged CEF keys into {}", plist_path.display());
    Ok(())
}

/// Ensure `LSEnvironment` dict exists and `MallocNanoZone` is `"0"`.
fn merge_ls_environment(dict: &mut plist::Dictionary) -> Result<()> {
    match dict.get("LSEnvironment") {
        None => {
            let mut env = plist::Dictionary::new();
            env.insert("MallocNanoZone".into(), Value::String("0".into()));
            dict.insert("LSEnvironment".into(), Value::Dictionary(env));
        }
        Some(Value::Dictionary(_)) => {
            // Existing dict — update MallocNanoZone in place.
            let env = dict
                .get_mut("LSEnvironment")
                .unwrap()
                .as_dictionary_mut()
                .unwrap();
            env.insert("MallocNanoZone".into(), Value::String("0".into()));
        }
        Some(_) => {
            bail!("LSEnvironment exists but is not a dictionary");
        }
    }
    Ok(())
}

/// Set `LSMinimumSystemVersion` to at least `"11.0"`.
fn merge_minimum_system_version(dict: &mut plist::Dictionary) {
    let key = "LSMinimumSystemVersion";
    let needs_update = match dict.get(key).and_then(|v| v.as_string()) {
        None => true,
        Some(existing) => version_less_than(existing, "11.0"),
    };
    if needs_update {
        dict.insert(key.into(), Value::String("11.0".into()));
    }
}

/// Simple major.minor version comparison (e.g. `"10.15"` < `"11.0"`).
fn version_less_than(a: &str, b: &str) -> bool {
    let parse =
        |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse::<u32>().ok()).collect() };
    parse(a) < parse(b)
}

// ---------------------------------------------------------------------------
// Clean old CEF files
// ---------------------------------------------------------------------------

/// Remove previously bundled CEF framework and helper apps to prevent stale
/// files from lingering after a CEF version update.
fn clean_old_cef_files(args: &Args, bin_name: &str) -> Result<()> {
    let frameworks = args.app.join("Contents/Frameworks");

    // Remove old CEF framework
    let old_cef = frameworks.join("Chromium Embedded Framework.framework");
    if old_cef.exists() {
        std::fs::remove_dir_all(&old_cef).with_context(|| {
            format!("failed to remove old CEF framework: {}", old_cef.display())
        })?;
        println!("Removed old CEF framework");
    }

    // Remove old helper apps
    for suffix in HELPER_SUFFIXES {
        let helper_name = format!("{bin_name} Helper{suffix}");
        let helper_app = frameworks.join(format!("{helper_name}.app"));
        if helper_app.exists() {
            std::fs::remove_dir_all(&helper_app).with_context(|| {
                format!("failed to remove old helper: {}", helper_app.display())
            })?;
            println!("Removed old helper: {helper_name}.app");
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// CEF framework copy
// ---------------------------------------------------------------------------

/// Copy the CEF framework into `Contents/Frameworks/`.
fn copy_cef_framework(args: &Args) -> Result<()> {
    let dest = args.app.join("Contents/Frameworks");
    std::fs::create_dir_all(&dest)?;

    run_cmd(
        "cp",
        &[
            "-R",
            &args.cef_framework.to_string_lossy(),
            &dest.to_string_lossy(),
        ],
        "copy CEF framework",
    )?;

    println!("Copied CEF framework to {}", dest.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Helper apps
// ---------------------------------------------------------------------------

/// Create the four helper app bundles.
fn create_helper_apps(args: &Args, bin_name: &str) -> Result<()> {
    let frameworks = args.app.join("Contents/Frameworks");

    for suffix in HELPER_SUFFIXES {
        let helper_name = format!("{bin_name} Helper{suffix}");
        let bundle_id = helper_bundle_id(&args.bundle_id_base, suffix);
        let helper_app = frameworks.join(format!("{helper_name}.app"));
        let helper_macos = helper_app.join("Contents/MacOS");

        std::fs::create_dir_all(&helper_macos)?;

        // Copy helper binary
        std::fs::copy(&args.helper_bin, helper_macos.join(&helper_name))
            .with_context(|| format!("failed to copy helper binary for {helper_name}"))?;

        // Write Info.plist
        let plist = build_helper_plist(&helper_name, &bundle_id);
        Value::Dictionary(plist)
            .to_file_xml(helper_app.join("Contents/Info.plist"))
            .with_context(|| format!("failed to write helper plist for {helper_name}"))?;

        println!("  Created {helper_name}.app (id: {bundle_id})");
    }

    Ok(())
}

/// Derive the bundle identifier for a helper app.
fn helper_bundle_id(base: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        format!("{base}.helper")
    } else {
        let raw = suffix.to_lowercase().replace([' ', '(', ')'], "");
        format!("{base}.helper.{raw}")
    }
}

/// Build an `Info.plist` dictionary for a helper app.
fn build_helper_plist(name: &str, bundle_id: &str) -> plist::Dictionary {
    let mut dict = plist::Dictionary::new();
    dict.insert("CFBundleExecutable".into(), Value::String(name.into()));
    dict.insert("CFBundleName".into(), Value::String(name.into()));
    dict.insert("CFBundleIdentifier".into(), Value::String(bundle_id.into()));
    dict.insert(
        "CFBundleInfoDictionaryVersion".into(),
        Value::String("6.0".into()),
    );
    dict.insert("CFBundlePackageType".into(), Value::String("APPL".into()));

    let mut env = plist::Dictionary::new();
    env.insert("MallocNanoZone".into(), Value::String("0".into()));
    dict.insert("LSEnvironment".into(), Value::Dictionary(env));

    dict.insert("LSUIElement".into(), Value::Boolean(true));

    dict
}

// ---------------------------------------------------------------------------
// xattr
// ---------------------------------------------------------------------------

/// Strip extended attributes from the entire app bundle.
fn strip_xattrs(args: &Args) -> Result<()> {
    run_cmd(
        "xattr",
        &["-cr", &args.app.to_string_lossy()],
        "strip extended attributes",
    )?;
    println!("Stripped extended attributes");
    Ok(())
}

// ---------------------------------------------------------------------------
// Codesign
// ---------------------------------------------------------------------------

/// Codesign the entire bundle in inside-out order, then verify.
fn codesign_bundle(args: &Args, bin_name: &str) -> Result<()> {
    let identity = &args.sign_identity;
    let contents = args.app.join("Contents");
    let frameworks = contents.join("Frameworks");
    let cef_fw = frameworks.join("Chromium Embedded Framework.framework");

    println!("Codesigning CEF components...");

    // 1. Sign dylibs in the framework
    let libs_dir = cef_fw.join("Libraries");
    if libs_dir.is_dir() {
        for entry in std::fs::read_dir(&libs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "dylib") {
                codesign(identity, &path)?;
            }
        }
    }

    // 2. Sign the main CEF framework binary
    codesign(identity, &cef_fw.join("Chromium Embedded Framework"))?;

    // 3. Sign the CEF framework bundle
    codesign(identity, &cef_fw)?;

    // 4. Sign each helper app
    for suffix in HELPER_SUFFIXES {
        let helper_name = format!("{bin_name} Helper{suffix}");
        codesign(identity, &frameworks.join(format!("{helper_name}.app")))?;
    }

    // 5. Sign the main app bundle
    codesign(identity, &args.app)?;

    // Verify
    let output = Command::new("codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(&args.app)
        .output()
        .context("failed to run codesign --verify")?;

    ensure!(
        output.status.success(),
        "codesign verification failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("Codesign verification passed");
    Ok(())
}

/// Run `codesign --force --sign <identity>` on a path.
fn codesign(identity: &str, path: &Path) -> Result<()> {
    run_cmd(
        "codesign",
        &["--force", "--sign", identity, &path.to_string_lossy()],
        &format!("codesign {}", path.display()),
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run a command, checking for success.
fn run_cmd(program: &str, args: &[&str], description: &str) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("failed to run: {description}"))?;

    ensure!(
        status.success(),
        "{description} failed (exit code {status})"
    );
    Ok(())
}
