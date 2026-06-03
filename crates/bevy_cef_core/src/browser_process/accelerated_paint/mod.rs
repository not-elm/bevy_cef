//! macOS GPU OSR: OnAcceleratedPaint + IOSurface texture import.
#![cfg(target_os = "macos")]

mod iosurface;
pub use iosurface::import_iosurface_to_wgpu;
