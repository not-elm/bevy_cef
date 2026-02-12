//! Shows how to emit an event from the host to the webview.

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_cef::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview),
        )
        .add_systems(Update, emit_count.run_if(on_timer(Duration::from_secs(1))))
        .run();
}

#[derive(Component)]
struct DebugWebview;

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
        DebugWebview,
        WebviewSource::local("host_emit.html"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}

fn emit_count(
    mut commands: Commands,
    mut count: Local<usize>,
    webviews: Query<Entity, With<DebugWebview>>,
) {
    *count += 1;
    commands.trigger(HostEmitEvent::new(
        webviews.single().unwrap(),
        "count",
        &*count,
    ));
}
