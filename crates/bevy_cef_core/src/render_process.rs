use crate::prelude::RenderProcessAppBuilder;
use cef::args::Args;
use cef::{api_hash, execute_process, sys};

pub mod app;
pub mod cef_api_handler;
pub mod render_process_handler;

/// Execute the CEF render process.
pub fn execute_render_process() {
    let args = Args::new();
    #[cfg(target_os = "macos")]
    let _loader = {
        let loader =
            cef::library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), true);
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
