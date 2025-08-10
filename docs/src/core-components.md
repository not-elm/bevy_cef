# Core Components

bevy_cef provides several essential components that control webview behavior. Understanding these components is crucial
for effective use of the library.

## Component Overview

| Component       | Purpose                       | Default Value   | Required     |
|-----------------|-------------------------------|-----------------|--------------|
| `CefWebviewUri` | Specifies web content URL     | None            | ✅ Primary    |
| `WebviewSize`   | Controls rendering resolution | 800×800         | ✅ Auto-added |
| `ZoomLevel`     | Controls webview zoom         | 0.0 (default)   | ✅ Auto-added |
| `AudioMuted`    | Controls audio output         | false (unmuted) | ✅ Auto-added |
| `HostWindow`    | Parent window specification   | Primary window  | ❌ Optional   |

## CefWebviewUri

The primary component that defines what web content to display.

### Usage

```rust
use bevy_cef::prelude::*;

// Remote web page
let webview = CefWebviewUri::new("https://example.com");

// Local HTML file from assets/ directory
let webview = CefWebviewUri::local("ui/menu.html");

// Equivalent to local() method
let webview = CefWebviewUri::new("cef://localhost/ui/menu.html");
```

### Implementation Details

```rust
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
#[require(WebviewSize, ZoomLevel, AudioMuted)]
pub struct CefWebviewUri(pub String);
```

The `#[require(...)]` attribute ensures that every webview automatically gets the essential supporting components.

### Methods

- **`new(uri)`**: Create with any valid URL (http, https, cef://localhost/)
- **`local(path)`**: Create with local file path, automatically prefixed with `cef://localhost/`

### Local File Serving

When using local files, bevy_cef serves them through a custom scheme:

- **Scheme**: `cef://localhost/`
- **Root Directory**: Your project's `assets/` folder
- **Path Resolution**: Relative paths from assets/ root

**Example File Structure:**

```
assets/
├── index.html          → cef://localhost/index.html
├── ui/
│   ├── menu.html      → cef://localhost/ui/menu.html
│   └── styles.css     → cef://localhost/ui/styles.css
└── js/
    └── app.js         → cef://localhost/js/app.js
```

## WebviewSize

Controls the internal rendering resolution of the webview, independent of the 3D object size.

### Usage

```rust
use bevy::math::Vec2;

// High resolution webview
WebviewSize(Vec2::new(1920.0, 1080.0))

// Standard resolution
WebviewSize(Vec2::new(800.0, 600.0))

// Square webview
WebviewSize(Vec2::splat(512.0))

// Default size
WebviewSize::default () // 800×800
```

### Performance Considerations

- **Higher Resolution**: Better quality, more memory usage
- **Lower Resolution**: Better performance, potential pixelation
- **Aspect Ratio**: Match your 3D mesh proportions for best results

```rust
// Example: Widescreen webview for cinematic content
commands.spawn((
CefWebviewUri::local("video-player.html"),
WebviewSize(Vec2::new(1920.0, 800.0)),  // 21:9 aspect ratio
Mesh3d(meshes.add(Quad::new(Vec2::new(4.8, 2.0)))), // Match aspect in 3D
MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default ())),
));
```

### Dynamic Resizing

You can change webview size at runtime:

```rust
fn resize_webview(
    mut webviews: Query<&mut WebviewSize>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for mut size in webviews.iter_mut() {
            size.0 *= 1.5; // Increase resolution by 50%
        }
    }
}
```

## ZoomLevel

Controls the zoom level of web content within the webview.

### Usage

```rust
// Default zoom (browser default)
ZoomLevel(0.0)

// Zoom in 20%
ZoomLevel(1.2)

// Zoom out 20%
ZoomLevel(0.8)

// Maximum zoom in
ZoomLevel(3.0)

// Maximum zoom out  
ZoomLevel(0.25)
```

### Zoom Behavior

- **0.0**: Browser default zoom level
- **Positive values**: Zoom in (1.2 = 120% of default)
- **Negative values**: Zoom out (0.8 = 80% of default)
- **Range**: Typically 0.25 to 3.0 (25% to 300%)

### Dynamic Zoom Control

```rust
fn zoom_control(
    mut webviews: Query<&mut ZoomLevel>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for mut zoom in webviews.iter_mut() {
        if keyboard.just_pressed(KeyCode::Equal) {
            zoom.0 = (zoom.0 + 0.1).min(3.0); // Zoom in
        }
        if keyboard.just_pressed(KeyCode::Minus) {
            zoom.0 = (zoom.0 - 0.1).max(0.25); // Zoom out
        }
        if keyboard.just_pressed(KeyCode::Digit0) {
            zoom.0 = 0.0; // Reset zoom
        }
    }
}
```

### Use Cases

- **Accessibility**: Larger text for readability
- **Dense Content**: Fit more information in limited space
- **Responsive Design**: Adapt to different screen sizes
- **User Preference**: Allow users to adjust comfortable viewing size

## AudioMuted

Controls whether audio from the webview is muted.

### Usage

```rust
// Audio enabled (default)
AudioMuted(false)

// Audio muted
AudioMuted(true)
```

### Dynamic Audio Control

```rust
fn toggle_audio(
    mut webviews: Query<&mut AudioMuted>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        for mut muted in webviews.iter_mut() {
            muted.0 = !muted.0; // Toggle mute state
        }
    }
}
```

### Use Cases

- **Background Content**: Mute decorative webviews
- **Multiple Webviews**: Prevent audio conflicts
- **User Control**: Provide mute/unmute functionality
- **Game State**: Mute during pause or cutscenes

## HostWindow (Optional)

Specifies which Bevy window should be the parent of the webview. If not provided, the primary window is used.

### Usage

```rust
// Use primary window (default behavior)
commands.spawn((
CefWebviewUri::local("ui.html"),
// No HostWindow component needed
));

// Specify a particular window
commands.spawn((
CefWebviewUri::local("ui.html"),
HostWindow(secondary_window_entity),
));
```

### Multi-Window Applications

```rust
fn setup_multi_window(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    // Create secondary window
    let secondary_window = commands.spawn(Window {
        title: "Secondary Display".to_string(),
        resolution: (800.0, 600.0).into(),
        ..default()
    }).id();

    // Main webview in primary window
    commands.spawn((
        CefWebviewUri::local("main-ui.html"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));

    // Secondary webview in secondary window
    commands.spawn((
        CefWebviewUri::local("secondary-ui.html"),
        HostWindow(secondary_window),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
```

## Component Combinations

### Common Patterns

**High-Resolution Interactive Display:**

```rust
commands.spawn((
CefWebviewUri::local("dashboard.html"),
WebviewSize(Vec2::new(2560.0, 1440.0)),
ZoomLevel(0.0),
AudioMuted(false),
));
```

**Compact Information Panel:**

```rust
commands.spawn((
CefWebviewUri::local("info-panel.html"),
WebviewSize(Vec2::new(400.0, 300.0)),
ZoomLevel(0.8),
AudioMuted(true),
));
```

**Video Player:**

```rust
commands.spawn((
CefWebviewUri::new("https://player.example.com"),
WebviewSize(Vec2::new(1920.0, 1080.0)),
ZoomLevel(0.0),
AudioMuted(false), // Keep audio for video
));
```

**Background Decoration:**

```rust
commands.spawn((
CefWebviewUri::local("animated-bg.html"),
WebviewSize(Vec2::new(1024.0, 1024.0)),
ZoomLevel(0.0),
AudioMuted(true), // No audio for decoration
));
```

## Component Lifecycle

### Automatic Requirements

When you add `CefWebviewUri`, the required components are automatically added with default values:

```rust
// You only need to specify this:
commands.spawn(CefWebviewUri::local("page.html"));

// But the entity automatically gets:
// - WebviewSize(Vec2::splat(800.0))
// - ZoomLevel(0.0)  
// - AudioMuted(false)
```

### Manual Override

You can override the defaults by adding components explicitly:

```rust
commands.spawn((
CefWebviewUri::local("page.html"),
WebviewSize(Vec2::new(1024.0, 768.0)), // Override default
ZoomLevel(1.2),                        // Override default
AudioMuted(true),                      // Override default
));
```

### Runtime Modification

All components can be modified at runtime through standard Bevy systems:

```rust
fn modify_webview_properties(
    mut query: Query<(&mut WebviewSize, &mut ZoomLevel, &mut AudioMuted)>,
    time: Res<Time>,
) {
    for (mut size, mut zoom, mut muted) in query.iter_mut() {
        // Dynamic effects based on time, input, game state, etc.
        let scale = (time.elapsed_secs().sin() + 1.0) / 2.0;
        zoom.0 = 0.8 + scale * 0.4; // Oscillate between 0.8 and 1.2
    }
}
```

## Best Practices

### Performance Optimization

```rust
// Good: Appropriate resolution for use case
WebviewSize(Vec2::new(800.0, 600.0))   // Standard UI

// Avoid: Excessive resolution unless needed
WebviewSize(Vec2::new(4096.0, 4096.0)) // Only for high-detail content
```

### Memory Management

```rust
// Good: Mute background content
commands.spawn((
CefWebviewUri::local("ambient-display.html"),
AudioMuted(true), // Prevent memory usage for audio processing
));
```

### User Experience

```rust
// Good: Consistent zoom across related webviews
let ui_zoom = ZoomLevel(1.1);
for ui_component in ["menu.html", "inventory.html", "settings.html"] {
commands.spawn((
CefWebviewUri::local(ui_component),
ui_zoom,
));
}
```

## Next Steps

- Learn about [Webview Rendering](webview-rendering.md) techniques
- Explore [Inter-Process Communication](ipc.md) patterns
- Check component-specific guides:
    - [CefWebviewUri Details](core-components/webview-uri.md)
    - [WebviewSize Details](core-components/webview-size.md)
    - [HostWindow Details](core-components/host-window.md)