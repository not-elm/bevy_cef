//! Shows how to change the zoom level of a webview.

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview),
        )
        .add_systems(Update, change_zoom_level)
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
        CefWebviewUri("https://bevy.org/".to_string()),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}

fn change_zoom_level(mut er: MessageReader<MouseWheel>, mut webviews: Query<&mut ZoomLevel>) {
    for event in er.read() {
        webviews.par_iter_mut().for_each(|mut level| {
            if event.y > 0.0 {
                level.0 += 0.1; // Zoom in
            } else if event.y < 0.0 {
                level.0 -= 0.1; // Zoom out
            }
        });
    }
}
