#[cfg(feature = "browser")]
mod browser_process;
#[cfg(target_os = "macos")]
mod debug;

pub mod custom_scheme;
mod macros;
mod render_process;
mod util;

pub mod prelude {
    #[cfg(all(feature = "browser", target_os = "windows"))]
    pub use crate::browser_process::cef_command::{BrowsersProxy, CefCommand};
    #[cfg(all(feature = "browser", target_os = "windows"))]
    pub use crate::browser_process::cef_thread::{drain_commands, init_cef_browsers};
    #[cfg(feature = "browser")]
    pub use crate::browser_process::display_handler::{
        AddressChangedMessage, AddressChangedSenderInner,
    };
    #[cfg(feature = "browser")]
    pub use crate::browser_process::drag_handler::DraggableRegionSenderInner;
    #[cfg(feature = "browser")]
    pub use crate::browser_process::*;
    #[cfg(all(feature = "browser", target_os = "macos"))]
    pub use crate::browser_process::accelerated_paint::{
        RetainedIoSurface, WebviewGpuSurface, import_iosurface_to_wgpu,
    };
    pub use crate::custom_scheme::{
        CefCustomScheme, CefSchemeBody, CefSchemeHandler, CefSchemeOptions, CefSchemeRequest,
        CefSchemeResponse, init_registered_schemes,
    };
    #[cfg(target_os = "macos")]
    pub use crate::debug::*;
    pub use crate::render_process::app::*;
    pub use crate::render_process::execute_render_process;
    pub use crate::render_process::render_process_handler::*;
    pub use crate::util::*;
    pub use cef::DraggableRegion;
    pub use cef::Rect;
}
