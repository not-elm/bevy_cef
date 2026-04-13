# Webview HiDPI Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `bevy_cef` render webviews at the display's native physical resolution on HiDPI monitors (Retina, Windows ≥125%), with live updates on monitor transitions and independent per-window DPR tracking.

**Architecture:** Implement CEF's `RenderHandler::screen_info` callback to report `device_scale_factor`; add a new `WebviewDpr` component auto-inserted on every webview and driven by a pair of Bevy systems (`seed_webview_dpr_system` for initial value + `refresh_on_scale_factor_changed_system` for runtime updates); commit DPR changes to CEF via `Browsers::set_dpr` + `notify_screen_info_changed` + explicit view invalidation. `WebviewSize` semantics change from "physical pixels" to "DIP (logical pixels)" — the physical texture scales with DPR automatically. Resize derive pipeline is simplified to drop its DPR multiplier.

**Tech Stack:** Rust edition 2024, Bevy 0.18 ECS, cef-rs 145.6.1 bindings, CEF offscreen rendering (windowless mode)

**Design Spec:** `docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md`

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `crates/bevy_cef_core/src/browser_process/renderer_handler.rs` | **Modify** | Add `SharedDpr` type alias; add `dpr: SharedDpr` field to `RenderHandlerBuilder`; implement `screen_info` trait method |
| `crates/bevy_cef_core/src/browser_process/browsers.rs` | **Modify** | Add `dpr` field to `WebviewBrowser`; add `initial_dpr: f32` param to `Browsers::create_browser` and `client_handler`; add `Browsers::set_dpr` and `notify_screen_info_changed` methods |
| `crates/bevy_cef_core/src/browser_process/cef_command.rs` | **Modify** | Add `initial_dpr: f32` to `CefCommand::CreateBrowser`; add `SetDpr` and `NotifyScreenInfoChanged` variants; add matching `BrowsersProxy` methods |
| `crates/bevy_cef_core/src/browser_process/cef_thread.rs` | **Modify** | Add `initial_dpr: f32` to `BrowsersCefSide::create_browser`; add `BrowsersCefSide::set_dpr` and `notify_screen_info_changed` methods; extend drain task `execute` match |
| `src/common/components.rs` | **Modify** | Add `WebviewDpr(pub f32)` component (moved from resize, with `PartialEq` added); add to `WebviewSource` auto-require list; register type in `WebviewCoreComponentsPlugin` |
| `src/common/dpi.rs` | **Create** | `WebviewDpiPlugin` + `seed_webview_dpr_system` + `refresh_on_scale_factor_changed_system` + platform-split `commit_webview_dpr_system` + unit tests |
| `src/common/mod.rs` | **Modify** | `pub mod dpi;` + export `WebviewDpiPlugin` |
| `src/resize/components.rs` | **Modify** | Remove `WebviewDpr` (moved); update `WebviewResizable` doc comments to DIP semantics |
| `src/resize/plugin.rs` | **Modify** | Remove DPR acquisition from `init_resizable_system` and `pending_basis_init_system`; remove `× dpr.0` from interactive resize math at line 274 |
| `src/resize/pipeline.rs` | **Modify** | Remove `&WebviewDpr` from `derive_pipeline_system` query; remove DPR from change detection |
| `src/resize/mod.rs` | **Modify** | Drop `dpr: f32` parameter from `derive_webview_size`; update unit tests |
| `src/webview.rs` | **Modify** | Add `WebviewSet::DpiSeed` variant; update `configure_sets` chain; add `&WebviewDpr` to `create_webview` / `create_webview_win` queries and pass as `initial_dpr` |
| `src/system_param/pointer.rs` | **Modify** | Extract pure `dip_to_pixel` helper + unit tests; update `is_transparent_at` to use it |
| `src/lib.rs` | **Modify** | Register `WebviewDpiPlugin` inside `CefPlugin::build` |
| `examples/hidpi.rs` | **Create** | Visual verification example with live DPR display |
| `assets/hidpi_demo.html` | **Create** | Inline HTML for `examples/hidpi.rs` |
| `docs/website/docs/concepts.md` | **Modify** | Rewrite `WebviewSize` section (lines 66-74) for DIP semantics |
| `docs/website/docs/getting-started/your-first-webview.md` | **Modify** | Update caution block (lines 69-73) for DIP semantics |
| `docs/website/docs/guides/sprite-rendering.md` | **Modify** | Update size-control paragraph (lines 59-64) for DIP semantics |
| `CHANGELOG.md` | **Modify** | Add HiDPI entries to `[Unreleased]` section (`Added` + `Changed`) |

---

## Task 1: `SharedDpr` type + `RenderHandlerBuilder` struct extension

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/renderer_handler.rs`

This task adds the thread-safe DPR slot type and adds a `dpr` field to `RenderHandlerBuilder` (without yet implementing `screen_info`). After this task the code still builds but does nothing new — callers must be updated in subsequent tasks.

- [ ] **Step 1: Add `SharedDpr` type alias near `SharedViewSize`**

Open `crates/bevy_cef_core/src/browser_process/renderer_handler.rs`. After line 45 (end of `SharedViewSize` definition), add:

```rust
/// Thread-safe slot for a webview's current `device_scale_factor`.
///
/// Mirrors `SharedViewSize`'s platform split: on non-Windows the CEF UI
/// thread is the Bevy main thread, so no locking is needed; on Windows the
/// CEF UI thread is separate, so an `Arc<Mutex<_>>` is required.
#[cfg(not(target_os = "windows"))]
pub type SharedDpr = std::rc::Rc<std::cell::Cell<f32>>;
#[cfg(target_os = "windows")]
pub type SharedDpr = std::sync::Arc<std::sync::Mutex<f32>>;
```

- [ ] **Step 2: Add `dpr: SharedDpr` field to `RenderHandlerBuilder`**

Locate the struct definition (around line 50). Add `dpr: SharedDpr` as the final field:

```rust
pub struct RenderHandlerBuilder {
    object: *mut RcImpl<sys::cef_render_handler_t, Self>,
    webview: Entity,
    #[cfg(not(target_os = "windows"))]
    view_slot: SharedTexture,
    #[cfg(not(target_os = "windows"))]
    popup_slot: SharedTexture,
    #[cfg(target_os = "windows")]
    texture_sender: TextureSender,
    size: SharedViewSize,
    dpr: SharedDpr,
}
```

- [ ] **Step 3: Update both `build()` signatures and constructors to take `dpr`**

Non-Windows version:

```rust
#[cfg(not(target_os = "windows"))]
pub fn build(
    webview: Entity,
    view_slot: SharedTexture,
    popup_slot: SharedTexture,
    size: SharedViewSize,
    dpr: SharedDpr,
) -> RenderHandler {
    RenderHandler::new(Self {
        object: std::ptr::null_mut(),
        webview,
        view_slot,
        popup_slot,
        size,
        dpr,
    })
}
```

Windows version:

```rust
#[cfg(target_os = "windows")]
pub fn build(
    webview: Entity,
    texture_sender: TextureSender,
    size: SharedViewSize,
    dpr: SharedDpr,
) -> RenderHandler {
    RenderHandler::new(Self {
        object: std::ptr::null_mut(),
        webview,
        texture_sender,
        size,
        dpr,
    })
}
```

- [ ] **Step 4: Update `Clone` impl to clone `dpr`**

Find the `impl Clone for RenderHandlerBuilder` block. Add `dpr: self.dpr.clone()` to the struct literal:

```rust
Self {
    object,
    webview: self.webview,
    #[cfg(not(target_os = "windows"))]
    view_slot: self.view_slot.clone(),
    #[cfg(not(target_os = "windows"))]
    popup_slot: self.popup_slot.clone(),
    #[cfg(target_os = "windows")]
    texture_sender: self.texture_sender.clone(),
    size: self.size.clone(),
    dpr: self.dpr.clone(),
}
```

- [ ] **Step 5: Build — will fail because callers don't yet pass `dpr`**

Run: `cargo check -p bevy_cef_core`
Expected: compile errors at the `RenderHandlerBuilder::build` call site(s) inside `browsers.rs` / `cef_thread.rs` — **"expected N arguments, got M"**. This is intentional; Task 2 wires the callers.

- [ ] **Step 6: Do NOT commit yet** — the workspace does not build. Proceed to Task 2 and commit after Task 3.

---

## Task 2: Add `screen_info` implementation to `RenderHandlerBuilder`

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/renderer_handler.rs`

This task adds the actual CEF callback that reports DPR to Chromium.

- [ ] **Step 1: Add `screen_info` method to the `ImplRenderHandler` impl block**

Locate the `impl ImplRenderHandler for RenderHandlerBuilder` block. It currently has `view_rect`, `on_paint`, and `get_raw`. Add `screen_info` **between** `view_rect` and `on_paint` (keeps call-related methods together):

```rust
fn screen_info(
    &self,
    _browser: Option<&mut Browser>,
    screen_info: Option<&mut cef::ScreenInfo>,
) -> c_int {
    let Some(info) = screen_info else { return 0; };

    #[cfg(not(target_os = "windows"))]
    let dpr = self.dpr.get();
    #[cfg(target_os = "windows")]
    let dpr = *self.dpr.lock().unwrap();

    info.device_scale_factor = dpr;
    info.depth = 24;
    info.depth_per_component = 8;
    info.is_monochrome = 0;
    // `rect` / `available_rect` describe the monitor in virtual-screen coords
    // per CEF (`cef_types.h:1911-1923`), not the view size. For HiDPI quality
    // only `device_scale_factor` matters — leave rects at their defaults.
    // Populating them with real monitor bounds would improve `window.screen.*`
    // JS API accuracy; deferred as a follow-up.
    info.rect = cef::Rect::default();
    info.available_rect = cef::Rect::default();
    1
}
```

- [ ] **Step 2: Add `cef::ScreenInfo` to the existing `use cef::*;` (if not already glob-imported)**

The file currently has `use cef::*;` at the top, so `cef::ScreenInfo` and `cef::Rect` should already be in scope. If a compiler error complains about `cef::ScreenInfo`, add explicit imports: `use cef::{ScreenInfo, Rect};`.

- [ ] **Step 3: Partial build check**

Run: `cargo check -p bevy_cef_core 2>&1 | head -40`
Expected: still fails at the `build()` callsite due to missing `dpr` argument (Task 1's error), but the `screen_info` method itself should compile cleanly. If `screen_info` has its own errors, fix them inline.

- [ ] **Step 4: Do NOT commit yet.** Proceed to Task 3.

---

## Task 3: Wire `SharedDpr` through `Browsers::create_browser` (non-Windows)

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/browsers.rs`

- [ ] **Step 1: Add `dpr` field to `WebviewBrowser` struct**

Locate the struct at line 44-52 and add the field:

```rust
pub struct WebviewBrowser {
    pub client: Browser,
    pub host: BrowserHost,
    pub size: SharedViewSize,
    pub dpr: SharedDpr,
    #[cfg(not(target_os = "windows"))]
    pub view_slot: SharedTexture,
    #[cfg(not(target_os = "windows"))]
    pub popup_slot: SharedTexture,
}
```

- [ ] **Step 2: Add `SharedDpr` to the imports**

At the top of the file, find the renderer_handler import(s). Ensure `SharedDpr` is imported:

```rust
use crate::browser_process::renderer_handler::{
    RenderHandlerBuilder, RenderTextureMessage, SharedDpr, SharedTexture, SharedViewSize,
    TextureSender,
};
```

(Exact path depends on existing imports — add `SharedDpr` to whichever glob or explicit import already covers `SharedViewSize`.)

- [ ] **Step 3: Add `initial_dpr: f32` parameter to `Browsers::create_browser` (non-Windows)**

Find `#[cfg(not(target_os = "windows"))] pub fn create_browser` (around line 62). Insert `initial_dpr: f32` immediately after `webview_size: Vec2`:

```rust
#[cfg(not(target_os = "windows"))]
#[allow(clippy::too_many_arguments)]
pub fn create_browser(
    &mut self,
    webview: Entity,
    uri: &str,
    webview_size: Vec2,
    initial_dpr: f32,
    requester: Requester,
    ipc_event_sender: Sender<IpcEventRaw>,
    brp_sender: Sender<BrpMessage>,
    system_cursor_icon_sender: SystemCursorIconSenderInner,
    drag_regions_sender: DraggableRegionSenderInner,
    initialize_scripts: &[String],
    _window_handle: Option<RawWindowHandle>,
) {
```

- [ ] **Step 4: Create the `SharedDpr` slot inside `create_browser`**

Find the line `let size: SharedViewSize = Rc::new(Cell::new(webview_size));` (around line 76). Add immediately below:

```rust
let size: SharedViewSize = Rc::new(Cell::new(webview_size));
let dpr: SharedDpr = Rc::new(Cell::new(initial_dpr));
```

- [ ] **Step 5: Pass `dpr.clone()` to `client_handler`**

Find the `client_handler(...)` call inside `create_browser`. It currently passes `webview, size.clone(), view_slot.clone(), popup_slot.clone(), ipc_event_sender, ...`. Add `dpr.clone()` after `popup_slot.clone()`:

```rust
Some(&mut self.client_handler(
    webview,
    size.clone(),
    view_slot.clone(),
    popup_slot.clone(),
    dpr.clone(),
    ipc_event_sender,
    brp_sender,
    system_cursor_icon_sender,
    drag_regions_sender,
)),
```

- [ ] **Step 6: Update `client_handler` signature to accept and forward `dpr`**

Find the `#[cfg(not(target_os = "windows"))] fn client_handler` definition (around line 437). Add `dpr: SharedDpr` parameter after `popup_slot: SharedTexture`:

```rust
#[cfg(not(target_os = "windows"))]
#[allow(clippy::too_many_arguments)]
fn client_handler(
    &self,
    webview: Entity,
    size: SharedViewSize,
    view_slot: SharedTexture,
    popup_slot: SharedTexture,
    dpr: SharedDpr,
    ipc_event_sender: Sender<IpcEventRaw>,
    brp_sender: Sender<BrpMessage>,
    system_cursor_icon_sender: SystemCursorIconSenderInner,
    drag_regions_sender: DraggableRegionSenderInner,
) -> Client {
    ClientHandlerBuilder::new(RenderHandlerBuilder::build(
        webview,
        view_slot,
        popup_slot,
        size.clone(),
        dpr,
    ))
    .with_display_handler(DisplayHandlerBuilder::build(system_cursor_icon_sender))
    .with_drag_handler(DragHandlerBuilder::build(webview, drag_regions_sender))
    .with_message_handler(JsEmitEventHandler::new(webview, ipc_event_sender))
    .with_message_handler(BrpHandler::new(brp_sender))
    .build()
}
```

- [ ] **Step 7: Construct `WebviewBrowser` with the `dpr` field**

Find the `WebviewBrowser { ... }` struct literal near the end of `create_browser` (around line 110). Add `dpr`:

```rust
let webview_browser = WebviewBrowser {
    host,
    client: browser,
    size,
    dpr,
    view_slot,
    popup_slot,
};
```

- [ ] **Step 8: Add `Browsers::set_dpr` and `notify_screen_info_changed` methods**

Still in `browsers.rs`, find the existing `pub fn resize(&self, webview: &Entity, size: Vec2)` method (around line 215). Add the two new methods immediately after it:

```rust
/// Update the stored device scale factor for the webview's backing browser.
///
/// Must be called before [`Self::notify_screen_info_changed`] — otherwise
/// CEF re-queries `GetScreenInfo` with the stale value.
pub fn set_dpr(&self, webview: &Entity, dpr: f32) {
    if let Some(browser) = self.browsers.get(webview) {
        #[cfg(not(target_os = "windows"))]
        browser.dpr.set(dpr);
        #[cfg(target_os = "windows")]
        {
            *browser.dpr.lock().unwrap() = dpr;
        }
    }
}

/// Tell CEF to re-query screen info and force an immediate repaint.
///
/// `notify_screen_info_changed` alone only updates Chromium's cached screen
/// metrics; it does not explicitly promise an `OnPaint` (cef_browser.h:710-730).
/// We follow the cefclient OSR convention and pair it with an explicit
/// `invalidate(PET_VIEW)` so the new DPR is visible on the next frame.
pub fn notify_screen_info_changed(&self, webview: &Entity) {
    if let Some(browser) = self.browsers.get(webview) {
        browser.host.notify_screen_info_changed();
        browser.host.invalidate(cef::PaintElementType::VIEW);
    }
}
```

**Note:** `cef::PaintElementType::VIEW` is the expected variant name for cef-rs 145.6.1. If the compiler complains, check the `cef` crate: it may be `PaintElementType::View` (CamelCase). Fix the casing inline and keep going.

- [ ] **Step 9: Import `cef::PaintElementType` if not already**

At the top of `browsers.rs`, the `use cef::{...}` block already imports many types. Add `PaintElementType` to the list.

- [ ] **Step 10: Partial build check**

Run: `cargo check -p bevy_cef_core 2>&1 | head -40`
Expected: non-Windows path builds cleanly. Windows path still fails because `BrowsersCefSide` hasn't been updated. That's Task 4.

- [ ] **Step 11: Do NOT commit yet.** Proceed to Task 4.

---

## Task 4: Update Windows `BrowsersCefSide::create_browser` + new methods

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/cef_thread.rs`

- [ ] **Step 1: Import `SharedDpr`**

At the top of `cef_thread.rs`, find the existing import from `renderer_handler`:

```rust
use crate::browser_process::renderer_handler::{
    RenderHandlerBuilder, RenderTextureMessage, SharedViewSize, TextureSender,
};
```

Add `SharedDpr` to the list:

```rust
use crate::browser_process::renderer_handler::{
    RenderHandlerBuilder, RenderTextureMessage, SharedDpr, SharedViewSize, TextureSender,
};
```

- [ ] **Step 2: Add `initial_dpr: f32` parameter to `BrowsersCefSide::create_browser`**

Find the method (grep for `fn create_browser` inside `impl BrowsersCefSide`). Add `initial_dpr: f32` after `webview_size: Vec2` — matching the order in the non-Windows `Browsers::create_browser`.

- [ ] **Step 3: Create the `SharedDpr` slot inside the Windows path**

Find where `SharedViewSize` is constructed (grep for `Arc::new(Mutex::new(webview_size))`). Add immediately after:

```rust
let size: SharedViewSize = Arc::new(Mutex::new(webview_size));
let dpr: SharedDpr = Arc::new(Mutex::new(initial_dpr));
```

- [ ] **Step 4: Pass `dpr.clone()` to `RenderHandlerBuilder::build`**

Find the `RenderHandlerBuilder::build(webview, texture_sender, size.clone())` call. Add `dpr.clone()`:

```rust
RenderHandlerBuilder::build(
    webview,
    texture_sender.clone(),
    size.clone(),
    dpr.clone(),
)
```

- [ ] **Step 5: Populate the `dpr` field in `WebviewBrowser`**

Find the `WebviewBrowser { ... }` literal. Add `dpr`:

```rust
let webview_browser = WebviewBrowser {
    host,
    client: browser,
    size,
    dpr,
};
```

(On Windows, there are no `view_slot`/`popup_slot` fields because of the cfg gates.)

- [ ] **Step 6: Add `BrowsersCefSide::set_dpr` and `notify_screen_info_changed` methods**

Inside `impl BrowsersCefSide`, add both methods near the existing `resize` method:

```rust
pub fn set_dpr(&self, webview: &Entity, dpr: f32) {
    if let Some(browser) = self.browsers.get(webview) {
        *browser.dpr.lock().unwrap() = dpr;
    }
}

pub fn notify_screen_info_changed(&self, webview: &Entity) {
    if let Some(browser) = self.browsers.get(webview) {
        browser.host.notify_screen_info_changed();
        browser.host.invalidate(cef::PaintElementType::VIEW);
    }
}
```

- [ ] **Step 7: Build check**

Run: `cargo check -p bevy_cef_core 2>&1 | head -40`
Expected: compile errors are now only at the `CefCommand::CreateBrowser` match arm (drain task calls `self.create_browser` without passing `initial_dpr`) — fixed in Task 5.

- [ ] **Step 8: Do NOT commit yet.** Proceed to Task 5.

---

## Task 5: Extend `CefCommand` + `BrowsersProxy` + drain-task match

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/cef_command.rs`
- Modify: `crates/bevy_cef_core/src/browser_process/cef_thread.rs`

- [ ] **Step 1: Add `initial_dpr: f32` to `CefCommand::CreateBrowser`**

In `cef_command.rs`, find the `CreateBrowser { ... }` variant (around line 38-49). Add `initial_dpr: f32` after `webview_size: Vec2`:

```rust
CreateBrowser {
    webview: Entity,
    uri: String,
    webview_size: Vec2,
    initial_dpr: f32,
    requester: Requester,
    ipc_event_sender: Sender<IpcEventRaw>,
    brp_sender: Sender<BrpMessage>,
    system_cursor_icon_sender: SystemCursorIconSenderInner,
    drag_regions_sender: DraggableRegionSenderInner,
    initialize_scripts: Vec<String>,
    window_handle: Option<SendRawWindowHandle>,
},
```

- [ ] **Step 2: Add the two new variants to `CefCommand`**

Add these variants anywhere inside the enum (grouped near `Resize` is natural):

```rust
/// Update the device_scale_factor for a webview's backing CEF browser.
SetDpr { entity: Entity, dpr: f32 },

/// Ask CEF to re-query screen info and force a repaint (paired with SetDpr).
NotifyScreenInfoChanged { entity: Entity },
```

- [ ] **Step 3: Update `BrowsersProxy::create_browser` signature**

Find the method (around line 162). Add `initial_dpr: f32` after `webview_size: Vec2`, both in the signature and in the `CefCommand::CreateBrowser { ... }` struct literal it constructs:

```rust
#[allow(clippy::too_many_arguments, deprecated)]
pub fn create_browser(
    &self,
    webview: Entity,
    uri: &str,
    webview_size: Vec2,
    initial_dpr: f32,
    requester: Requester,
    ipc_event_sender: Sender<IpcEventRaw>,
    brp_sender: Sender<BrpMessage>,
    system_cursor_icon_sender: SystemCursorIconSenderInner,
    drag_regions_sender: DraggableRegionSenderInner,
    initialize_scripts: &[String],
    window_handle: Option<RawWindowHandle>,
) {
    let _ = self.tx.send_blocking(CefCommand::CreateBrowser {
        webview,
        uri: uri.to_owned(),
        webview_size,
        initial_dpr,
        requester,
        ipc_event_sender,
        brp_sender,
        system_cursor_icon_sender,
        drag_regions_sender,
        initialize_scripts: initialize_scripts.to_vec(),
        window_handle: window_handle.map(SendRawWindowHandle),
    });
}
```

- [ ] **Step 4: Add `BrowsersProxy::set_dpr` and `notify_screen_info_changed`**

Add these methods inside `impl BrowsersProxy`, right after the existing `resize` method:

```rust
pub fn set_dpr(&self, entity: &Entity, dpr: f32) {
    let _ = self.tx.send_blocking(CefCommand::SetDpr {
        entity: *entity,
        dpr,
    });
}

pub fn notify_screen_info_changed(&self, entity: &Entity) {
    let _ = self
        .tx
        .send_blocking(CefCommand::NotifyScreenInfoChanged { entity: *entity });
}
```

- [ ] **Step 5: Update drain-task match in `cef_thread.rs`**

Open `cef_thread.rs` and find the `fn execute(&mut self, cmd: CefCommand)` method (around line 73). Update the `CreateBrowser` match arm to destructure and pass `initial_dpr`:

```rust
CefCommand::CreateBrowser {
    webview,
    uri,
    webview_size,
    initial_dpr,
    requester,
    ipc_event_sender,
    brp_sender,
    system_cursor_icon_sender,
    drag_regions_sender,
    initialize_scripts,
    window_handle,
} => {
    #[allow(deprecated)]
    let raw_handle = window_handle.map(|h| h.0);
    self.create_browser(
        webview,
        &uri,
        webview_size,
        initial_dpr,
        requester,
        ipc_event_sender,
        brp_sender,
        system_cursor_icon_sender,
        drag_regions_sender,
        &initialize_scripts,
        raw_handle,
    );
}
```

- [ ] **Step 6: Add match arms for the two new variants**

Add immediately after the `CefCommand::Resize { entity, size } => self.resize(&entity, size),` line:

```rust
CefCommand::SetDpr { entity, dpr } => self.set_dpr(&entity, dpr),
CefCommand::NotifyScreenInfoChanged { entity } => {
    self.notify_screen_info_changed(&entity)
}
```

- [ ] **Step 7: Full build check**

Run: `cargo check --workspace`
Expected: clean build on the current platform. If on Windows, it may still fail at `create_webview_win` in `src/webview.rs` because that call site hasn't been updated to pass `initial_dpr` yet — that's Task 12. For now, a temporary workaround is to pass `1.0` as a placeholder at the call site just to see the rest of the crate compile; we'll wire the real value in Task 12.

Actually don't make temporary fixes. Simply accept that the build breaks at `create_webview` / `create_webview_win` — that's the exact boundary between Tasks 5 and 12.

- [ ] **Step 8: First commit — CEF core HiDPI plumbing**

At this point everything in `bevy_cef_core` compiles, but the top-level `bevy_cef` crate does not (because `create_webview` hasn't been updated). We'll commit this first chunk because it's logically complete — the CEF core crate is self-consistent.

```bash
cd crates/bevy_cef_core
cargo check 2>&1 | tail -5  # confirm bevy_cef_core builds standalone
cd ../..
git add crates/bevy_cef_core/src/browser_process/renderer_handler.rs \
        crates/bevy_cef_core/src/browser_process/browsers.rs \
        crates/bevy_cef_core/src/browser_process/cef_command.rs \
        crates/bevy_cef_core/src/browser_process/cef_thread.rs
git commit -m "feat(core): add CEF screen_info + SharedDpr plumbing

Implement ImplRenderHandler::screen_info returning device_scale_factor
from a new SharedDpr slot, add Browsers::set_dpr and
notify_screen_info_changed methods, extend CefCommand with SetDpr and
NotifyScreenInfoChanged variants, and thread initial_dpr through
create_browser. No behavioral change until the Bevy side is wired in
a follow-up commit.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 6: Move `WebviewDpr` to `common` with `PartialEq`

**Files:**
- Modify: `src/common/components.rs`
- Modify: `src/resize/components.rs`

- [ ] **Step 1: Remove `WebviewDpr` from `src/resize/components.rs`**

Open `src/resize/components.rs` and delete lines 75-85 (the entire `WebviewDpr` component definition and its `Default` impl). Also remove any re-exports mentioning `WebviewDpr`.

Expected remaining imports in that file: only those needed by `DisplaySize`, `BaseRenderScale`, `QualityMultiplier`, `WebviewBasis2d`, `PendingBasisInit`, `WebviewResizable`, `AspectLockMode`.

- [ ] **Step 2: Add `WebviewDpr` to `src/common/components.rs`**

Open `src/common/components.rs`. Immediately after the `WebviewSize` definition (around line 79, after its `Default` impl), add:

```rust
/// Device pixel ratio (DPR) for the webview's backing CEF render buffer.
///
/// Automatically set and kept up-to-date by `WebviewDpiPlugin`: seeded from
/// the host window's `scale_factor()` at spawn, refreshed on
/// `WindowScaleFactorChanged`. User code normally does not need to write this
/// component, but may override it (e.g. to force 2× rendering for screenshots).
///
/// `WebviewSize` is interpreted in logical pixels (DIP). The actual GPU
/// texture CEF allocates is `WebviewSize × WebviewDpr` physical pixels.
#[derive(Reflect, Component, Debug, Copy, Clone, PartialEq, Deref, DerefMut)]
#[reflect(Component, Debug, Default)]
pub struct WebviewDpr(pub f32);

impl Default for WebviewDpr {
    fn default() -> Self {
        Self(1.0)
    }
}
```

- [ ] **Step 3: Register `WebviewDpr` in `WebviewCoreComponentsPlugin`**

Still in `src/common/components.rs`, find the plugin at the top (lines 5-16). Add a `.register_type::<WebviewDpr>()` call:

```rust
impl Plugin for WebviewCoreComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WebviewSize>()
            .register_type::<WebviewSource>()
            .register_type::<HostWindow>()
            .register_type::<ZoomLevel>()
            .register_type::<AudioMuted>()
            .register_type::<PreloadScripts>()
            .register_type::<WebviewDpr>();
    }
}
```

- [ ] **Step 4: Find and fix all remaining references to `WebviewDpr` in the resize module**

Run: `grep -rn WebviewDpr src/resize/ 2>&1 | head -20`
(Use the Grep tool, not bash.) Expected hits are in `plugin.rs` and `pipeline.rs`. These will be removed entirely in Tasks 14-15, but for now just add an import at the top of each file so the module compiles:

```rust
use crate::common::WebviewDpr;
```

Delete the old `use crate::resize::components::WebviewDpr;` (or `use super::components::WebviewDpr;`) line if present.

- [ ] **Step 5: Update `src/resize/mod.rs` re-exports (if any)**

If `resize/mod.rs` has `pub use self::components::{..., WebviewDpr, ...};`, remove `WebviewDpr` from the list.

- [ ] **Step 6: Update `src/common/mod.rs` to re-export `WebviewDpr`**

Open `src/common/mod.rs`. Find where `WebviewSize` is re-exported (usually a `pub use components::{...}` line). Add `WebviewDpr`:

```rust
pub use components::{
    AudioMuted, HostWindow, PreloadScripts, WebviewCoreComponentsPlugin, WebviewDpr,
    WebviewSize, WebviewSource, ZoomLevel,
};
```

(Exact list depends on what's currently there — just add `WebviewDpr` to it.)

- [ ] **Step 7: Build check**

Run: `cargo check --workspace 2>&1 | tail -30`
Expected: the `WebviewDpr` move is clean. Remaining errors (if any) are from `create_webview` still not passing `initial_dpr` — that's Task 12.

- [ ] **Step 8: Commit — `WebviewDpr` component move**

```bash
git add src/common/components.rs src/common/mod.rs src/resize/components.rs \
        src/resize/plugin.rs src/resize/pipeline.rs src/resize/mod.rs
git commit -m "refactor(common): move WebviewDpr to common module

Promote WebviewDpr from a resize-specific component to a shared one
used by all webviews. Add PartialEq derive (required by set_if_neq
in the upcoming WebviewDpiPlugin). Register the type for reflection.
The resize module still imports WebviewDpr from common during this
transition; the dependency will be removed once the DPI pipeline is
in place.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 7: Add `WebviewDpr` to `WebviewSource` auto-require

**Files:**
- Modify: `src/common/components.rs`

- [ ] **Step 1: Extend the `#[require(...)]` attribute**

Open `src/common/components.rs` and find `WebviewSource` (around line 25-28). Update the `#[require(...)]` list to include `WebviewDpr`:

```rust
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Debug)]
#[require(WebviewSize, ZoomLevel, AudioMuted, PreloadScripts, WebviewDpr)]
pub enum WebviewSource {
```

- [ ] **Step 2: Build check**

Run: `cargo check --workspace 2>&1 | tail -20`
Expected: same errors as Task 6 step 7 (only the `create_webview` mismatch). The require change compiles cleanly.

- [ ] **Step 3: Commit**

```bash
git add src/common/components.rs
git commit -m "feat(common): auto-require WebviewDpr on every webview

WebviewSource now requires WebviewDpr, so every spawned webview gets
a default-valued component (1.0) that the upcoming WebviewDpiPlugin
will overwrite with the host window's real scale factor on the same
frame.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 8: Add `WebviewSet::DpiSeed` system set

**Files:**
- Modify: `src/webview.rs`

- [ ] **Step 1: Add `DpiSeed` variant to `WebviewSet`**

Open `src/webview.rs`. Find the `WebviewSet` enum (around line 102-111). Add `DpiSeed` between `ResizeInteraction` and `DerivePipeline`:

```rust
#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub enum WebviewSet {
    /// Resize drag tracking writes DisplaySize.
    ResizeInteraction,
    /// Seeds and refreshes WebviewDpr from host window scale factors.
    DpiSeed,
    /// Derives WebviewSize from pipeline components.
    DerivePipeline,
    /// Creates CEF browser instances.
    CreateBrowser,
    /// Commits WebviewSize changes to CEF via browsers.resize().
    CommitResize,
}
```

- [ ] **Step 2: Update `configure_sets` chain**

Find `app.configure_sets(Update, (...).chain())` (around line 119-128). Add `DpiSeed` to the chain in the right position:

```rust
app.configure_sets(
    Update,
    (
        WebviewSet::ResizeInteraction,
        WebviewSet::DpiSeed,
        WebviewSet::DerivePipeline,
        WebviewSet::CreateBrowser,
        WebviewSet::CommitResize,
    )
        .chain(),
);
```

- [ ] **Step 3: Build check**

Run: `cargo check --workspace 2>&1 | tail -15`
Expected: same `create_webview` errors from before. The set addition is clean.

- [ ] **Step 4: Commit**

```bash
git add src/webview.rs
git commit -m "feat(webview): add WebviewSet::DpiSeed between resize and derive

New system set slot for DPR seed/refresh systems, ordered before
DerivePipeline (and therefore before CreateBrowser) so the initial
WebviewDpr is written on the same frame as browser creation.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 9: Create `src/common/dpi.rs` with `seed_webview_dpr_system` (TDD)

**Files:**
- Create: `src/common/dpi.rs`
- Modify: `src/common/mod.rs`

This task uses TDD for the seed system. Bevy `App::new().update()` gives us a real ECS world to test against, so we can write actual unit tests for the seed behavior.

- [ ] **Step 1: Create `src/common/dpi.rs` with the skeleton and one failing test**

Create file `src/common/dpi.rs`:

```rust
//! DPI / device-scale-factor tracking for webviews.
//!
//! `WebviewDpiPlugin` maintains each webview's `WebviewDpr` component,
//! seeding it from the host window at spawn and refreshing it when the
//! host window's `scale_factor` changes (monitor move, OS DPI setting).
//! The change is then committed to CEF via `notify_screen_info_changed`.

use crate::common::{HostWindow, WebviewDpr, WebviewSource};
use crate::webview::WebviewSet;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowScaleFactorChanged};

pub struct WebviewDpiPlugin;

impl Plugin for WebviewDpiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                seed_webview_dpr_system,
                refresh_on_scale_factor_changed_system,
            )
                .in_set(WebviewSet::DpiSeed),
        );

        #[cfg(not(target_os = "windows"))]
        app.add_systems(
            Update,
            commit_webview_dpr_system.in_set(WebviewSet::CommitResize),
        );

        #[cfg(target_os = "windows")]
        app.add_systems(
            Update,
            commit_webview_dpr_system_win.in_set(WebviewSet::CommitResize),
        );
    }
}

fn seed_webview_dpr_system(
    mut webviews: Query<
        (&mut WebviewDpr, Option<&HostWindow>),
        Added<WebviewSource>,
    >,
    windows: Query<&Window>,
    primary: Query<&Window, With<PrimaryWindow>>,
) {
    for (mut dpr, host) in webviews.iter_mut() {
        let resolved = host
            .and_then(|hw| windows.get(hw.0).ok())
            .or_else(|| primary.single().ok())
            .map(|w| w.scale_factor())
            .unwrap_or_else(|| {
                warn!("No window found when seeding WebviewDpr; falling back to 1.0");
                1.0
            });
        dpr.0 = resolved;
    }
}

// TODO: implement in Task 10
fn refresh_on_scale_factor_changed_system() {}

// TODO: implement in Task 11
#[cfg(not(target_os = "windows"))]
fn commit_webview_dpr_system() {}

#[cfg(target_os = "windows")]
fn commit_webview_dpr_system_win() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{WebviewDpr, WebviewSource};

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::window::WindowPlugin::default())
            .add_systems(Update, seed_webview_dpr_system);
        app
    }

    #[test]
    fn seed_falls_back_to_1_0_when_no_windows_exist() {
        let mut app = make_app();
        // Minimal WebviewSource spawn (no WebviewSize auto-require in this
        // cut-down test App — insert WebviewDpr manually to simulate the
        // auto-require default).
        let entity = app
            .world_mut()
            .spawn((
                WebviewSource::new("https://example.com"),
                WebviewDpr(1.0),
            ))
            .id();
        app.update();
        let dpr = app.world().get::<WebviewDpr>(entity).unwrap();
        assert_eq!(dpr.0, 1.0);
    }
}
```

- [ ] **Step 2: Register the `dpi` module in `src/common/mod.rs`**

Open `src/common/mod.rs`. Add `pub mod dpi;` near the other module declarations. Add `WebviewDpiPlugin` to the re-export list:

```rust
pub mod dpi;
// ... other modules ...
pub use dpi::WebviewDpiPlugin;
```

- [ ] **Step 3: Run the new test**

Run: `cargo test --workspace -p bevy_cef seed_falls_back 2>&1 | tail -20`
Expected: PASS. (The `warn!` about no windows will print but the fallback to `1.0` is correct.)

If the test fails to compile because `WebviewSource::new` requires more auto-required components than we inserted, that's fine — the test App uses `MinimalPlugins` which doesn't auto-require, so manual component insertion works.

- [ ] **Step 4: Add a second test — with PrimaryWindow present**

Add to the same `#[cfg(test)] mod tests` block:

```rust
#[test]
fn seed_uses_primary_window_scale_factor_when_no_host_window() {
    let mut app = make_app();
    // Spawn a fake PrimaryWindow with scale_factor = 2.0
    let window = Window {
        resolution: bevy::window::WindowResolution::new(800.0, 600.0)
            .with_scale_factor_override(2.0),
        ..default()
    };
    app.world_mut().spawn((window, PrimaryWindow));

    let entity = app
        .world_mut()
        .spawn((
            WebviewSource::new("https://example.com"),
            WebviewDpr(1.0),
        ))
        .id();
    app.update();
    let dpr = app.world().get::<WebviewDpr>(entity).unwrap();
    assert_eq!(dpr.0, 2.0);
}
```

- [ ] **Step 5: Run the new test**

Run: `cargo test --workspace -p bevy_cef seed_uses_primary 2>&1 | tail -20`
Expected: PASS. (If `with_scale_factor_override` is the wrong API name in Bevy 0.18, check `bevy_window::WindowResolution` and adjust — the equivalent constructor sets `scale_factor`.)

- [ ] **Step 6: Commit**

```bash
git add src/common/dpi.rs src/common/mod.rs
git commit -m "feat(common): add WebviewDpiPlugin skeleton + seed system

New src/common/dpi.rs implements seed_webview_dpr_system which
populates WebviewDpr on newly spawned webviews from their HostWindow
(or PrimaryWindow fallback). Refresh and commit systems are stubbed
pending Tasks 10-11. Covered by two unit tests.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 10: Implement `refresh_on_scale_factor_changed_system`

**Files:**
- Modify: `src/common/dpi.rs`

- [ ] **Step 1: Replace the stub with the real implementation**

In `src/common/dpi.rs`, replace the `fn refresh_on_scale_factor_changed_system() {}` stub with:

```rust
fn refresh_on_scale_factor_changed_system(
    mut er: MessageReader<WindowScaleFactorChanged>,
    mut webviews: Query<(&mut WebviewDpr, Option<&HostWindow>), With<WebviewSource>>,
    primary: Query<Entity, With<PrimaryWindow>>,
) {
    for event in er.read() {
        let changed_window = event.window;
        let primary_entity = primary.single().ok();
        for (mut dpr, host) in webviews.iter_mut() {
            let target = host.map(|h| h.0).or(primary_entity);
            if target == Some(changed_window) {
                dpr.set_if_neq(WebviewDpr(event.scale_factor as f32));
            }
        }
    }
}
```

- [ ] **Step 2: Build check**

Run: `cargo check --workspace 2>&1 | tail -15`
Expected: cleanly compiles (apart from the ongoing `create_webview` mismatch, which is Task 12's responsibility).

- [ ] **Step 3: Add a unit test for the refresh path**

In the `#[cfg(test)] mod tests` block, add:

```rust
#[test]
fn refresh_updates_only_webviews_on_the_changed_window() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::window::WindowPlugin::default())
        .add_message::<WindowScaleFactorChanged>()
        .add_systems(Update, refresh_on_scale_factor_changed_system);

    // Two fake windows with different scale factors
    let win_a = app
        .world_mut()
        .spawn(Window {
            resolution: bevy::window::WindowResolution::new(800.0, 600.0)
                .with_scale_factor_override(1.0),
            ..default()
        })
        .id();
    let win_b = app
        .world_mut()
        .spawn(Window {
            resolution: bevy::window::WindowResolution::new(800.0, 600.0)
                .with_scale_factor_override(2.0),
            ..default()
        })
        .id();

    let wv_a = app
        .world_mut()
        .spawn((
            WebviewSource::new("https://a.example"),
            WebviewDpr(1.0),
            HostWindow(win_a),
        ))
        .id();
    let wv_b = app
        .world_mut()
        .spawn((
            WebviewSource::new("https://b.example"),
            WebviewDpr(1.0),
            HostWindow(win_b),
        ))
        .id();

    // Fire a ScaleFactorChanged event for window B only
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<WindowScaleFactorChanged>>()
        .send(WindowScaleFactorChanged {
            window: win_b,
            scale_factor: 2.0,
        });

    app.update();

    let dpr_a = app.world().get::<WebviewDpr>(wv_a).unwrap();
    let dpr_b = app.world().get::<WebviewDpr>(wv_b).unwrap();
    assert_eq!(dpr_a.0, 1.0, "webview A should be untouched");
    assert_eq!(dpr_b.0, 2.0, "webview B should be refreshed");
}
```

- [ ] **Step 4: Run the test**

Run: `cargo test --workspace -p bevy_cef refresh_updates_only 2>&1 | tail -25`
Expected: PASS. If the `Messages::send` API is different in Bevy 0.18 (it may be `Messages::write` or similar — grep the project for `.send(Window` to find the correct call), fix the test accordingly.

- [ ] **Step 5: Commit**

```bash
git add src/common/dpi.rs
git commit -m "feat(dpi): implement refresh_on_scale_factor_changed_system

React to WindowScaleFactorChanged events, updating WebviewDpr only on
webviews whose HostWindow matches the changed window (or
PrimaryWindow fallback). Covered by a unit test that verifies
cross-window isolation in a multi-window scenario.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 11: Implement `commit_webview_dpr_system` (both platforms)

**Files:**
- Modify: `src/common/dpi.rs`

- [ ] **Step 1: Replace the non-Windows commit stub**

In `src/common/dpi.rs`, replace the `#[cfg(not(target_os = "windows"))] fn commit_webview_dpr_system() {}` stub with:

```rust
#[cfg(not(target_os = "windows"))]
fn commit_webview_dpr_system(
    browsers: bevy::ecs::system::NonSend<bevy_cef_core::prelude::Browsers>,
    webviews: Query<(Entity, &WebviewDpr), Changed<WebviewDpr>>,
) {
    for (entity, dpr) in webviews.iter() {
        browsers.set_dpr(&entity, dpr.0);
        browsers.notify_screen_info_changed(&entity);
    }
}
```

- [ ] **Step 2: Replace the Windows commit stub**

```rust
#[cfg(target_os = "windows")]
fn commit_webview_dpr_system_win(
    proxy: Res<bevy_cef_core::prelude::BrowsersProxy>,
    webviews: Query<(Entity, &WebviewDpr), Changed<WebviewDpr>>,
) {
    for (entity, dpr) in webviews.iter() {
        proxy.set_dpr(&entity, dpr.0);
        proxy.notify_screen_info_changed(&entity);
    }
}
```

- [ ] **Step 3: Clean up the imports**

At the top of `src/common/dpi.rs`, replace the fully-qualified `bevy_cef_core::prelude::*` references with explicit imports:

```rust
use crate::common::{HostWindow, WebviewDpr, WebviewSource};
use crate::webview::WebviewSet;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowScaleFactorChanged};
#[cfg(not(target_os = "windows"))]
use bevy_cef_core::prelude::Browsers;
#[cfg(target_os = "windows")]
use bevy_cef_core::prelude::BrowsersProxy;
```

Update the system signatures accordingly:

```rust
#[cfg(not(target_os = "windows"))]
fn commit_webview_dpr_system(
    browsers: NonSend<Browsers>,
    webviews: Query<(Entity, &WebviewDpr), Changed<WebviewDpr>>,
) {
    // same body
}
```

- [ ] **Step 4: Build check**

Run: `cargo check --workspace 2>&1 | tail -15`
Expected: clean (apart from the pending `create_webview` mismatch).

- [ ] **Step 5: Commit**

```bash
git add src/common/dpi.rs
git commit -m "feat(dpi): wire commit system to Browsers / BrowsersProxy

Commit WebviewDpr changes to CEF via Browsers::set_dpr (non-Windows)
or BrowsersProxy::set_dpr (Windows) plus notify_screen_info_changed.
Runs in WebviewSet::CommitResize so DPR changes are pushed to CEF
on the same frame as resize commits.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 12: Register `WebviewDpiPlugin` + update `create_webview`

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/webview.rs`

- [ ] **Step 1: Register `WebviewDpiPlugin` inside `CefPlugin`**

Open `src/lib.rs`. Find `impl Plugin for CefPlugin { fn build(...) }`. Locate where `WebviewCoreComponentsPlugin` is added and add `WebviewDpiPlugin` immediately after:

```rust
use crate::common::{WebviewCoreComponentsPlugin, WebviewDpiPlugin};
// ...
app.add_plugins((
    // ... other plugins ...
    WebviewCoreComponentsPlugin,
    WebviewDpiPlugin,
    // ... other plugins ...
));
```

(The exact structure depends on how `CefPlugin` is currently composed — just add `WebviewDpiPlugin` alongside `WebviewCoreComponentsPlugin`.)

- [ ] **Step 2: Update `create_webview` Query to include `&WebviewDpr` (non-Windows)**

Open `src/webview.rs`. Find `fn create_webview` (around line 246). Add `&WebviewDpr` to the `webviews` query tuple:

```rust
webviews: Query<
    (
        Entity,
        &ResolvedWebviewUri,
        &WebviewSize,
        &WebviewDpr,
        &PreloadScripts,
        Option<&HostWindow>,
    ),
    Added<ResolvedWebviewUri>,
>,
```

- [ ] **Step 3: Destructure the extra field in the loop**

Find the `for (entity, uri, size, initialize_scripts, host_window) in webviews.iter()` line. Add `dpr`:

```rust
for (entity, uri, size, dpr, initialize_scripts, host_window) in webviews.iter() {
```

- [ ] **Step 4: Pass `dpr.0` to `browsers.create_browser`**

In the `browsers.create_browser(...)` call, add `dpr.0` after `size.0`:

```rust
browsers.create_browser(
    entity,
    &uri.0,
    size.0,
    dpr.0,
    requester.clone(),
    ipc_event_sender.0.clone(),
    brp_sender.clone(),
    cursor_icon_sender.clone(),
    drag_regions_sender.0.clone(),
    &initialize_scripts.0,
    host_window,
);
```

- [ ] **Step 5: Add `WebviewDpr` import at the top of `src/webview.rs`**

Find the existing `use crate::common::{...}` line (line 2-4). Add `WebviewDpr`:

```rust
use crate::common::{
    HostWindow, IpcEventRawSender, ResolvedWebviewUri, WebviewDpr, WebviewSize, WebviewSource,
};
```

- [ ] **Step 6: Apply the same changes to `create_webview_win`**

Find `fn create_webview_win` (around line 353). Apply the same three changes: add `&WebviewDpr` to the query, destructure `dpr`, pass `dpr.0` to `proxy.create_browser`.

- [ ] **Step 7: Full workspace build check**

Run: `cargo check --workspace 2>&1 | tail -30`
Expected: **clean build**. This is the first point in the plan where the whole workspace compiles again.

- [ ] **Step 8: Run unit tests**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: all tests pass (including the three DPI tests from Tasks 9-10).

- [ ] **Step 9: Commit**

```bash
git add src/lib.rs src/webview.rs
git commit -m "feat(webview): register WebviewDpiPlugin + seed initial_dpr

Register WebviewDpiPlugin inside CefPlugin so WebviewDpr is
maintained for every webview. Update create_webview and
create_webview_win to read WebviewDpr and pass it as initial_dpr to
Browsers::create_browser / BrowsersProxy::create_browser. At this
point startup-time HiDPI works end-to-end on the current platform.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 13: Remove `WebviewDpr` from resize derive formula

**Files:**
- Modify: `src/resize/mod.rs`
- Modify: `src/resize/pipeline.rs`

- [ ] **Step 1: Drop `dpr` parameter from `derive_webview_size`**

Open `src/resize/mod.rs`. Find `fn derive_webview_size` (around line 160). Current signature:

```rust
pub fn derive_webview_size(
    display: Vec2,
    base: Vec2,
    quality: f32,
    dpr: f32,
) -> Vec2 {
    // ... uses dpr ...
}
```

Change to:

```rust
pub fn derive_webview_size(display: Vec2, base: Vec2, quality: f32) -> Vec2 {
    (display * base * quality).round()
}
```

Delete the `dpr` parameter and any usage of it in the function body.

- [ ] **Step 2: Update existing unit tests in `src/resize/mod.rs`**

Find the `#[cfg(test)] mod tests` block. Any test calling `derive_webview_size(..., dpr)` must drop the `dpr` argument. Example:

```rust
#[test]
fn derive_produces_expected_size_at_unit_inputs() {
    let size = derive_webview_size(Vec2::splat(100.0), Vec2::splat(2.0), 1.0);
    assert_eq!(size, Vec2::splat(200.0));
}
```

- [ ] **Step 3: Update `derive_pipeline_system` in `src/resize/pipeline.rs`**

Open `src/resize/pipeline.rs`. Find `fn derive_pipeline_system` (around line 10). Remove `&WebviewDpr` from the query tuple and remove `dpr.0` from the `derive_webview_size` call. Example before/after:

Before:
```rust
fn derive_pipeline_system(
    mut webviews: Query<
        (
            &DisplaySize,
            &BaseRenderScale,
            &QualityMultiplier,
            &WebviewDpr,
            &mut WebviewSize,
        ),
        (Or<(Changed<DisplaySize>, Changed<BaseRenderScale>, Changed<QualityMultiplier>, Changed<WebviewDpr>, Changed<WebviewResizable>)>, Without<PendingBasisInit>),
    >,
) {
    for (display, base, quality, dpr, mut webview_size) in webviews.iter_mut() {
        let new = derive_webview_size(display.0, base.0, quality.0, dpr.0);
        // ...
    }
}
```

After:
```rust
fn derive_pipeline_system(
    mut webviews: Query<
        (
            &DisplaySize,
            &BaseRenderScale,
            &QualityMultiplier,
            &mut WebviewSize,
        ),
        (
            Or<(
                Changed<DisplaySize>,
                Changed<BaseRenderScale>,
                Changed<QualityMultiplier>,
                Changed<WebviewResizable>,
            )>,
            Without<PendingBasisInit>,
        ),
    >,
) {
    for (display, base, quality, mut webview_size) in webviews.iter_mut() {
        let new = derive_webview_size(display.0, base.0, quality.0);
        // keep the rest of the loop body (clamping, set_if_neq, etc.)
    }
}
```

- [ ] **Step 4: Remove `use ... WebviewDpr` if no longer needed**

If `pipeline.rs` no longer references `WebviewDpr` after the edit, remove the import.

- [ ] **Step 5: Build + test**

Run: `cargo test --workspace 2>&1 | tail -25`
Expected: all tests pass. The derive_webview_size tests were updated in Step 2.

- [ ] **Step 6: Commit**

```bash
git add src/resize/mod.rs src/resize/pipeline.rs
git commit -m "refactor(resize): drop DPR from derive_webview_size formula

WebviewSize is now DIP (matches WebviewSize semantics change in the
HiDPI spec). CEF multiplies by device_scale_factor internally when
allocating the physical backing texture, so the resize derive
pipeline only needs DisplaySize × BaseRenderScale × QualityMultiplier.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 14: Remove DPR from `init_resizable_system` and interactive resize

**Files:**
- Modify: `src/resize/plugin.rs`

- [ ] **Step 1: Remove the `windows` query and `dpr` computation from `init_resizable_system`**

Open `src/resize/plugin.rs`. Find `fn init_resizable_system` (around line 381). Delete:

- The `windows: Query<&Window>` parameter
- The `let dpr = windows.iter().next().map(|w| w.scale_factor()).unwrap_or(1.0);` block (lines ~397-401)

- [ ] **Step 2: Update `BaseRenderScale` computation in the 3D mesh path**

Find the `let base = Vec2::new(webview_size.0.x / (world_size.x * dpr), ...)` block. Change to:

```rust
let base = Vec2::new(
    webview_size.0.x / world_size.x,
    webview_size.0.y / world_size.y,
);
```

- [ ] **Step 3: Update `BaseRenderScale` computation in the 2D sprite path**

Same file, slightly lower. Find:

```rust
let base = Vec2::new(
    webview_size.0.x / (display_size.x * dpr),
    webview_size.0.y / (display_size.y * dpr),
);
```

Change to:

```rust
let base = Vec2::new(
    webview_size.0.x / display_size.x,
    webview_size.0.y / display_size.y,
);
```

- [ ] **Step 4: Remove `WebviewDpr` inserts**

`init_resizable_system` currently inserts `WebviewDpr(dpr)` on each entity. Delete every `WebviewDpr(dpr)` entry from the `.insert((...))` tuples — the component is now maintained by `WebviewDpiPlugin`. Leave the other components (`DisplaySize`, `BaseRenderScale`, `QualityMultiplier`, `WebviewBasis2d`, `PendingBasisInit`) alone.

- [ ] **Step 5: Same edits in `pending_basis_init_system`**

Find `fn pending_basis_init_system` (around line 458). Remove `&WebviewDpr` from the query, remove `dpr` from the destructuring, drop `dpr.0` from the `BaseRenderScale` computation, and remove any `WebviewDpr` insertion.

- [ ] **Step 6: Interactive resize math at line ~274**

Find the `let scale_factor = Vec2::new(base.0.x * quality.0 * dpr.0, base.0.y * quality.0 * dpr.0);` line. Change to:

```rust
let scale_factor = Vec2::new(base.0.x * quality.0, base.0.y * quality.0);
```

The enclosing query destructuring has a `dpr` binding (`let Ok((mut tf, mut display_size, resizable, base, quality, dpr)) = webviews.get_mut(webview) else {...};`). Remove `dpr` from both the tuple and the query definition. Find the query signature (a few dozen lines above line 274) and drop the `&WebviewDpr` from it.

- [ ] **Step 7: Clean up unused imports**

The top of `src/resize/plugin.rs` probably still has `use crate::common::WebviewDpr;` (added in Task 6). If there are no more `WebviewDpr` references in the file, remove the import.

- [ ] **Step 8: Build + test**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: all tests pass. Run the resize example to smoke-test interactively:

```bash
cargo run --example resize 2>&1 | head -20
```

Expected: launches without panicking. Drag the corners — the webview resizes. Quit with Ctrl+C.

- [ ] **Step 9: Commit**

```bash
git add src/resize/plugin.rs
git commit -m "refactor(resize): drop DPR from init_resizable and interactive math

init_resizable_system no longer queries Window for scale_factor and
no longer inserts WebviewDpr (WebviewDpiPlugin handles that now).
BaseRenderScale is computed without the DPR divisor. Interactive
resize's scale_factor drops its × dpr.0 factor. At DPR=1.0 behavior
is identical; at DPR>1 the math is correct because WebviewSize is
DIP and CEF applies the physical-pixel multiplication downstream.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 15: Update `WebviewResizable` doc comments to DIP

**Files:**
- Modify: `src/resize/components.rs`

- [ ] **Step 1: Rewrite the doc comments on `edge_thickness`, `min_size`, `max_size`**

Open `src/resize/components.rs`. Find `WebviewResizable` (lines 12-34). Update the field doc comments:

```rust
pub struct WebviewResizable {
    /// Width of the invisible resize border, in **logical pixels (DIP)**.
    /// Default: 16.
    pub edge_thickness: u32,
    /// Minimum size in **logical pixels (DIP)**. Default: (100, 100).
    pub min_size: UVec2,
    /// Maximum size in **logical pixels (DIP)**. `None` = no cap.
    pub max_size: Option<UVec2>,
    /// Aspect-lock behavior during resize drag.
    pub aspect_lock: AspectLockMode,
}
```

- [ ] **Step 2: Build check**

Run: `cargo check --workspace 2>&1 | tail -10`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src/resize/components.rs
git commit -m "docs(resize): redefine WebviewResizable units as DIP

Match the WebviewSize DIP semantics introduced in the HiDPI spec.
Default values are unchanged numerically (16, (100,100)) — they just
now mean logical pixels instead of physical pixels.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 16: Extract `dip_to_pixel` helper in `pointer.rs` (TDD)

**Files:**
- Modify: `src/system_param/pointer.rs`

- [ ] **Step 1: Add the failing unit test first**

Open `src/system_param/pointer.rs`. Add at the bottom of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dip_to_pixel_identity_at_dpr_1() {
        // 800 DIP -> 800 px
        let result = dip_to_pixel(Vec2::new(100.0, 200.0), UVec2::new(800, 800), Vec2::new(800.0, 800.0));
        assert_eq!(result, UVec2::new(100, 200));
    }

    #[test]
    fn dip_to_pixel_scales_by_dpr_2() {
        // 800 DIP, 1600 px image -> 2x scaling
        let result = dip_to_pixel(Vec2::new(100.0, 200.0), UVec2::new(1600, 1600), Vec2::new(800.0, 800.0));
        assert_eq!(result, UVec2::new(200, 400));
    }

    #[test]
    fn dip_to_pixel_scales_by_dpr_1_5() {
        let result = dip_to_pixel(Vec2::new(100.0, 100.0), UVec2::new(1200, 900), Vec2::new(800.0, 600.0));
        assert_eq!(result, UVec2::new(150, 150));
    }

    #[test]
    fn dip_to_pixel_clamps_to_image_bounds() {
        // pos larger than dip size would overflow — must clamp to img_size - 1
        let result = dip_to_pixel(Vec2::new(1000.0, 1000.0), UVec2::new(800, 800), Vec2::new(800.0, 800.0));
        assert_eq!(result, UVec2::new(799, 799));
    }

    #[test]
    fn dip_to_pixel_zero_position_is_origin() {
        let result = dip_to_pixel(Vec2::ZERO, UVec2::new(1600, 1600), Vec2::new(800.0, 800.0));
        assert_eq!(result, UVec2::ZERO);
    }
}
```

- [ ] **Step 2: Run the failing test**

Run: `cargo test --workspace -p bevy_cef dip_to_pixel 2>&1 | tail -20`
Expected: FAIL with "cannot find function `dip_to_pixel` in this scope".

- [ ] **Step 3: Write the helper**

Add near the top of `src/system_param/pointer.rs` (below the imports, above `WebviewPointer`):

```rust
/// Convert a DIP (logical-pixel) coordinate to a physical pixel index
/// inside an image of size `img_size`, given a logical viewport of `dip_size`.
///
/// Clamps to `img_size - 1` on each axis so it is safe to use as an index
/// into the image's byte buffer. Returns `UVec2::ZERO` when `dip_size` has
/// a zero component (caller is expected to early-out on invalid inputs).
fn dip_to_pixel(pos: Vec2, img_size: UVec2, dip_size: Vec2) -> UVec2 {
    if dip_size.x <= 0.0 || dip_size.y <= 0.0 || img_size.x == 0 || img_size.y == 0 {
        return UVec2::ZERO;
    }
    let sx = img_size.x as f32 / dip_size.x;
    let sy = img_size.y as f32 / dip_size.y;
    let x = ((pos.x * sx).floor() as u32).min(img_size.x - 1);
    let y = ((pos.y * sy).floor() as u32).min(img_size.y - 1);
    UVec2::new(x, y)
}
```

- [ ] **Step 4: Run tests — should pass**

Run: `cargo test --workspace -p bevy_cef dip_to_pixel 2>&1 | tail -20`
Expected: all 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/system_param/pointer.rs
git commit -m "feat(pointer): add dip_to_pixel helper with unit tests

Pure function that converts a DIP coordinate to a physical pixel
index inside a CEF-delivered Image. Callers currently index the
Image buffer directly using DIP coordinates, which is wrong at DPR>1
once WebviewSize becomes DIP. Wired into is_transparent_at in the
next commit.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 17: Fix `is_transparent_at` to use `dip_to_pixel`

**Files:**
- Modify: `src/system_param/pointer.rs`

- [ ] **Step 1: Rewrite `is_transparent_at` to use the helper**

Find the existing `fn is_transparent_at` (lines 89-108). Replace it with:

```rust
fn is_transparent_at(&self, webview: Entity, pos: Vec2) -> bool {
    let Ok(surface) = self.surfaces.get(webview) else {
        return false;
    };
    let Some(image) = self.images.get(surface.0.id()) else {
        return false;
    };
    let Ok((_, webview_size)) = self.webviews.get(webview) else {
        return false;
    };
    let img_size = UVec2::new(image.width(), image.height());
    if img_size.x == 0 || img_size.y == 0 || webview_size.0.x <= 0.0 || webview_size.0.y <= 0.0 {
        return false;
    }
    let px = dip_to_pixel(pos, img_size, webview_size.0);
    let offset = ((px.y * img_size.x + px.x) * 4 + 3) as usize;
    let Some(data) = image.data.as_ref() else {
        return false;
    };
    data.len() > offset && data[offset] == 0
}
```

- [ ] **Step 2: Build + existing tests still pass**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: all tests still pass. No new tests are added here because `is_transparent_at` is tightly coupled to Bevy queries and `Assets<Image>`; the pure helper it delegates to is the testable part.

- [ ] **Step 3: Commit**

```bash
git add src/system_param/pointer.rs
git commit -m "fix(pointer): scale DIP hit-test coords to physical pixels

is_transparent_at used to index image.data using WebviewSize DIP
coordinates. After WebviewSize became DIP (HiDPI spec), the backing
image is DIP × DPR physical pixels, so the index was wrong by a
factor of DPR on HiDPI displays — transparent-region click-through
reported the wrong pixel. Route through the dip_to_pixel helper.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 18: Create the `hidpi` example

**Files:**
- Create: `examples/hidpi.rs`
- Create: `assets/hidpi_demo.html`

- [ ] **Step 1: Write the HTML fixture**

Create `assets/hidpi_demo.html`:

```html
<!doctype html>
<html>
<head>
<meta charset="utf-8">
<title>bevy_cef HiDPI demo</title>
<style>
  body {
    margin: 0;
    padding: 24px;
    background: #111;
    color: #eee;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  }
  h1 { font-size: 28px; margin: 0 0 8px 0; }
  .dpr { font-size: 64px; color: #4af; margin: 16px 0; }
  .hint { font-size: 14px; color: #888; margin-top: 12px; }
  p { font-size: 12px; line-height: 1.4; }
</style>
</head>
<body>
  <h1>bevy_cef HiDPI</h1>
  <div class="dpr">DPR: <span id="dpr">?</span></div>
  <p>
    Lorem ipsum dolor sit amet, consectetur adipiscing elit.
    Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
    Ut enim ad minim veniam, quis nostrud exercitation ullamco.
  </p>
  <p class="hint">Move this window between monitors with different DPI to verify the value updates.</p>
  <script>
    const el = document.getElementById('dpr');
    function render() { el.textContent = window.devicePixelRatio.toFixed(2); }
    render();
    setInterval(render, 250);
  </script>
</body>
</html>
```

- [ ] **Step 2: Write the Bevy example**

Create `examples/hidpi.rs`:

```rust
//! HiDPI / device_scale_factor demo.
//!
//! Shows a 3D plane webview that displays its current
//! `window.devicePixelRatio` live. Move the window between monitors
//! with different DPI to verify the plugin updates the rendering
//! resolution on the fly.

use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    #[cfg(target_os = "windows")]
    bevy_cef::prelude::early_exit_if_subprocess();

    App::new()
        .add_plugins((DefaultPlugins, CefPlugin::default()))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::default().looking_at(Vec3::new(1.0, -1.0, -1.0), Vec3::Y),
    ));
    commands.spawn((
        WebviewSource::local("hidpi_demo.html"),
        WebviewSize::new(800.0, 800.0),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    ));
}
```

**Note:** `WebviewSize::new` may take `f32` or `UVec2` depending on the current type definition — check `src/common/components.rs:73` and use whichever matches. If `WebviewSize(pub Vec2)`, use `WebviewSize::new(800.0, 800.0)` or `WebviewSize(Vec2::splat(800.0))`.

- [ ] **Step 3: Build the example**

Run: `cargo build --example hidpi`
Expected: clean build.

- [ ] **Step 4: Run the example**

Run: `cargo run --example hidpi` (on macOS add `--features debug` per the project's `CLAUDE.md`)

Expected: a window opens, showing a 3D plane with the inline HTML. The "DPR:" text should show the display's scale factor (1.0 on a 1× monitor, 2.0 on Retina, 1.5 on Windows at 150% scaling). Text should be crisp even on HiDPI. Close the window when done verifying.

- [ ] **Step 5: Commit**

```bash
git add examples/hidpi.rs assets/hidpi_demo.html
git commit -m "docs(example): add hidpi example for visual verification

examples/hidpi.rs spawns a 3D plane webview rendering a live
window.devicePixelRatio display. Used for the manual verification
matrix (scenarios A-F) in the HiDPI spec.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 19: Update user-facing docs

**Files:**
- Modify: `docs/website/docs/concepts.md`
- Modify: `docs/website/docs/getting-started/your-first-webview.md`
- Modify: `docs/website/docs/guides/sprite-rendering.md`

- [ ] **Step 1: Rewrite `WebviewSize` section in `concepts.md`**

Open `docs/website/docs/concepts.md`. Find the `## WebviewSize {#webview-size}` section (around line 66). Replace the section body (lines 66-74) with:

```md
## WebviewSize {#webview-size}

`WebviewSize` controls the **logical pixel (DIP) resolution** of the webview's
CSS viewport, not the physical size of the mesh in your scene. The default is
800×800 DIP.

On HiDPI displays (Retina, Windows scaling > 100%), the underlying GPU texture
is automatically rendered at `WebviewSize × device_pixel_ratio` physical pixels,
so text and imagery stay sharp without any additional configuration.

- To make a webview appear larger in your 3D scene, change the mesh dimensions
  or scale the entity's `Transform`.
- To make the rendered content use a different viewport size (e.g. to match a
  page designed for 1920×1080), set `WebviewSize::new(1920.0, 1080.0)`.
- On a 2× display, `WebviewSize::new(800.0, 800.0)` allocates roughly a
  1600×1600 physical-pixel texture. Keep this in mind for GPU memory budgeting.

The webview's CSS layout uses `WebviewSize` as its viewport dimensions, just as
if it were displayed in a browser window of that size. `window.devicePixelRatio`
in JavaScript reflects the host window's actual DPR.
```

- [ ] **Step 2: Update caution block in `your-first-webview.md`**

Open `docs/website/docs/getting-started/your-first-webview.md`. Find the `:::caution WebviewSize is texture resolution, not mesh size` block (around line 69). Replace with:

```md
:::caution WebviewSize is the logical-pixel viewport, not the mesh size

`WebviewSize` controls the **logical pixel (DIP)** resolution of the rendered
web content (default 800×800). It does not affect the physical size of the
3D mesh. To make the webview appear larger in the scene, scale the mesh or
change the plane dimensions — not `WebviewSize`. On HiDPI displays the actual
GPU texture is automatically scaled up by the display's pixel ratio, so
content stays sharp without manual tuning.

:::
```

- [ ] **Step 3: Update size-control paragraph in `sprite-rendering.md`**

Open `docs/website/docs/guides/sprite-rendering.md`. Find the `## Controlling the Display Size` section (around line 57). Replace the paragraph (lines 59-64) with:

```md
`Sprite::custom_size` controls how large the webview appears on screen, in
world units. `WebviewSize` (default 800×800 DIP) controls the logical-pixel
viewport size the web page sees. On HiDPI displays the backing texture is
automatically allocated at `WebviewSize × device_pixel_ratio`, so increasing
`WebviewSize` is usually unnecessary for sharpness — only do it if the page
is designed for a specific viewport size (e.g. 1920×1080).
```

- [ ] **Step 4: Commit**

```bash
git add docs/website/docs/concepts.md \
        docs/website/docs/getting-started/your-first-webview.md \
        docs/website/docs/guides/sprite-rendering.md
git commit -m "docs: update WebviewSize wording for DIP semantics

Rewrite the three user-facing docs that describe WebviewSize to
reflect that it now measures logical pixels (DIP), matching the CSS
viewport, and that HiDPI scaling is automatic.

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 20: Update `CHANGELOG.md`

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add HiDPI entries under `[Unreleased]`**

Open `CHANGELOG.md`. Find the `## [Unreleased]` section (or add it under the most recent version heading if missing). Insert the following. If `Added` / `Changed` subsections already exist under `[Unreleased]`, append to them; otherwise create them.

```md
## [Unreleased]

### Added

- HiDPI / `device_scale_factor` support for CEF OSR rendering. Webview
  textures now automatically use the host window's pixel ratio, producing
  sharp output on Retina and Windows-scaled displays. Monitor transitions
  are handled via `WindowScaleFactorChanged`.
- `WebviewDpiPlugin` (included in `CefPlugin`) seeds and refreshes
  `WebviewDpr` on every webview.

### Changed

- **Breaking:** `WebviewSize` is now interpreted as **logical pixels (DIP)**,
  matching CSS viewport semantics. On HiDPI displays the underlying GPU
  texture is allocated at `WebviewSize × DPR` physical pixels. Users who
  previously set `WebviewSize` expecting physical pixels may see larger
  textures on high-DPI monitors; reduce `WebviewSize` to compensate if
  needed.
- **Breaking:** `WebviewResizable::min_size`, `max_size`, and
  `edge_thickness` are now measured in DIP, not physical pixels.
- The `WebviewDpr` component has moved from the `resize` module to the
  common module and is now auto-inserted on every webview via `WebviewSource`.
- `RenderHandler::screen_info` is now implemented and returns the host
  window's `device_scale_factor`; the resize derive pipeline no longer
  applies a DPR multiplier (CEF handles physical-pixel scaling internally).
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs(changelog): add HiDPI support entry to [Unreleased]

Refs: docs/superpowers/specs/2026-04-13-webview-hidpi-support-design.md"
```

---

## Task 21: Final verification

**Files:** none (verification only)

- [ ] **Step 1: Run lint**

Run: `make fix-lint`
Expected: zero clippy warnings, zero format diffs.

- [ ] **Step 2: Run full test suite**

Run: `cargo test --workspace --all-features 2>&1 | tail -40`
Expected: all tests pass, including the 8 new unit tests added in Tasks 9, 10, and 16.

- [ ] **Step 3: Smoke-test existing examples to confirm no regressions**

For each of the following, run and visually confirm the example still works (on macOS add `--features debug`). Close each window before starting the next.

```bash
cargo run --example simple
cargo run --example resize
cargo run --example inline_html
cargo run --example host_emit
cargo run --example sprite
cargo run --example navigation
```

Expected: all six launch without panicking and render their expected content.

- [ ] **Step 4: Run the new hidpi example and verify scenarios**

Run: `cargo run --example hidpi`

Verify the six scenarios from the spec's Manual Verification section:
- **A**: baseline at 100% scaling — DPR reads `1.00`.
- **B**: at 150% / 200% scaling — DPR reads `1.50` / `2.00` and text is visibly sharper.
- **C**: drag window between monitors with different DPR — DPR value updates live.
- **D**: drag the other direction — DPR updates the other way, GPU memory drops.
- **E**: (optional, requires editing the example to spawn two webviews) — both webviews show matching DPR.
- **F**: (optional, requires a transparent-PNG webview) — click on transparent regions passes through correctly.

On platforms where all-DPI scenarios aren't physically available (e.g. a machine with only one 100% monitor), document which scenarios were not tested in the PR description.

- [ ] **Step 5: Downstream regression check (optional if workspace is isolated)**

Run:

```bash
cd ../desktop_homunculus 2>/dev/null && make debug || echo "desktop_homunculus worktree not adjacent, skipping"
cd -
```

Expected: either the downstream app builds and runs, or the message "not adjacent, skipping" prints.

- [ ] **Step 6: Final status check**

Run: `git status`
Expected: working tree clean. All work is committed.

Run: `git log --oneline main..HEAD`
Expected: ~13 commits from this plan (Tasks 5-7 were one commit each, Tasks 8-14 mostly one each, plus hidpi example, docs, changelog).

- [ ] **Step 7: No commit** — this is verification only. The plan is complete.

---

## Rollout Notes

- The breaking `WebviewSize` semantics change must be called out in the PR description and release notes.
- On Windows the first-run HiDPI effect may only be visible after `cargo install bevy_cef_render_process` refresh if users have a stale subprocess binary.
- The `rect`/`available_rect` fields of `ScreenInfo` are left at defaults — follow-ups may populate them with monitor bounds for more accurate `window.screen.*` JS APIs.
- Out-of-scope items from the spec (popup `ty` bug, `PendingBasisInit` race, sampler/mipmap config, begin-frame throttle interaction, camera-projection-driven `WebviewSize`) remain open and should be filed as follow-up issues.
