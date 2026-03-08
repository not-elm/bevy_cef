---
sidebar_position: 2
---

# Installation

This guide walks you through adding bevy_cef to your Bevy project and setting up the CEF runtime for your platform.

## Prerequisites

- **Rust** (stable toolchain)
- **Bevy 0.18+**

## Add bevy_cef to Your Project

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
bevy_cef = "0.4.0"
```

On macOS, you will also need the `debug` feature enabled during development:

```toml
[dependencies]
bevy_cef = { version = "0.4.0", features = ["debug"] }
```

## Platform Setup

bevy_cef requires the CEF runtime binaries and a render process executable to be installed on your system. The steps differ by platform.

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="macos" label="macOS" default>

### 1. Install the CEF Runtime

Download and install the CEF framework to the default location:

```bash
cargo install export-cef-dir
export-cef-dir --force $HOME/.local/share
```

This places the Chromium Embedded Framework at `$HOME/.local/share/Chromium Embedded Framework.framework`.

### 2. Install the Debug Render Process

During development, install the debug render process binary and copy it into the framework directory:

```bash
cargo install bevy_cef_debug_render_process
cp $HOME/.cargo/bin/bevy_cef_debug_render_process \
   "$HOME/.local/share/Chromium Embedded Framework.framework/Libraries/bevy_cef_debug_render_process"
```

### 3. Use the `debug` Feature

On macOS, always use the `debug` feature when running examples or during development. This links to the local CEF framework:

```bash
cargo run --example simple --features debug
```

</TabItem>
<TabItem value="windows" label="Windows">

### 1. Install the CEF Runtime

Download and install the CEF binaries:

```powershell
cargo install export-cef-dir --force
export-cef-dir --force "$env:USERPROFILE/.local/share/cef"
```

The `build.rs` in `bevy_cef_core` automatically copies all required CEF files (DLLs, PAK resources, locale data) from this directory into your target build directory. No manual file copying is needed after this step.

### 2. Install the Render Process Binary (Recommended)

Install a dedicated render process executable to avoid a brief window flash when CEF launches subprocesses:

```powershell
cargo install bevy_cef_render_process
```

The build script automatically detects and copies this binary to the target directory.

### 3. Run Your Project

On Windows, no special feature flags are needed:

```powershell
cargo run --example simple
```

</TabItem>
</Tabs>

## Verify Your Setup

Run the `simple` example to confirm everything is working:

<Tabs>
<TabItem value="macos" label="macOS" default>

```bash
cargo run --example simple --features debug
```

</TabItem>
<TabItem value="windows" label="Windows">

```powershell
cargo run --example simple
```

</TabItem>
</Tabs>

You should see a Bevy window with a rendered webview. If the webview appears and loads content, your setup is complete.

:::info Subprocess fallback on Windows

If you choose not to install the dedicated render process binary (`bevy_cef_render_process`), you must call `bevy_cef::prelude::early_exit_if_subprocess()` at the very start of your `main()` function, before any Bevy initialization:

```rust
fn main() {
    bevy_cef::prelude::early_exit_if_subprocess();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CefPlugin::default())
        // ...
        .run();
}
```

This prevents CEF subprocesses from re-executing your full application, which would cause a visible window flash on each subprocess launch.

:::
