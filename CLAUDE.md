# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is `bevy_cef`, a Bevy plugin that integrates the Chromium Embedded Framework (CEF) into Bevy applications, allowing webviews to be rendered as 3D objects in the game world or as UI overlays.

## Architecture

### Multi-Process Design
- **Browser Process**: Main application process running Bevy (`bevy_cef_core::browser_process`)
- **Render Process**: Separate CEF render process (`bevy_cef_core::render_process`)
- Communication through IPC channels and Bevy Remote Protocol (BRP)

### Core Components
- `CefWebviewUri`: URL specification (`CefWebviewUri::new("url")` or `CefWebviewUri::local("file.html")`)
- `WebviewSize`: Rendering dimensions (default 800x800), controls texture resolution not 3D size
- `WebviewExtendStandardMaterial`: Primary material for 3D mesh rendering
- `WebviewSpriteMaterial`: Material for 2D sprite rendering
- `HostWindow`: Optional parent window (defaults to PrimaryWindow)
- `ZoomLevel`: f64 zoom control (0.0 = default)
- `AudioMuted`: bool for audio control
- `PreloadScripts`: Vec<String> of scripts to execute before page scripts
- `CefExtensions`: Custom JS extensions via `register_extension` (global to all webviews)

### Plugin Architecture
The main `CefPlugin` accepts `CommandLineConfig` for CEF command-line switches and `CefExtensions` for custom JavaScript APIs. Sub-plugins:
- `LocalHostPlugin`: Serves local assets via `cef://localhost/` scheme
- `MessageLoopPlugin`: CEF message loop integration (macOS uses `CefDoMessageLoopWork()`)
- `WebviewCoreComponentsPlugin`: Core component registration
- `WebviewPlugin`: Webview lifecycle and DevTools
- `IpcPlugin`: IPC containing `IpcRawEventPlugin` and `HostEmitPlugin`
- `KeyboardPlugin`, `SystemCursorIconPlugin`, `NavigationPlugin`, `ZoomPlugin`, `AudioMutePlugin`
- `RemotePlugin`: Auto-added for BRP support if not present

### IPC System
Three communication patterns:
1. **JS Emit**: Webview → Bevy via `JsEmitEventPlugin<E>` where E: `DeserializeOwned + Send + Sync + 'static`
   - Events wrapped in `Receive<E>` EntityEvent
   - JavaScript: `window.cef.emit('event_name', data)`
2. **Host Emit**: Bevy → Webview via `HostEmitEvent` (EntityEvent)
   - JavaScript: `window.cef.listen('event_name', callback)`
3. **BRP**: Bidirectional RPC via `bevy_remote`
   - JavaScript: `await window.cef.brp({ method: 'method_name', params: {...} })`

### EntityEvent Pattern
Navigation and DevTools events are `EntityEvent` types requiring explicit `webview: Entity` field:
- `HostEmitEvent`, `RequestGoBack`, `RequestGoForward`, `RequestShowDevTool`, `RequestCloseDevtool`

## Development Commands

```bash
# Fix and format code
make fix

# Run examples (macOS requires debug feature)
cargo run --example simple --features debug

# Install debug render process tool
make install
```

### Debug Tools Setup (macOS)
Manual installation required before running with `debug` feature:
```bash
cargo install export-cef-dir
export-cef-dir --force $HOME/.local/share
cargo install bevy_cef_debug_render_process
mv $HOME/.cargo/bin/bevy_cef_debug_render_process "$HOME/.local/share/Chromium Embedded Framework.framework/Libraries/bevy_cef_debug_render_process"
```

## Local Asset Loading

Local HTML/assets served via `cef://localhost/` scheme:
- Place assets in `assets/` directory
- Reference as `CefWebviewUri::local("filename.html")`

## Testing

No automated tests. Testing done through examples:
- `cargo test --workspace --all-features` (for any future tests)
- Examples: simple, js_emit, host_emit, brp, navigation, zoom_level, sprite, devtool, custom_material, preload_scripts, extensions

## Platform Notes

- Primary platform: macOS (uses `objc` crate for window handling)
- CEF framework location: `$HOME/.local/share/Chromium Embedded Framework.framework`
- Windows/Linux: Infrastructure ready but needs testing
- Key resources (`Browsers`, library loaders) are `NonSend` - CEF is not thread-safe

## Workspace Structure

- Root crate: `bevy_cef` (public API)
- `crates/bevy_cef_core`: Core CEF integration logic
- `crates/bevy_cef_debug_render_process`: Debug render process executable