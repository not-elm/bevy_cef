---
sidebar_position: 2
---

# Navigation

bevy_cef supports browser-style back and forward navigation through Bevy's EntityEvent trigger/observer pattern. You can programmatically navigate a webview's history in response to user input or game logic.

## Navigation Commands

Two EntityEvent types control navigation:

- `RequestGoBack` -- navigates the webview to the previous page in its history.
- `RequestGoForward` -- navigates the webview to the next page in its history.

Both require you to specify which webview entity to navigate via the `webview` field.

## Example: Keyboard-Driven Navigation

This example binds the left and right arrow keys to back and forward navigation:

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(Startup, (spawn_camera, spawn_webview))
        .add_systems(Update, handle_navigation)
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

fn handle_navigation(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    webviews: Query<Entity, With<WebviewSource>>,
) {
    let Ok(webview) = webviews.single() else {
        return;
    };
    if keys.just_pressed(KeyCode::ArrowLeft) {
        commands.trigger(RequestGoBack { webview });
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        commands.trigger(RequestGoForward { webview });
    }
}
```

## How It Works

`RequestGoBack` and `RequestGoForward` are Bevy EntityEvents. When you call `commands.trigger()`, bevy_cef's internal observers pick up the event and forward the navigation command to the underlying CEF browser instance. This follows the same pattern used by other bevy_cef commands like `RequestShowDevTool`.

:::note

Navigation only works if the webview has browsing history. Calling `RequestGoBack` on a webview that has only loaded one page has no effect.

:::
