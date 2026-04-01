mod browser_process;
#[cfg(target_os = "macos")]
mod debug;

mod render_process;
mod util;

pub mod prelude {
    #[cfg(target_os = "windows")]
    pub use crate::browser_process::cef_command::{BrowsersProxy, CefCommand};
    #[cfg(target_os = "windows")]
    pub use crate::browser_process::cef_thread::{drain_commands, init_cef_browsers};
    pub use crate::browser_process::*;
    #[cfg(target_os = "macos")]
    pub use crate::debug::*;
    pub use crate::render_process::app::*;
    pub use crate::render_process::execute_render_process;
    pub use crate::render_process::render_process_handler::*;
    pub use crate::util::*;
}
