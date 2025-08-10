#[cfg(all(target_os = "macos", feature = "debug"))]
use std::env::home_dir;
#[cfg(all(target_os = "macos", feature = "debug"))]
use std::process::Command;

fn main() -> std::io::Result<()> {
    println!("cargo::rerun-if-changed=build.rs");
    #[cfg(all(target_os = "macos", feature = "debug"))]
    {
        install_bevy_cef_debug_render_process()?;
        install_export_cef_dir()?;
        export_cef_dir()?;
    }
    Ok(())
}

#[cfg(all(target_os = "macos", feature = "debug"))]
fn install_bevy_cef_debug_render_process() -> std::io::Result<()> {
    let bevy_cef_render_process_path = home_dir()
        .unwrap()
        .join(".cargo")
        .join("bin")
        .join("bevy_cef_debug_render_process");
    if !bevy_cef_render_process_path.exists() {
        Command::new("cargo")
            .args(["install", "bevy_cef_debug_render_process"])
            .spawn()?;
    }
    Ok(())
}

#[cfg(all(target_os = "macos", feature = "debug"))]
fn install_export_cef_dir() -> std::io::Result<()> {
    let export_cef_dir_path = home_dir()
        .unwrap()
        .join(".cargo")
        .join("bin")
        .join("export-cef-dir");
    if !export_cef_dir_path.exists() {
        Command::new("cargo")
            .args(["install", "export-cef-dir"])
            .spawn()?;
    }
    Ok(())
}

#[cfg(all(target_os = "macos", feature = "debug"))]
fn export_cef_dir() -> std::io::Result<()> {
    let cef_dir = home_dir().unwrap().join(".local").join("share").join("cef");
    if cef_dir.exists() {
        return Ok(());
    }
    let export_cef_dir_path = home_dir()
        .unwrap()
        .join(".cargo")
        .join("bin")
        .join("export-cef-dir");
    Command::new(export_cef_dir_path)
        .arg("--force")
        .arg(cef_dir)
        .spawn()?
        .wait()?;
    Ok(())
}
