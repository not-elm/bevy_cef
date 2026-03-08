---
sidebar_position: 6
---

# Preload Scripts

Preload scripts let you inject JavaScript that runs before any page scripts execute. This is useful for setting up global variables, polyfills, or bridges between your web content and Bevy.

## Adding Preload Scripts

Add the `PreloadScripts` component to your webview entity:

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        WebviewSource::new("https://example.com"),
        PreloadScripts::from([
            "console.log('Preload script running before page scripts');",
            "window.BEVY_ENV = 'development';",
        ]),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
```

## Execution Order

Preload scripts execute in the CEF render process **before** any `<script>` tags on the page run. This guarantees that any global variables or functions you define are available when the page's own JavaScript starts. Scripts are executed in the order they appear in the `PreloadScripts` vector.

## Per-Webview Scope

`PreloadScripts` is a per-entity component. Each webview can have its own set of preload scripts, or none at all. This lets you customize behavior for individual webviews:

```rust
// Dashboard webview with analytics setup
commands.spawn((
    WebviewSource::local("dashboard.html"),
    PreloadScripts::from(["window.ANALYTICS_ENABLED = true;"]),
    // ... mesh and material
));

// Settings webview with no preload scripts
commands.spawn((
    WebviewSource::local("settings.html"),
    // ... mesh and material
));
```

## Use Cases

- **Environment configuration** -- Set `window` variables that your web app reads at startup.
- **API injection** -- Define helper functions that wrap `window.cef.emit()` or `window.cef.brp()` calls.
- **Polyfills** -- Add compatibility shims before the page loads.
- **Debug tooling** -- Inject logging or error-reporting hooks during development.

:::tip Preload Scripts vs Extensions

Preload scripts are **per-webview** and run on every page navigation. [Extensions](./extensions.md) are **global** to all webviews and registered once at startup. Use preload scripts when you need different behavior per webview; use extensions when you want shared functionality everywhere.

:::
