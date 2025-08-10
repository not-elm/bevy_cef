# Basic Concepts

Understanding these fundamental concepts will help you make the most of bevy_cef in your projects.

## Multi-Process Architecture

bevy_cef follows a multi-process architecture similar to modern web browsers:

### Browser Process
- **Main Application**: Your Bevy game/app runs here
- **CEF Management**: Handles browser creation and management
- **IPC Coordination**: Manages communication with render processes

### Render Process
- **Web Content**: Each webview runs in a separate process
- **Isolation**: Crashes in web content don't affect your main application
- **Sandboxing**: Enhanced security through process separation

### Benefits
- **Stability**: Web content crashes won't crash your game
- **Performance**: CPU-intensive web content won't block your game loop
- **Security**: Sandboxed execution of untrusted web content

## Core Components

bevy_cef provides several key components that work together:

### CefWebviewUri
The primary component that defines what web content to display:

```rust
// Remote web page
CefWebviewUri::new("https://example.com")

// Local HTML file from assets/
CefWebviewUri::local("ui/menu.html")

// Equivalent to above
CefWebviewUri::new("cef://localhost/ui/menu.html")
```

### WebviewSize
Controls the rendering resolution of the webview (not the 3D object size):

```rust
WebviewSize(Vec2::new(1920.0, 1080.0))  // High resolution
WebviewSize(Vec2::new(800.0, 600.0))    // Standard resolution
WebviewSize::default()                   // 800x800 pixels
```

### Material Integration
Webviews integrate with Bevy's material system:

```rust
// Standard material with webview texture
WebviewExtendStandardMaterial::default()

// Custom material properties
WebviewExtendStandardMaterial {
    base: StandardMaterial {
        unlit: true,
        emissive: Color::WHITE.into(),
        ..default()
    },
    ..default()
}
```

## Component Requirements

When you add a `CefWebviewUri` component, bevy_cef automatically requires several other components:

```rust
#[require(WebviewSize, ZoomLevel, AudioMuted)]
pub struct CefWebviewUri(pub String);
```

This means every webview entity automatically gets:
- **WebviewSize**: Default 800x800 resolution  
- **ZoomLevel**: Default zoom (0.0 = browser default)
- **AudioMuted**: Default unmuted (false)

## Rendering Modes

bevy_cef supports different rendering approaches:

### 3D Mesh Rendering
Render web content on 3D objects in your world:

```rust
commands.spawn((
    CefWebviewUri::local("interface.html"),
    Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
));
```

### 2D Sprite Rendering
Render web content as UI overlays:

```rust
commands.spawn((
    CefWebviewUri::local("hud.html"),
    Sprite::default(),
    WebviewSpriteMaterial::default(),
));
```

## Local Asset Serving

bevy_cef includes a built-in web server that serves files from your `assets/` directory:

### Custom Scheme
- **Scheme**: `cef://localhost/`
- **Path Mapping**: `assets/` directory maps to the root
- **Example**: `assets/ui/menu.html` → `cef://localhost/ui/menu.html`

### Supported File Types
- HTML, CSS, JavaScript
- Images (PNG, JPG, SVG, etc.)
- Fonts (WOFF, TTF, etc.)
- JSON, XML, and other data files

### Asset Organization
```
assets/
├── ui/
│   ├── menu.html
│   ├── styles.css
│   └── script.js
├── images/
│   └── logo.png
└── data/
    └── config.json
```

## Inter-Process Communication (IPC)

bevy_cef provides three communication patterns:

### 1. JavaScript to Bevy (JS Emit)
Send events from web content to Bevy systems:

```javascript
// In your HTML/JavaScript
window.cef.emit('player_action', { action: 'jump', power: 10 });
```

```rust
fn main(){
    App::new()
        .add_plugins(JsEmitEventPlugin::<PlayerAction>::default())
    // ...
}

// In your Bevy system
#[derive(Event, Deserialize)]
struct PlayerAction {
    action: String,
    power: i32,
}

fn handle_player_action(trigger: Trigger<PlayerAction>) {
    let action = trigger.event();
    info!("Player action: {} with power {}", action.action, action.power);
}
```

### 2. Bevy to JavaScript (Host Emit)
Send events from Bevy to web content:

```rust
// In your Bevy system
commands.entity(webview_entity).trigger(HostEmitEvent {
    event_name: "score_update".to_string(),
    data: json!({ "score": 1000, "level": 3 }),
});
```

```javascript
// In your HTML/JavaScript
window.cef.listen('score_update', (data) => {
    document.getElementById('score').textContent = data.score;
    document.getElementById('level').textContent = data.level;
});
```

### 3. Bevy Remote Protocol (BRP)

Please see [here](https://gist.github.com/coreh/1baf6f255d7e86e4be29874d00137d1d) for about the Bevy Remote Protocol (BRP).

Bidirectional RPC calls for complex interactions:

```rust
// Register BRP method in Bevy
app.add_plugins(RemotePlugin::default().with_method("get_player_stats", get_stats));
```

```javascript
// Call from JavaScript
const stats = await window.cef.brp({ 
    method: 'get_player_stats',
    params: { playerId: 42 }
});
```

## User Interaction

### Input Handling
Webviews automatically receive keyboard and mouse input when focused:
- **Keyboard**: All keyboard events are forwarded
- **Mouse**: Click, scroll, and hover events work naturally
- **Focus**: Multiple webviews can coexist; input goes to the focused one

### Navigation
Control web navigation programmatically:

```rust
commands.entity(webview).trigger(RequestGoBack);
commands.entity(webview).trigger(ReqeustGoForward);
```

### Zoom Control
Manage zoom levels per webview:

```rust
// Set zoom level (0.0 = default, positive = zoom in, negative = zoom out)
commands.entity(webview).insert(ZoomLevel(1.2));

// Reset to default zoom
commands.entity(webview).insert(ZoomLevel(0.0));
```

## Developer Experience

### Developer Tools
Access Chrome DevTools for debugging:

```rust
// Show developer tools for a webview
commands.entity(webview).trigger(RequestShowDevTool);

// Close developer tools
commands.entity(webview).trigger(RequestCloseDevtool);
```

### Hot Reload
Local assets automatically reload when changed during development.

## Best Practices

### Component Organization
```rust
// Good: Group related components
commands.spawn((
    // The uri convert to `cef://localhost/index.html`
    CefWebviewUri::local("index.html"),
    WebviewSize(Vec2::new(1920.0, 200.0)),
    ZoomLevel(0.8),
    AudioMuted(true),
    Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
    Mesh3d(meshes.add(Quad::new(Vec2::new(4.0, 1.0)))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
));
```

## Next Steps

Now that you understand the basic concepts:

- Explore [Core Components](core-components.md) in detail
- Learn about [Webview Rendering](webview-rendering.md) techniques  
- Dive into [Inter-Process Communication](ipc.md) patterns
- Check out practical [Examples](examples/simple.md)