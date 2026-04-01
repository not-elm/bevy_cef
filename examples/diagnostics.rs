//! # CEF Diagnostics Example
//!
//! Demonstrates how to use [`CefDiagnosticsPlugin`] to monitor bevy_cef's
//! runtime performance through Bevy's standard diagnostics system.
//!
//! ## Running this example
//!
//! ```sh
//! # macOS (requires debug feature)
//! cargo run --example diagnostics --features debug
//!
//! # Windows
//! cargo run --example diagnostics
//! ```
//!
//! ## What is `CefDiagnosticsPlugin`?
//!
//! An opt-in plugin that registers 5 performance metrics into Bevy's
//! [`DiagnosticsStore`]. It does NOT include FPS — add
//! [`FrameTimeDiagnosticsPlugin`] separately if you want that.
//!
//! ## Metrics
//!
//! | Metric | Unit | Description |
//! |--------|------|-------------|
//! | `cef/message_loop_time` | ms | Wall-clock duration of CEF's `cef_do_message_loop_work()` per frame. High values indicate CEF is doing significant work (layout, JS execution, etc.). |
//! | `cef/texture_transfer_time` | ms | Time from CEF's `on_paint` callback to when Bevy receives the texture via async channel. Includes channel latency and frame scheduling. |
//! | `cef/ipc_processing_time` | ms | Bevy-side time to deserialize an IPC event and queue the EntityEvent trigger. Only recorded when JS→Bevy events are actually received. |
//! | `cef/texture_buffer_memory` | bytes | Total texture buffer bytes received in the current frame. For an 800×800 BGRA webview, each frame is ~2.56MB. Scales linearly with webview count and resolution. |
//! | `cef/webview_count` | count | Number of active CEF browser instances. Useful for correlating performance with webview count. |
//!
//! ## Two ways to read diagnostics
//!
//! ### 1. Automatic console output (easiest)
//!
//! Add [`LogDiagnosticsPlugin`] — it prints all registered diagnostics to the
//! console every second. This is the approach used in this example.
//!
//! ```rust,no_run
//! app.add_plugins(LogDiagnosticsPlugin::default());
//! ```
//!
//! ### 2. Programmatic access via `DiagnosticsStore`
//!
//! Read metrics directly in a system. This example includes a custom system
//! `log_cef_diagnostics` that demonstrates this approach.
//!
//! ```rust,ignore
//! fn my_system(diagnostics: Res<DiagnosticsStore>) {
//!     if let Some(d) = diagnostics.get(&CefDiagnosticsPlugin::TEXTURE_TRANSFER_TIME) {
//!         // d.value()    — latest raw measurement
//!         // d.average()  — simple moving average over history window
//!         // d.smoothed() — exponential moving average (less noisy)
//!     }
//! }
//! ```
//!
//! ## How it works internally
//!
//! ```text
//! ┌─────────────────────────┐     ┌──────────────────────────────┐
//! │   Measurement Points    │     │   Diagnostics Collection     │
//! │                         │     │                              │
//! │  on_paint (CEF)         │────►│  send_render_textures()      │
//! │    → Instant::now()     │     │    → CefTextureDiagnostics   │
//! │    → RenderTextureMsg   │     │                              │
//! │                         │     │  cef_diagnostics_system()    │
//! │  cef_do_message_loop()  │────►│    → reads all resources     │
//! │    → CefMsgLoopDuration │     │    → Diagnostics::add_...()  │
//! │                         │     │    → resets for next frame   │
//! │  receive_events<E>()    │────►│                              │
//! │    → CefIpcDiagnostics  │     │  LogDiagnosticsPlugin        │
//! │                         │     │    → prints every 1 second   │
//! └─────────────────────────┘     └──────────────────────────────┘
//! ```
//!
//! When `CefDiagnosticsPlugin` is NOT added, existing systems skip diagnostics
//! recording via `Option<ResMut<...>>` checks — the only overhead is a single
//! `Instant::now()` (~25ns) per texture creation.

use bevy::diagnostic::{
    DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy_cef::diagnostics::CefDiagnosticsPlugin;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin::default(),
            // CEF performance metrics (5 diagnostics)
            CefDiagnosticsPlugin,
            // FPS / frame time (Bevy standard)
            FrameTimeDiagnosticsPlugin::default(),
            // Prints all diagnostics to the console every second
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview),
        )
        // Custom system: demonstrates programmatic access to CEF diagnostics
        .add_systems(Update, log_cef_diagnostics)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 0., 3.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_directional_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(1., 1., 1.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        WebviewSource::new("https://github.com/not-elm/bevy_cef"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}

/// Demonstrates reading CEF diagnostics programmatically via [`DiagnosticsStore`].
///
/// This system logs a compact summary every 2 seconds. In a real application you
/// might use these values to drive a debug UI overlay, trigger alerts, or feed an
/// external monitoring system.
fn log_cef_diagnostics(diagnostics: Res<DiagnosticsStore>, time: Res<Time>, mut timer: Local<f64>) {
    *timer += time.delta_secs_f64();
    if *timer < 2.0 {
        return;
    }
    *timer = 0.0;

    let msg_loop = diagnostics
        .get(&CefDiagnosticsPlugin::MESSAGE_LOOP_TIME)
        .and_then(|d| d.smoothed());

    let texture = diagnostics
        .get(&CefDiagnosticsPlugin::TEXTURE_TRANSFER_TIME)
        .and_then(|d| d.smoothed());

    let memory = diagnostics
        .get(&CefDiagnosticsPlugin::TEXTURE_BUFFER_MEMORY)
        .and_then(|d| d.smoothed());

    let webviews = diagnostics
        .get(&CefDiagnosticsPlugin::WEBVIEW_COUNT)
        .and_then(|d| d.value());

    info!(
        "[CEF Diagnostics] msg_loop={:.2}ms, texture_transfer={:.2}ms, buffer={:.0} bytes, webviews={}",
        msg_loop.unwrap_or(0.0),
        texture.unwrap_or(0.0),
        memory.unwrap_or(0.0),
        webviews.unwrap_or(0.0) as u32,
    );
}
