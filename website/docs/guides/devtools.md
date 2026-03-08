---
sidebar_position: 3
---

# DevTools

bevy_cef lets you open Chrome DevTools for any webview, giving you access to the full suite of debugging tools -- DOM inspector, console, network monitor, and more.

## Opening and Closing DevTools

Use the `RequestShowDevTool` and `RequestCloseDevtool` EntityEvents to control DevTools:

```rust
// Open DevTools for a webview
commands.trigger(RequestShowDevTool { webview });

// Close DevTools for a webview
commands.trigger(RequestCloseDevtool { webview });
```

Both events require you to specify the target webview entity.

## Example: Toggle DevTools with a Key

This example opens DevTools when F12 is pressed and closes them when Escape is pressed:

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(Startup, (spawn_camera, spawn_webview))
        .add_systems(Update, toggle_devtools)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        WebviewSource::new("https://example.com"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}

fn toggle_devtools(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    webviews: Query<Entity, With<WebviewSource>>,
) {
    let Ok(webview) = webviews.single() else {
        return;
    };
    if keys.just_pressed(KeyCode::F12) {
        commands.trigger(RequestShowDevTool { webview });
    }
    if keys.just_pressed(KeyCode::Escape) {
        commands.trigger(RequestCloseDevtool { webview });
    }
}
```

## How It Works

DevTools opens as a separate CEF window. The `RequestShowDevTool` and `RequestCloseDevtool` events follow the same EntityEvent trigger/observer pattern used by navigation commands. bevy_cef's internal observers receive the event and forward it to the CEF browser instance.

:::tip

DevTools is invaluable during development for inspecting the DOM, debugging JavaScript, and profiling performance. You can use the console to test `window.cef.emit()` and `window.cef.brp()` calls interactively.

:::
