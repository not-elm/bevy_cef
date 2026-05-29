# macOS `terminate:` Swizzle to Prevent Ctrl+C Crash Report

**Date:** 2026-05-28
**Status:** Approved design, pending implementation plan
**Scope:** `bevy_cef`, macOS only, debug builds only

## Problem

On macOS, running `cargo run --example simple --features debug` and pressing Ctrl+C while the app is running produces:

1. SIGABRT
2. The macOS crash report dialog ("simple quit unexpectedly")
3. An entry in `~/Library/Logs/DiagnosticReports/`

The crash chain (verified from `crash.log` against winit 0.30.13 source):

```
SIGINT
  → CEF (inside cef_do_message_loop_work, on the main thread)
  → [NSApplication terminate:]
  → AppKit posts NSApplicationWillTerminateNotification
  → winit applicationWillTerminate: observer runs
  → EventHandler::handle_event tries try_borrow_mut on RefCell
  → RefCell is already mutably borrowed (winit handler is on the stack above)
  → panic!("tried to handle event while another event is currently being handled")
  → panic propagates across objc2 declare_class!'s extern "C" boundary
  → panic_cannot_unwind → abort() → SIGABRT → ReportCrash
```

`disable_signal_handlers: true` in CefSettings (`src/common/message_loop.rs:132`) does not help — that flag only gates the POSIX `SetupSignalHandlers()` in Chromium's `content/app/content_main.cc`. The macOS `[NSApp terminate:]` path is a separate Cocoa-side route.

Bevy's `TerminalCtrlCHandlerPlugin` is too late — it only sets an atomic flag in the signal handler and emits `AppExit` from an `Update` system. The crash happens synchronously *inside* the CEF pump tick, well before that `Update` system can run.

## Goal

Prevent the crash dialog by intercepting `[NSApplication terminate:]` and routing termination through Bevy's existing `AppExit` shutdown path, so:
- `NSApplicationWillTerminateNotification` is never posted
- winit's `applicationWillTerminate:` observer never runs
- `cef::shutdown()` runs once cleanly via the existing `cef_shutdown` system
- The process exits with code 130 (conventional SIGINT exit code)

## Scope

**Compile-time gate:** `#[cfg(all(target_os = "macos", feature = "debug"))]`. Release macOS builds, Windows, and Linux are untouched.

**Known limitation (intentional, per scope):** Release builds (no `debug` feature) remain vulnerable. This includes Cmd-Q from the standard winit menu (`winit-0.30.13/src/platform_impl/macos/menu.rs:66-72` binds Cmd-Q to `sel!(terminate:)`). If Cmd-Q reliability becomes a release-build issue, a follow-up can promote the swizzle to always-on.

**Out of scope:**
- Linux support (the platform itself is not yet supported by `bevy_cef`).
- Suppressing the crash dialog via OS-level settings (`defaults write com.apple.CrashReporter DialogType none`) — that hides the symptom and does not fix the abort.
- Any user-visible API change.

## Approach

True Objective-C method swizzling via `method_exchangeImplementations` on `NSApplication`'s `terminate:`. The replacement IMP sets an atomic flag and returns without calling the original. A Bevy system added to `Main` with `.before(cef_do_message_loop_work)` observes the flag and emits `AppExit::from_code(130)`. The existing `cef_shutdown` system (`src/common/message_loop.rs:95`) then runs as usual.

This mirrors the canonical pattern used by CEF's own `tests/cefsimple/cefsimple_mac.mm`, which overrides `-[NSApplication terminate:]` to intercept Chromium's shutdown.

### Why not other approaches

- **`class_replaceMethod`** (overwrite IMP outright): loses the original IMP irreversibly. `method_exchangeImplementations` keeps the original recoverable under the placeholder selector, which is the lower-risk pattern. Otherwise equivalent for the current behavior.
- **Hook `applicationShouldTerminate:` on the delegate**: not viable. winit owns its `NSApplicationDelegate` via `objc2::declare_class!` (`winit-0.30.13/src/platform_impl/macos/app_state.rs:69`). Replacing the delegate breaks winit's event routing; injecting methods into winit's class is fragile across winit versions.
- **`class_addMethod` (the existing pattern in `install_cef_app_protocol`)**: only *adds* methods that don't exist. `terminate:` already exists on `NSApplication`, so `class_addMethod` returns false for it. The existing precedent works for `isHandlingSendEvent` precisely because that selector is *not* on `NSApplication` by default.

## Components

All inside the existing `macos` submodule at `src/common/message_loop.rs:287-340`. No new files. No public API changes.

### 1. `TERMINATE_REQUESTED: AtomicBool`

Module-level static. Mirrors the existing `IS_HANDLING_SEND_EVENT` pattern at line 305. `Ordering::Relaxed` is sufficient (see Concurrency note below).

### 2. `extern "C" fn swizzled_terminate(_: &Object, _: Sel, _sender: *mut Object)`

The replacement IMP. Body:

```rust
extern "C" fn swizzled_terminate(_: &Object, _: Sel, _sender: *mut Object) {
    TERMINATE_REQUESTED.store(true, Ordering::Relaxed);
    // Intentionally does NOT call the original terminate:.
    // Calling it would post NSApplicationWillTerminateNotification and re-introduce the crash.
}
```

Method type encoding: `"v@:@"` (void return; self, _cmd, sender id).

### 3. `install_terminate_swizzle()`

Installer. Called from `install_cef_app_protocol()` after the existing `setHandlingSendEvent:` block. Steps:

1. `Class::get("NSApplication")` — reuse the existing handle.
2. `class_addMethod` with selector `cef_swizzled_terminate:` and IMP `swizzled_terminate`. Assert success.
3. `class_getInstanceMethod` for both `sel!(terminate:)` and `sel!(cef_swizzled_terminate:)`. Assert non-null.
4. `method_exchangeImplementations(terminate_method, swizzled_method)`.

After step 4, calling `[NSApp terminate:]` dispatches to `swizzled_terminate`. The original IMP is preserved under `cef_swizzled_terminate:`, recoverable if ever needed.

**Pre-implementation check on `objc 0.2.7` symbol availability.** docs.rs indicates `objc::runtime::class_getInstanceMethod` and `objc::runtime::method_exchangeImplementations` are re-exported, in which case the implementation can use them directly without `unsafe extern "C"` declarations. Before coding, grep the installed crate to confirm:

```bash
rg -n "pub fn (class_getInstanceMethod|method_exchangeImplementations|class_addMethod)" \
   ~/.cargo/registry/src/index.crates.io-*/objc-0.2.7/src/runtime.rs
```

If either function is missing or has a signature mismatch (the existing precedent declares `class_addMethod` as `unsafe extern "C"` at `src/common/message_loop.rs:296-303`, suggesting one of those reasons for that specific function), fall back to declaring the missing ones in the same `unsafe extern "C"` block, matching the existing pattern.

### 4. `pub(crate) fn observe_terminate_request(mut writer: MessageWriter<AppExit>)`

Bevy system. Body:

```rust
pub(crate) fn observe_terminate_request(mut writer: MessageWriter<AppExit>) {
    if TERMINATE_REQUESTED.swap(false, Ordering::Relaxed) {
        log::info!("Termination intercepted, requesting AppExit");
        writer.write(AppExit::from_code(130));
    }
}
```

`swap(false, Relaxed)` atomically reads-and-clears the flag, ensuring `AppExit` is emitted exactly once even if the system runs again before shutdown completes.

### 5. Plugin wiring

In `MessageLoopPlugin::build`, under `#[cfg(all(target_os = "macos", feature = "debug"))]`:

```rust
app.add_systems(
    Main,
    macos::observe_terminate_request.before(cef_do_message_loop_work),
);
```

Adding to `Main` with an explicit `.before(cef_do_message_loop_work)` ordering puts the observer in the same schedule as the CEF pump and guarantees it runs first. See "Schedule ordering" below for why this matters.

## Data Flow

**Frame N (Ctrl+C arrives):**

1. SIGINT delivered.
2. CEF — on the main thread, inside the current `cef_do_message_loop_work` tick — invokes `[NSApp terminate:]`.
3. `swizzled_terminate` runs: `TERMINATE_REQUESTED.store(true)`. Returns immediately. **No notification posted.**
4. CEF's pump completes its tick. `cef_do_message_loop_work` returns to Bevy. Frame N finishes.

**Frame N+1:**

5. `Main` schedule, ordered first: `observe_terminate_request` sees the flag, writes `AppExit::from_code(130)`, clears the flag via `swap(false, Relaxed)`.
6. `Main` schedule: `cef_do_message_loop_work` runs once more. Harmless — CEF may pump pending work.
7. `Update` schedule: existing `cef_shutdown.run_if(on_message::<AppExit>)` fires → `cef::shutdown()`.
8. Bevy propagates `AppExit` to winit. winit exits its run loop cleanly. Process returns from `main()` with status 130. No abort, no crash dialog.

### Critical invariant

The swizzled `terminate:` MUST NEVER call the original `terminate:` — not on first call, not on retry, not at shutdown. Any call to the original IMP re-introduces the crash because it posts `NSApplicationWillTerminateNotification`.

### Schedule ordering

`cef_do_message_loop_work` is added to the `Main` schedule directly (`src/common/message_loop.rs:91`), not to any sub-schedule. `Main` invokes the sub-schedules (`First → PreUpdate → … → Last`) via its built-in `run_main` system. The order between "systems added to `Main` directly" and "systems running inside `Main`'s sub-schedules" is not guaranteed without explicit ordering constraints.

To make the ordering unambiguous, `observe_terminate_request` is added to `Main` (the same schedule as `cef_do_message_loop_work`) with an explicit `.before(cef_do_message_loop_work)` constraint. The Bevy scheduler then guarantees the observer runs first in each `Main` tick.

**Design is also robust to ordering slip.** Even in the unlikely case that the constraint were dropped and the observer ran *after* `cef_do_message_loop_work` in the same frame, the flag persists into the next frame. `AppExit` would still be queued, `cef_shutdown` would still fire in the `Update` sub-schedule (unambiguously after `Main`-direct systems via `MainScheduleOrder`), and the worst-case impact is one extra frame of latency before shutdown. The explicit `.before` constraint exists for correctness clarity, not because the design is fragile without it.

### Concurrency

The crash log confirms `terminate:` runs on `CrBrowserMain` / `com.apple.main-thread`, the same thread as Bevy systems. No cross-thread synchronization is needed; `Ordering::Relaxed` matches the existing `IS_HANDLING_SEND_EVENT` precedent in the same file. If a future investigation shows CEF can invoke `terminate:` from a non-main thread, the ordering should be upgraded to `SeqCst` and the system audited.

## Error Handling

Three install-time failure points, all panic on failure to match the existing precedent in `install_cef_app_protocol`:

1. `Class::get("NSApplication")` returns null — reuses existing call at line 317.
2. `class_addMethod` for the placeholder returns false — assert with a clear message. Would only happen if `cef_swizzled_terminate:` somehow already exists on `NSApplication`; the selector name is deliberately unique to avoid this.
3. `class_getInstanceMethod` returns null for either selector — assert. The placeholder was just added, and `terminate:` is guaranteed by AppKit; the assert surfaces unexpected runtime state.

`method_exchangeImplementations` returns void.

**Runtime errors:** none by construction. The swizzled IMP does an atomic store and returns. The Bevy system does a compare-exchange and conditionally writes one `AppExit`. Both are infallible.

**Deliberate non-choices:**
- No logging from inside the swizzled IMP. Calling Rust's `log` machinery from an `extern "C"` boundary invoked from CEF/AppKit shutdown is risky. Logging happens in `observe_terminate_request` on the normal Bevy thread.
- No `Drop` / uninstall logic. The swizzle persists for process lifetime, matching the existing `install_cef_app_protocol`.

## Testing

**Manual verification (primary):**

1. `cargo run --example simple --features debug`
2. Press Ctrl+C while the window is visible.
3. Expected: process exits with code 130; no crash dialog; no new entry in `~/Library/Logs/DiagnosticReports/simple-*.ips`.
4. With `RUST_LOG=info`: confirm "Termination intercepted, requesting AppExit" appears exactly once, and existing CEF shutdown log lines follow it.
5. Without `--features debug` (release build): confirm the swizzle is *not* installed and the original crash still occurs (documented limitation, not a regression).
6. With `--features debug`, quit via Cmd-Q from the standard menu (if present): confirm clean exit.

**Automated tests:** none. The crash is a function of CEF + AppKit + winit + signal delivery; none can be faithfully exercised in `#[test]`. Per `CLAUDE.md`: "No automated tests. Testing done through examples."

**Regression guard:** Add a brief comment in `cef_do_message_loop_work` referencing the swizzle so a future maintainer who removes the `debug`-feature gate understands the implications for release builds.

## Sources / Evidence

- Crash trace: `/Users/watanabe/workspace/bevy_cef/crash.log`
- CEF settings on macOS: `src/common/message_loop.rs:119-134` (`external_message_pump: true`, `disable_signal_handlers: true`)
- CEF pump system: `src/common/message_loop.rs:87,91,218-244`
- Existing shutdown wiring: `src/common/message_loop.rs:94-95,246-248`
- Existing swizzle precedent: `src/common/message_loop.rs:287-340` (`install_cef_app_protocol`)
- winit re-entrancy panic: `winit-0.30.13/src/platform_impl/macos/event_handler.rs:117-136`
- winit `applicationWillTerminate:` selector: `winit-0.30.13/src/platform_impl/macos/app_state.rs:69-72`
- winit Cmd-Q menu binding: `winit-0.30.13/src/platform_impl/macos/menu.rs:66-72`
- CEF reference pattern: `tests/cefsimple/cefsimple_mac.mm` (CEF upstream)
- Related upstream issues: rust-windowing/winit#3992, #3668, #4260
