#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use bevy_cef_core::prelude::*;
use cef::{args::Args, *};

#[cfg(target_os = "macos")]
enum RenderProcessLoader {
    Custom(CustomLibraryLoader),
    Default(library_loader::LibraryLoader),
}

#[cfg(target_os = "macos")]
impl RenderProcessLoader {
    fn load(&self) -> bool {
        match self {
            Self::Custom(l) => l.load(),
            Self::Default(l) => l.load(),
        }
    }
}

fn main() {
    let args = Args::new();

    #[cfg(target_os = "macos")]
    let (_, framework_binary) = {
        let (dir, bin, _) = resolve_custom_cef_paths(None, None);
        (dir, bin)
    };

    #[cfg(target_os = "macos")]
    let _sandbox = {
        let exe = std::env::current_exe().ok();
        let expected_sandbox = exe.as_ref().and_then(|e| e.parent()).map(|p| {
            p.join("../../../Chromium Embedded Framework.framework/Libraries/libcef_sandbox.dylib")
        });

        if expected_sandbox.as_ref().is_some_and(|p| p.exists()) {
            std::panic::catch_unwind(|| {
                let mut sandbox = cef::sandbox::Sandbox::new();
                sandbox.initialize(args.as_main_args());
                sandbox
            })
            .ok()
        } else {
            None
        }
    };
    #[cfg(target_os = "macos")]
    let _loader = {
        let exe = std::env::current_exe().unwrap();

        let sibling = exe
            .parent()
            .unwrap()
            .join("Chromium Embedded Framework.framework/Chromium Embedded Framework");

        let loader = if let Some(bin) = framework_binary.filter(|b| b.exists()) {
            RenderProcessLoader::Custom(CustomLibraryLoader::new(bin))
        } else if sibling.exists() {
            RenderProcessLoader::Custom(CustomLibraryLoader::new(sibling))
        } else {
            RenderProcessLoader::Default(library_loader::LibraryLoader::new(&exe, true))
        };
        assert!(loader.load());
        loader
    };
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let mut app = RenderProcessAppBuilder::build();
    let code = execute_process(
        Some(args.as_main_args()),
        Some(&mut app),
        std::ptr::null_mut(),
    );
    std::process::exit(code);
}
