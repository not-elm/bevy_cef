# Quick Start

Get up and running with bevy_cef in just a few minutes! This guide will walk you through creating your first webview-enabled Bevy application.

## Create a New Project

Start by creating a new Bevy project:

```bash
cargo new my_webview_app
cd my_webview_app
```

## Add Dependencies

Add bevy_cef to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.16"
bevy_cef = { version = "0.1", features = ["debug"] }
```

## Your First Webview

Replace the contents of `src/main.rs` with:

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
    // Spawn a camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn a light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(1.0, 1.0, 1.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn a webview on a 3D plane
    commands.spawn((
        CefWebviewUri::new("https://bevy.org"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
```

## Run Your Application

```bash
cargo run
```

That's it! You should see the Bevy website rendered on a 3D plane in your application. The first run might take a moment as bevy_cef downloads and sets up the CEF framework automatically.

## What Just Happened?

Let's break down the key components:

### 1. CefPlugin
```rust
.add_plugins((DefaultPlugins, CefPlugin))
```
The `CefPlugin` initializes the CEF framework and sets up all necessary systems for webview rendering.

### 2. CefWebviewUri
```rust
CefWebviewUri::new("https://bevy.org")
```
This component specifies what web content to load. You can use:
- **Remote URLs**: `"https://example.com"`
- **Local files**: `CefWebviewUri::local("index.html")`

### 3. WebviewExtendStandardMaterial
```rust
MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default()))
```
This special material renders the webview content as a texture on your 3D mesh.

## Try Local Content

Create an `assets/` directory and add a simple HTML file:

```bash
mkdir assets
```

Create `assets/hello.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Hello bevy_cef!</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            background: linear-gradient(45deg, #ff6b6b, #4ecdc4);
            color: white;
            text-align: center;
            padding: 50px;
        }
    </style>
</head>
<body>
    <h1>Hello from bevy_cef! 🎮</h1>
    <p>This HTML is rendered directly on a 3D mesh!</p>
    <p>Current time: <span id="time"></span></p>
    
    <script>
        function updateTime() {
            document.getElementById('time').textContent = new Date().toLocaleTimeString();
        }
        setInterval(updateTime, 1000);
        updateTime();
    </script>
</body>
</html>
```

Update your webview URI in `main.rs`:

```rust
CefWebviewUri::local("hello.html"),  // Load local file
```

Run again:

```bash
cargo run
```

You'll see your custom HTML content with a live-updating clock!

## Next Steps

### Explore More Features

- **[JavaScript Communication](ipc/js-emit.md)**: Send data from web pages to Bevy
- **[Host Communication](ipc/host-emit.md)**: Send data from Bevy to web pages  
- **[2D Sprites](webview-rendering/2d-sprite.md)**: Render webviews as UI elements
- **[Developer Tools](developer-tools.md)**: Debug your web content

### Try More Examples

- **Navigation**: Add back/forward buttons
- **Zoom Controls**: Implement zoom in/out functionality
- **Multiple Webviews**: Render different content on multiple objects

### Learn the Architecture

- **[Basic Concepts](basic-concepts.md)**: Understand core components
- **[Multi-Process Design](architecture/multi-process.md)**: How CEF integration works
- **[Plugin System](architecture/plugin-system.md)**: Deep dive into the plugin architecture

## Common First Steps

### Adding Interaction
Make your webview interactive by enabling input:

```rust
// In your setup system, also spawn input handling
commands.spawn((
    CefWebviewUri::local("interactive.html"),
    WebviewSize(Vec2::new(800.0, 600.0)),
    // Webview will automatically receive keyboard and mouse input
    Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
));
```

### Sizing Your Webview
Control the resolution of your webview:

```rust
WebviewSize(Vec2::new(1920.0, 1080.0)),  // High resolution
WebviewSize(Vec2::new(800.0, 600.0)),    // Standard resolution
```

### Multiple Webviews
You can have multiple webviews in the same scene:

```rust
// First webview
commands.spawn((
    CefWebviewUri::new("https://news.ycombinator.com"),
    Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0)),
    Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
));

// Second webview  
commands.spawn((
    CefWebviewUri::local("dashboard.html"),
    Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
    Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
));
```

You're now ready to build amazing applications that blend 3D graphics with modern web technologies!