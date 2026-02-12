//! Example demonstrating custom JavaScript extensions via CEF's register_extension.
//!
//! This example shows how to create global JavaScript APIs that are available
//! in all webviews before any page scripts run.

use bevy::prelude::*;
use bevy_cef::prelude::*;
use serde::Deserialize;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin {
                extensions: CefExtensions::new().add(
                    "myGame",
                    r#"
                    var myGame = {
                        version: "1.0.0",
                        sendScore: function(score) {
                            window.cef.emit('score_update', { score: score });
                        }
                    };
                "#,
                ),
                ..Default::default()
            },
            JsEmitEventPlugin::<ScoreUpdate>::default(),
        ))
        .add_systems(
            Startup,
            (spawn_camera, spawn_directional_light, spawn_webview),
        )
        .add_observer(on_score_update)
        .run();
}

#[derive(Deserialize, Debug)]
struct ScoreUpdate {
    score: u32,
}

fn on_score_update(trigger: On<Receive<ScoreUpdate>>) {
    info!("Received score update: {:?}", trigger.score);
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
        WebviewSource::local("extensions.html"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
