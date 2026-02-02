//! Shows how to navigate a webview using keyboard input.
//!
//! ## Keyboard Controls
//!
//! - Press `Z` to go back in history.
//! - Press `X` to go forward in history.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview),
        )
        .add_systems(
            Update,
            (
                request_go_back.run_if(input_just_pressed(KeyCode::KeyZ)),
                request_go_forward.run_if(input_just_pressed(KeyCode::KeyX)),
            ),
        )
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
        CefWebviewUri::new("https://github.com/not-elm/bevy_cef"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}

fn request_go_back(mut commands: Commands, webviews: Query<Entity, With<DebugWebview>>) {
    for webview in webviews.iter() {
        commands.trigger(RequestGoBack { webview });
    }
}

fn request_go_forward(mut commands: Commands, webviews: Query<Entity, With<DebugWebview>>) {
    for webview in webviews.iter() {
        commands.trigger(RequestGoForward { webview });
    }
}
