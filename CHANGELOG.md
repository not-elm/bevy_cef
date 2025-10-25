## Unreleased

### Features

- Support Bevy 0.17
- Added `PreloadScripts` component for specifying JavaScript to be executed when the page is initialized.
- Enables GPU when debugging.

### Bug Fixes

- Fixed so that the webview can detect pointers correctly even if it is not the root entity.
- Avoid a crash when updating the cursor icon

### Breaking Changes

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

## v0.1.0

First release
