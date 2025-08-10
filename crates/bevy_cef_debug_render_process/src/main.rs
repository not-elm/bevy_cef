use bevy_cef_core::prelude::*;
use cef::{args::Args, *};

fn main() {
    let args = Args::new();
    #[cfg(target_os = "macos")]
    let _loader = {
        let loader = DebugLibraryLoader::new();
        assert!(loader.load());
        loader
    };
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let mut app = RenderProcessAppBuilder::build();
    execute_process(
        Some(args.as_main_args()),
        Some(&mut app),
        std::ptr::null_mut(),
    );
}
