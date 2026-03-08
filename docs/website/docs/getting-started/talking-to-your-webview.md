---
sidebar_position: 2
---

# Talking to Your Webview

Now that you have a webview rendering in your scene, the next step is communication. bevy_cef provides three IPC patterns, but the simplest starting point is **JS Emit** -- sending events from your webview's JavaScript into Bevy.

## JS Emit: Webview to Bevy

JS Emit is a fire-and-forget pattern. JavaScript calls `window.cef.emit()` with an event name and data, and Bevy receives it as an `EntityEvent` on the webview entity.

### Step 1: Define a Message Struct

Create a Rust struct that matches the shape of the data your JavaScript will send. It must implement `Deserialize`:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Message {
    count: u32,
}
```

### Step 2: Register the Plugin

Add a `JsEmitEventPlugin` for your message type. This tells bevy_cef to listen for events named after your type (lowercase) and deserialize them into your struct:

```rust
app.add_plugins(JsEmitEventPlugin::<Message>::default());
```

### Step 3: Add an Observer

Use Bevy's observer pattern to react when a `Receive<Message>` event fires on a webview entity:

```rust
fn on_message(trigger: On<Receive<Message>>) {
    info!("Received count: {}", trigger.count);
}

// In your app setup:
app.add_observer(on_message);
```

The `Receive<T>` wrapper is an `EntityEvent` -- it fires on the specific webview entity that emitted it, so you always know which webview the event came from.

### Step 4: Emit from JavaScript

In your HTML file, call `window.cef.emit()` with the event name and a data object:

```html
<!DOCTYPE html>
<html>
<body>
  <button id="btn">Click me</button>
  <script>
    let count = 0;
    document.getElementById('btn').addEventListener('click', () => {
      count += 1;
      window.cef.emit('message', { count });
    });
  </script>
</body>
</html>
```

Load this file with `WebviewSource::local("js_emit.html")` and place it in your `assets/` directory.

### Putting It Together

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Message {
    count: u32,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin::default(),
            JsEmitEventPlugin::<Message>::default(),
        ))
        .add_systems(Startup, spawn_webview)
        .add_observer(on_message)
        .run();
}

fn on_message(trigger: On<Receive<Message>>) {
    info!("Button clicked {} times", trigger.count);
}

fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        WebviewSource::local("js_emit.html"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
```

## What's Next

JS Emit handles the webview-to-Bevy direction. To send data the other way (Bevy to webview) or make async RPC calls, see the [Communication](../communication/host-emit.md) section:

- [Sending Events to Webview](../communication/host-emit.md) -- push state updates from Bevy into JavaScript
- [Calling Bevy APIs from JS](../communication/brp.md) -- async bidirectional RPC via the Bevy Remote Protocol
- [Choosing an IPC Pattern](../communication/index.md) -- a decision guide for when to use each approach
