---
sidebar_position: 3
---

# JavaScript API

bevy_cef injects a `window.cef` object into every webview. This object provides three methods for communicating with the Bevy application.

## window.cef.emit(eventName, data) {#emit}

Sends a fire-and-forget event from JavaScript to Bevy.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `eventName` | `string` | The event name. Must match the Rust type name (lowercased) registered with `JsEmitEventPlugin<T>`. |
| `data` | `object` | A JSON-serializable object. Its shape must match the fields of the corresponding Rust struct. |

**Behavior:**

The data is serialized to JSON, sent from the render process to the browser process via CEF IPC (`PROCESS_MESSAGE_JS_EMIT`), deserialized into the Rust type `T`, and delivered as a `Receive<T>` EntityEvent on the webview entity that emitted it.

**Event naming convention:** The event name is the Rust struct name converted to lowercase. A struct named `ChatMessage` listens for the event name `"chatmessage"`. This is case-insensitive matching based on the type name.

**Example:**

```html
<script>
  // Rust side: JsEmitEventPlugin::<PlayerAction>::default()
  // Rust struct: #[derive(Deserialize)] struct PlayerAction { action: String, value: f64 }

  document.getElementById('jump').addEventListener('click', () => {
    window.cef.emit('playeraction', {
      action: 'jump',
      value: 1.5
    });
  });
</script>
```

## window.cef.listen(eventName, callback) {#listen}

Registers a listener for events sent from Bevy via `HostEmitEvent`.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `eventName` | `string` | The event name. Must match the `event_name` string passed to `HostEmitEvent::new()` on the Rust side. |
| `callback` | `function(data)` | A function that receives the deserialized data. The data type depends on what Bevy serialized. |

**Callback signature:**

```typescript
(data: any) => void
```

The `data` argument is the JSON-deserialized value of whatever the Rust side serialized. If Bevy sent an integer, `data` is a number. If Bevy sent a struct, `data` is an object with matching field names.

**Behavior:**

When Bevy triggers a `HostEmitEvent` targeting this webview, the data travels from the browser process to the render process via CEF IPC (`PROCESS_MESSAGE_HOST_EMIT`), is deserialized from JSON, and the callback is invoked with the resulting value.

You can register multiple listeners for the same event name. All registered callbacks will be invoked in registration order.

**Example:**

```html
<script>
  // Listen for score updates from Bevy
  window.cef.listen('score_update', (score) => {
    document.getElementById('score').textContent = score;
  });

  // Listen for complex data
  window.cef.listen('player_state', (state) => {
    // state is an object: { health: 100, position: { x: 1.0, y: 2.0 } }
    document.getElementById('health').textContent = state.health;
  });
</script>
```

## window.cef.brp(request) {#brp}

Makes an asynchronous RPC call to Bevy through the Bevy Remote Protocol. Returns a `Promise` that resolves with the response data.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `request` | `object` | An object with `method` and optional `params` fields. |
| `request.method` | `string` | The BRP method name (e.g., `"bevy/query"`, `"bevy/spawn"`). |
| `request.params` | `object` (optional) | Method-specific parameters, passed as-is to the BRP handler. |

**Return value:** `Promise<any>`

The promise resolves with the response data from the BRP handler. If the RPC call fails, the promise rejects with an error.

**Behavior:**

The request is serialized to JSON, sent from the render process to the browser process via CEF IPC (`PROCESS_MESSAGE_BRP`), proxied through Bevy's `RemotePlugin` to the appropriate BRP handler, and the response is sent back to the render process where the V8 promise is resolved.

Because BRP is async, it is the only IPC pattern that supports request/response semantics. Use it when JavaScript needs to read data from the Bevy world or wait for confirmation that an operation completed.

**Example:**

```html
<script>
  // Query entities with a specific component
  async function getPlayers() {
    try {
      const result = await window.cef.brp({
        method: 'bevy/query',
        params: {
          data: {
            components: ['my_game::Player'],
          },
        },
      });
      console.log('Players:', result);
    } catch (error) {
      console.error('BRP call failed:', error);
    }
  }

  // Call a custom BRP method
  async function saveGame() {
    const result = await window.cef.brp({
      method: 'my_game/save',
      params: { slot: 1 },
    });
    console.log('Save result:', result);
  }
</script>
```

## Availability

The `window.cef` object is available after the page's JavaScript context is created. It is safe to use in inline `<script>` tags, `DOMContentLoaded` handlers, and dynamically loaded scripts. It is not available during CEF extension execution (use `CefExtensions` for code that must run before context creation).

If you need to run JavaScript before the page's own scripts, use the `PreloadScripts` component on the Rust side rather than relying on `window.cef` availability timing.
