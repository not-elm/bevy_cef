---
slug: /
sidebar_position: 1
---

# What is bevy_cef?

**bevy_cef** is a Bevy plugin that integrates the Chromium Embedded Framework (CEF) into Bevy applications, allowing you to render fully interactive webviews as 3D mesh textures or 2D sprites. It provides bidirectional communication between JavaScript running in the webview and your Bevy ECS world, enabling rich UI overlays, in-game browsers, and hybrid rendering workflows.

## Key Features

- **Webviews as 3D textures** -- Render HTML/CSS/JS content onto any Bevy mesh, making web content a first-class part of your 3D scene.
- **2D sprite rendering** -- Use webviews as sprite materials for HUD elements, menus, or full-screen overlays.
- **Bidirectional IPC** -- Three communication patterns between Bevy and JavaScript:
  - **JS Emit**: Webview sends events to Bevy via `window.cef.emit()`.
  - **Host Emit**: Bevy triggers events that JavaScript listens for via `window.cef.listen()`.
  - **BRP**: Async bidirectional RPC using `await window.cef.brp()`, proxied through Bevy Remote Protocol.
- **Local asset loading** -- Serve local HTML, CSS, and JS files through a built-in `cef://localhost/` scheme powered by Bevy's asset system.
- **Inline HTML** -- Render HTML strings directly without external files using `WebviewSource::inline()`.
- **Full input forwarding** -- Keyboard and mouse events are forwarded from Bevy to CEF, supporting text input, clicks, scrolling, and hover states.
- **DevTools support** -- Open Chrome DevTools for any webview during development.
- **Zoom and audio control** -- Per-webview zoom level and audio mute settings.

## Architecture

bevy_cef uses a **multi-process architecture**, mirroring CEF's own design. Your main Bevy application runs as the browser process, managing CEF initialization, browser instances, and input. A separate render process executable handles V8 JavaScript execution and communicates back via IPC. This separation keeps the main application responsive even under heavy JavaScript workloads.

For a deeper look at how the pieces fit together, see the [Concepts](./concepts) page.

## Supported Platforms

| Platform | Status |
|----------|--------|
| macOS    | Fully supported |
| Windows  | Fully supported |
| Linux    | Planned |

## Version Compatibility

| Bevy | bevy_cef | CEF |
|------|----------|-----|
| 0.18+ | 0.4.0-dev | 144.4.0 |
| 0.16 | 0.1.0 | 139 |

## Next Steps

Head to the [Installation](./installation.md) guide to get bevy_cef set up in your project.
