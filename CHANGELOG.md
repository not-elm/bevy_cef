## v0.11.0

### Security

- **Breaking:** bevy_cef no longer enables `disable-web-security`,
  `ignore-certificate-errors`, `ignore-ssl-errors`, or
  `allow-running-insecure-content` by default. Apps that relied on these must opt in
  explicitly, e.g. `CommandLineConfig::default().with_switch(switches::DISABLE_WEB_SECURITY)`.
  User-supplied switches are now forwarded to CEF child processes (so the opt-in
  reaches the renderer/network process), and a startup warning is logged whenever a
  security-relaxing switch is active.
  - The `disable-session-crashed-bubble` switch has been relocated into `CommandLineConfig::default()`,
    so it now applies to both the browser and child processes (previously child-process only);
    this is benign and overridable via `CommandLineConfig`.
- Added `CefPlugin::sandbox: SandboxMode` to control Chromium's OS-level sandbox.
  `SandboxMode::PlatformDefault` (the default) preserves current per-platform behavior.

### Added

- macOS: GPU offscreen rendering via CEF `OnAcceleratedPaint` + IOSurface. The webview is
  imported as a Metal/IOSurface texture and blitted into a Bevy texture inside a custom
  render-graph node (no CPU readback). Supported for 3D mesh (`WebviewExtendStandardMaterial`
  and custom `WebviewExtendedMaterial<E>`), `bevy_ui` (`WebviewUiMaterial`), and 2D `Sprite`
  webviews. The CPU `OnPaint` path is no longer used on macOS.

### Features

- Add `WebviewTitle` component and `TitleChanged` Event trigger to propagate the webview title changes to the ECS.
- Add `WebviewTextureTarget` for headless webviews: a webview with no display component renders into a
  user-supplied `Handle<Image>`, so third-party materials can sample it (e.g. a terminal shader compositing
  an inline webview). On rebind (first frame / resize / handle swap) bevy_cef touches the target image,
  firing `AssetEvent::Modified`; pair `WebviewTextureSlot` + `WebviewTargetUiMaterialPlugin<M>` for turnkey
  bind-group rebinds, or handle the event manually. macOS GPU path only. See the `headless_texture` example.

### Bug Fixes

- macOS: clipboard/editing keyboard shortcuts (⌘C / ⌘X / ⌘V / ⌘A, ⌘Z / ⇧⌘Z) now work in offscreen
  (OSR) webviews. Without a real `NSView`, AppKit never translates these shortcuts into editor commands
  and Blink treats them as command-key system keys it ignores, so forwarding the key event alone did
  nothing. The focused webview's shortcut is now dispatched to `CefFrame::Copy/Cut/Paste/SelectAll/Undo/Redo`
  (in addition to the existing key-event forwarding, so the DOM `keydown` still fires). macOS only;
  Windows/Linux already handle these renderer-side and are unchanged.
- macOS: special keys (Backspace, Delete, arrows, F-keys, navigation keys) now carry the correct
  `NSEvent.characters` value. Backspace was sent as U+0008 (`NSBackspaceCharacter`/Ctrl-H) instead of
  U+007F (`NSDeleteCharacter`), which made Blink delete two characters per press.

### Notes

- macOS: set `CefPlugin { root_cache_path: Some(...), .. }` (CEF's default-cache process-singleton
  otherwise can make `cef_initialize` fail).
- macOS known limitations of the GPU path: CEF popup widgets (`PET_POPUP` — e.g. `<select>`
  dropdown lists) are not rendered yet; on a sprite webview, transparent regions still block
  lower pickable entities (bevy's sprite picking backend reads the CPU placeholder), though
  pointer events on transparent pixels are no longer forwarded to CEF; the GPU path requires
  the Metal wgpu backend (no software-rendering fallback).

## v0.10.0

### Breaking Changes

- macOS (debug builds): the local CEF framework now lives under `$HOME/.local/share/cef/` instead of directly in `$HOME/.local/share/`, matching the Windows/Linux layout. The previously documented setup command `export-cef-dir --force $HOME/.local/share` was destructive: `export-cef-dir --force` renames the target directory to `old_<name>` and then `remove_dir_all`s it, so pointing it at the shared XDG data directory wiped unrelated data (e.g. `~/.local/share/nvim`). The runtime framework loader (`DebugLibraryLoader`), the `debug_chromium_embedded_framework_dir_path` / `debug_render_process_path` helpers, `bevy_cef_bundle_app`'s default framework path, the `Makefile`, and the docs all now use `$HOME/.local/share/cef/Chromium Embedded Framework.framework`. **Migration:** re-run the macOS setup (`make setup-macos`, or `export-cef-dir --force $HOME/.local/share/cef` followed by copying the debug render process into `.../cef/Chromium Embedded Framework.framework/Libraries/`), or move your existing framework into the new `cef/` subdirectory.

## v0.9.1

### Bug Fixes

- macOS: consecutive presses of the same key (e.g. Vimium-style `gg`) now register. The synthesized `KEYUP` was sent with `character` and `unmodified_character` both `0`, so the same `NSFlagsChanged` reclassification swallowed the key release; Blink kept the key logically held and dropped the next same-key `RAWKEYDOWN` as a duplicate, so only the first of any consecutive same-key press produced a `keydown`. The macOS `KEYUP` now carries the character — derived from `logical_key`, since winit leaves `text` empty on release — so releases register and repeated presses dispatch `keydown` again.

## v0.9.0

### Bug Fixes

- macOS (debug builds): fixed the crash-report dialog that appeared when force-terminating an example with Ctrl+C. `-[NSApplication terminate:]` is now swizzled to set an internal flag instead of posting `NSApplicationWillTerminateNotification`; a Bevy system in `Main` reads the flag and emits `AppExit::from_code(130)`, routing through the existing `cef_shutdown` path. This avoids the re-entrancy panic in winit's `applicationWillTerminate:` observer that previously produced SIGABRT. Gated behind `feature = "debug"`; release builds remain vulnerable to the same crash on Cmd-Q / Ctrl+C.
- macOS / non-Windows: in-page DOM `keydown` events now fire. `create_cef_key_events` previously emitted only a `CHAR` event on key press off Windows, which delivered text input but never a DOM `keydown`, so page-side keyboard handlers and shortcuts never ran. A key press now emits a `RAWKEYDOWN` (which drives `keydown`) followed by `CHAR` for character keys, mirroring the native WM_KEYDOWN → WM_CHAR sequence already used on Windows. On macOS the key-down additionally carries `character` / `unmodified_character`, because CEF builds the native `NSEvent` from `native_key_code` + `character` and reclassifies a key event whose `character` and `unmodified_character` are both `0` as `NSFlagsChanged` — which never produces a `keydown`. The `CHAR` text-input path and Windows behavior are unchanged.
- Keyboard and IME input is now delivered only to the explicitly focused webview. When `FocusedWebview` was `None`, `send_key_event` and `ime_event` broadcast input to every browser with a focused CEF frame. Because CEF's `focused_frame()` survives `set_focus(false)`, a webview that had lost focus kept receiving keystrokes and IME composition. Input now goes solely to the focused webview; when focus is on a non-webview surface (e.g. a terminal pane in an embedder), no webview receives input. Applies to both the macOS / non-Windows and Windows paths.

### Features

- Support custom scheme handlers.
- Support Linux.

## v0.8.1

### Features

- Added `LoadHandler` for page load lifecycle events (`OnLoadingStateChange`, `OnLoadStart`, `OnLoadEnd`, `OnLoadError`). New `LoadHandlerBuilder` wires load state into Bevy via `NavigationPlugin`.
- Added `AddressChanged` event to `DisplayHandler`, enabling Bevy systems to react to URL navigation changes.

### Bug Fixes

- Fixed stale V8 callback crashes by using per-context value storage instead of global pointers.
- Added `ctx.enter()` return value check before V8 operations to prevent use-after-free in the render process.
- Resolved clippy warnings in `LoadHandlerBuilder`.

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
