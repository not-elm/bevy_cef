# Introduction to bevy_cef

**bevy_cef** is a powerful Bevy plugin that integrates the Chromium Embedded Framework (CEF) into Bevy applications, enabling you to render web content as 3D objects in your game world or as UI overlays.

## What is bevy_cef?

bevy_cef bridges the gap between modern web technologies and Bevy's 3D engine by:

- **Embedding webviews** as textures on 3D meshes or 2D sprites
- **Supporting bidirectional communication** between JavaScript and Bevy systems
- **Providing a multi-process architecture** for stability and performance
- **Offering local asset serving** through a custom URL scheme
- **Enabling developer tools integration** for debugging web content

## Key Features

### 🌐 Web Content Rendering
- Render any web page as a texture on 3D objects
- Support for HTML5, CSS3, and modern JavaScript
- Local file serving via the `cef://localhost/` scheme
- Remote web page loading with full browser compatibility

### 🔄 Inter-Process Communication (IPC)
- **JS Emit**: Send events from JavaScript to Bevy systems
- **Host Emit**: Send events from Bevy to JavaScript
- **Bevy Remote Protocol (BRP)**: Bidirectional RPC communication

### 🎮 Interactive Controls
- Keyboard input forwarding to webviews
- Mouse interaction support
- Navigation controls (back, forward, refresh)
- Zoom level management
- Audio muting capabilities

### 🔧 Developer Experience
- Chrome DevTools integration for debugging
- Hot-reload support for local assets
- Comprehensive error handling and logging
- Extensive customization options

## Architecture Overview

bevy_cef uses a multi-process architecture similar to modern web browsers:

- **Browser Process**: The main Bevy application process
- **Render Process**: Separate CEF process for web content rendering
- **IPC Communication**: Secure inter-process communication channels

This design ensures stability - if a web page crashes, it won't bring down your entire application.

## Use Cases

### Game UI
Create rich, responsive game interfaces using familiar web technologies:
```rust
commands.spawn((
    CefWebviewUri::local("ui/main-menu.html"),
    // Render as 2D sprite overlay
));
```

### In-World Displays
Embed interactive web content directly in your 3D world:
```rust
commands.spawn((
    CefWebviewUri::new("https://example.com"),
    Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
));
```

### Data Visualization
Display real-time data using web-based charting libraries:
```rust
// Load a local HTML file with Chart.js or D3.js
commands.spawn((
    CefWebviewUri::local("charts/dashboard.html"),
    WebviewSize(Vec2::new(1920.0, 1080.0)),
));
```

### Development Tools
Integrate web-based development and debugging interfaces directly into your game editor or development build.

## Getting Started

Ready to integrate web content into your Bevy application? Check out the [Quick Start](quick-start.md) guide to get up and running in minutes, or dive into [Basic Concepts](basic-concepts.md) to understand the fundamental components and systems.

## Platform Support

Currently, bevy_cef focuses on macOS development with plans for expanded platform support. The plugin automatically handles CEF framework installation and configuration on supported platforms.