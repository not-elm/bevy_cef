---
sidebar_position: 6
---

# Concepts

This page explains the core ideas behind bevy_cef's design. Understanding these concepts will help you make better decisions when building your application and troubleshoot issues when they arise.

## Architecture {#architecture}

bevy_cef follows CEF's **multi-process architecture**. When your application runs, there are at least two OS processes involved:

- **Browser process** -- your main Bevy application. It manages CEF initialization, creates and owns browser instances, handles input forwarding, and receives rendered textures from CEF's offscreen rendering pipeline.
- **Render process** -- a separate executable that CEF spawns to run V8 JavaScript. This process handles all JavaScript execution, DOM manipulation, and web page rendering. It communicates with the browser process through CEF's internal IPC using named process messages (`PROCESS_MESSAGE_BRP`, `PROCESS_MESSAGE_HOST_EMIT`, `PROCESS_MESSAGE_JS_EMIT`).

This separation exists for two reasons. First, it is a security boundary -- JavaScript runs in an isolated process with limited system access. Second, it keeps the main Bevy application responsive. Heavy JavaScript workloads (complex DOM updates, long-running scripts) execute in the render process without blocking your game loop.

On **macOS**, the render process binary is located inside the CEF framework bundle. On **Windows**, a separate `bevy_cef_render_process.exe` binary is recommended. If no dedicated render process binary is installed, the main application executable re-launches itself as a subprocess, which can cause a brief window flash on Windows. To avoid this, either install the dedicated render process binary or call `early_exit_if_subprocess()` at the start of `main()`.

## Message Loop {#message-loop}

CEF requires a message loop to process internal events (network requests, rendering, IPC). bevy_cef runs CEF in **external message pump** mode, meaning it does not hand control of the main thread over to CEF. Instead, `cef_do_message_loop_work()` is called once per Bevy frame in the `Main` schedule.

This design integrates naturally with Bevy's frame-based execution model. Each Bevy frame, the message loop plugin gives CEF a chance to process pending work -- delivering rendered frames, dispatching IPC messages, handling network responses. The trade-off is that CEF's responsiveness is tied to your frame rate: at 60 FPS, CEF processes events roughly every 16ms, which is sufficient for most use cases.

You do not need to configure the message loop directly. The `MessageLoopPlugin`, included automatically by `CefPlugin`, handles initialization and per-frame pumping.

## EntityEvent Pattern {#entity-event-pattern}

bevy_cef uses Bevy's **trigger/observer** pattern for commands that target a specific webview. Rather than calling methods on a resource, you trigger an `EntityEvent` with an explicit `webview: Entity` field, and bevy_cef's internal observers handle the rest.

This pattern is used for:

- **Navigation**: `RequestGoBack { webview }`, `RequestGoForward { webview }`
- **DevTools**: `RequestShowDevTool { webview }`, `RequestCloseDevtool { webview }`
- **Host Emit**: `HostEmitEvent::new(webview, event_name, &data)`
- **Incoming JS events**: `Receive<T>` fires as an `EntityEvent` on the webview entity

The `webview` field is always the `Entity` of the webview you want to target. This makes multi-webview setups straightforward -- you query for the webview entity you care about and trigger the event on it:

```rust
fn go_back(mut commands: Commands, webviews: Query<Entity, With<MyBrowser>>) {
    let webview = webviews.single().unwrap();
    commands.trigger(RequestGoBack { webview });
}
```

For receiving events, you register an observer. The event fires on the specific webview entity that produced it, so you always know which webview sent the data:

```rust
fn on_message(trigger: On<Receive<ChatMessage>>) {
    info!("Message from webview {:?}: {}", trigger.target(), trigger.text);
}
```

## NonSend Constraints {#nonsend-constraints}

The `Browsers` resource and CEF library loaders are marked as **`NonSend`** in Bevy. This means they can only be accessed from the main thread. CEF's C API is not thread-safe -- its functions must be called from the thread that initialized the library.

In practice, this means:

- Systems that access `Browsers` or other CEF-internal resources cannot run in parallel with other systems on different threads.
- You do not typically interact with `Browsers` directly. bevy_cef's built-in systems and observers handle CEF operations on your behalf, and those systems are already correctly configured to run on the main thread.
- If you are writing advanced integrations that need direct CEF access, your systems must accept `NonSend<Browsers>` or `NonSendMut<Browsers>` and will be scheduled accordingly by Bevy.

## WebviewSize {#webview-size}

`WebviewSize` is a common source of confusion. It controls the **pixel resolution** of the texture that CEF renders into, not the physical size of the mesh in your scene. The default is 800x800 pixels.

- To make a webview appear larger in your 3D scene, change the mesh dimensions or scale the entity's `Transform`.
- To make the rendered content sharper (higher DPI), increase `WebviewSize`. This allocates a larger texture and uses more GPU memory.
- To make the rendered content fit a non-square aspect ratio, set `WebviewSize` to match your desired ratio (e.g., 1920x1080).

The webview's CSS layout uses `WebviewSize` as its viewport dimensions. A `WebviewSize` of 1920x1080 means the web page sees a 1920x1080 viewport, just as if it were displayed in a browser window of that size.

## Pointer Interaction {#pointer-interaction}

bevy_cef translates Bevy pointer input (mouse clicks, movement, scrolling) into CEF browser events so that web content remains interactive. The pipeline works as follows:

1. **Screen-space to UV mapping** -- A custom `WebviewPointer` system parameter takes the screen-space pointer position and projects it onto the webview's mesh using AABB/mesh bounds and camera transforms. This produces UV coordinates (0.0 to 1.0 in each axis) representing where on the texture the pointer is located.
2. **UV to pixel coordinates** -- The UV coordinates are multiplied by the `WebviewSize` to get pixel coordinates within the rendered web page.
3. **Hit testing** -- bevy_cef performs alpha-channel hit testing on the rendered texture. If the pixel under the pointer is fully transparent, the click passes through the webview to whatever is behind it. This enables non-rectangular webview shapes and transparent overlays.
4. **Event forwarding** -- The final pixel coordinates are sent to CEF as mouse events (move, click, scroll), which CEF delivers to the web page's DOM as standard browser events.

This means web content responds to hover states, button clicks, text selection, and scrolling just as it would in a regular browser, even when the webview is rendered onto a 3D mesh at an angle to the camera.

## Plugin Tree {#plugin-tree}

`CefPlugin` is the root plugin that composes all of bevy_cef's functionality from smaller, focused sub-plugins. You add only `CefPlugin` to your app; it includes everything else automatically.

<details>
<summary>Full plugin composition tree</summary>

```
CefPlugin (root — accepts CommandLineConfig, CefExtensions, root_cache_path)
├── LocalHostPlugin (cef://localhost/ scheme for local assets)
├── MessageLoopPlugin (CEF init + per-frame cef_do_message_loop_work())
├── WebviewCoreComponentsPlugin (component registration)
├── WebviewPlugin → MeshWebviewPlugin (lifecycle, materials, DevTools)
├── IpcPlugin (IpcRawEventPlugin + HostEmitPlugin)
├── KeyboardPlugin, SystemCursorIconPlugin, NavigationPlugin
├── ZoomPlugin, AudioMutePlugin
└── RemotePlugin (auto-added for BRP if not present)
```

</details>

Each sub-plugin handles a single concern: `NavigationPlugin` registers observers for `RequestGoBack` and `RequestGoForward`, `ZoomPlugin` watches for `ZoomLevel` changes and forwards them to CEF, and so on. This composition means you always get the full feature set without manual wiring, while the internal code remains modular and maintainable.
