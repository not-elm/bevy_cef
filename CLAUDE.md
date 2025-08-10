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
- `CefWebviewUri`: Component specifying webview URL (remote or local via `cef://localhost/`)
- `WebviewSize`: Controls webview rendering dimensions (default 800x800)
- `WebviewExtendStandardMaterial`: Material for rendering webviews on 3D meshes
- `HostWindow`: Optional parent window specification
- `ZoomLevel`: Webview zoom control
- `AudioMuted`: Audio muting control

### Plugin Architecture
The main `CefPlugin` orchestrates several sub-plugins:
- `LocalHostPlugin`: Serves local assets via custom scheme
- `MessageLoopPlugin`: CEF message loop integration
- `WebviewCoreComponentsPlugin`: Core component registration
- `WebviewPlugin`: Main webview management
- `IpcPlugin`: Inter-process communication
- `KeyboardPlugin`, `NavigationPlugin`, `ZoomPlugin`, `AudioMutePlugin`: Feature-specific functionality

### IPC System
Three communication patterns:
1. **JS Emit**: Webview → Bevy app via `JsEmitEventPlugin<T>`
2. **Host Emit**: Bevy app → Webview via event emission
3. **BRP (Bevy Remote Protocol)**: Bidirectional RPC calls

## Development Commands

### Code Quality
```bash
# Fix and format code
make fix
# Which runs:
# cargo clippy --fix --allow-dirty --allow-staged --workspace --all --all-features
# cargo fmt --all
```

### Development Setup
The build system automatically handles CEF dependencies on macOS with debug feature:
- Installs `bevy_cef_debug_render_process` tool
- Installs `export-cef-dir` tool  
- Downloads/extracts CEF framework to `$HOME/.local/share/cef`

### Manual Installation
```bash
# Install debug render process tool
make install
# Or: cargo install --path ./crates/bevy_cef_debug_render_process --force
```

## Key Examples

- `examples/simple.rs`: Basic webview on 3D plane
- `examples/js_emit.rs`: JavaScript to Bevy communication
- `examples/host_emit.rs`: Bevy to JavaScript communication  
- `examples/brp.rs`: Bidirectional RPC with devtools
- `examples/navigation.rs`: Page navigation controls
- `examples/zoom_level.rs`: Zoom functionality
- `examples/sprite.rs`: Webview as 2D sprite
- `examples/devtool.rs`: Chrome DevTools integration

## Local Asset Loading

Local HTML/assets are served via the custom `cef://localhost/` scheme:
- Place assets in `assets/` directory
- Reference as `CefWebviewUri::local("filename.html")`
- Or manually: `cef://localhost/filename.html`

## Testing

No automated tests are present in this codebase. Testing is done through the example applications.

### Manually Testing

- Run tests with `cargo test --workspace --all-features`

## Platform Notes

- Currently focused on macOS development (see Cargo.toml target-specific dependencies)
- CEF framework must be available at `$HOME/.local/share/cef`
- Uses `objc` crate for macOS-specific window handling
- DYLD environment variables required for CEF library loading

## Workspace Structure

This is a Cargo workspace with:
- Root crate: `bevy_cef` (public API)
- `crates/bevy_cef_core`: Core CEF integration logic
- `crates/bevy_cef_debug_render_process`: Debug render process executable
- `examples/demo`: Standalone demo application