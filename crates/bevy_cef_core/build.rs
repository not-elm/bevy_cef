fn main() {
    #[cfg(target_os = "windows")]
    windows::copy_cef_files();
}

#[cfg(target_os = "windows")]
mod windows {
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::io::ErrorKind;
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

        println!("cargo:rerun-if-changed={}", cef_dir.display());

        println!(
            "cargo:warning=Copying CEF files from {:?} to {:?}",
            cef_dir, target_dir
        );
        copy_cef_runtime_files(&cef_dir, &target_dir);

        println!(
            "cargo:warning=Hard-linking CEF files into {:?}",
            examples_dir
        );
        fs::create_dir_all(&examples_dir).unwrap();
        link_cef_runtime_files(&target_dir, &examples_dir);
    }

    fn find_cef_dir() -> Option<PathBuf> {
        let home = env::var("USERPROFILE").ok()?;
        let cef_dir = PathBuf::from(home).join(".local/share/cef");
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

    fn link_cef_runtime_files(src: &Path, dst: &Path) {
        let entries = fs::read_dir(src).unwrap_or_else(|e| {
            panic!("Failed to read directory {:?}: {}", src, e);
        });

        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = entry.file_name();
            match path.is_dir() {
                true if file_name == "locales" => {
                    link_locales_file(dst, path.as_path(), &file_name);
                }
                false if is_runtime_file(&path) => {
                    link_runtime_file(dst, path.as_path(), &file_name);
                }
                _ => {}
            }
        }
    }

    fn link_locales_file(dst: &Path, path: &Path, file_name: &OsString) {
        let dest_dir = dst.join(file_name);
        fs::create_dir_all(&dest_dir).unwrap();
        link_cef_runtime_files(path, &dest_dir);
    }

    fn link_runtime_file(dst: &Path, path: &Path, file_name: &OsString) {
        let dest = dst.join(file_name);
        match fs::hard_link(path, &dest) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                //Skip if already exists runtime files.
            }
            Err(_) => {
                fs::copy(path, &dest).unwrap_or_else(|e| {
                    panic!("Failed to copy {:?} to {:?}: {}", path, dest, e);
                });
            }
        }
    }
}
