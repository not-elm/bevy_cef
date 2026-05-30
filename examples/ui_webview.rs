//! Renders a webview inside a bevy_ui flex layout via WebviewUiMaterial.

use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    #[cfg(not(target_os = "macos"))]
    bevy_cef::prelude::early_exit_if_subprocess();

    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(Startup, (spawn_camera, spawn_ui))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_ui(mut commands: Commands, mut materials: ResMut<Assets<WebviewUiMaterial>>) {
    // Root column: a header bar + a webview filling the rest.
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                Node { width: Val::Percent(100.0), height: Val::Px(40.0), ..default() },
                BackgroundColor(Color::srgb(0.1, 0.1, 0.12)),
            ));
            root.spawn((
                WebviewSource::inline(
                    r#"<!DOCTYPE html><html><body style="margin:0;background:#222;color:#0f0;font-family:sans-serif">
                    <h1>Webview in a bevy_ui node</h1></body></html>"#,
                ),
                Node { width: Val::Percent(100.0), flex_grow: 1.0, ..default() },
                MaterialNode(materials.add(WebviewUiMaterial::default())),
            ));
        });
}
