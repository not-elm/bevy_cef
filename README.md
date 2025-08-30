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

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.16"
bevy_cef = "0.1.0"
```

### Platform Requirements

On macOS, using CEF typically requires creating an app bundle.
For development, this library provides a `debug` feature flag.
Once enabled, you can run the app without needing the bundle.

> [!NOTE]
> Use this feature only during development; for releases, bundle the renderer process and the CEF framework inside the
> app.

### Installation debug tools(macOS)

When using `debug`, you need to prepare a separate CEF framework and debug render process.
Please follow the steps below to set it up.

```shell
> cargo install export-cef-dir
> export-cef-dir --force $HOME/.local/share/cef
> cargo install bevy_cef_debug_render_process "$HOME/.local/share/cef/Chromium Embedded Framework.framework/Libraries"
> mv $HOME/.cargo/bin/bevy_cef_debug_render_process $HOME/.cargo/bin/bevy_cef_render_process
```

## Examples

See [`examples/`](./examples).

On macOS, you need to enable `debug` feature enabled:
```shell
cargo run --example simple --features debug
```

## 🌍 Platform Support

| Platform | Status     | Notes                             |
|----------|------------|-----------------------------------|
| macOS    | ✅ Full     | Primary development platform      |
| Windows  | ⚠️ Planned | CEF support ready, testing needed |
| Linux    | ⚠️ Planned | CEF support ready, testing needed |

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

| Bevy | bevy_cef | CEF | Status    |
|------|----------|-----|-----------|
| 0.16 | 0.1.0    | 139 | ✅ Current |

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE2) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Contact

- [Discord](https://discord.com/channels/691052431525675048/1404180578969981018)