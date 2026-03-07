# bevy_cef

A powerful Bevy plugin for embedding web content using the Chromium Embedded Framework (CEF).
Render websites, local HTML files, and web applications directly onto 3D meshes or 2D sprites with full interactivity
and bidirectional communication between JavaScript and Bevy.

[![Crates.io](https://img.shields.io/crates/v/bevy_cef)](https://crates.io/crates/bevy_cef)
[![Documentation](https://docs.rs/bevy_cef/badge.svg)](https://docs.rs/bevy_cef)
[![License](https://img.shields.io/badge/license-Apache%202.0%20OR%20MIT-blue.svg)](https://github.com/not-elm/bevy_cef#license)

https://github.com/user-attachments/assets/54f476d0-8eda-4030-a3f6-dc4f2f54209f

## ✨ Features

- **🌐 Full Web Browser Integration** - Embed complete web pages with CSS, JavaScript, and modern web APIs
- **🎮 3D Mesh & 2D Sprite Rendering** - Render web content on any 3D surface or 2D sprite
- **⚡ Interactive Input** - Full mouse, keyboard, and touch input support with automatic event forwarding
- **🔄 Bidirectional Communication** - Seamless data exchange between JavaScript and Bevy systems
- **📁 Local Asset Serving** - Serve local HTML/CSS/JS files with hot reload support
- **🛠️ Developer Tools** - Full Chrome DevTools integration for debugging
- **🎯 Navigation Controls** - Browser history, zoom, audio controls, and more
- **🔒 Multi-Process Architecture** - Secure CEF multi-process design for stability

## Platform Requirements

### Macos

On macOS, using CEF typically requires creating an app bundle.
For development, this library provides a `debug` feature flag.
Once enabled, you can run the app without needing the bundle.

> [!NOTE]
> Use this feature only during development; for releases, bundle the renderer process and the CEF framework inside the
> app.

#### Installation debug tools

When using `debug`, you need to prepare a separate CEF framework and debug render process.
Please follow the steps below to set it up.

```shell
> cargo install export-cef-dir
> export-cef-dir --force $HOME/.local/share
> cargo install bevy_cef_debug_render_process
> cp $HOME/.cargo/bin/bevy_cef_debug_render_process "$HOME/.local/share/Chromium Embedded Framework.framework/Libraries/bevy_cef_debug_render_process"
```

#### Bundling for Release

Install the bundling tools:

```shell
> cargo install bevy_cef_render_process
> cargo install bevy_cef_bundle_app
```

Bundle CEF into your `.app`:

```shell
> bevy_cef_bundle_app --app path/to/YourApp.app --bundle-id-base com.example.yourapp
```

Run `bevy_cef_bundle_app --help` for additional options.

### Windows

On Windows, you need to place CEF libraries in the same directory as the application executable.
Please run the following command to install the CEF framework to local.
When you build the project, the libraries will be automatically copied to the executable's direcotry.

```powershell
>	cargo install export-cef-dir --force
>	export-cef-dir --force "$HOME/.local/share/cef"
```

### Linux

Install system dependencies (Ubuntu/Debian):

```bash
sudo apt-get install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev \
  pkg-config libnss3 libnspr4 libatk1.0-0 libatk-bridge2.0-0 libcups2 \
  libdrm2 libgbm1 libpango-1.0-0 libcairo2 libgtk-3-0 cmake ninja-build
```

Install the CEF framework:

```bash
make setup-linux
```

When you build the project, the CEF runtime files will be automatically copied to the executable's directory.
To run examples, set `LD_LIBRARY_PATH` to point at the directory containing `libcef.so`:

```bash
LD_LIBRARY_PATH=$HOME/.local/share/cef cargo run --example simple
```

> [!NOTE]
> **Known Limitations:**
> - **Sandbox:** Disabled by default. Production use requires a SUID-root `chrome-sandbox` binary.
> - **Wayland:** X11 is fully supported. Wayland is experimental (dialogs/context menus may not position correctly).
> - **WSL2:** May need `CommandLineConfig::default().with_switch("disable-gpu")`.
> - **LD_LIBRARY_PATH:** Required unless rpath is configured in your binary's `build.rs`.

## Examples

See [`examples/`](./examples).

On macOS, you need to enable `debug` feature enabled:

```shell
cargo run --example simple --features debug
```

## 🌍 Platform Support

| Platform | Status     |
| -------- | ---------- |
| macOS    | ✅ Full    |
| Windows  | ✅ Full    |
| Linux    | ✅ Full    |

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **🐛 Bug Reports** - Open an issue with detailed reproduction steps
2. **💡 Feature Requests** - Suggest new features or improvements
3. **🔧 Pull Requests** - Submit bug fixes or new features
4. **📚 Documentation** - Improve docs, examples, or tutorials
5. **🧪 Testing** - Help test on different platforms

### Development Setup

1. Clone the repository
2. Install Rust and Cargo
3. Install the debugging tool with reference to [Installation debug tools](#installation-debug-toolsmacos).
4. Run `cargo build --features debug` to build the project

## Version Compatibility

| Bevy   | bevy_cef | CEF     |
| ------ | -------- | ------- |
| 0.18 ~ | 0.2.0    | 144.4.0 |
| 0.16   | 0.1.0    | 139     |

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE2) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
