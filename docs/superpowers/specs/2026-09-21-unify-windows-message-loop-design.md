# Unify Windows onto `external_message_pump` — Design

Date: 2026-09-21
Status: Approved (design sections approved in brainstorming; validated by a throwaway spike on Windows hardware)
Target version: 0.13.0 (breaking)

## Goal

Remove the Windows / non-Windows split in how bevy_cef talks to CEF, by removing its single
root cause: Windows runs CEF with `multi_threaded_message_loop` (MTML) while macOS and Linux
run it with `external_message_pump`.

Today that one difference produces:

- 132 `target_os = "windows"` cfg branches across 16 files in `src/`, including 24 `*_win`
  duplicate functions and 8 same-name cfg'd function pairs (4 in `webview/ui/input.rs`, 4 in
  `navigation.rs`). Every one exists only because the
  system parameter is `Res<BrowsersProxy>` on Windows and `NonSend<Browsers>` elsewhere.
- 974 lines of Windows-only core code (`cef_command.rs` 376 + `cef_thread.rs` 598).
  `BrowsersCefSide` in `cef_thread.rs` is a near line-for-line copy of `Browsers`.
- Drift: `BrowsersProxy` has no `can_go_back`, `can_go_forward`, `zoom_level`, or
  `exec_edit_command`, so those public `Browsers` APIs are unusable on Windows. Windows also
  lost the latest-frame-wins texture slot (PR #37) and uses an unbounded channel instead.

After this change every platform uses `NonSend<Browsers>` and the only platform axis left in
the webview pipeline is "macOS (GPU / IOSurface) vs. everything else (CPU paint)".

## Background: why this is safe

- `multi_threaded_message_loop` is documented by CEF as "only supported on Windows and Linux".
  macOS cannot use it, so `external_message_pump` is the only mode all platforms share.
- `external_message_pump` has no platform restriction; CEF ships
  `main_message_loop_external_pump_win.cc` in cefclient.
- bevy_cef shipped Windows on `external_message_pump` from v0.3.0 through v0.5.3. PR #40
  (2026-04-05) switched Windows to MTML with the rationale that calling
  `cef_do_message_loop_work()` each frame "blocks Bevy's render loop". The CEF header for that
  function states "This function will not block.", and the actual cost concern (pumping at an
  uncapped frame rate) was fixed four days later by PR #39 (4 ms minimum pump interval,
  30 Hz max-delay timer) — on the non-Windows path only.
- Spike result (Windows 11, debug build, same probe on both builds). Note: the machine's
  CEF runtime was 152.0.6 while this branch still pinned the 145.6.1 bindings, so these
  numbers and the DevTools begin-frame stall below were observed on 152.0.6. The branch was
  later rebased onto `main`, which pins 152.3.0+152.0.6, and re-probed with the same results.
  The MTML-vs-pump comparison is like-for-like:

  | Metric | MTML (current) | external pump |
  |---|---|---|
  | Bevy frame rate | 60.0 | 60.0 |
  | Steady-state max frame time | 35.1 ms | 19.8–32.7 ms |
  | Page rAF rate | 60.1 | 59.3–60 (CEF-driven compositing) |
  | Host→JS→host round trip p50 / p95 | 17.75 / 22.00 ms | 16.77–16.86 / 18.89–19.05 ms |
  | CPU, all processes, per 10 s | 31.3 s | 27.6–31.9 s |

  `devtool`, `resize`, `toolbar_drag`, `ui_webview`, `navigation` start and run without panics.
  Interactive check on hardware: with DevTools open and clicked, the Bevy-side webview keeps
  rendering and accepting input.

## Requirements

1. Windows initializes CEF with `external_message_pump: true` and never sets
   `multi_threaded_message_loop`.
2. `cef_do_message_loop_work` (with the existing 4 ms throttle and 30 Hz max-delay timer) is
   registered on every platform.
3. `Browsers` is the only CEF-calling implementation and is a `NonSend` resource on every
   platform. `cef_command.rs` and `cef_thread.rs` are deleted.
4. **Windows keeps `external_begin_frame_enabled: false`** and does not schedule the
   `send_external_begin_frame` system (both driven by one `EXTERNAL_BEGIN_FRAME` const); CEF drives compositing at `windowless_frame_rate: 60`.
   macOS and Linux keep `external_begin_frame_enabled: true` unchanged.
   Rationale: the spike showed that with `true` on Windows, opening DevTools permanently stops
   the inspected webview's rAF and painting (JS keeps running, so it looks like dead input).
5. Windows uses the same CPU texture path as Linux: the `Rc<Cell<Option<RenderTextureMessage>>>`
   latest-frame-wins view/popup slots and `Rc<Cell<_>>` `SharedViewSize` / `SharedDpr`. The
   Windows-only `async_channel` texture channel and `Arc<Mutex<_>>` variants are removed.
6. On Windows `create_browser` leaves `WindowInfo::parent_window` null; Linux keeps `0`; macOS
   keeps `parent_view`. (Amended after code review: `cef_thread.rs` *textually* passed the Bevy
   `HWND`, but `create_webview_win` ran off the main thread where winit's window lookup always
   failed, so the effective 0.12 value was null. Passing a real `HWND` switches on CEF's default
   OSR context menu — mis-positioned, since `GetScreenPoint` is not implemented — and native JS
   dialogs. Follow-up: wire the context-menu handler, then a real `HWND` can be passed.)
7. No behavior change on macOS or Linux.
8. All `*_win` functions and Windows-side cfg pairs in `src/` are removed; the surviving
   function loses its `#[cfg(not(target_os = "windows"))]`.
9. Docs and changelog describe the new architecture and the breaking API removals.

## Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| How to unify | Remove the threading-model difference (approach E) instead of hiding it behind a facade (`CefAccess` SystemParam), a trait (`CefOps`), an event bus, or an all-platform command queue | A facade leaves the 974-line duplicate implementation and the drift; an all-platform queue makes macOS/Linux worse (deferred calls, no return values, IOSurface pull cannot be a command); an event bus does not remove the cfg at the consuming end. Removing the cause deletes both the call-site branches and the duplicate implementation. |
| Begin-frame driving on Windows | CEF-driven (`external_begin_frame_enabled: false`) | Verified on hardware: externally driven begin frames stall when DevTools opens. This is also what Windows does today, so Windows rendering cadence is unchanged. Costs one remaining cfg pair. |
| Expressing the begin-frame split | One `pub const EXTERNAL_BEGIN_FRAME: bool = !cfg!(target_os = "windows");` in `bevy_cef_core` (exported via the prelude). `create_browser` sets `external_begin_frame_enabled: EXTERNAL_BEGIN_FRAME as _`; `WebviewPlugin` does `if EXTERNAL_BEGIN_FRAME { app.add_systems(Main, send_external_begin_frame); }`. The system function itself stays unconditional. | The compiler enforces that both sites agree; no `#[cfg(windows)]` remains in `src/webview.rs`; no dead-code warning because the function is still referenced. |
| `BeginFrameInterval` on Windows | Resource is initialized on all platforms (so `Res<BeginFrameInterval>` exists everywhere) but is documented as having no effect on Windows, where CEF composites at 60 Hz (macOS/Linux default to 30 fps via this resource) | Avoids a cfg in user code; honest docs. |
| `Browsers::send_mouse_move` signature | Unchanged (`impl IntoIterator<Item = &MouseButton>`) | Only the Windows call sites built a `Vec`; they are deleted. |
| Texture cfg | `#[cfg(target_os = "linux")]` on the CPU slot path becomes `#[cfg(not(target_os = "macos"))]` | Windows joins the Linux path; macOS stays on IOSurface. |
| Focus gate on clicks (`get_focused_browser`) | Unchanged | Not the cause of the DevTools symptom; out of scope. |
| Version | 0.13.0 | Public API removals. |

## Architecture

### Core (`crates/bevy_cef_core`)

- `browser_process.rs`: remove `pub mod cef_command;` and `pub mod cef_thread;`.
- `lib.rs` prelude: remove the `BrowsersProxy`, `CefCommand`, `drain_commands`,
  `init_cef_browsers` exports.
- Delete `browser_process/cef_command.rs` and `browser_process/cef_thread.rs`.
- `browsers.rs`:
  - Drop every `#[cfg(not(target_os = "windows"))]` gate. The `macos` / `not(macos)` gates stay.
  - Drop the `#[cfg(target_os = "windows")]` arms (e.g. the `Arc<Mutex>` size/DPR writes in
    `resize` / `set_dpr`).
  - `#[cfg(target_os = "linux")]` → `#[cfg(not(target_os = "macos"))]` for `SharedTexture`
    slots, `try_receive_textures`, and the `client_handler` slot arguments.
  - `create_browser`: `parent_window` stays null on Windows (see Requirement 6), `0` on Linux;
    `external_begin_frame_enabled: EXTERNAL_BEGIN_FRAME as _`.
  - Define and export `EXTERNAL_BEGIN_FRAME` (see Design Decisions).
  - `modifiers_from_mouse_buttons` / `make_underlines_for` stay; they are used by `Browsers`.
- `renderer_handler.rs`: remove the Windows `TextureSender` (`async_channel`) and
  `Arc<Mutex>` variants; Linux slot path becomes `not(macos)`; the `not(windows)` slot block
  inside `on_paint` becomes unconditional. Update the comments that describe the Windows
  `TextureSender` path and the `SharedDpr` "platform split".
- Genuine OS differences are untouched: key-code tables in `browsers/keyboard.rs`, the
  `HICON__` cursor argument in `display_handler.rs`, the `.exe` suffix in `util.rs`,
  Linux-only switches in `command_line_config.rs`.

### Plugin crate (`src/`)

- `common/message_loop.rs`:
  - Non-macOS `cef_initialize`: `external_message_pump: true` unconditionally.
  - The pump channel `(tx, rx)`, `MessageLoopWorkingReceiver`, and the
    `cef_do_message_loop_work` system are unconditional.
  - Remove `CommandChannelReceiver`, `TextureReceiverRes`, `TextureSenderRes`, and the block
    that creates the command/texture channels and inserts `BrowsersProxy`.
  - Update the plugin doc comment.
- `webview.rs`:
  - Remove the Windows block (`init_cef_browsers` task, `post_drain_task`,
    `win_commands_pending`, drain `Task` impl) and the Windows imports.
  - The non-Windows setup block becomes unconditional (including
    `init_resource::<BeginFrameInterval>()`), except that `send_external_begin_frame` is only
    added `if EXTERNAL_BEGIN_FRAME`.
  - `on_despawn` hook: single `world.non_send_mut::<Browsers>().close(&ctx.entity)`.
  - Remove `create_webview_win`, `navigate_on_source_change_win`, `resize_win`,
    `apply_request_show_devtool_win`, `apply_request_close_devtool_win`.
  - `BeginFrameInterval` doc: note it has no effect on Windows.
- `webview/mesh/webview_material.rs`: remove the Windows `send_render_textures`
  (`TextureReceiverRes`) variant; the `try_receive_textures` variant becomes `not(macos)`.
- `webview/mesh.rs`, `webview/webview_sprite.rs`, `webview/ui/input.rs`: remove the `*_win`
  observers / Windows cfg'd same-name observers. macOS IOSurface alpha hit-testing stays
  behind its existing `cfg(target_os = "macos")`; `resolve_ui_pos` becomes unconditional.
- `keyboard.rs`: remove `send_key_event_win`, `ime_event_win`; single system registration.
- `focus.rs`, `zoom.rs`, `mute.rs`, `navigation.rs`, `common/dpi.rs`,
  `common/ipc/host_emit.rs`, `common/localhost/responser.rs`: collapse each identical pair.
- `drag.rs`, `resize/plugin.rs`: cfg'd parameter pair → `NonSend<Browsers>`; cfg'd
  `send_mouse_move` call blocks → the non-Windows form.
- `async-channel` stays a dependency (IPC, cursor, drag, navigation, title still use it).

### Behavior change (Windows only)

- CEF calls become immediate (were queued to the CEF UI thread and drained on a later frame).
- Texture delivery becomes latest-frame-wins (was an unbounded channel).
- `Browsers::can_go_back`, `can_go_forward`, `zoom_level`, `exec_edit_command` work.
- Rendering cadence is unchanged (CEF-driven at 60).
- CEF browser-process work now runs only when Bevy's `Main` schedule runs (as it already does
  on macOS/Linux). Under MTML CEF's UI thread ran independently. Webviews therefore slow down
  or pause, where they did not before, when Bevy runs below 60 fps, when the app uses
  reactive / low-power `WinitSettings` while unfocused, and during Win32 modal move/size loops
  (e.g. while dragging a native window owned by the main thread, such as DevTools).

## Public API (breaking, 0.13.0)

Removed:

- `bevy_cef_core::prelude::{BrowsersProxy, CefCommand, drain_commands, init_cef_browsers}`
  (including `BrowsersProxy::{is_empty, sender}`, which have no `Browsers` equivalent)
- The `cef_command` and `cef_thread` modules and their other public items:
  `cef_command::SendRawWindowHandle`, `cef_thread::BrowsersCefSide`
- `TextureSender` (Windows-only type alias in `renderer_handler.rs`)
- `bevy_cef::common::{CommandChannelReceiver, TextureReceiverRes, TextureSenderRes}`

Changed on Windows only: `SharedViewSize` / `SharedDpr` become `Rc<Cell<_>>` (were
`Arc<Mutex<_>>`), `WebviewBrowser` gains the `view_slot` / `popup_slot` fields, and
`RenderHandlerBuilder::build` takes the slot arguments — i.e. Windows now matches Linux.

Migration: Windows code using `Res<BrowsersProxy>` switches to `NonSend<Browsers>` (same
method names); callers of `create_browser` / `close` need `NonSendMut<Browsers>`. Code that only uses the EntityEvents (`RequestGoBack`, `RequestNavigate`,
`HostEmitEvent`, `RequestShowDevTool`, …) or components is unaffected.

## Error Handling

No new error paths. `cef_initialize`'s existing assertion covers initialization failure.

## Verification

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --all-features -- -Dwarnings` (Windows)
3. `cargo test --workspace --all-features` — existing tests pass.
4. Residue greps return nothing in `src/` and `crates/`:
   `BrowsersProxy`, `BrowsersCefSide`, `CefCommand`, `SendRawWindowHandle`, `drain_commands`,
   `init_cef_browsers`, `CommandChannelReceiver`, `TextureReceiverRes`, `TextureSenderRes`,
   `TextureSender`, `fn \w+_win\b`, `multi_threaded_message_loop: true`.
   Remaining `target_os = "windows"` occurrences are limited to: `browsers/keyboard.rs`,
   `display_handler.rs`, `util.rs`, `crates/bevy_cef_core/build.rs`, the `windows_subsystem`
   attribute in both render-process `main.rs` files, and in `browsers.rs` the
   `EXTERNAL_BEGIN_FRAME` const. `src/` has none.
5. Every example builds individually (`cargo build --example <name>`) and the interactive
   ones survive a 9-second startup smoke run without `panicked` in the log.
6. Probe run (scratchpad `spike_probe.rs`, copied in temporarily, never committed), adapted to
   `NonSend<Browsers>`: Bevy ≈ 60 fps, rAF ≈ 60, rAF continues after DevTools opens, round-trip
   p50 within one frame, exit code 0 when DevTools is not opened.
7. Compile gate for macOS and Linux: the three-OS CI matrix (`.github/workflows/ci.yml`) on
   the PR. Local verification is Windows only.
8. Manual (requested from the maintainer after the pipeline): Windows `devtool` example
   including one IME input; macOS `simple` and `devtool`. Linux via CI.

## Deliverables

- Core and plugin changes above; two files deleted.
- `CHANGELOG.md`: `## v0.13.0` with Breaking (API removals + migration) and Changed (Windows
  back on `external_message_pump`, why, and the measured numbers in one or two sentences).
- `Cargo.toml`: workspace version and the `bevy_cef` / `bevy_cef_core` workspace dependency
  pins → `0.13.0`; `Cargo.lock` updated accordingly.
- `CLAUDE.md`: Multi-Process Design / message loop bullets, Key Non-Obvious Patterns, Platform
  Notes (Windows), Version Compatibility table.
- Version compatibility tables: `README.md` and `CLAUDE.md`. The website tables
  (`docs/website/docs/intro.md`, `reference/version-compatibility.md`) are already stale
  independently of this change (they list `0.4.0-dev`) and are left alone.
- `docs/website/docs/concepts.md`: the "called once per Bevy frame" sentence.
- In-code comments: the plugin doc and inline comments in `src/common/message_loop.rs`, the
  Windows comment block in `src/webview.rs`, and the `renderer_handler.rs` comments.
  (No `.md` file outside the changelog mentions MTML or `BrowsersProxy` today.)

## Out of Scope

- Crash on exit while DevTools is open (pre-existing on MTML too: `STATUS_ACCESS_VIOLATION`).
- Whether macOS/Linux also stall rendering when DevTools opens (they use
  `external_begin_frame_enabled: true`); to be checked on macOS separately.
- Letting clicks bypass the focused-frame gate.
- The `cargo build --examples` (all at once) "required to be available in rlib format" error.
- A Trigger/EntityEvent input API for embedders.
- Review suggestions deferred: extending CI clippy to all three OSes, extracting the pump
  throttle decision into a unit-tested pure helper, waking the winit event loop from
  `on_schedule_message_pump_work` (would decouple CEF from reactive update modes), merging the
  two `cef_initialize` functions.
