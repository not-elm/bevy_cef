## v0.8.0

### Features

- Added `can_go_back` / `can_go_forward` public methods to `Browsers` for querying webview navigation history state.

## v0.7.0

### Features

- Added `ResizePlugin` for drag-to-resize of 3D mesh webviews. Users add `WebviewResizable` to a webview entity to enable edge/corner resize handles with configurable edge thickness, min/max size constraints, and aspect-ratio locking (always, never, or shift-key toggle). Cursor icon updates automatically on hover. New `resize` example.
- Added HiDPI / `device_scale_factor` support. Implemented CEF's `RenderHandler::screen_info` so webview textures render at the display's native physical resolution on Retina and Windows-scaled monitors. A new `WebviewDpiPlugin` (included in `CefPlugin`) seeds `WebviewDpr` on every webview from its `HostWindow` at spawn and refreshes it on `WindowScaleFactorChanged`, including multi-window setups. Alpha-channel hit-testing now correctly converts DIP coordinates to physical pixels. New `hidpi` example. (#45)

### Breaking Changes

- `WebviewSize` is now interpreted as **logical pixels (DIP)**, matching CSS viewport semantics. On HiDPI displays the backing texture is allocated at `WebviewSize × DPR` physical pixels — reduce `WebviewSize` if GPU memory matters. `WebviewResizable::min_size`, `max_size`, and `edge_thickness` are likewise DIP now (numeric defaults unchanged). The `WebviewDpr` component has moved from `bevy_cef::resize` to `bevy_cef::common` (still re-exported through the prelude). (#45)

## v0.6.0

### Features

- Added support for CSS `-webkit-app-region` draggable regions. HTML elements with `-webkit-app-region: drag` can now be used to drag-move the host window. CEF's `OnDraggableRegionsChanged` callback feeds a `DraggableRegions` component onto the webview entity; pointer press detection starts window drag, CEF mouse events are suppressed during drag, and hover state is restored after release. New `toolbar_drag` example. (#42)

### Performance

- Throttled CEF `send_external_begin_frame` and `cef_do_message_loop_work` calls. Instead of calling the CEF compositor at Bevy's uncapped frame rate (500+ calls/sec), begin-frame is now gated by a configurable `BeginFrameInterval` resource (default 30 fps) using a Bevy `Timer`, and the message pump enforces a 4 ms minimum interval (~250 pumps/sec). (#39)

## v0.5.4

### Features

- Windows: switched to CEF `multi_threaded_message_loop` so CEF owns its own UI thread. Introduced a `BrowsersProxy` resource and `CefCommand` command-drain architecture; webview systems now use `Res<BrowsersProxy>` on Windows instead of `NonSend<Browsers>`. macOS behavior is unchanged. (#40)

### Performance

- Eliminated the ~2.56 MB per-frame texture buffer clone in the `on_paint` pipeline. `update_webview_image` now borrows `&RenderTextureMessage` and reuses `Image.data` via `copy_from_slice` when dimensions are stable. (#36, #37)
- Replaced the shared `async_channel::unbounded()` with per-webview `Rc<Cell<Option<RenderTextureMessage>>>` slots (separate for View and Popup paint types) for latest-frame-wins semantics with bounded memory. (#37)

## v0.5.3

### Bug Fixes

- Fixed IME composition cancel via BackSpace deleting extra committed characters on Windows. When pressing BackSpace during IME candidate selection, the IME emits `Commit("")` followed by a raw BackSpace key event in the same frame. The BackSpace is now suppressed alongside the existing Enter suppression after IME commit. Also improved empty preedit handling to use `ime_cancel_composition()` instead of `set_ime_composition("")`.

## v0.5.2

### Bug Fixes

- Fixed keyboard event handling for CEF WebViews on Windows:
  - Skip sending CHAR events with NUL character when IME finalizes with `text: None`, which previously caused CEF to suppress the preceding RAWKEYDOWN's DOM keydown dispatch.
  - Use Chromium-format scan codes for `native_key_code` instead of VK codes, fixing empty `KeyboardEvent.code` in JavaScript.

## v0.5.1

### Bug Fixes

- Fixed keyboard input by distinguishing RAWKEYDOWN from CHAR events — non-character keys (F-keys, arrows, modifiers, etc.) now correctly send RAWKEYDOWN, while character keys send CHAR with the proper character code.

## 0.4.1

### Bug Fixes

- Fixed failed localhost asset loads returning a crash instead of a proper error response, and re-enabled CEF signal handlers.
- Hide console window for render process binaries (`bevy_cef_render_process`, `bevy_cef_debug_render_process`) on Windows release builds.

## v0.4.0

### Features

- Added `root_cache_path` option to `CefPlugin` for configurable CEF cache directory.

## v0.3.0

### Features

- Support Windows platform.

## v0.2.1

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
