//! Shows how to emit a message from the webview to the application.

use bevy::prelude::*;
use bevy_cef::prelude::*;
use serde::Deserialize;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin::new(),
            JsEmitEventPlugin::<Message>::default(),
        ))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview),
        )
        .add_observer(apply_receive_message)
        .run();
}

#[derive(Deserialize)]
struct Message {
    count: u32,
}

fn apply_receive_message(trigger: On<Receive<Message>>) {
    info!("Received: {:?}", trigger.count);
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
        CefWebviewUri::local("js_emit.html"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
