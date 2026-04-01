//! # CEF Diagnostics Example
//!
//! Demonstrates how to use [`CefDiagnosticsPlugin`] to monitor bevy_cef's
//! runtime performance via an on-screen overlay.
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
//! [`DiagnosticsStore`]. This example displays the 3 timing metrics as an
//! on-screen overlay using Bevy UI text nodes:
//!
//! | Metric | Description |
//! |--------|-------------|
//! | `cef/message_loop_time` | Wall-clock duration of CEF's `cef_do_message_loop_work()` per frame |
//! | `cef/texture_transfer_time` | Time from CEF `on_paint` to Bevy channel receive |
//! | `cef/ipc_processing_time` | Bevy-side IPC event deserialization + trigger queue time |
//!
//! Two additional metrics (`cef/texture_buffer_memory`, `cef/webview_count`)
//! are quasi-static and omitted from the overlay. Access them programmatically:
//!
//! ```rust,ignore
//! fn my_system(store: Res<DiagnosticsStore>) {
//!     if let Some(d) = store.get(&CefDiagnosticsPlugin::TEXTURE_BUFFER_MEMORY) {
//!         if let Some(bytes) = d.smoothed() { /* use bytes */ }
//!     }
//! }
//! ```
//!
//! ## How it works
//!
//! 1. `CefDiagnosticsPlugin` records metrics into intermediate resources each frame
//! 2. A collection system feeds them into Bevy's `DiagnosticsStore`
//! 3. This example spawns a UI text node and updates it every second from the store
//!
//! The overlay uses `.smoothed()` (exponential moving average) for stable readings.
//! Text update overhead is ~0.1ms per 1-second cycle — negligible relative to the
//! sub-millisecond CEF metrics being measured.

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy_cef::diagnostics::CefDiagnosticsPlugin;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin::default(),
            CefDiagnosticsPlugin,
        ))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview, spawn_overlay),
        )
        .add_systems(Update, update_overlay)
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

// --- Overlay ---

/// Marker component for the overlay text node.
#[derive(Component)]
struct DiagnosticsOverlayText;

fn spawn_overlay(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)))
        .insert(GlobalZIndex(i32::MAX - 100))
        .with_child((
            Text::new("CEF Diagnostics\n---"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgba(0.0, 1.0, 0.0, 1.0)),
            DiagnosticsOverlayText,
        ));
}

fn update_overlay(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<DiagnosticsOverlayText>>,
    time: Res<Time>,
    mut timer: Local<f64>,
) {
    *timer += time.delta_secs_f64();
    if *timer < 1.0 {
        return;
    }
    *timer = 0.0;

    let msg_loop = diagnostics
        .get(&CefDiagnosticsPlugin::MESSAGE_LOOP_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let texture = diagnostics
        .get(&CefDiagnosticsPlugin::TEXTURE_TRANSFER_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let ipc = diagnostics
        .get(&CefDiagnosticsPlugin::IPC_PROCESSING_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    for mut text in &mut query {
        **text = format!(
            "CEF Diagnostics\nMessage Loop:     {msg_loop:.2} ms\nTexture Transfer: {texture:.2} ms\nIPC Processing:   {ipc:.2} ms"
        );
    }
}
