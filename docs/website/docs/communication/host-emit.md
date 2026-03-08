---
sidebar_position: 1
---

# Sending Events to Webview

**Host Emit** lets you push data from Bevy into your webview's JavaScript. It is a fire-and-forget pattern: Bevy triggers an event, and any JavaScript listeners registered with `window.cef.listen()` receive the data.

## How It Works

Host Emit uses Bevy's `EntityEvent` trigger pattern. You create a `HostEmitEvent` specifying the target webview entity, an event name, and serializable data, then trigger it via `commands.trigger()`.

On the JavaScript side, `window.cef.listen()` registers a callback for that event name. When the event arrives, the callback receives the deserialized data.

## Rust Side

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

#[derive(Component)]
struct MyWebview;

fn emit_count(
    mut commands: Commands,
    mut count: Local<usize>,
    webviews: Query<Entity, With<MyWebview>>,
) {
    *count += 1;
    commands.trigger(HostEmitEvent::new(
        webviews.single().unwrap(),
        "count",
        &*count,
    ));
}
```

`HostEmitEvent::new()` takes three arguments:

1. **`webview: Entity`** -- the target webview entity. Host Emit must know which webview to send to.
2. **`event_name: &str`** -- the name JavaScript uses in `window.cef.listen()`.
3. **`data: &T`** -- any `Serialize`-implementing value. It is serialized to JSON before being sent to the webview.

You can call this from any system. A common pattern is to run it on a timer or in response to game state changes:

```rust
app.add_systems(Update, emit_count.run_if(on_timer(Duration::from_secs(1))));
```

## JavaScript Side

```html
<!DOCTYPE html>
<html>
<body>
  <p>Count: <span id="count">0</span></p>
  <script>
    window.cef.listen('count', (data) => {
      document.getElementById('count').textContent = data;
    });
  </script>
</body>
</html>
```

`window.cef.listen(eventName, callback)` registers a listener. The `callback` receives the deserialized data directly -- if Bevy sent an integer, JavaScript receives a number; if Bevy sent an object, JavaScript receives an object.

## Multiple Webviews

Since `HostEmitEvent` requires an explicit `Entity`, you can target different webviews independently:

```rust
fn update_webviews(
    mut commands: Commands,
    score_view: Query<Entity, With<ScoreWebview>>,
    chat_view: Query<Entity, With<ChatWebview>>,
) {
    commands.trigger(HostEmitEvent::new(
        score_view.single().unwrap(),
        "score_update",
        &42,
    ));
    commands.trigger(HostEmitEvent::new(
        chat_view.single().unwrap(),
        "new_message",
        &"Hello from Bevy!",
    ));
}
```

## See Also

- [Calling Bevy APIs from JS](./brp.md) -- for async request/response communication
- [Choosing an IPC Pattern](./index.md) -- when to use Host Emit vs other patterns
