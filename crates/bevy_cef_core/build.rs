fn main() {
    #[cfg(target_os = "windows")]
    windows::copy_cef_files();

    #[cfg(target_os = "linux")]
    linux::copy_cef_files();
}

#[cfg(target_os = "windows")]
mod windows {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    const RUNTIME_EXTENSIONS: &[&str; 6] = &["dll", "lib", "pak", "dat", "bin", "json"];

    pub fn copy_cef_files() {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-env-changed=USERPROFILE");

        let Some(cef_dir) = find_cef_dir() else {
            return;
        };

        let target_dir = find_target_profile_dir();
        let examples_dir = target_dir.join("examples");

        println!(
            "cargo:warning=Copying CEF files from {:?} to {:?}",
            cef_dir, target_dir
        );
        copy_cef_runtime_files(&cef_dir, &target_dir);

        println!("cargo:warning=Copying CEF files into {:?}", examples_dir);
        fs::create_dir_all(&examples_dir).unwrap();
        copy_cef_runtime_files(&target_dir, &examples_dir);
    }

    fn find_cef_dir() -> Option<PathBuf> {
        let home = env::var("USERPROFILE").ok()?;
        let cef_dir = PathBuf::from(home).join(".local/share/cef");
        println!("cargo:rerun-if-changed={}", cef_dir.display());
        if !cef_dir.exists() {
            println!(
                "cargo:warning=CEF directory not found at {:?}. Run `make setup-windows` first. Skipping CEF file copy.",
                cef_dir
            );
            return None;
        }
        Some(cef_dir)
    }

    fn find_target_profile_dir() -> PathBuf {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let mut dir = out_dir.as_path();
        loop {
            if dir.file_name().map(|n| n == "build").unwrap_or(false) {
                return dir.parent().unwrap().to_path_buf();
            }
            dir = dir
                .parent()
                .expect("Could not find target profile directory from OUT_DIR");
        }
    }

    fn is_runtime_file(path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        RUNTIME_EXTENSIONS.contains(&ext.as_str())
    }

    fn copy_cef_runtime_files(src: &Path, dst: &Path) {
        let entries = fs::read_dir(src).unwrap_or_else(|e| {
            panic!("Failed to read CEF directory {:?}: {}", src, e);
        });

        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = entry.file_name();

            if path.is_dir() {
                if file_name == "locales" {
                    let dest_dir = dst.join(&file_name);
                    fs::create_dir_all(&dest_dir).unwrap();
                    copy_cef_runtime_files(&path, &dest_dir);
                }
                // Skip other directories (include/, cmake/, libcef_dll/)
            } else if is_runtime_file(&path) {
                let dest = dst.join(&file_name);
                if dest.exists() {
                    let src_modified = fs::metadata(&path).unwrap().modified().unwrap();
                    let dst_modified = fs::metadata(&dest).unwrap().modified().unwrap();
                    if dst_modified >= src_modified {
                        continue;
                    }
                }
                fs::copy(&path, &dest).unwrap_or_else(|e| {
                    panic!("Failed to copy {:?} to {:?}: {}", path, dest, e);
                });
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    const RUNTIME_EXTENSIONS: &[&str; 5] = &["so", "pak", "dat", "bin", "json"];

    pub fn copy_cef_files() {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-env-changed=CEF_PATH");

        let Some(cef_dir) = find_cef_dir() else {
            return;
        };

        let target_dir = find_target_profile_dir();
        let examples_dir = target_dir.join("examples");

        println!(
            "cargo:warning=Copying CEF files from {:?} to {:?}",
            cef_dir, target_dir
        );
        copy_cef_runtime_files(&cef_dir, &target_dir);

        println!("cargo:warning=Copying CEF files into {:?}", examples_dir);
        fs::create_dir_all(&examples_dir).unwrap();
        copy_cef_runtime_files(&target_dir, &examples_dir);
    }

    fn find_cef_dir() -> Option<PathBuf> {
        // 1. Check DEP_CEF_PATH (set by cef-dll-sys via cargo:root= metadata)
        // 2. Check CEF_PATH env var
        // 3. Fall back to $HOME/.local/share/cef
        let cef_dir = env::var("DEP_CEF_PATH")
            .ok()
            .map(PathBuf::from)
            .or_else(|| env::var("CEF_PATH").ok().map(PathBuf::from))
            .or_else(|| {
                env::var("HOME")
                    .ok()
                    .map(|home| PathBuf::from(home).join(".local/share/cef"))
            })?;

        println!("cargo:rerun-if-changed={}", cef_dir.display());
        if !cef_dir.exists() {
            println!(
                "cargo:warning=CEF directory not found at {:?}. Run `make setup-linux` first. Skipping CEF file copy.",
                cef_dir
            );
            return None;
        }
        Some(cef_dir)
    }

    fn find_target_profile_dir() -> PathBuf {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let mut dir = out_dir.as_path();
        loop {
            if dir.file_name().map(|n| n == "build").unwrap_or(false) {
                return dir.parent().unwrap().to_path_buf();
            }
            dir = dir
                .parent()
                .expect("Could not find target profile directory from OUT_DIR");
        }
    }

    fn is_runtime_file(path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        RUNTIME_EXTENSIONS.contains(&ext.as_str())
    }

    fn copy_cef_runtime_files(src: &Path, dst: &Path) {
        let entries = fs::read_dir(src).unwrap_or_else(|e| {
            panic!("Failed to read CEF directory {:?}: {}", src, e);
        });

        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = entry.file_name();

            if path.is_dir() {
                if file_name == "locales" {
                    let dest_dir = dst.join(&file_name);
                    fs::create_dir_all(&dest_dir).unwrap();
                    copy_cef_runtime_files(&path, &dest_dir);
                }
            } else if is_runtime_file(&path) {
                let dest = dst.join(&file_name);
                if dest.exists() {
                    let src_modified = fs::metadata(&path).unwrap().modified().unwrap();
                    let dst_modified = fs::metadata(&dest).unwrap().modified().unwrap();
                    if dst_modified >= src_modified {
                        continue;
                    }
                }
                fs::copy(&path, &dest).unwrap_or_else(|e| {
                    panic!("Failed to copy {:?} to {:?}: {}", path, dest, e);
                });
            }
        }
    }
}
