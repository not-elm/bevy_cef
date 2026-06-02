//! Renders two interactive webviews inside a bevy_ui flex layout via
//! WebviewUiMaterial. Demonstrates pointer + focus-based keyboard input: click a
//! webview's text field, type into it, scroll it, then click the other and
//! confirm keystrokes follow focus.

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
    const PAGE: &str = r#"<!DOCTYPE html>
    <html>
        <body style="margin:0;background:#222;color:#0f0;font-family:sans-serif;height:1500px">
            <h2>Type here, then scroll</h2>
            <input style="font-size:20px;width:80%" placeholder="focus me and type" />
            <p>Scroll down — this page is tall.</p>
            <div style="margin-top:1200px">Bottom of page.</div>
        </body>
    </html>"#;

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|root| {
            for _ in 0..2 {
                root.spawn((
                    WebviewSource::inline(PAGE),
                    Node {
                        height: Val::Percent(100.0),
                        flex_grow: 1.0,
                        ..default()
                    },
                    MaterialNode(materials.add(WebviewUiMaterial::default())),
                ));
            }
        });
}
