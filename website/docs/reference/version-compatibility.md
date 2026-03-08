---
sidebar_position: 4
---

# Version Compatibility

## Version Table

| Bevy   | bevy_cef  | CEF     |
|--------|-----------|---------|
| 0.18+  | 0.4.0-dev | 144.4.0 |
| 0.16   | 0.1.0     | 139     |

bevy_cef tracks Bevy's release cycle. Each bevy_cef version targets a specific Bevy major version and a specific CEF version. Mixing versions across these boundaries is not supported.

## Feature Flags

### `debug`

Enables the debug render process for **macOS development**. When active, bevy_cef links against the local CEF framework at `$HOME/.local/share/Chromium Embedded Framework.framework` and uses the `bevy_cef_debug_render_process` binary. This allows faster iteration without building a full release bundle.

This feature is macOS-only and should not be enabled for release builds or on Windows.

```bash
cargo run --example simple --features debug   # macOS development
```

### `serialize`

Enables Bevy's serialization feature within bevy_cef. Use this if you need Bevy's `Reflect`-based serialization support for bevy_cef components.

```bash
cargo build --features serialize
```

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS    | Fully supported | CEF framework at `$HOME/.local/share/Chromium Embedded Framework.framework`. Uses `objc` crate for native window integration. |
| Windows  | Fully supported | CEF at `$USERPROFILE/.local/share/cef`. Build script auto-copies DLLs, PAK files, and locales to target directory. Dedicated render process binary recommended to avoid subprocess window flash. |
| Linux    | Planned | Not yet supported. |

### macOS Details

On macOS, the CEF framework is installed as a framework bundle. The `debug` feature flag links to this framework directly for development. For release distribution, use the `bevy_cef_bundle_app` crate to create a proper `.app` bundle with the framework embedded.

Running examples on macOS requires the `debug` feature flag:

```bash
cargo run --example simple --features debug
```

### Windows Details

On Windows, CEF runtime files (DLLs, `.pak` resource files, locale data) are stored at `$USERPROFILE/.local/share/cef`. The `build.rs` script in `bevy_cef_core` automatically copies these files to the target output directory during compilation.

A dedicated render process binary (`bevy_cef_render_process.exe`) is recommended. Without it, the main application executable re-launches itself as a CEF subprocess, which causes a brief window flash. If you cannot install the dedicated binary, call `bevy_cef::prelude::early_exit_if_subprocess()` at the very start of `main()` to exit the subprocess before any Bevy initialization occurs.

Running examples on Windows does not require the `debug` feature:

```bash
cargo run --example simple
```

## Upgrade Notes

### 0.1.0 to 0.4.0-dev

- **Bevy version**: Upgraded from Bevy 0.16 to Bevy 0.18+.
- **CEF version**: Upgraded from CEF 139 to CEF 144.4.0.
- **Windows support**: Added full Windows platform support. See the [Installation](../installation.md) guide for Windows setup instructions.
- **Subprocess strategy**: Added `early_exit_if_subprocess()` for Windows builds without a dedicated render process binary.
- **root_cache_path**: Added `root_cache_path` option to `CefPlugin` for configuring persistent cache storage.
- **`WebviewSource`** (previously `CefWebviewUri`): The component for specifying webview content has been renamed. Update `CefWebviewUri::new()` to `WebviewSource::new()`, `CefWebviewUri::local()` to `WebviewSource::local()`.
