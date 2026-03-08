---
---

# Choosing an IPC Pattern

bevy_cef provides three communication patterns between Bevy and JavaScript. Each serves a different purpose, and you will often use more than one in the same application.

## Comparison

| Pattern | Direction | Async? | Returns a value? | Use case |
|---------|-----------|--------|-------------------|----------|
| **JS Emit** | Webview to Bevy | No (fire-and-forget) | No | UI events, user actions, form submissions |
| **Host Emit** | Bevy to Webview | No (fire-and-forget) | No | State updates, notifications, pushing data to UI |
| **BRP** | Bidirectional | Yes (Promise-based) | Yes | Querying game state, RPC calls, async operations |

## When to Use Each

### JS Emit -- "Something happened in the UI"

Use JS Emit when the webview needs to notify Bevy about user actions. The webview fires an event and does not wait for a response.

```rust
// Rust: listen for button clicks
#[derive(Deserialize)]
struct ButtonClicked { id: String }

app.add_plugins(JsEmitEventPlugin::<ButtonClicked>::default());
app.add_observer(|trigger: On<Receive<ButtonClicked>>| {
    info!("Button {} was clicked", trigger.id);
});
```

```javascript
// JS: notify Bevy when a button is clicked
document.getElementById('start').addEventListener('click', () => {
    window.cef.emit('button_clicked', { id: 'start' });
});
```

Good for: button clicks, menu selections, form inputs, drag events, any user-initiated action.

### Host Emit -- "Here is new data for the UI"

Use Host Emit when Bevy needs to push state into the webview. The webview passively listens and updates its display.

```rust
// Rust: push score updates to the webview
fn update_score(mut commands: Commands, webview: Query<Entity, With<ScoreUi>>, score: Res<Score>) {
    if score.is_changed() {
        commands.trigger(HostEmitEvent::new(
            webview.single().unwrap(),
            "score",
            &*score,
        ));
    }
}
```

```javascript
// JS: react to score changes
window.cef.listen('score', (score) => {
    document.getElementById('score').textContent = score.value;
});
```

Good for: game state display, health bars, inventory updates, chat messages, any data that flows from the game to the UI.

### BRP -- "I need to ask Bevy something"

Use BRP when JavaScript needs to request data from Bevy and wait for a response. This is the only pattern that returns a value.

```rust
// Rust: expose a method that returns player info
app.add_plugins(
    RemotePlugin::default().with_method("get_player", get_player)
);

fn get_player(In(_): In<Option<serde_json::Value>>) -> BrpResult {
    Ok(serde_json::json!({ "name": "Player 1", "level": 5 }))
}
```

```javascript
// JS: fetch player info on page load
const player = await window.cef.brp({ method: 'get_player', params: 'null' });
document.getElementById('name').textContent = player.name;
```

Good for: initial data loading, querying entity state, performing calculations on the Bevy side, any request/response interaction.

## Combining Patterns

Most applications use multiple patterns together. A typical setup:

- **Host Emit** pushes real-time game state to the webview (health, score, position)
- **JS Emit** sends user actions back to Bevy (button clicks, menu selections)
- **BRP** handles one-off queries when the UI first loads (fetch initial state, list available items)

```rust
app.add_plugins((
    CefPlugin::default(),
    JsEmitEventPlugin::<MenuAction>::default(),
    RemotePlugin::default().with_method("get_inventory", get_inventory),
))
.add_systems(Update, push_health_to_ui)
.add_observer(handle_menu_action);
```

This combination gives you a reactive, bidirectional communication layer without any of the patterns stepping on each other.
