---
sidebar_position: 1
---

# Components & Events

This page lists all public components, events, and resources provided by bevy_cef.

## Components

| Name | Type | Description | Related Guide |
|------|------|-------------|---------------|
| `WebviewSource` | Component | Specifies the content to render: a remote URL, local file, or inline HTML. Constructed via `WebviewSource::new()`, `WebviewSource::local()`, or `WebviewSource::inline()`. | [Your First Webview](../getting-started/your-first-webview.md) |
| `WebviewSize` | Component | Pixel resolution of the rendered webview texture. Default is 800x800. Controls the CSS viewport size, not the 3D mesh dimensions. | [Your First Webview](../getting-started/your-first-webview.md) |
| `WebviewExtendStandardMaterial` | Material | Material for rendering webview textures on 3D meshes. Extends Bevy's `StandardMaterial` with webview texture support. Used via `MeshMaterial3d<WebviewExtendStandardMaterial>`. | [Your First Webview](../getting-started/your-first-webview.md) |
| `HostWindow` | Component | Associates a webview with a specific window entity. Defaults to `PrimaryWindow` if not provided. | -- |
| `ZoomLevel` | Component | Controls the zoom level of the webview as an `f64`. `0.0` is the default (100%) zoom. Positive values zoom in, negative values zoom out. | -- |
| `AudioMuted` | Component | Controls whether audio is muted for the webview. A `bool` value. | -- |
| `PreloadScripts` | Component | A `Vec<String>` of JavaScript code that executes before the page's own scripts load. Useful for injecting polyfills or configuration. | -- |

## EntityEvents

| Name | Type | Description | Related Guide |
|------|------|-------------|---------------|
| `Receive<T>` | EntityEvent | Fired on a webview entity when JavaScript calls `window.cef.emit()`. `T` must implement `Deserialize`. Requires `JsEmitEventPlugin::<T>` to be registered. | [Talking to Your Webview](../getting-started/talking-to-your-webview.md) |
| `HostEmitEvent` | EntityEvent | Sends data from Bevy to a webview's JavaScript. Constructed via `HostEmitEvent::new(webview, event_name, &data)`. The `webview` field specifies the target entity. | [Sending Events to Webview](../communication/host-emit.md) |
| `RequestGoBack` | EntityEvent | Navigates the target webview to the previous page in its history. Has a `webview: Entity` field. | [Navigation](../guides/navigation.md) |
| `RequestGoForward` | EntityEvent | Navigates the target webview to the next page in its history. Has a `webview: Entity` field. | [Navigation](../guides/navigation.md) |
| `RequestShowDevTool` | EntityEvent | Opens Chrome DevTools for the target webview. Has a `webview: Entity` field. | -- |
| `RequestCloseDevtool` | EntityEvent | Closes Chrome DevTools for the target webview. Has a `webview: Entity` field. | -- |

## Resources

| Name | Type | Description | Related Guide |
|------|------|-------------|---------------|
| `Browsers` | NonSend Resource | Manages all active CEF browser instances. Used internally by bevy_cef's systems. Not typically accessed directly. | [Concepts](../concepts.md#nonsend-constraints) |

## Plugins

| Name | Description | Related Guide |
|------|-------------|---------------|
| `CefPlugin` | Root plugin that includes all bevy_cef functionality. Accepts `CommandLineConfig`, `CefExtensions`, and `root_cache_path`. | [Plugin Configuration](./plugin-configuration.md) |
| `JsEmitEventPlugin<T>` | Registers a JS Emit event type. Must be added for each `T` you want to receive via `Receive<T>`. | [Talking to Your Webview](../getting-started/talking-to-your-webview.md) |
