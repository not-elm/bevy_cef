---
sidebar_position: 7
---

# Extensions

CEF extensions let you register custom JavaScript that is available globally in every webview. Unlike preload scripts, extensions are registered once at startup and become part of the V8 JavaScript context itself.

## Registering Extensions

Pass `CefExtensions` to `CefPlugin` during app setup:

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin {
                extensions: CefExtensions::new().add(
                    "myGame",
                    r#"
                    var myGame = {
                        version: "1.0.0",
                        sendScore: function(score) {
                            window.cef.emit('score_update', { score: score });
                        },
                        getPlayerName: function() {
                            return "Player1";
                        }
                    };
                    "#,
                ),
                ..Default::default()
            },
        ))
        .add_systems(Startup, (spawn_camera, spawn_webview))
        .run();
}
```

With this extension registered, every webview can call `myGame.sendScore(100)` or `myGame.getPlayerName()` directly.

## Combining with JS Emit

Extensions work well with the IPC system. Define extension functions that call `window.cef.emit()`, then handle those events in Bevy:

```rust
// In your app setup, register the JsEmitEventPlugin for your event type
app.add_plugins(JsEmitEventPlugin::<ScoreUpdate>::new("score_update"));

// Define the event struct
#[derive(Deserialize)]
struct ScoreUpdate {
    score: u32,
}

// Handle the event with an observer
fn on_score_update(trigger: Trigger<Receive<ScoreUpdate>>) {
    let score = trigger.event().payload.score;
    println!("Score updated: {score}");
}
```

In the browser, calling `myGame.sendScore(42)` fires the `score_update` event, which the Bevy observer receives.

## Global Scope

Extensions are evaluated in every frame's V8 context, iframes included, and apply to **all** webviews. In each context, `window.cef` is defined first, then the extensions run in name order, then [Preload Scripts](./preload-scripts.md), then the page's own scripts. You cannot register different extensions for different webviews. If you need per-webview customization, use [Preload Scripts](./preload-scripts.md) instead.

## Multiple Extensions

Chain `.add()` calls to register multiple extensions:

```rust
CefExtensions::new()
    .add("myGame", r#"var myGame = { /* ... */ };"#)
    .add("analytics", r#"var analytics = { /* ... */ };"#)
```

Extensions run in name order, so `analytics` runs before `myGame` here whatever the order of the `.add()` calls. Adding a second extension with the same name replaces the first.

:::caution

Extension code must be valid JavaScript. An error in an extension is reported to the page console, and from there to the Bevy log, in a message that names `cef://localhost/__extensions__/<name>.js` with the line and column. The other extensions and the page keep running.

:::
