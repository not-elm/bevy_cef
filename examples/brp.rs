//! Shows how to use Bevy Remote Protocol (BRP) with the webview.
//!
//! Please see [here](https://gist.github.com/coreh/1baf6f255d7e86e4be29874d00137d1d) for more about BRP.

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_cef::prelude::*;
use bevy_remote::{BrpResult, RemotePlugin};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RemotePlugin::default().with_method("greet", greet),
            CefPlugin::default(),
        ))
        .add_systems(
            Startup,
            (ime, spawn_camera, spawn_directional_light, spawn_webview),
        )
        .add_systems(
            Update,
            show_devtool.run_if(on_timer(Duration::from_secs(1))),
        )
        .run();
}

fn greet(In(name): In<Option<serde_json::Value>>) -> BrpResult {
    let name = name.unwrap_or_default();
    Ok(serde_json::Value::String(format!("Hello, {name}!")))
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
        WebviewSource::local("brp.html"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial {
            base: StandardMaterial {
                unlit: true,
                emissive: Color::WHITE.into(),
                ..default()
            },
            ..default()
        })),
    ));
}

fn ime(mut windows: Query<&mut bevy::prelude::Window>) {
    for mut window in windows.iter_mut() {
        window.ime_enabled = true;
    }
}

fn show_devtool(
    mut commands: Commands,
    webviews: Query<Entity, With<WebviewSource>>,
    mut initialized: Local<bool>,
) {
    if *initialized {
        return;
    }
    *initialized = true;
    for webview in webviews.iter() {
        commands.trigger(RequestShowDevTool { webview });
    }
}
