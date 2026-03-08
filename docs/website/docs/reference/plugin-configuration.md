---
sidebar_position: 2
---

# Plugin Configuration

`CefPlugin` is the single entry point for adding bevy_cef to your application. It accepts three configuration fields and composes all internal sub-plugins automatically.

## CefPlugin Fields

```rust
pub struct CefPlugin {
    pub command_line_config: CommandLineConfig,
    pub extensions: CefExtensions,
    pub root_cache_path: Option<String>,
}
```

### command_line_config

`CommandLineConfig` controls the command-line arguments passed to CEF during initialization. CEF supports a wide range of switches that affect rendering, security, and debugging behavior.

```rust
use bevy_cef::prelude::*;

let plugin = CefPlugin {
    command_line_config: CommandLineConfig::new()
        .arg("--disable-gpu")
        .arg("--remote-debugging-port=9222"),
    ..default()
};
```

Use this to pass CEF-specific flags such as `--disable-gpu`, `--remote-debugging-port`, or `--enable-media-stream`. Refer to the [CEF command-line flags documentation](https://peter.sh/experiments/chromium-command-line-switches/) for available options.

### extensions

`CefExtensions` registers custom JavaScript extensions that are available globally to all webviews. Extensions are JavaScript code that runs in the V8 context of the render process, making them available before any page scripts execute.

```rust
use bevy_cef::prelude::*;

let mut extensions = CefExtensions::default();
extensions.register_extension(
    "my_extension",
    r#"
        var myGlobal = {
            version: '1.0.0',
            greet: function(name) { return 'Hello, ' + name; }
        };
    "#,
);

let plugin = CefPlugin {
    extensions,
    ..default()
};
```

Unlike `PreloadScripts` (which are per-webview and run after the page's context is created), extensions are registered once at the CEF level and are available in every webview. They run in V8's extension context, which means they execute before any page scripts and cannot access the DOM directly.

### root_cache_path

`root_cache_path` sets the root directory where CEF stores its cache data (cookies, localStorage, IndexedDB, and other persistent browser state).

```rust
let plugin = CefPlugin {
    root_cache_path: Some("/path/to/cache".to_string()),
    ..default()
};
```

When set to `None` (the default), CEF uses an in-memory cache that is discarded when the application exits. Set this to a directory path if you need persistent browser state across application sessions.

## Default Configuration

For most use cases, the default configuration is sufficient:

```rust
app.add_plugins((DefaultPlugins, CefPlugin::default()));
```

This initializes CEF with no custom command-line arguments, no extensions, and an in-memory cache.

## Sub-Plugin Tree

`CefPlugin` internally adds the following sub-plugins. You do not add these individually -- they are included automatically:

| Sub-Plugin | Purpose |
|-----------|---------|
| `LocalHostPlugin` | Registers the `cef://localhost/` scheme for serving local assets from Bevy's asset system. |
| `MessageLoopPlugin` | Initializes CEF and calls `cef_do_message_loop_work()` once per frame. |
| `WebviewCoreComponentsPlugin` | Registers core webview components (`WebviewSource`, `WebviewSize`, etc.) with Bevy. |
| `WebviewPlugin` / `MeshWebviewPlugin` | Manages webview lifecycle: creation, texture delivery, material assignment, and DevTools. |
| `IpcPlugin` | Composes `IpcRawEventPlugin` and `HostEmitPlugin` for bidirectional IPC. |
| `KeyboardPlugin` | Forwards keyboard events from Bevy to CEF. |
| `SystemCursorIconPlugin` | Updates the system cursor icon based on the webview's CSS cursor property. |
| `NavigationPlugin` | Registers observers for `RequestGoBack` and `RequestGoForward` events. |
| `ZoomPlugin` | Watches for `ZoomLevel` changes and forwards them to CEF. |
| `AudioMutePlugin` | Watches for `AudioMuted` changes and forwards them to CEF. |
| `RemotePlugin` | Adds Bevy's `RemotePlugin` if not already present, enabling BRP communication. |
