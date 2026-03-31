use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::prelude::*;
use bevy_cef_core::prelude::Browsers;
use std::time::Duration;

use crate::common::CefMessageLoopDuration;

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

/// Cached webview count for diagnostics (bridges NonSend `Browsers` → Send resource).
#[derive(Resource, Default)]
pub struct CefWebviewCount(pub usize);

/// Registers CEF performance diagnostics into Bevy's `DiagnosticsStore`.
///
/// This plugin does NOT include FPS measurement — add
/// `FrameTimeDiagnosticsPlugin` separately if needed.
///
/// # Usage
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
/// # use bevy_cef::prelude::*;
/// # use bevy_cef::diagnostics::CefDiagnosticsPlugin;
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(CefPlugin::default())
///     .add_plugins(CefDiagnosticsPlugin)
///     .add_plugins(FrameTimeDiagnosticsPlugin::default())
///     .add_plugins(LogDiagnosticsPlugin::default());
/// ```
pub struct CefDiagnosticsPlugin;

impl CefDiagnosticsPlugin {
    /// Wall-clock duration of one `cef_do_message_loop_work()` call (ms).
    pub const MESSAGE_LOOP_TIME: DiagnosticPath =
        DiagnosticPath::const_new("cef/message_loop_time");

    /// Elapsed time from CEF `on_paint` to channel receive in `send_render_textures` (ms).
    pub const TEXTURE_TRANSFER_TIME: DiagnosticPath =
        DiagnosticPath::const_new("cef/texture_transfer_time");

    /// Bevy-side IPC receive → EntityEvent trigger processing time (ms).
    pub const IPC_PROCESSING_TIME: DiagnosticPath =
        DiagnosticPath::const_new("cef/ipc_processing_time");

    /// Sum of all texture buffer bytes received from channel in the frame.
    pub const TEXTURE_BUFFER_MEMORY: DiagnosticPath =
        DiagnosticPath::const_new("cef/texture_buffer_memory");

    /// Number of active browser instances.
    pub const WEBVIEW_COUNT: DiagnosticPath = DiagnosticPath::const_new("cef/webview_count");
}

impl Plugin for CefDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(
            Diagnostic::new(Self::MESSAGE_LOOP_TIME).with_suffix("ms"),
        )
        .register_diagnostic(
            Diagnostic::new(Self::TEXTURE_TRANSFER_TIME).with_suffix("ms"),
        )
        .register_diagnostic(
            Diagnostic::new(Self::IPC_PROCESSING_TIME).with_suffix("ms"),
        )
        .register_diagnostic(
            Diagnostic::new(Self::TEXTURE_BUFFER_MEMORY).with_suffix("bytes"),
        )
        .register_diagnostic(Diagnostic::new(Self::WEBVIEW_COUNT))
        .init_resource::<CefMessageLoopDuration>()
        .init_resource::<CefTextureDiagnostics>()
        .init_resource::<CefIpcDiagnostics>()
        .init_resource::<CefWebviewCount>()
        .add_systems(
            Update,
            (update_webview_count, cef_diagnostics_system),
        );
    }
}

fn update_webview_count(browsers: NonSend<Browsers>, mut count: ResMut<CefWebviewCount>) {
    count.0 = browsers.len();
}

fn cef_diagnostics_system(
    mut diagnostics: Diagnostics,
    mut message_loop: ResMut<CefMessageLoopDuration>,
    mut texture: ResMut<CefTextureDiagnostics>,
    mut ipc: ResMut<CefIpcDiagnostics>,
    webview_count: Res<CefWebviewCount>,
) {
    if let Some(duration) = message_loop.0.take() {
        diagnostics.add_measurement(&CefDiagnosticsPlugin::MESSAGE_LOOP_TIME, || {
            duration.as_secs_f64() * 1000.0
        });
    }

    if let Some(duration) = texture.last_transfer_time.take() {
        diagnostics.add_measurement(&CefDiagnosticsPlugin::TEXTURE_TRANSFER_TIME, || {
            duration.as_secs_f64() * 1000.0
        });
    }

    if texture.total_buffer_bytes > 0 {
        let bytes = texture.total_buffer_bytes;
        texture.total_buffer_bytes = 0;
        diagnostics
            .add_measurement(&CefDiagnosticsPlugin::TEXTURE_BUFFER_MEMORY, || bytes as f64);
    }

    if let Some(duration) = ipc.last_processing_time.take() {
        diagnostics.add_measurement(&CefDiagnosticsPlugin::IPC_PROCESSING_TIME, || {
            duration.as_secs_f64() * 1000.0
        });
    }

    diagnostics
        .add_measurement(&CefDiagnosticsPlugin::WEBVIEW_COUNT, || webview_count.0 as f64);
}
