## Unreleased

### Bug Fixes

- Set `disable_signal_handlers = true` in CEF settings to avoid crashes caused by signal handler conflicts on POSIX systems.

## v0.2.0

### Breaking Changes

- Support Bevy 0.18
- Update CEF version to 144.4.0
- Improve message loop handling
- We can now specify command-line switches when creating the `CefPlugin`.
  - As a result, `CefPlugin` is no longer a unit struct.
- Demo example removed from workspace
- Changed `JsEmitEventPlugin` to use `Receive<E>` wrapper for events
  - Events no longer need to implement the `Event` trait, only `DeserializeOwned + Send + Sync + 'static`
- Changed `HostEmitEvent` to `EntityEvent` with required `webview` field
  - `Default` trait is no longer implemented
- Changed navigation events `RequestGoBack` and `RequestGoForward` to `EntityEvent`
  - Both events now require a `webview: Entity` field
  - `Default` trait is no longer implemented
- Changed DevTools events `RequestShowDevTool` and `RequestCloseDevtool` to `EntityEvent`
  - Both events now require a `webview: Entity` field
  - `Default` trait is no longer implemented
- Remove auto install debug tools
  - Please refer to [README.md](./README.md) and install manually from now on.

### Features

- Added `PreloadScripts` component for specifying JavaScript to be executed when the page is initialized.
- Added `CefExtensions` type for registering custom JavaScript APIs via CEF's `register_extension`
  - Extensions are global and load before any page scripts
  - New `extensions` example demonstrating custom JS APIs
- Refactored `window.cef` API (`brp`, `emit`, `listen`) to be registered as a CEF extension during `on_web_kit_initialized`
  - The API is now available earlier in the page lifecycle

### Bug Fixes

- Fixed so that the webview can detect pointers correctly even if it is not the root entity.
- Avoid a crash when updating the cursor icon
- Fixed IME input not working due to `bevy_winit` not calling `set_ime_allowed()` on initial window creation

## v0.1.0

First release
