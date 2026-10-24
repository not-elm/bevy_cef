#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use bevy_cef_core::prelude::*;
use cef::{args::Args, *};

#[cfg(target_os = "macos")]
enum DebugRenderProcessLoader {
    Custom(CustomLibraryLoader),
    Debug(DebugLibraryLoader),
}

#[cfg(target_os = "macos")]
impl DebugRenderProcessLoader {
    fn load(&self) -> bool {
        match self {
            Self::Custom(l) => l.load(),
            Self::Debug(l) => l.load(),
        }
    }
}

fn main() {
    let args = Args::new();

    #[cfg(target_os = "macos")]
    let _loader = {
        let (_, framework_binary, _) = resolve_custom_cef_paths(None, None);

        let loader = if let Some(bin) = framework_binary.filter(|b| b.exists()) {
            DebugRenderProcessLoader::Custom(CustomLibraryLoader::new(bin))
        } else {
            DebugRenderProcessLoader::Debug(DebugLibraryLoader::new())
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
