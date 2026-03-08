---
sidebar_position: 2
---

# Calling Bevy APIs from JS

**BRP (Bevy Remote Protocol)** enables async, bidirectional RPC between your webview's JavaScript and Bevy. Unlike JS Emit and Host Emit which are fire-and-forget, BRP calls return a value -- JavaScript `await`s a Promise that resolves with the result from Bevy.

## How It Works

BRP is built on top of Bevy's `RemotePlugin`. You register named methods in Rust, and JavaScript calls them with `await window.cef.brp()`. Under the hood, the call is proxied through CEF's V8 engine as a Promise, sent via IPC to the browser process, dispatched through `bevy_remote`, and the result is sent back to resolve the Promise.

`CefPlugin` automatically adds `RemotePlugin` if your app does not already include it, so you do not need to add it manually.

## Registering a Method

Define a Bevy system that takes an `In<Option<serde_json::Value>>` parameter and returns a `BrpResult`:

```rust
use bevy::prelude::*;
use bevy_remote::BrpResult;

fn greet(In(params): In<Option<serde_json::Value>>) -> BrpResult {
    let name = params
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "World".to_string());
    Ok(serde_json::Value::String(format!("Hello, {name}!")))
}
```

Register it when building your app:

```rust
use bevy_remote::RemotePlugin;

app.add_plugins(
    RemotePlugin::default().with_method("greet", greet)
);
```

## Calling from JavaScript

```html
<!DOCTYPE html>
<html>
<body>
  <p id="result">Loading...</p>
  <script>
    async function callGreet() {
      const result = await window.cef.brp({
        method: 'greet',
        params: '"Bevy"'
      });
      document.getElementById('result').textContent = result;
    }
    callGreet();
  </script>
</body>
</html>
```

`window.cef.brp()` accepts an object with:

- **`method`** -- the method name registered in Rust.
- **`params`** -- a JSON-encoded string of the parameters. This is passed as `Option<serde_json::Value>` to the Rust handler.

The returned Promise resolves with the value returned from `Ok(...)` in your handler, or rejects if the handler returns an `Err(...)`.

## Full Example

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;
use bevy_remote::{BrpResult, RemotePlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin::default(),
            RemotePlugin::default().with_method("greet", greet),
        ))
        .add_systems(Startup, spawn_webview)
        .run();
}

fn greet(In(params): In<Option<serde_json::Value>>) -> BrpResult {
    let name = params
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "World".to_string());
    Ok(serde_json::Value::String(format!("Hello, {name}!")))
}

fn spawn_webview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        WebviewSource::local("brp.html"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
```

## Error Handling

If your handler returns an error, the Promise on the JavaScript side rejects:

```rust
fn risky_operation(In(_params): In<Option<serde_json::Value>>) -> BrpResult {
    Err(bevy_remote::BrpError {
        code: -1,
        message: "Something went wrong".into(),
        data: None,
    })
}
```

```javascript
try {
  const result = await window.cef.brp({ method: 'risky_operation', params: 'null' });
} catch (error) {
  console.error('BRP call failed:', error);
}
```

## See Also

- [Sending Events to Webview](./host-emit.md) -- for fire-and-forget Bevy-to-JS communication
- [Choosing an IPC Pattern](./index.md) -- when to use BRP vs other patterns
