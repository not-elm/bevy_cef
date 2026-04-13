//! HiDPI / device_scale_factor demo.
//!
//! Shows a 3D plane webview that displays its current
//! `window.devicePixelRatio` live. Move the window between monitors with
//! different DPI to verify the plugin updates the rendering resolution on
//! the fly, or press `1` / `2` / `3` in the host window to force DPR
//! 1.0 / 2.0 / 3.0 at runtime (useful for single-monitor testing).

use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview),
        )
        .add_systems(Update, change_dpr_on_keypress)
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
        WebviewSource::local("hidpi_demo.html"),
        WebviewSize(Vec2::new(800.0, 800.0)),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}

fn change_dpr_on_keypress(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut webviews: Query<&mut WebviewDpr, With<WebviewSource>>,
) {
    let new_dpr = if keyboard.just_pressed(KeyCode::Digit1) {
        1.0
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        2.0
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        3.0
    } else {
        return;
    };
    for mut dpr in webviews.iter_mut() {
        dpr.set_if_neq(WebviewDpr(new_dpr));
    }
}
