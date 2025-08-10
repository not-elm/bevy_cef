# Installation

This guide will help you set up bevy_cef in your Bevy project.

## Prerequisites

### System Requirements

- **macOS**: Currently the primary supported platform
- **Rust**: 1.70 or later
- **Bevy**: 0.16 or later

## Adding bevy_cef to Your Project

### 1. Add the Dependency

Add bevy_cef to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.16"
bevy_cef = { version = "0.1", features = ["debug"] }
```

### 2. Enable the Plugin

Add the `CefPlugin` to your Bevy app:

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin))
        .run();
}
```

## Automatic Setup with Debug Feature

When you enable the `debug` feature (recommended for development), the build system automatically handles everything for you:

✅ **Downloads** and extracts CEF framework to `$HOME/.local/share/cef`  
✅ **Installs** the `bevy_cef_debug_render_process` tool  
✅ **Installs** the `export-cef-dir` tool  
✅ **Configures** all necessary environment variables  

Simply run your project:

```bash
cargo run
```

That's it! No manual installation required.

## Production Builds

For production builds where you want to minimize dependencies, you can omit the debug feature:

```toml
[dependencies]
bevy = "0.16"
bevy_cef = "0.1"  # No debug feature
```

However, you'll need to ensure the CEF framework is available at runtime. The debug feature is recommended for most development scenarios.

## Verification

To verify your installation is working correctly, try running this simple example:

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CefPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Webview
    commands.spawn((
        CefWebviewUri::new("https://bevy.org"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
```

If you see the Bevy website rendered on a plane in your 3D scene, your installation is successful!

## Troubleshooting

### Common Installation Issues

#### Build Fails with Debug Feature
- Ensure you have internet connection for CEF download
- Check available disk space in `$HOME/.local/share/`
- Try cleaning and rebuilding: `cargo clean && cargo build`

#### CEF Framework Issues
- The debug feature handles CEF installation automatically
- If issues persist, delete `$HOME/.local/share/cef` and rebuild

#### Linker Errors on macOS
- Verify Xcode Command Line Tools are installed: `xcode-select --install`
- The debug feature should handle DYLD environment variables automatically

For more troubleshooting information, see the [Troubleshooting](troubleshooting/common-issues.md) section.

## Next Steps

Now that you have bevy_cef installed, check out:

- [Quick Start](quick-start.md) - Build your first webview in minutes
- [Basic Concepts](basic-concepts.md) - Understand the core components
- [Examples](examples/simple.md) - Explore practical examples