use bevy::prelude::*;
use std::time::Duration;

/// Intermediate resource for texture transfer diagnostics.
///
/// Written by `send_render_textures`, read and reset by `cef_diagnostics_system`.
#[derive(Resource, Default)]
pub struct CefTextureDiagnostics {
    pub last_transfer_time: Option<Duration>,
    pub total_buffer_bytes: u64,
}

/// Intermediate resource for IPC processing diagnostics.
///
/// Written by `receive_events`, read and reset by `cef_diagnostics_system`.
#[derive(Resource, Default)]
pub struct CefIpcDiagnostics {
    pub last_processing_time: Option<Duration>,
}

/// Cached webview count for diagnostics (bridges NonSend Browsers to Send resource).
#[derive(Resource, Default)]
pub struct CefWebviewCount(pub usize);
