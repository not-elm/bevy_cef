# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`bevy_cef` is a Bevy plugin integrating the Chromium Embedded Framework (CEF) into Bevy applications, rendering webviews as 3D mesh textures or 2D sprites with full interactivity and bidirectional JS↔Bevy communication.

## Architecture

### Multi-Process Design
- **Browser Process**: Main Bevy app (`bevy_cef_core::browser_process`) — manages CEF initialization, browser instances, input forwarding
- **Render Process**: Separate CEF executable (`bevy_cef_core::render_process`) — handles V8 JavaScript execution, sends events back via IPC
- Communication through CEF process messages with named handlers (`PROCESS_MESSAGE_BRP`, `PROCESS_MESSAGE_HOST_EMIT`, `PROCESS_MESSAGE_JS_EMIT`)

### Plugin Composition
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

### Core Components
- `WebviewSource`: URL spec enum — `WebviewSource::new("url")`, `WebviewSource::local("file.html")`, or `WebviewSource::inline("<h1>Hello</h1>")`; auto-requires `WebviewSize`, `ZoomLevel`, `AudioMuted`, `PreloadScripts`
- `WebviewSize`: Texture resolution (default 800×800), not 3D mesh size
- `WebviewExtendStandardMaterial`: Material for 3D mesh rendering
- `WebviewSpriteMaterial`: Material for 2D sprite rendering
- `HostWindow`: Optional parent window (defaults to PrimaryWindow)
- `ZoomLevel`: f64 zoom (0.0 = default)
- `AudioMuted`: bool audio control
- `PreloadScripts`: Vec<String> scripts executed before page scripts
- `CefExtensions`: Custom JS extensions via `register_extension` (global to all webviews)
- `WebviewTextureTarget`: Headless render target — user-supplied `Handle<Image>` the webview renders into, for sampling from third-party materials without any display component (macOS GPU path only; pair with `WebviewTextureSlot` + `WebviewTargetUiMaterialPlugin` for bind-group rebinds)

### Webview Lifecycle (spans multiple files)
1. User adds `WebviewSource` component → auto-requires `WebviewSize`, `ZoomLevel`, `AudioMuted`, `PreloadScripts`
2. System resolves `WebviewSource` → internal `ResolvedWebviewUri` (lazy, change detection); runtime changes trigger navigation without browser recreation
3. `WebviewPlugin` detects new `ResolvedWebviewUri` → calls `Browsers::create_browser()`
4. CEF renders offscreen → `TextureSender` delivers texture to Bevy
5. `WebviewMaterialPlugin` applies texture to mesh/sprite material
6. User input (mouse/keyboard) → observers → `Browsers` methods forward to CEF

### IPC System
Three communication patterns:
1. **JS Emit** (Webview → Bevy): `JsEmitEventPlugin<E>` where E: `DeserializeOwned + Send + Sync + 'static`
   - JS: `window.cef.emit('event_name', data)` → V8 handler → process message → `IpcEventRaw` channel → deserialize → `Receive<E>` EntityEvent
   - Events wrapped in `Receive<E>` EntityEvent on the webview entity
2. **Host Emit** (Bevy → Webview): Trigger `HostEmitEvent` EntityEvent on webview entity
   - JS: `window.cef.listen('event_name', callback)`
3. **BRP** (Bidirectional RPC): `await window.cef.brp({ method: 'method_name', params: {...} })`
   - Async via V8 promises, proxied through `bevy_remote`

### EntityEvent Pattern
Navigation and DevTools use Bevy's trigger/observer pattern. These require explicit `webview: Entity`:
- `HostEmitEvent`, `RequestGoBack`, `RequestGoForward`, `RequestShowDevTool`, `RequestCloseDevtool`

### Key Non-Obvious Patterns
- **NonSend resources**: `Browsers` and CEF library loaders are `NonSend` — CEF is not thread-safe
- **Message loop**: Uses CEF's `external_message_pump` mode; `cef_do_message_loop_work()` called every Bevy frame in `Main` schedule
- **Pointer interaction**: Custom `WebviewPointer` SystemParam converts screen-space pointer → webview UV via AABB/mesh bounds + camera transforms; alpha channel hit-testing for transparent pixels
- **Localhost scheme**: Static assets via Bevy asset system; inline HTML via `cef://localhost/__inline__/{id}` with auto-cleanup on component remove
- **Secure-by-default switches**: no security-relaxing CEF switches are enabled by default. Users opt into `disable-web-security` etc. via `CommandLineConfig::default().with_switch(switches::DISABLE_WEB_SECURITY)`; opt-in switches are forwarded to all CEF child processes (CORS is enforced in the network process). `CefPlugin::sandbox: SandboxMode` controls the OS sandbox (`PlatformDefault` preserves per-platform behavior; enabling is best-effort and needs platform setup).

## Development Commands

```bash
# Lint and format (runs clippy --fix then cargo fmt)
make fix-lint

# Run examples — macOS requires debug feature, Windows does not
cargo run --example simple --features debug   # macOS
cargo run --example simple                     # Windows

# Install debug render process (macOS)
make install-debug-render-process

# Setup CEF on Windows (installs CEF + render process binary into ~/.local/share/cef)
make setup-windows
```

**Note:** Workspace uses Rust edition 2024.

### Debug Tools Setup (macOS)
```bash
cargo install export-cef-dir --version 145.6.1+145.0.28
export-cef-dir --force $HOME/.local/share/cef
cargo install bevy_cef_debug_render_process
cp $HOME/.cargo/bin/bevy_cef_debug_render_process "$HOME/.local/share/cef/Chromium Embedded Framework.framework/Libraries/bevy_cef_debug_render_process"
```

### Windows Setup
```powershell
cargo install export-cef-dir --force
export-cef-dir --force "$env:USERPROFILE/.local/share/cef"
# Recommended: install dedicated render process to avoid window flash on subprocess launch
cargo install bevy_cef_render_process
```
The `build.rs` in `bevy_cef_core` automatically copies CEF runtime files (DLLs, PAKs, locales) and the render process binary from `$USERPROFILE/.local/share/cef` to the target directory.

If the render process binary is not installed, call `bevy_cef::prelude::early_exit_if_subprocess()` at the start of `main()` before any Bevy initialization to prevent subprocess window flash.

## Features

- `debug`: Enables debug render process (macOS development — auto-links to local CEF framework)
- `serialize`: Enables Bevy's serialization feature

## Testing

No automated tests. Testing done through examples:
- `cargo test --workspace --all-features` (for any future tests)
- Examples: simple, inline_html, js_emit, host_emit, brp, navigation, zoom_level, sprite, devtool, custom_material, custom_scheme, preload_scripts, extensions, headless_texture, hidpi, resize, toolbar_drag, ui_webview

## Workspace Structure

- Root crate `bevy_cef` (`src/`): Public API, plugin composition, user-facing components
- `crates/bevy_cef_core`: Core CEF integration (browser process, render process, IPC internals, V8 bridge)
- `crates/bevy_cef_render_process`: Release render process executable
- `crates/bevy_cef_debug_render_process`: Debug render process executable (development only)
- `crates/bevy_cef_bundle_app`: macOS `.app` bundling tool for release builds

## Platform Notes

- **macOS**: Full support. Uses `objc` crate for window handling. CEF framework at `$HOME/.local/share/cef/Chromium Embedded Framework.framework`. All webviews (mesh + bevy_ui + sprite) render via the GPU `OnAcceleratedPaint` + IOSurface path — a render-world system (`webview_blit`, `RenderGraph` schedule) imports the IOSurface as a Metal texture and blits it into the Bevy texture each frame (no CPU readback; requires the Metal wgpu backend). `root_cache_path` must be set in `CefPlugin` to avoid `cef_initialize` failures from CEF's process-singleton lock. Known limitations: CEF popup widgets (`PET_POPUP`, e.g. `<select>` dropdowns) are not rendered yet, and sprite webviews' transparent regions still block lower pickable entities (sprite picking reads the CPU placeholder).
- **Windows**: Full support. CEF at `$USERPROFILE/.local/share/cef`, auto-copied by build.rs. Separate render process binary recommended
- **Linux**: Supported. CEF at `$HOME/.local/share/cef`, auto-copied by `build.rs`. Run `make setup-linux` to install CEF + `bevy_cef_render_process`. `--no-zygote` is set in the default `CommandLineConfig` to avoid `chrome-sandbox` dependencies (combined with `no_sandbox: true`).

## Version Compatibility

| Bevy   | bevy_cef       | CEF              |
| ------ | -------------- | ---------------- |
| 0.19   | 0.12.0         | 145.6.1+145.0.28 |
| 0.18   | 0.5.0 – 0.11.0 | 145.6.1+145.0.28 |
| 0.16   | 0.1.0          | 139              |
