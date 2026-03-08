---
sidebar_position: 1
---

# Local Assets

bevy_cef includes a built-in `cef://localhost/` scheme that serves local files through Bevy's asset system. This lets you load HTML, CSS, JavaScript, and images from your project's `assets/` directory without running a separate web server.

## Loading a Local HTML File

Use `WebviewSource::local()` to load an HTML file from your assets directory:

```rust
commands.spawn((
    WebviewSource::local("ui/index.html"),
    Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
));
```

This resolves to `cef://localhost/ui/index.html`. Inside your HTML, you can reference other local assets using relative paths:

```html
<link rel="stylesheet" href="ui/styles.css" />
<script src="ui/app.js"></script>
<img src="ui/logo.png" />
```

All paths are relative to the `assets/` directory at the root of your Bevy project.

## Inline HTML

For small snippets or dynamically generated content, use `WebviewSource::inline()`:

```rust
commands.spawn((
    WebviewSource::inline(r#"
        <html>
            <body>
                <h1>Hello from Bevy!</h1>
            </body>
        </html>
    "#),
    Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
));
```

Inline HTML is served internally via `cef://localhost/__inline__/{id}`, where each webview gets a unique identifier. When the webview entity is removed, the inline content is automatically cleaned up.

## Asset Directory Structure

A typical project layout looks like this:

```
my_project/
├── assets/
│   ├── ui/
│   │   ├── index.html
│   │   ├── styles.css
│   │   └── app.js
│   └── images/
│       └── logo.png
├── src/
│   └── main.rs
└── Cargo.toml
```

:::tip

The `cef://localhost/` scheme uses Bevy's asset system under the hood, so any file accessible through `AssetServer` is also accessible from your webview.

:::
