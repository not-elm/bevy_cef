# Webview HiDPI Support — Design

**Date:** 2026-04-13
**Status:** Approved (brainstorming complete)
**Branch:** `scale`
**Related:** PR #44 (drag-to-resize), investigation log at `.claude/plans/fluffy-foraging-clock.md`

## Context

The user reports that webview rendering sometimes appears coarse / low-resolution. A parallel investigation (Codex + Claude Code Agent) traced this to multiple independent root causes, the largest of which is that **`bevy_cef` never tells CEF the display's `device_scale_factor`**. `RenderHandlerBuilder` in `crates/bevy_cef_core/src/browser_process/renderer_handler.rs:130-185` implements only `view_rect` and `on_paint`; a workspace-wide grep for `get_screen_info|device_scale_factor|notify_screen_info` returns zero matches. As a result, CEF always renders as if on a 1.0× DPI monitor — on Retina, Windows 150%/200% scaling, and other HiDPI displays, Chromium lays out pages at 1× and the bitmap is visibly soft when displayed.

A secondary contributor is that `WebviewSize` defaults to 800×800 and is decoupled from on-screen pixel coverage. Users spawning webviews without explicit sizing get an 800-physical-pixel texture regardless of monitor DPR.

This spec addresses the HiDPI axis end-to-end: CEF integration (`screen_info` + `notify_screen_info_changed`), Bevy wiring (initial seed + `WindowScaleFactorChanged` refresh + multi-window via `HostWindow`), and the semantic change to `WebviewSize` that the CEF contract requires. Other contributors identified in the investigation (popup paint-type bug, `PendingBasisInit` spawn-frame race, sampler/mipmap configuration, begin-frame throttle interaction, camera-projection-driven `WebviewSize`) are deliberately out of scope and will be addressed in separate follow-ups.

## Goals

1. On HiDPI displays (Retina, Windows ≥125%), webview text and imagery render at the display's native physical resolution without user configuration.
2. When a window is moved between monitors with different DPRs, in-flight webviews on that window update their rendering resolution automatically.
3. Webviews attached to specific windows via `HostWindow` track their host's DPR independently of other windows in multi-window setups.
4. `WebviewSize` semantics align with browser / CSS conventions: it is the logical-pixel viewport size, and the physical texture scales with DPR.

## Non-Goals

Deliberately out of scope (tracked in `.claude/plans/fluffy-foraging-clock.md`):

- Popup `RenderPaintElementType` being ignored on the Bevy side (`renderer_handler.rs:25, 153-170` — popup paints overwrite the main-view `Image`)
- `PendingBasisInit` spawn-frame race in the resize pipeline
- Sampler / mipmap configuration on the webview `Image`
- 30 FPS `BeginFrameInterval` interaction with resize repaints
- Camera-projection-driven automatic `WebviewSize` computation
- DevTools rendering (uses native OS windows, not OSR — OS handles DPI)

## Design Decisions

Three decisions locked in during brainstorming:

### 1. Scope: full multi-window HiDPI with runtime transitions

Rejected alternatives: "startup only" (breaks laptop + external monitor), "no multi-window" (breaks `HostWindow` users). Decision: track `WindowScaleFactorChanged` and honour `HostWindow` in every DPR resolution.

### 2. `WebviewSize` becomes DIP (logical pixels). Backward compatibility is not preserved.

Rationale:
- CEF OSR contract: `view_rect` is DIP, `on_paint` delivers `view_rect × device_scale_factor` physical pixels. The cleanest mapping is for `WebviewSize` to *be* DIP.
- Matches CSS / `window.innerWidth` conventions — most web developers already think in these terms.
- CSS layout stays stable across monitor transitions (`@media` queries don't flip when moving between displays).
- Resize derive pipeline simplifies: DPR drops out of the formula entirely and becomes purely a CEF-side concern.

Trade-off accepted: memory use grows by DPR² on HiDPI displays. Users who need to bound GPU memory can lower `WebviewSize`.

### 3. `WebviewDpr` component stays, promoted to every webview

Rejected alternative: component-less, drive `SharedDpr` directly from a Bevy system. Rationale for keeping the component:
- `bevy_cef` is ECS-native — every other piece of webview state is a component.
- Change detection (`Changed<WebviewDpr>`) gives a clean commit trigger without polling.
- Inspector / debug visibility.
- User code can override DPR per webview if they want (e.g. to force 2× rendering for screenshots).

The existing `WebviewDpr` in `src/resize/components.rs:75-85` has a comment "Phase 2 will update this on monitor transitions" — this spec implements Phase 2.

## Architecture

### Data flow

**Startup / spawn:**

```
PrimaryWindow or HostWindow (Window::scale_factor)
        │
        ▼
seed_webview_dpr_system  (runs on Added<WebviewSource>)
        │
        ▼
WebviewDpr (ECS component, overwrites auto-require default of 1.0)
        │
        ▼
create_webview reads WebviewDpr and passes initial_dpr to
Browsers::create_browser / BrowsersProxy::create_browser
        │
        ▼
SharedDpr slot initialised inside WebviewBrowser at browser creation time
        │
        ▼
RenderHandler::screen_info returns the initial DPR to CEF
        │
        ▼
First on_paint arrives at DIP × DPR physical pixels
```

**Runtime DPR change (monitor transition, OS setting):**

```
WindowScaleFactorChanged event
        │
        ▼
refresh_on_scale_factor_changed_system
  (runs in WebviewSet::DpiSeed, matches webviews by HostWindow / PrimaryWindow)
        │
        ▼
WebviewDpr mutated  (set_if_neq — no-op if unchanged)
        │
        │ Changed<WebviewDpr>
        ▼
commit_webview_dpr_system (non-Windows) / commit_webview_dpr_system_win (Windows)
  (runs in WebviewSet::CommitResize)
        │
        ├─► Browsers::set_dpr   / BrowsersProxy::set_dpr   (updates SharedDpr slot)
        │
        └─► Browsers::notify_screen_info_changed / proxy equivalent
              (CEF re-queries RenderHandler::screen_info and repaints)
```

### System ordering

A new `WebviewSet::DpiSeed` variant slots between `ResizeInteraction` and `DerivePipeline`:

```
ResizeInteraction → DpiSeed → DerivePipeline → CreateBrowser → CommitResize
```

`DpiSeed` runs both `seed_webview_dpr_system` (one-shot per newly added webview) and `refresh_on_scale_factor_changed_system` (event-driven). Placing `DpiSeed` before `CreateBrowser` ensures that `create_webview` reads a correctly-seeded `WebviewDpr` on the same frame the browser is created, eliminating the spawn-time DPR race.

### Component boundaries

| Unit | Responsibility | Inputs | Outputs |
|---|---|---|---|
| `WebviewDpr(f32)` | Single source of truth for a webview's current DPR | Written by Bevy-side systems only | Consumed by `commit_webview_dpr_system` and `create_webview` |
| `SharedDpr` | Thread-safe handoff slot for Bevy→CEF | Written by `Browsers::set_dpr` | Read by `RenderHandler::screen_info` |
| `seed_webview_dpr_system` | Initial DPR population for new webviews | `Added<WebviewSource>`, `HostWindow`, `PrimaryWindow` | Writes `WebviewDpr` |
| `refresh_on_scale_factor_changed_system` | React to OS DPR changes | `WindowScaleFactorChanged`, `HostWindow`, `PrimaryWindow` | Writes `WebviewDpr` |
| `commit_webview_dpr_system` | Propagate DPR changes to CEF | `Changed<WebviewDpr>` | Calls `Browsers::set_dpr` + `notify_screen_info_changed` |
| `RenderHandler::screen_info` | Supply CEF with the DPR for the next repaint | Reads `SharedDpr` | Populates `cef::ScreenInfo` |

## Implementation — CEF core (`bevy_cef_core`)

### `SharedDpr` type

New type alias in `crates/bevy_cef_core/src/browser_process/renderer_handler.rs`, mirroring `SharedViewSize`:

```rust
#[cfg(not(target_os = "windows"))]
pub type SharedDpr = std::rc::Rc<std::cell::Cell<f32>>;
#[cfg(target_os = "windows")]
pub type SharedDpr = std::sync::Arc<std::sync::Mutex<f32>>;
```

### `RenderHandlerBuilder` extension

Add `dpr: SharedDpr` field. `build()`, `Clone`, and constructor callers receive an extra `dpr: SharedDpr` parameter. `wrap_rc` and `Rc` impls are unchanged.

New trait method implementation on `ImplRenderHandler`:

```rust
fn screen_info(
    &self,
    _browser: Option<&mut Browser>,
    screen_info: Option<&mut cef::ScreenInfo>,
) -> c_int {
    let Some(info) = screen_info else { return 0 };

    #[cfg(not(target_os = "windows"))]
    let dpr = self.dpr.get();
    #[cfg(target_os = "windows")]
    let dpr = *self.dpr.lock().unwrap();

    info.device_scale_factor = dpr;
    info.depth = 24;
    info.depth_per_component = 8;
    info.is_monochrome = 0;
    // `rect` / `available_rect` describe the *monitor* in virtual-screen
    // coordinates per CEF (`cef_types.h:1911-1923`), not the view size.
    // For HiDPI rendering quality only `device_scale_factor` matters, so
    // leave the rects at their defaults. Populating them with real monitor
    // bounds would improve `window.screen.*` JS API accuracy — deferred
    // as a follow-up since it requires a window→monitor lookup.
    info.rect = cef::Rect::default();
    info.available_rect = cef::Rect::default();
    1
}
```

The `ScreenInfo.size` FFI field (struct size in bytes) is populated automatically by cef-rs's `Default` impl (verified in `cef-145.6.1+145.0.28/src/bindings/x86_64_pc_windows_msvc.rs:1000`), so we don't set it explicitly. `rect`/`available_rect` are left at defaults (all zeros); this affects `window.screen.width`/`screen.height` in JavaScript but does not impact bitmap rendering quality or the `device_scale_factor` path.

### `WebviewBrowser` extension

Add `pub dpr: SharedDpr` field to `WebviewBrowser` in `browsers.rs`.

### `Browsers::create_browser` signature

Add `initial_dpr: f32` parameter (both non-Windows and Windows variants). Inside:

```rust
let dpr: SharedDpr = Rc::new(Cell::new(initial_dpr));
// ...
client_handler(webview, size.clone(), view_slot.clone(), popup_slot.clone(),
               dpr.clone(), /* ... */)
```

`client_handler()` signature also gains `dpr: SharedDpr`, which is forwarded to `RenderHandlerBuilder::build`.

### New `Browsers` methods

```rust
pub fn set_dpr(&self, webview: &Entity, dpr: f32) {
    if let Some(browser) = self.browsers.get(webview) {
        #[cfg(not(target_os = "windows"))]
        browser.dpr.set(dpr);
        #[cfg(target_os = "windows")]
        { *browser.dpr.lock().unwrap() = dpr; }
    }
}

pub fn notify_screen_info_changed(&self, webview: &Entity) {
    if let Some(browser) = self.browsers.get(webview) {
        // `notify_screen_info_changed` alone updates Chromium's cached
        // screen metrics but does NOT run `ResizeRootLayer` /
        // `SynchronizeVisualProperties`. Only `was_resized()` pushes new
        // `VisualProperties` (including the new `device_scale_factor`) to
        // Blink. Without the pair, the CSS viewport ends up laid out as
        // `view_rect × DSF` DIP wide and on-screen text shrinks by
        // exactly `1/DSF`. Matches the cefclient OSR convention at
        // `tests/cefclient/browser/osr_window_win.cc::SetDeviceScaleFactor`.
        browser.host.notify_screen_info_changed();
        browser.host.was_resized();
    }
}
```

**Call order at the call-site:** `set_dpr` first, then `notify_screen_info_changed`. Reversing the order would cause CEF to re-query with the stale DPR. (`was_resized` inside `notify_screen_info_changed` fires *after* the CEF cache update, so Blink reflows with the new DSF on the next frame.)

**Threading note:** `CefBrowserHost` methods may be called from any browser-process thread unless explicitly restricted, and `NotifyScreenInfoChanged` has no UI-thread-only note (`cef_browser.h:276`, `:730`). On non-Windows we call from the Bevy main thread (= CEF UI thread under `external_message_pump`). On Windows we enqueue through the drain task and execute on the CEF UI thread. Both paths are valid.

### Windows `CefCommand` additions

In `crates/bevy_cef_core/src/browser_process/cef_command.rs`:

```rust
pub enum CefCommand {
    // ... existing variants ...

    CreateBrowser {
        // ... existing fields ...
        initial_dpr: f32,   // new
    },

    SetDpr { entity: Entity, dpr: f32 },                    // new
    NotifyScreenInfoChanged { entity: Entity },             // new
}
```

Matching `BrowsersProxy` methods:

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

`BrowsersProxy::create_browser` gains `initial_dpr: f32`. The Windows drain task (in `cef_thread.rs`) adds match arms for both new variants, calling the new `Browsers` methods on the CEF UI thread. FIFO ordering of `async_channel` guarantees `SetDpr` is processed before `NotifyScreenInfoChanged` when the commit system enqueues them in that order.

## Implementation — Bevy side

### Component move

`WebviewDpr` moves from `src/resize/components.rs:75-85` to `src/common/components.rs` (near `WebviewSize`). Update its doc comment to reflect the new owner (`WebviewDpiPlugin`) and remove the Phase 1 / Phase 2 wording. All the DPR handling currently in `src/resize/plugin.rs:397-451` disappears (it is superseded by `WebviewDpiPlugin`).

**Required derive change:** add `PartialEq` to the derive list. `set_if_neq` in `refresh_on_scale_factor_changed_system` requires `Self::Inner: PartialEq`, which the current `WebviewDpr` definition lacks.

```rust
#[derive(Component, Debug, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct WebviewDpr(pub f32);
```

### `WebviewSource` auto-require

```rust
#[require(WebviewSize, ZoomLevel, AudioMuted, PreloadScripts, WebviewDpr)]
pub enum WebviewSource { /* ... */ }
```

Every webview now has `WebviewDpr` automatically. Default is `1.0`; the real value is written by `seed_webview_dpr_system` on the same frame.

### `WebviewSet::DpiSeed`

New variant in `src/webview.rs:102-111`:

```rust
pub enum WebviewSet {
    ResizeInteraction,
    DpiSeed,           // new
    DerivePipeline,
    CreateBrowser,
    CommitResize,
}
```

Update `configure_sets` chain accordingly.

### New module `src/common/dpi.rs`

Contains `WebviewDpiPlugin` and its systems. Registered from `CefPlugin::build()` immediately after `WebviewCoreComponentsPlugin`.

```rust
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
```

#### `seed_webview_dpr_system`

```rust
fn seed_webview_dpr_system(
    mut webviews: Query<(&mut WebviewDpr, Option<&HostWindow>), Added<WebviewSource>>,
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
```

#### `refresh_on_scale_factor_changed_system`

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

Only webviews whose `HostWindow` matches the event (or `PrimaryWindow`-attached webviews when the primary window is the one changing) are updated. Other monitors' webviews stay put.

Note the two systems intentionally use different `primary` query types: `seed_webview_dpr_system` reads `&Window` (it needs `scale_factor()`), while `refresh_on_scale_factor_changed_system` reads `Entity` (it only needs the ID to compare against `event.window`).

#### `commit_webview_dpr_system` (non-Windows)

```rust
#[cfg(not(target_os = "windows"))]
fn commit_webview_dpr_system(
    browsers: NonSend<Browsers>,
    webviews: Query<(Entity, &WebviewDpr), Changed<WebviewDpr>>,
) {
    for (entity, dpr) in webviews.iter() {
        browsers.set_dpr(&entity, dpr.0);
        browsers.notify_screen_info_changed(&entity);
    }
}
```

Runs in `WebviewSet::CommitResize`.

#### `commit_webview_dpr_system_win` (Windows)

```rust
#[cfg(target_os = "windows")]
fn commit_webview_dpr_system_win(
    proxy: Res<BrowsersProxy>,
    webviews: Query<(Entity, &WebviewDpr), Changed<WebviewDpr>>,
) {
    for (entity, dpr) in webviews.iter() {
        proxy.set_dpr(&entity, dpr.0);
        proxy.notify_screen_info_changed(&entity);
    }
}
```

### `create_webview` / `create_webview_win` update

Add `&WebviewDpr` to the `Query`. Pass `dpr.0` as the new `initial_dpr` argument to `browsers.create_browser(...)` / `proxy.create_browser(...)`.

### Hit-test fix in `src/system_param/pointer.rs`

The current `is_transparent_at` at `pointer.rs:89-108` indexes the CEF-delivered `Image` (physical pixels) using coordinates that are in `WebviewSize` DIP space — after the DIP migration this is wrong by a factor of DPR. Replace with:

```rust
fn is_transparent_at(&self, webview: Entity, pos: Vec2) -> bool {
    let Ok(surface) = self.surfaces.get(webview) else { return false };
    let Some(image) = self.images.get(surface.0.id()) else { return false };
    let Ok((_, webview_size)) = self.webviews.get(webview) else { return false };
    let img_w = image.width();
    let img_h = image.height();
    if img_w == 0 || img_h == 0 || webview_size.0.x <= 0.0 || webview_size.0.y <= 0.0 {
        return false;
    }
    let sx = img_w as f32 / webview_size.0.x;
    let sy = img_h as f32 / webview_size.0.y;
    let x = ((pos.x * sx).floor() as u32).min(img_w - 1);
    let y = ((pos.y * sy).floor() as u32).min(img_h - 1);
    let offset = ((y * img_w + x) * 4 + 3) as usize;
    let Some(data) = image.data.as_ref() else { return false };
    data.len() > offset && data[offset] == 0
}
```

Extract the DIP-to-pixel conversion as a pure helper (`dip_to_pixel(pos, img_size, dip_size) -> UVec2`) to enable unit testing.

`pointer_to_webview_uv` (`pointer.rs:125-162`) and the sprite-path `obtain_relative_pos` (`webview_sprite.rs:307-322`) are unchanged — they return DIP coordinates, which is what CEF's `send_mouse_*_event` functions expect.

### Resize module changes

1. **Derive formula** (`src/resize/mod.rs:160-184`):
   ```rust
   pub fn derive_webview_size(display: Vec2, base: Vec2, quality: f32) -> Vec2 {
       (display * base * quality).round()
   }
   ```
   (drop `dpr: f32` parameter entirely)

2. **`derive_pipeline_system`** (`src/resize/pipeline.rs:10-30`):
   - Remove `&WebviewDpr` from query
   - Remove DPR from change detection
   - Update `derive_webview_size` call site

3. **`init_resizable_system`** (`src/resize/plugin.rs:380-455`):
   - Remove `windows` query and `dpr` computation
   - `BaseRenderScale = WebviewSize / world_size` (drop DPR from denominator)
   - Remove `WebviewDpr` inserts (Plugin handles this now)
   - Update `PendingBasisInit` path the same way

4. **`pending_basis_init_system`** (`src/resize/plugin.rs:458-487`):
   - Remove `&WebviewDpr` from query
   - `BaseRenderScale = WebviewSize / world_size`

5. **Interactive resize math** (`src/resize/plugin.rs:274`):
   ```rust
   let scale_factor = Vec2::new(base.0.x * quality.0, base.0.y * quality.0);
   ```
   (drop `× dpr.0`)
   Remove `dpr` from the surrounding query.

6. **`WebviewResizable` doc comments** (`src/resize/components.rs:14-23`):
   Redefine `edge_thickness`, `min_size`, `max_size` as **DIP (logical pixels)**, matching the new `WebviewSize` semantics. Default values (16, (100,100)) are unchanged numerically — they just now mean DIP.

## Fallback behaviour

| Situation | Behaviour |
|---|---|
| Webview has `HostWindow` pointing to a valid window | Use that window's `scale_factor()` |
| Webview has no `HostWindow`, `PrimaryWindow` exists | Use `PrimaryWindow.scale_factor()` |
| No `PrimaryWindow` and no `HostWindow` (e.g. headless test) | Default to `1.0`, emit `warn!` once |
| `WindowScaleFactorChanged` for an unrelated window | Skip — only update webviews whose `HostWindow` (or primary fallback) matches the changed window |

## Atomicity (SharedViewSize vs SharedDpr)

`SharedViewSize` and `SharedDpr` are separate slots. When both size and DPR change simultaneously (e.g. the user drags the resize handle while dragging the window across monitors), the two commit systems may run in sequence and briefly leave CEF with `new_dpr + old_size` before `new_size` arrives. This is acceptable:
- CEF's next `on_paint` delivers the final consistent bitmap.
- CSS layout does not depend on DPR in a visible way (layout uses DIP; DPR only affects the physical resolution of the backing store).
- No user-visible tearing expected in practice.

If this assumption fails in the wild, future work can coalesce the two slots into a single `SharedScreenState { size, dpr }` struct without breaking public API.

## Testing Strategy

`bevy_cef` has no automated tests today (per project `CLAUDE.md`). This spec adds targeted unit tests for the new pure-function surfaces and a visual example for end-to-end verification.

### Unit tests

1. **`src/system_param/pointer.rs`** — `#[cfg(test)] mod tests` covering the extracted `dip_to_pixel` helper at DPR=1.0, 1.5, 2.0, at boundaries (origin, exact `dip` value, oversized input clamping).
2. **`src/resize/mod.rs`** — `derive_webview_size` unit tests for typical, zero, very-large, and rounding-boundary inputs.
3. **`src/common/dpi.rs`** — Bevy `App::new().update()` tests covering:
   - `seed_webview_dpr_system` with `HostWindow` set → uses that window
   - `seed_webview_dpr_system` without `HostWindow` → falls back to `PrimaryWindow`
   - `seed_webview_dpr_system` with no windows → `1.0` + warn
   - `refresh_on_scale_factor_changed_system` only updates matching webviews in multi-window setups
   - `Changed<WebviewDpr>` fires exactly once per DPR change (via `set_if_neq`)

### New visual example `examples/hidpi.rs`

Spawns one 3D mesh webview with inline HTML showing `window.devicePixelRatio` live and a paragraph of small text. Used for manual verification scenarios A–F below.

## Documentation Updates

| File | Change |
|---|---|
| `docs/website/docs/concepts.md:66-74` | Rewrite `WebviewSize` section to describe DIP semantics, HiDPI auto-scaling, and `window.devicePixelRatio` |
| `docs/website/docs/getting-started/your-first-webview.md:69-73` | Update caution block with DIP wording |
| `docs/website/docs/guides/sprite-rendering.md:59-64` | Update size-control paragraph with DIP wording |
| `CHANGELOG.md` | New `[Unreleased]` entries under `Added` (HiDPI, `WebviewDpiPlugin`) and `Changed` (breaking: `WebviewSize` / `WebviewResizable` units, resize derive formula, `RenderHandler::screen_info`) |

## Manual Verification

Run after implementation is complete.

1. `make fix-lint` — zero clippy warnings
2. Build:
   - non-Windows: `cargo build --workspace --all-features`
   - Windows: `cargo build --workspace`
3. `cargo test --workspace --all-features` — unit tests pass
4. Smoke-test existing examples to confirm no regressions:
   `simple`, `resize`, `inline_html`, `host_emit`, `sprite`, `navigation`
5. Run the new `examples/hidpi.rs` through these scenarios:
   - **A**: start on 100% monitor → DPR reads `1.0`, baseline sharpness
   - **B**: start on 150% / 200% monitor → DPR reads `1.5` / `2.0`, text visibly sharper than A
   - **C**: drag window from 100% to 200% monitor → DPR updates to `2.0` live, text re-sharpens
   - **D**: drag window from 200% to 100% monitor → DPR updates to `1.0`, GPU memory drops
   - **E**: multiple webviews in the same window → all identical DPR
   - **F**: transparent-PNG webview → click on transparent regions passes through correctly on 2× monitors (hit-test fix verification)
6. Platform matrix:
   - macOS: Retina (DPR=2.0) on Apple Silicon and Intel
   - Windows: 100%, 125%, 150%, 200% scaling settings, and multi-monitor mixed-DPI drag
   - Linux: not supported yet
7. Downstream regression: `cd ../desktop_homunculus && make debug` still builds and runs; existing drag-to-resize feature (#44) still functions.

## Implementation Order (low risk → high risk)

1. Add `SharedDpr` type + `RenderHandlerBuilder::screen_info` impl, hardcoded `1.0` at the call-sites — ships green.
2. Add `dpr` field to `WebviewBrowser`; add `initial_dpr` parameter to `Browsers::create_browser` (non-Windows first).
3. Extend Windows path: `CefCommand::CreateBrowser` gets `initial_dpr`, add `CefCommand::SetDpr` / `NotifyScreenInfoChanged`, add `BrowsersProxy` methods and drain-task handlers.
4. Move `WebviewDpr` from `resize/components.rs` to `common/components.rs`; auto-require it on `WebviewSource`.
5. Add `WebviewSet::DpiSeed`; create `src/common/dpi.rs` with `seed_webview_dpr_system` only.
6. Update `create_webview` / `create_webview_win` to read `WebviewDpr` and pass it as `initial_dpr`. **← startup HiDPI works here.**
7. Add `refresh_on_scale_factor_changed_system` and `commit_webview_dpr_system` (both platforms). **← monitor-transition HiDPI works here.**
8. Fix `is_transparent_at` to convert DIP → physical pixels; extract pure helper for unit tests.
9. Strip DPR from the resize module (derive formula, `init_resizable_system`, `pending_basis_init_system`, interactive resize math, `WebviewResizable` docs).
10. Add unit tests (`pointer`, `resize`, `dpi`).
11. Add `examples/hidpi.rs`.
12. Update docs (`concepts.md`, `your-first-webview.md`, `sprite-rendering.md`, `CHANGELOG.md`).
13. `make fix-lint` + full smoke test across platforms.

## Resolved FFI Notes (verified during Codex review)

1. `cef::ScreenInfo::default()` auto-populates the `size` struct-size field (cef-rs binding `x86_64_pc_windows_msvc.rs:1000`). No explicit `size_of::<ScreenInfo>()` needed.
2. `ScreenInfo.rect` / `available_rect` describe the **monitor** in virtual-screen coordinates (`cef_types.h:1911-1923`), not the view rectangle — separate from `GetViewRect`, which is DIP. The spec leaves these at `cef::Rect::default()`; `device_scale_factor` is the only HiDPI-relevant field.
3. `CefBrowserHost::notify_screen_info_changed` may be called from any browser-process thread (`cef_browser.h:276, 730` — no UI-thread-only note). Both the non-Windows direct-call path and the Windows drain-task path are valid.
4. `notify_screen_info_changed` updates Chromium's cached screen metrics but does NOT run `ResizeRootLayer` / `SynchronizeVisualProperties` (`cef_browser.h:710-730`). Only `was_resized()` pushes new `VisualProperties` (including `device_scale_factor`) to Blink. The spec therefore pairs `notify_screen_info_changed` with `host.was_resized()` — matching cefclient's `osr_window_win.cc::SetDeviceScaleFactor`. The original PR mis-chose `invalidate(PET_VIEW)` (an initial guess that turned out to be wrong and produced a visible 1/DSF text shrink at DPR>1); the fix commit replaced it.

## References

- `.claude/plans/fluffy-foraging-clock.md` — original parallel-research investigation report covering all identified causes
- `docs/superpowers/specs/2026-04-12-webview-resize-handler-design.md` — drag-to-resize (#44) design; this spec layers on top
- CEF: `CefRenderHandler::GetScreenInfo` contract
- cefclient: `osr_window_win.cc` — canonical `GetScreenInfo` + `NotifyScreenInfoChanged` example
