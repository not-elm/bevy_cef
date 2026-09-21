# Unify Windows onto `external_message_pump` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Windows run CEF with `external_message_pump` like macOS/Linux, so every platform uses `NonSend<Browsers>` and the Windows-only `BrowsersProxy` / `CefCommand` / `BrowsersCefSide` stack (974 lines) plus 132 call-site cfg branches disappear.

**Architecture:** This is a deletion-heavy refactor, not new behavior. `Browsers` (already the macOS/Linux implementation) becomes the only CEF-calling implementation. Windows joins Linux's CPU paint path (`Rc<Cell>` latest-frame-wins texture slots). The single remaining Windows difference in the webview pipeline is that CEF drives compositing itself (`external_begin_frame_enabled: false`), expressed through one const `EXTERNAL_BEGIN_FRAME`.

**Tech Stack:** Rust 2024, Bevy 0.19, `cef` / `cef-dll-sys` 145.6.1. Development machine is **Windows**; macOS/Linux are compile-checked by CI only.

**Spec:** `docs/superpowers/specs/2026-09-21-unify-windows-message-loop-design.md` — read it first; it explains *why* each item below exists.

## Global Constraints

- No behavior change on macOS or Linux. Never edit code under `#[cfg(target_os = "macos")]` except where this plan says so.
- Windows MUST keep `external_begin_frame_enabled: false` and MUST NOT schedule `send_external_begin_frame` (with `true`, opening DevTools permanently stops the webview's rAF and painting — verified on hardware).
- `Browsers::send_mouse_move` keeps its signature: `buttons: impl IntoIterator<Item = &'a MouseButton>`.
- Do not change `get_focused_browser` / the focused-frame gate.
- Genuine OS differences stay untouched: `crates/bevy_cef_core/src/browser_process/browsers/keyboard.rs`, the `HICON__` argument in `display_handler.rs`, the `.exe` suffix in `util.rs`, `crates/bevy_cef_core/build.rs`, `windows_subsystem` in both render-process `main.rs`, Linux switches in `command_line_config.rs`, `src/lib.rs` sandbox/zygote gates.
- `async-channel` and `raw-window-handle` remain dependencies (still used).
- Target version is `0.13.0`.
- Rust file ordering convention: module decls → imports → constants → entry points → public fns → pub(crate) fns → private fns → tests.
- When running `cargo check` / `cargo clippy`, use the `forte:rust-diagnostics` skill if available; if its runner errors, fall back to plain cargo with `--message-format=short`.
- Build examples one at a time (`cargo build --example <name>`); `cargo build --examples` fails on this machine with an unrelated "required to be available in rlib format" error.
- Commit messages follow the repo style (`fix:`, `feat:`, `remove:`, `docs:`, `update:`) and end with:
  `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`

**Why there are no new unit tests:** nothing here adds logic. CEF cannot run inside `cargo test`. The test cycle for each task is: compile (the compiler proves every Windows call site now resolves against `Browsers`), the existing unit tests, clippy with `-Dwarnings`, and residue greps that prove the deleted items are gone. Task 4 adds a runtime probe.

**Note on intermediate state:** after Task 1 the workspace root crate does not compile on Windows (it still imports the removed items). Task 1 is verified with `-p bevy_cef_core` only; Task 2 restores the full workspace build.

---

### Task 1: Core — `Browsers` is the only implementation on every OS

**Files:**
- Delete: `crates/bevy_cef_core/src/browser_process/cef_command.rs`
- Delete: `crates/bevy_cef_core/src/browser_process/cef_thread.rs`
- Modify: `crates/bevy_cef_core/src/browser_process.rs` (module decls, lines 6-9)
- Modify: `crates/bevy_cef_core/src/lib.rs` (prelude, lines 14-17)
- Modify: `crates/bevy_cef_core/src/browser_process/renderer_handler.rs`
- Modify: `crates/bevy_cef_core/src/browser_process/browsers.rs`

**Interfaces:**
- Consumes: nothing.
- Produces (Task 2 relies on these, all reachable via `bevy_cef_core::prelude::*`):
  - `pub const EXTERNAL_BEGIN_FRAME: bool` — `true` on macOS/Linux, `false` on Windows.
  - `Browsers` compiled on every OS with its full method set, including
    `create_browser(&mut self, …, _window_handle: Option<RawWindowHandle>)`, `close(&mut self, &Entity)`, `send_external_begin_frame(&mut self)`.
  - `Browsers::try_receive_textures(&self) -> impl Iterator<Item = RenderTextureMessage> + '_` available under `#[cfg(not(target_os = "macos"))]`.
  - Removed: `BrowsersProxy`, `CefCommand`, `SendRawWindowHandle`, `BrowsersCefSide`, `drain_commands`, `init_cef_browsers`, `TextureSender`.

- [ ] **Step 1: Record the baseline**

Run: `cargo test -p bevy_cef_core --all-features 2>&1 | tail -5`
Expected: `test result: ok.` Note the passed count so you can compare in Step 8.

- [ ] **Step 2: Remove the Windows-only modules and their exports**

In `crates/bevy_cef_core/src/browser_process.rs` delete these four lines:

```rust
#[cfg(target_os = "windows")]
pub mod cef_command;
#[cfg(target_os = "windows")]
pub mod cef_thread;
```

In `crates/bevy_cef_core/src/lib.rs` delete these four lines from `pub mod prelude`:

```rust
    #[cfg(all(feature = "browser", target_os = "windows"))]
    pub use crate::browser_process::cef_command::{BrowsersProxy, CefCommand};
    #[cfg(all(feature = "browser", target_os = "windows"))]
    pub use crate::browser_process::cef_thread::{drain_commands, init_cef_browsers};
```

Then delete the files:

```bash
git rm crates/bevy_cef_core/src/browser_process/cef_command.rs crates/bevy_cef_core/src/browser_process/cef_thread.rs
```

- [ ] **Step 3: `renderer_handler.rs` — Windows joins the Linux slot path**

Apply all of the following:

1. Line 5: `#[cfg(target_os = "linux")]` on `use std::cell::Cell;` → `#[cfg(not(target_os = "macos"))]`.
2. Replace the `SharedTexture` doc tail and gate, and delete `TextureSender`:

```rust
/// Non-macOS only: the CPU `OnPaint` path (Linux and Windows). macOS uses the GPU
/// IOSurface accelerated-paint path (no slots).
#[cfg(not(target_os = "macos"))]
pub type SharedTexture = std::rc::Rc<Cell<Option<RenderTextureMessage>>>;
```

   (delete the two lines `#[cfg(target_os = "windows")]` / `pub type TextureSender = async_channel::Sender<RenderTextureMessage>;`)
3. Replace both cfg'd `SharedViewSize` / `SharedDpr` pairs with single unconditional aliases:

```rust
pub type SharedViewSize = std::rc::Rc<std::cell::Cell<Vec2>>;

/// Slot for a webview's current `device_scale_factor`.
///
/// The CEF UI thread is the Bevy main thread on every platform
/// (`external_message_pump`), so no locking is needed.
pub type SharedDpr = std::rc::Rc<std::cell::Cell<f32>>;
```

4. In `struct RenderHandlerBuilder`: the two `#[cfg(target_os = "linux")]` on `view_slot` / `popup_slot` → `#[cfg(not(target_os = "macos"))]`; delete the `#[cfg(target_os = "windows")] texture_sender: TextureSender,` field.
5. In `impl RenderHandlerBuilder`: the Linux `build` gate → `#[cfg(not(target_os = "macos"))]`; delete the whole `#[cfg(target_os = "windows")] pub fn build(…texture_sender…)`.
6. In `impl Clone`: the two linux gates → `#[cfg(not(target_os = "macos"))]`; delete the `#[cfg(target_os = "windows")] texture_sender: self.texture_sender.clone(),` pair.
7. `view_rect`: replace the four cfg'd lines with `let size = self.size.get();`.
8. `screen_info`: replace the four cfg'd lines with `let dpr = self.dpr.get();`.
9. `on_paint` tail: replace both cfg'd blocks with the unconditional slot write:

```rust
        let slot = match ty {
            RenderPaintElementType::Popup => &self.popup_slot,
            RenderPaintElementType::View => &self.view_slot,
        };
        slot.set(Some(texture));
```

- [ ] **Step 4: `browsers.rs` — drop the Windows gates**

1. Delete every `#[cfg(not(target_os = "windows"))]` attribute line in this file (imports at the top, `create_browser`, `request_context`, `client_handler`, `create_extra_info`). Where the attribute is followed by `#[allow(deprecated)]` or `#[allow(clippy::too_many_arguments)]`, keep the `allow`.
2. Replace every `#[cfg(target_os = "linux")]` in this file with `#[cfg(not(target_os = "macos"))]` — including the inline parameter attributes in `client_handler` (`#[cfg(target_os = "linux")] view_slot: SharedTexture,` → `#[cfg(not(target_os = "macos"))] view_slot: SharedTexture,`), the `WebviewBrowser` fields, the locals/arguments in `create_browser`, the `RenderHandlerBuilder::build` call in `client_handler`, and `try_receive_textures`.
   **Exception:** the `parent_window` line in `create_browser` (next item).
3. In `create_browser`, replace

```rust
                // Windowless rendering does not require a parent window handle on Linux.
                #[cfg(target_os = "linux")]
                parent_window: 0,
```

   with

```rust
                // Windowless rendering does not require a parent window handle on Linux.
                #[cfg(target_os = "linux")]
                parent_window: 0,
                #[cfg(target_os = "windows")]
                parent_window: match _window_handle {
                    Some(RawWindowHandle::Win32(handle)) => {
                        cef_dll_sys::HWND(handle.hwnd.get() as _)
                    }
                    _ => cef_dll_sys::HWND(std::ptr::null_mut()),
                },
```

4. In `create_browser`, replace `external_begin_frame_enabled: true as _,` with `external_begin_frame_enabled: EXTERNAL_BEGIN_FRAME as _,`.
5. Add the const directly after the `pub use keyboard::*;` line (constants come after imports):

```rust
/// Whether bevy_cef drives CEF compositing with `SendExternalBeginFrame`.
///
/// `false` on Windows: CEF composites on its own at `windowless_frame_rate`. With
/// externally driven begin frames, opening DevTools on Windows permanently stops the
/// inspected webview's rAF and painting. This one flag decides both
/// `WindowInfo::external_begin_frame_enabled` and whether the `send_external_begin_frame`
/// system is scheduled, so the two can never disagree.
pub const EXTERNAL_BEGIN_FRAME: bool = !cfg!(target_os = "windows");
```

6. `resize`: replace the cfg'd pair with `browser.size.set(size);`. `set_dpr`: replace the cfg'd pair with `browser.dpr.set(dpr);`.
7. Update the `try_receive_textures` doc line `/// Linux-only: the CPU `OnPaint` path. macOS uses the GPU IOSurface path.` → `/// Non-macOS only: the CPU `OnPaint` path. macOS uses the GPU IOSurface path.`

- [ ] **Step 5: Check the core crate compiles**

Run: `cargo check -p bevy_cef_core --all-features --message-format=short 2>&1 | grep -E "^error|error\[|warning: unused|Finished"`
Expected: `Finished`, no errors, no unused-import warnings. If `send_external_begin_frame` or another item is reported unused, do NOT delete it — it is `pub` API; re-check that you did not remove its `pub`.

- [ ] **Step 6: Clippy the core crate**

Run: `cargo clippy -p bevy_cef_core --all-features --all-targets --message-format=short -- -Dwarnings 2>&1 | grep -E "^error|error\[|warning|Finished"`
Expected: `Finished` with no warnings.

- [ ] **Step 7: Residue grep**

Run:
```bash
grep -rnE "BrowsersProxy|BrowsersCefSide|CefCommand|SendRawWindowHandle|drain_commands|init_cef_browsers|TextureSender|Arc<Mutex|not\(target_os = \"windows\"\)" crates/bevy_cef_core/src
```
Expected: no output.

Run: `grep -rn 'target_os = "windows"' crates/bevy_cef_core/src/browser_process/browsers.rs crates/bevy_cef_core/src/browser_process/renderer_handler.rs`
Expected: exactly two hits, both in `browsers.rs`: the `#[cfg(target_os = "windows")] parent_window` arm and the `EXTERNAL_BEGIN_FRAME` const.

- [ ] **Step 8: Run the core tests**

Run: `cargo test -p bevy_cef_core --all-features 2>&1 | tail -5`
Expected: `test result: ok.` with the same passed count as Step 1.

- [ ] **Step 9: Commit**

```bash
cargo fmt --all
git add -A crates/bevy_cef_core
git commit -m "remove: Windows BrowsersProxy/CefCommand stack; Browsers is the only CEF implementation

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 2: Plugin crate — external pump on Windows, collapse every Windows branch in `src/`

**Files (all Modify):**
- `src/common/message_loop.rs`
- `src/webview.rs`
- `src/webview/mesh/webview_material.rs`
- `src/webview/mesh.rs`, `src/webview/webview_sprite.rs`, `src/webview/ui/input.rs`
- `src/keyboard.rs`, `src/focus.rs`, `src/zoom.rs`, `src/mute.rs`, `src/navigation.rs`
- `src/common/dpi.rs`, `src/common/ipc/host_emit.rs`, `src/common/localhost/responser.rs`
- `src/drag.rs`, `src/resize/plugin.rs`

**Interfaces:**
- Consumes from Task 1: `bevy_cef_core::prelude::{Browsers, EXTERNAL_BEGIN_FRAME}`; `Browsers::try_receive_textures` under `not(macos)`.
- Produces: a workspace that compiles on Windows with zero `target_os = "windows"` occurrences in `src/`.

**The collapse recipe** (used by Steps 3–6). For each Windows/non-Windows pair:
1. Delete the whole item gated by `#[cfg(target_os = "windows")]` (function, `use`, statement, or `{}` block), including its attribute line and any doc comment that belongs to it.
2. On the surviving twin, delete the `#[cfg(not(target_os = "windows"))]` attribute line and keep everything else byte-for-byte (keep `#[allow(...)]` lines).
3. For inline parameter attributes, `#[cfg(not(target_os = "windows"))] browsers: NonSend<…Browsers>,` becomes `browsers: NonSend<…Browsers>,` and the `#[cfg(target_os = "windows")] browsers: Res<…BrowsersProxy>,` line is deleted.
4. Never touch `#[cfg(target_os = "macos")]` / `#[cfg(not(target_os = "macos"))]` items.

- [ ] **Step 1: `src/common/message_loop.rs` — external pump everywhere**

1. Replace the plugin doc comment (the `///` block above `pub struct MessageLoopPlugin`) with:

```rust
/// Controls the CEF message loop.
///
/// Every platform runs CEF in `external_message_pump` mode: CEF asks for work through
/// `on_schedule_message_pump_work`, and [`CefDoMessageLoopWork`](https://cef-builds.spotifycdn.com/docs/106.1/cef__app_8h.html#a830ae43dcdffcf4e719540204cefdb61)
/// runs from a system in the `Main` schedule (throttled to a 4 ms minimum interval, with a
/// 30 Hz max-delay fallback). The CEF UI thread is therefore the Bevy main thread, which is
/// why `Browsers` is a `NonSend` resource.
///
/// `multi_threaded_message_loop` is not used: CEF does not support it on macOS, and
/// `cef_do_message_loop_work` does not block.
```

2. Replace the cfg'd channel creation (the comment block starting `// On Windows with multi_threaded_message_loop` through the two `let (tx, …)` lines) with the single line:

```rust
        let (tx, rx) = std::sync::mpsc::channel();
```

3. Delete the whole `#[cfg(target_os = "windows")] { … }` block that creates `cmd_tx` / `tex_tx` and inserts `BrowsersProxy`, `CommandChannelReceiver`, `TextureReceiverRes`, `TextureSenderRes`, together with the comment block above it (starting `// On Windows, CEF runs its own message loop thread`).
4. Un-gate the pump registration: replace

```rust
        // On non-Windows platforms, use the external message pump.
        #[cfg(not(target_os = "windows"))]
        {
            app.insert_non_send(MessageLoopWorkingReceiver(rx));
            app.add_systems(Main, cef_do_message_loop_work);

            #[cfg(all(target_os = "macos", feature = "debug"))]
            app.add_systems(
                Main,
                macos::observe_terminate_request.before(cef_do_message_loop_work),
            );
        }
```

   with

```rust
        app.insert_non_send(MessageLoopWorkingReceiver(rx));
        app.add_systems(Main, cef_do_message_loop_work);

        #[cfg(all(target_os = "macos", feature = "debug"))]
        app.add_systems(
            Main,
            macos::observe_terminate_request.before(cef_do_message_loop_work),
        );
```

5. In the non-macOS `cef_initialize`, replace

```rust
        #[cfg(target_os = "windows")]
        multi_threaded_message_loop: true as _,
        #[cfg(not(target_os = "windows"))]
        external_message_pump: true as _,
```

   with `        external_message_pump: true as _,`.
6. Delete the three Windows resource structs with their doc comments: `CommandChannelReceiver`, `TextureReceiverRes`, `TextureSenderRes`.
7. Delete the `#[cfg(not(target_os = "windows"))]` line above `fn cef_do_message_loop_work`.

- [ ] **Step 2: `src/webview.rs` — one plugin body**

1. Delete the four lines importing `CommandChannelReceiver` / `TextureSenderRes` (lines 21-24).
2. In `WebviewPlugin::build`, replace the two platform blocks (from the comment `// macOS/Linux: direct NonSend<Browsers>` through the end of the `#[cfg(target_os = "windows")] { … }` block) with:

```rust
        app.init_non_send::<Browsers>()
            .init_resource::<BeginFrameInterval>()
            .add_plugins((MeshWebviewPlugin, UiWebviewPlugin))
            .add_systems(
                Update,
                (
                    resize.run_if(any_resized).in_set(WebviewSet::CommitResize),
                    create_webview
                        .run_if(added_webview)
                        .in_set(WebviewSet::CreateBrowser),
                    navigate_on_source_change,
                ),
            )
            .add_observer(apply_request_show_devtool)
            .add_observer(apply_request_close_devtool);

        // Windows lets CEF drive compositing; see `EXTERNAL_BEGIN_FRAME`.
        if EXTERNAL_BEGIN_FRAME {
            app.add_systems(Main, send_external_begin_frame);
        }

        #[cfg(target_os = "macos")]
        app.add_plugins(crate::webview::gpu_surface::WebviewGpuInjectPlugin);
```

3. Replace the despawn hook (from the comment `// Platform-conditional despawn hook`) with:

```rust
        app.world_mut()
            .register_component_hooks::<WebviewSource>()
            .on_despawn(|mut world: DeferredWorld, ctx: HookContext| {
                world.non_send_mut::<Browsers>().close(&ctx.entity);
            });
```

4. Delete the `#[cfg(not(target_os = "windows"))]` line above each of: `send_external_begin_frame`, `create_webview`, `navigate_on_source_change`, `resize`, `apply_request_show_devtool`, `apply_request_close_devtool`.
5. Delete these Windows-only functions entirely (attribute + body): `win_commands_pending`, `post_drain_task`, `create_webview_win`, `navigate_on_source_change_win`, `resize_win`, `apply_request_show_devtool_win`, `apply_request_close_devtool_win`.
6. Append to the `BeginFrameInterval` doc comment (directly above `#[derive(Resource)] pub struct BeginFrameInterval`), keeping the existing text and example:

```rust
///
/// Has no effect on Windows: there CEF drives compositing itself at 60 Hz
/// (see `EXTERNAL_BEGIN_FRAME`), so no external begin frames are sent.
```

- [ ] **Step 3: `src/webview/mesh/webview_material.rs` — one texture pump**

In `WebviewMaterialPlugin::build`, replace

```rust
        #[cfg(target_os = "linux")]
        app.add_systems(Update, send_render_textures);

        #[cfg(target_os = "windows")]
        app.add_systems(Update, send_render_textures_win);
```

with

```rust
        #[cfg(not(target_os = "macos"))]
        app.add_systems(Update, send_render_textures);
```

Change the gate on `fn send_render_textures` from `#[cfg(target_os = "linux")]` to `#[cfg(not(target_os = "macos"))]`, and delete `send_render_textures_win` entirely. If a `use bevy_cef_core::prelude::Browsers`-style import in this file is gated on `linux`, widen it to `not(target_os = "macos")` the same way.

- [ ] **Step 4: Apply the collapse recipe to the input observers**

- `src/webview/mesh.rs`: delete `setup_observers_win`, `on_pointer_move_win`, `on_pointer_pressed_win`, `on_pointer_released_win`, `on_mouse_wheel_win`, the Windows `use`, and the Windows registration in the plugin `build`; un-gate the twins.
- `src/webview/webview_sprite.rs`: delete `setup_observers_win`, `apply_on_pointer_move_win`, `apply_on_pointer_pressed_win`, `apply_on_pointer_released_win`, `on_mouse_wheel_win`, the Windows `use`, and the Windows registration; un-gate the twins. The `WebviewIoSurface` / `sprite_pos_transparent` pieces are macOS-gated — leave them.
- `src/webview/ui/input.rs`: delete the four `#[cfg(target_os = "windows")]` functions (`on_ui_pointer_move`, `on_ui_pointer_pressed`, `on_ui_pointer_released`, `on_ui_pointer_scroll` — the second definition of each, the ones taking `proxy: Res<BrowsersProxy>`) and the Windows `use`; un-gate the first definitions and `resolve_ui_pos`. `resolve_ui_pos` already contains the `#[cfg(not(target_os = "macos"))]` branch Windows will use. Also fix the now-stale sentence in the `setup_ui_observers` doc comment: replace "The platform split lives in the observer functions, not here." with "The macOS alpha hit-test split lives in `resolve_ui_pos`, not here."

- [ ] **Step 5: Apply the collapse recipe to the identical pairs**

- `src/keyboard.rs`: delete `send_key_event_win`, `ime_event_win`, the Windows `use`, and the `#[cfg(target_os = "windows")] app.add_systems(…)` block in `KeyboardPlugin::build`; un-gate the twin registration, `send_key_event`, and `ime_event`. Keep `#[cfg(target_os = "macos")]` inside `send_key_event` and the `#[cfg_attr(not(target_os = "macos"), allow(dead_code))]` near line 364.
- `src/focus.rs`: delete `apply_webview_focus_win` + Windows `use` + Windows registration; un-gate the twins. In the surviving `apply_webview_focus`, keep the long `// NOTE:` comment.
- `src/zoom.rs`: delete `sync_zoom_win`; `src/mute.rs`: delete `sync_audio_mute_win`; `src/common/dpi.rs`: delete `commit_webview_dpr_system_win`; `src/common/ipc/host_emit.rs`: delete `host_emit_win`; `src/common/localhost/responser.rs`: delete `hot_reload_win`. In each, also delete the Windows `use` and Windows registration, and un-gate the twins.
- `src/navigation.rs`: delete the four `#[cfg(target_os = "windows")]` observers (`apply_request_go_back`, `apply_request_go_forward`, `apply_request_navigate`, `apply_request_reload` — the ones taking `proxy: Res<BrowsersProxy>`) and the Windows `use`; un-gate the first definitions.

- [ ] **Step 6: `src/drag.rs` and `src/resize/plugin.rs` — cfg'd parameters and call blocks**

In both files each affected system has a parameter pair and one or more call-site pairs. Apply recipe item 3 to the parameters. For each call-site pair keep the non-Windows form. Example from `src/drag.rs` — replace

```rust
    #[cfg(not(target_os = "windows"))]
    browsers.send_mouse_move(/* … */ input.get_pressed(), /* … */);
    #[cfg(target_os = "windows")]
    {
        let buttons: Vec<MouseButton> = input.get_pressed().copied().collect();
        browsers.send_mouse_move(/* … */ &buttons, /* … */);
    }
```

with just the first call (without its attribute), keeping its actual arguments exactly as they are in the file. Sites: `src/drag.rs` around lines 155-156, 203-215, 285-286, 298-301; `src/resize/plugin.rs` around lines 71-72, 135-148, 175-188.

- [ ] **Step 7: Compile the workspace**

Run: `cargo check --workspace --all-features --all-targets --message-format=short 2>&1 | grep -E "^error|error\[|^warning|Finished"`
Expected: `Finished`. One warning is pre-existing and out of scope: `unused import: common::*` at `src/lib.rs:40` (it exists at the pipeline start commit `b97db28`). Fix every other warning. Typical fallout: an import that was only used by a deleted `_win` function (remove it), or `MouseButton` no longer needed in a file (remove it).

- [ ] **Step 8: Residue greps**

Run:
```bash
grep -rnE "BrowsersProxy|CefCommand|drain_commands|init_cef_browsers|CommandChannelReceiver|TextureReceiverRes|TextureSenderRes|TextureSender|multi_threaded_message_loop" src crates examples
grep -rnE "fn \w+_win\b" src crates
grep -rn 'target_os = "windows"' src
```
Expected: no output from any of the three.

- [ ] **Step 9: Tests, clippy, fmt**

Run: `cargo test --workspace --all-features 2>&1 | grep -E "^test result|FAILED|panicked"`
Expected: every `test result:` line is `ok`.

Run: `cargo clippy --workspace --all-targets --all-features --message-format=short -- -Dwarnings 2>&1 | grep -E "^error|error\[|^warning|Finished"`
Expected: `Finished`. If the ONLY diagnostic is the pre-existing `src/lib.rs:40` `unused import: common::*`, do not touch `src/lib.rs`; re-run without `-- -Dwarnings`, confirm it is the sole warning, and record that in your report.

Run: `cargo fmt --all --check`
Expected: no output.

- [ ] **Step 10: Smoke-run one example**

```bash
cargo build --example simple --message-format=short 2>&1 | grep -E "^error|Finished"
./target/debug/examples/simple.exe > /tmp/simple_smoke.log 2>&1 & pid=$!; sleep 9
kill -0 $pid && echo ALIVE; powershell -NoProfile -Command "Stop-Process -Name simple -Force -ErrorAction SilentlyContinue"
grep -ci panicked /tmp/simple_smoke.log
```
Expected: `Finished`, `ALIVE`, and `0`.

- [ ] **Step 11: Commit**

```bash
git add -A src
git commit -m "feat!: run Windows on external_message_pump; remove all Windows call-site branches

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 3: Version 0.13.0, changelog, and docs

**Files (all Modify):** `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `README.md`, `CLAUDE.md`, `docs/website/docs/concepts.md`

**Interfaces:** Consumes the final state of Tasks 1–2. Produces nothing code-facing.

- [ ] **Step 1: Bump the version**

In `Cargo.toml` change `version = "0.12.0"` (line 19) to `version = "0.13.0"`, and the two workspace dependency pins (lines 46-47) from `version = "0.12.0"` to `version = "0.13.0"`.

Run: `cargo check --workspace --message-format=short 2>&1 | grep -E "^error|Finished"` (this refreshes `Cargo.lock`).
Run: `git diff --stat Cargo.lock`
Expected: `Cargo.lock` changed; `grep -n -A1 'name = "bevy_cef' Cargo.lock` shows `0.13.0` for every workspace crate.

- [ ] **Step 2: Changelog**

Insert at the very top of `CHANGELOG.md`, above `## v0.12.0`:

```markdown
## v0.13.0

### Breaking

- Windows: removed the `multi_threaded_message_loop` architecture. Removed
  `BrowsersProxy` (incl. `is_empty` / `sender`), `CefCommand`, `drain_commands`,
  `init_cef_browsers`, the `cef_command` / `cef_thread` modules (`SendRawWindowHandle`,
  `BrowsersCefSide`), `TextureSender`, `CommandChannelReceiver`, `TextureReceiverRes`, and
  `TextureSenderRes`. Use `NonSend<Browsers>` on every platform (same method names;
  `create_browser` / `close` need `NonSendMut<Browsers>`). Code that only uses components
  and EntityEvents (`RequestGoBack`, `HostEmitEvent`, …) is unaffected.
- Windows: `SharedViewSize` / `SharedDpr` are now `Rc<Cell<_>>` (were `Arc<Mutex<_>>`),
  `WebviewBrowser` has `view_slot` / `popup_slot`, and `RenderHandlerBuilder::build` takes
  the slot arguments — identical to Linux.

### Changed

- Windows now runs CEF with `external_message_pump`, like macOS and Linux, so the CEF UI
  thread is the Bevy main thread on every platform. This removes ~970 lines of duplicated
  Windows-only core code and every Windows call-site branch in the plugin crate. The
  rationale for the earlier switch (#40) was that `cef_do_message_loop_work()` blocks the
  render loop; CEF documents that it does not block, and the pump throttle from #39 already
  bounds its cost. Measured on Windows 11 (debug build): 60 fps, page rAF ≈ 60, host→JS→host
  round trip p50 ≈ 17 ms, CPU on par with the previous architecture.
- Windows: CEF calls are immediate instead of queued to another thread; CPU textures use the
  latest-frame-wins slot instead of an unbounded channel; `Browsers::can_go_back`,
  `can_go_forward`, `zoom_level`, and `exec_edit_command` now work on Windows.
- Windows: CEF work now runs only when Bevy's `Main` schedule runs (as on macOS/Linux).
  Webviews slow down or pause when Bevy runs below 60 fps, under reactive / low-power
  `WinitSettings` while unfocused, and during Win32 modal move/size loops.
- Windows keeps CEF-driven compositing (`EXTERNAL_BEGIN_FRAME == false`);
  `BeginFrameInterval` has no effect there.
```

- [ ] **Step 3: Version tables**

`README.md` line 160 and `CLAUDE.md` line 137: change the row

```
| 0.19   | 0.12.0         | 145.6.1+145.0.28 |
```

to

```
| 0.19   | 0.12.0 – 0.13.0 | 145.6.1+145.0.28 |
```

and re-align the table columns in each file so the pipes line up (widen the `bevy_cef` column by one character in every row of that table).

Do NOT edit `docs/website/docs/intro.md` or `docs/website/docs/reference/version-compatibility.md`: their tables are already stale independently of this change (they still list `0.4.0-dev`), so they carry no `0.12.0` row to bump.

- [ ] **Step 4: Architecture prose**

`CLAUDE.md`, section "Key Non-Obvious Patterns": replace the **Message loop** bullet with

```markdown
- **Message loop**: Every platform (Windows included) uses CEF's `external_message_pump` mode; `cef_do_message_loop_work()` runs from a system in the `Main` schedule when CEF requests work (4 ms minimum interval, 30 Hz max-delay fallback). `multi_threaded_message_loop` is not used — CEF does not support it on macOS.
- **Begin frames**: macOS/Linux drive compositing with `send_external_begin_frame` (`BeginFrameInterval`, default 30 fps). Windows lets CEF composite on its own (`EXTERNAL_BEGIN_FRAME == false`); externally driven begin frames stall the webview there once DevTools opens.
```

`CLAUDE.md`, section "Platform Notes", replace the **Windows** bullet with

```markdown
- **Windows**: Full support. CEF at `$USERPROFILE/.local/share/cef`, auto-copied by build.rs. Separate render process binary recommended. Shares the CPU `OnPaint` texture path with Linux; CEF drives compositing (`BeginFrameInterval` has no effect).
```

`docs/website/docs/concepts.md` line 22: replace the sentence "Instead, `cef_do_message_loop_work()` is called once per Bevy frame in the `Main` schedule." with "Instead, on every platform `cef_do_message_loop_work()` runs from a system in the `Main` schedule whenever CEF requests work (throttled to a 4 ms minimum interval, with a 30 Hz fallback)."

- [ ] **Step 5: Verify and commit**

Run: `grep -rnE "BrowsersProxy|multi_threaded|MTML" --include=*.md . | grep -v "^./CHANGELOG.md\|^./docs/superpowers/\|^./target"`
Expected: no output.

Run: `cargo fmt --all --check && cargo check --workspace --locked --message-format=short 2>&1 | grep -E "^error|Finished"`
Expected: `Finished`.

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md README.md CLAUDE.md docs/website/docs/concepts.md
git commit -m "update: version to 0.13.0; document the unified message loop

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 4: Verification sweep (Windows hardware)

**Files:** none committed, except fixes for anything this task uncovers.
- Temporary (never commit): `examples/spike_probe.rs`, copied from
  `C:\Users\elmpr\AppData\Local\Temp\claude\C--Users-elmpr-workspace-bevy-cef-wt-input-intergration\c116b8af-53ec-4216-932e-6a419dc757cd\scratchpad\spike_probe.rs`

**Interfaces:** Consumes the finished workspace. Produces a results table for the final report / PR "Test Pattern" section.

- [ ] **Step 1: Whole-repo residue and cfg whitelist**

Run:
```bash
grep -rnE "BrowsersProxy|BrowsersCefSide|CefCommand|SendRawWindowHandle|drain_commands|init_cef_browsers|CommandChannelReceiver|TextureReceiverRes|TextureSenderRes|TextureSender|multi_threaded_message_loop: true" src crates examples
grep -rnE "fn \w+_win\b" src crates
grep -rln 'target_os = "windows"' src crates
```
Expected: first two print nothing. The third prints exactly these files:
`crates/bevy_cef_core/build.rs`, `crates/bevy_cef_core/src/browser_process/browsers.rs`, `crates/bevy_cef_core/src/browser_process/browsers/keyboard.rs`, `crates/bevy_cef_core/src/browser_process/display_handler.rs`, `crates/bevy_cef_core/src/util.rs`, `crates/bevy_cef_debug_render_process/src/main.rs`, `crates/bevy_cef_render_process/src/main.rs`.

- [ ] **Step 2: Build every example individually and smoke-run the windowed ones**

```bash
for ex in $(ls examples/*.rs | xargs -n1 basename | sed 's/\.rs$//'); do
  r=$(cargo build --example $ex --message-format=short 2>&1 | grep -E "^error|Finished" | head -1)
  ./target/debug/examples/$ex.exe > /tmp/smoke_$ex.log 2>&1 & pid=$!; sleep 9
  alive=no; kill -0 $pid 2>/dev/null && alive=yes
  powershell -NoProfile -Command "Stop-Process -Name $ex -Force -ErrorAction SilentlyContinue"; wait $pid 2>/dev/null
  echo "$ex | ${r:0:12} | alive=$alive | panics=$(grep -ci panicked /tmp/smoke_$ex.log)"
done
```
Expected: every line shows `Finished`, `alive=yes`, `panics=0`. (`spike_probe` must not be in `examples/` yet for this step.)

- [ ] **Step 3: Probe run**

Copy the probe in. It already uses `NonSend<bevy_cef_core::prelude::Browsers>` for its synthetic clicks, which is now valid on Windows.

```bash
cp "/c/Users/elmpr/AppData/Local/Temp/claude/C--Users-elmpr-workspace-bevy-cef-wt-input-intergration/c116b8af-53ec-4216-932e-6a419dc757cd/scratchpad/spike_probe.rs" examples/spike_probe.rs
cargo build --example spike_probe --message-format=short 2>&1 | grep -E "^error|Finished"
PROBE_SECS=20 ./target/debug/examples/spike_probe.exe > /tmp/probe_plain.log 2>&1; echo "exit=$?"
PROBE_SECS=22 PROBE_DEVTOOLS=1 ./target/debug/examples/spike_probe.exe > /tmp/probe_devtools.log 2>&1; echo "exit=$?"
grep -E "rtt_ms|raf_mean" /tmp/probe_plain.log
grep -E "raf_fps|devtools=open" /tmp/probe_devtools.log | sed 's/PROBE //;s/raf_fps=//' | tr '\n' ' '
grep -E "js_clicks" /tmp/probe_devtools.log | tail -1
```
Expected:
- plain run: `exit=0`; `raf_mean` ≥ 55; `rtt_ms` p50 ≤ 20.
- DevTools run: `raf_fps` values keep appearing (≥ 40) **after** `devtools=open`; `js_clicks` keeps increasing to the end. A non-zero exit code on the DevTools run is a known pre-existing crash-on-exit and is out of scope — record it, do not fix it.

- [ ] **Step 4: Remove the probe and confirm a clean tree**

```bash
rm -f examples/spike_probe.rs
git status --short
```
Expected: no output. If Steps 1–3 required fixes, commit them now with a `fix:` message and the co-author trailer, then re-run the failing check.

- [ ] **Step 5: Record the results**

Write the numbers from Steps 2–3 (example count, rAF mean, RTT p50/p95, rAF-after-DevTools, exit codes) into the task report so the final report and PR description can quote them.
