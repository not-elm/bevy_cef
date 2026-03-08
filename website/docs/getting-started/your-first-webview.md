---
sidebar_position: 1
---

# Your First Webview

In this guide, you will render a webpage onto a 3D plane in a Bevy scene. By the end, you will have a working application that displays a live web page as a texture on a mesh.

## Full Example

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(Startup, (spawn_camera, spawn_directional_light, spawn_webview))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_directional_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::default().looking_at(Vec3::new(1.0, -1.0, -1.0), Vec3::Y),
    ));
}

fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        WebviewSource::new("https://github.com/not-elm/bevy_cef"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
```

## Understanding the Components

The webview entity is built from three key components:

### `WebviewSource`

This tells bevy_cef what content to render. You can provide:

- A remote URL: `WebviewSource::new("https://example.com")`
- A local HTML file: `WebviewSource::local("my_page.html")` (served via the built-in `cef://localhost/` scheme)
- Inline HTML: `WebviewSource::inline("<h1>Hello</h1>")`

### `Mesh3d`

A standard Bevy mesh component. The webview texture is painted onto this mesh. You can use any mesh shape -- a plane, a cube, a cylinder -- but a flat plane facing the camera is the most common choice.

### `MeshMaterial3d<WebviewExtendStandardMaterial>`

This material receives the webview texture from CEF and applies it to the mesh. `WebviewExtendStandardMaterial` extends Bevy's `StandardMaterial`, so it responds to lighting and supports all the usual material properties.

:::caution WebviewSize is texture resolution, not mesh size

`WebviewSize` controls the **pixel resolution** of the rendered web content (default 800x800). It does not affect the physical size of the 3D mesh. To make the webview appear larger or smaller in the scene, scale the mesh or change the plane dimensions -- not `WebviewSize`. Increasing `WebviewSize` gives you sharper text and images at the cost of more GPU memory.

:::

## What's Next

Your webview is rendering, but it is a one-way street -- the web page cannot talk to your Bevy app yet. Head to [Talking to Your Webview](./talking-to-your-webview.md) to learn how to send events between JavaScript and Bevy.
