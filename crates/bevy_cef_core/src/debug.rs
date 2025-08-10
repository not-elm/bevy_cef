use cef::{load_library, unload_library};
use std::env::home_dir;

/// This loader is a modified version of [LibraryLoader](cef::library_loader::LibraryLoader) that can load the framework located in the home directory.
pub struct DebugLibraryLoader {
    path: std::path::PathBuf,
}

impl Default for DebugLibraryLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugLibraryLoader {
    const FRAMEWORK_PATH: &'static str =
        "Chromium Embedded Framework.framework/Chromium Embedded Framework";

    pub fn new() -> Self {
        let path = home_dir()
            .unwrap()
            .join(".local")
            .join("share")
            .join("cef")
            .join(Self::FRAMEWORK_PATH)
            .canonicalize()
            .unwrap();

        Self { path }
    }

    // See [cef_load_library] for more documentation.
    pub fn load(&self) -> bool {
        Self::load_library(&self.path)
    }

    fn load_library(name: &std::path::Path) -> bool {
        use std::os::unix::ffi::OsStrExt;
        let Ok(name) = std::ffi::CString::new(name.as_os_str().as_bytes()) else {
            return false;
        };
        unsafe { load_library(Some(&*name.as_ptr().cast())) == 1 }
    }
}

impl Drop for DebugLibraryLoader {
    fn drop(&mut self) {
        if unload_library() != 1 {
            eprintln!("cannot unload framework {}", self.path.display());
        }
    }
}
