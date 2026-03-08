---
sidebar_position: 8
---

# Zoom & Audio

bevy_cef provides per-webview zoom and audio mute controls through simple ECS components. Changes are reactive -- Bevy's change detection automatically propagates updates to the underlying CEF browser.

## Zoom Level

The `ZoomLevel` component controls the zoom level of a webview. It wraps an `f64` value where `0.0` is the default zoom. Positive values zoom in, negative values zoom out.

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
        ZoomLevel(1.5), // Zoomed in
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
```

### Dynamic Zoom with Mouse Wheel

A common pattern is binding the scroll wheel to zoom:

```rust
fn change_zoom_level(
    mut er: MessageReader<MouseWheel>,
    mut webviews: Query<&mut ZoomLevel>,
) {
    for event in er.read() {
        webviews.par_iter_mut().for_each(|mut level| {
            if event.y > 0.0 {
                level.0 += 0.1;
            } else if event.y < 0.0 {
                level.0 -= 0.1;
            }
        });
    }
}
```

## Audio Mute

The `AudioMuted` component controls whether a webview's audio output is muted. It wraps a `bool` value:

```rust
commands.spawn((
    WebviewSource::new("https://example.com"),
    AudioMuted(true), // Start muted
    // ... mesh and material
));
```

### Toggling Audio at Runtime

```rust
fn toggle_audio(
    keys: Res<ButtonInput<KeyCode>>,
    mut webviews: Query<&mut AudioMuted>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        for mut muted in webviews.iter_mut() {
            muted.0 = !muted.0;
        }
    }
}
```

## Reactive Updates

Both `ZoomLevel` and `AudioMuted` are standard Bevy components. bevy_cef uses change detection internally, so mutating these components through a `Mut<ZoomLevel>` or `Mut<AudioMuted>` reference is all you need -- the changes are forwarded to CEF automatically. There is no need to send commands or trigger events.

:::tip Default Values

When not explicitly added, `ZoomLevel` defaults to `0.0` (100% zoom) and `AudioMuted` defaults to `false` (audio enabled). These components are auto-required by `WebviewSource`, so they are always present on webview entities.

:::
