---
sidebar_position: 4
---

# Sprite Rendering

In addition to rendering webviews onto 3D meshes, bevy_cef supports 2D sprite rendering. This is useful for HUD elements, menus, or any scenario where you want a flat webview in a 2D scene.

## Basic Setup

Sprite rendering requires no special material. Combine a `WebviewSource` with a standard Bevy `Sprite` component:

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(Startup, (spawn_camera, spawn_sprite_webview))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_sprite_webview(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((
        WebviewSource::new("https://github.com/not-elm/bevy_cef"),
        Pickable::default(),
        Sprite {
            image: images.add(Image::default()),
            custom_size: Some(Vec2::splat(500.0)),
            ..default()
        },
    ));
}
```

## Key Differences from 3D Rendering

| | 3D Mesh | 2D Sprite |
|---|---|---|
| Camera | `Camera3d` | `Camera2d` |
| Material | `MeshMaterial3d<WebviewExtendStandardMaterial>` | Not needed |
| Mesh | `Mesh3d` with any shape | `Sprite` with `custom_size` |
| Lighting | Responds to scene lighting | No lighting needed |
| Size control | Mesh dimensions | `Sprite::custom_size` |

## Input Handling

Add `Pickable::default()` to your sprite entity to enable mouse interaction. This allows bevy_cef to forward click, scroll, and hover events from Bevy's pointer system to the CEF browser.

Without `Pickable`, the webview will render but will not respond to mouse input.

## Controlling the Display Size

The `Sprite::custom_size` field controls how large the webview appears on screen, in world units. The `WebviewSize` component (default 800x800) controls the pixel resolution of the rendered web content. For sharp text, increase `WebviewSize` while keeping `custom_size` at your desired display dimensions.

```rust
commands.spawn((
    WebviewSource::new("https://example.com"),
    WebviewSize::new(1920, 1080),
    Pickable::default(),
    Sprite {
        image: images.add(Image::default()),
        custom_size: Some(Vec2::new(960.0, 540.0)),
        ..default()
    },
));
```
