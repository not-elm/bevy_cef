# macOS `terminate:` Swizzle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent the macOS crash-report dialog that appears when `cargo run --example simple --features debug` is force-terminated with Ctrl+C, by swizzling `-[NSApplication terminate:]` to route through Bevy's `AppExit` shutdown instead of letting AppKit post `NSApplicationWillTerminateNotification`.

**Architecture:** All changes live in `src/common/message_loop.rs` inside the existing `#[cfg(target_os = "macos")] mod macos { … }` submodule, gated by `#[cfg(feature = "debug")]`. The swizzle replaces `terminate:`'s IMP with one that sets an atomic flag and returns without notifying. A Bevy system added to the `Main` schedule with `.before(cef_do_message_loop_work)` polls the flag and emits `AppExit::from_code(130)`, which the existing `cef_shutdown` system handles in `Update`. Spec: `docs/superpowers/specs/2026-05-28-macos-terminate-swizzle-design.md`.

**Tech Stack:** Rust edition 2024, Bevy 0.18, `objc` 0.2.7 (existing dep — same crate the existing CefAppProtocol swizzle uses), CEF 145.6.1.

---

## File Structure

**Modify:** `src/common/message_loop.rs` — single file. No new files, no public API changes.

Within that file, the changes are localized to two regions:
- `MessageLoopPlugin::build` (around line 87-92) — one new `add_systems` call.
- `cef_do_message_loop_work` (line 218) — one explanatory comment.
- `mod macos { … }` (lines 287-340) — new static, new IMP, new extern declarations, new `install_terminate_swizzle` function, new call inside `install_cef_app_protocol`, new `observe_terminate_request` Bevy system.

The codebase has no automated tests (per `CLAUDE.md`: "No automated tests. Testing done through examples."). Verification is via running the example and observing behavior. Each task ends with `cargo build --features debug` to confirm compile, plus a final manual verification task.

---

## Task 1: Confidence check — verify `objc 0.2.7` runtime exports

**Goal:** Confirm whether `class_getInstanceMethod` and `method_exchangeImplementations` are re-exported by `objc::runtime` in version 0.2.7. This is informational — the plan uses `unsafe extern "C"` declarations regardless (matching the existing `class_addMethod` precedent at `src/common/message_loop.rs:296-303`), so the result does not change later tasks. If the user later prefers to drop the extern decls in favor of crate imports, this task's output tells them which symbols are available.

**Files:** None modified.

- [ ] **Step 1: Find the installed crate**

Run:
```bash
ls ~/.cargo/registry/src/index.crates.io-*/objc-0.2.7/src/runtime.rs 2>/dev/null || \
  echo "objc 0.2.7 not in registry — run 'cargo fetch' first"
```

Expected: a single path printed, or the fallback message.

- [ ] **Step 2: Grep for the relevant function definitions**

Run:
```bash
rg -n "pub (unsafe )?fn (class_getInstanceMethod|method_exchangeImplementations|class_addMethod)\b" \
   ~/.cargo/registry/src/index.crates.io-*/objc-0.2.7/src/runtime.rs
```

Expected: zero, one, two, or three matches. Note which functions appear and what their signatures are. No action needed beyond recording the result for later reference.

- [ ] **Step 3: No commit (informational task)**

---

## Task 2: Add the swizzle infrastructure (flag, IMP, install function, call site)

**Goal:** Add the atomic flag, the replacement IMP, the new `extern "C"` declarations for the two runtime functions we need, the `install_terminate_swizzle` function, and the call to it from `install_cef_app_protocol`. All gated behind `#[cfg(feature = "debug")]`. After this task, the swizzle is installed at startup but no Bevy system reads the flag yet — `cargo build --features debug` must succeed and the example must still launch normally.

**Files:**
- Modify: `src/common/message_loop.rs:287-340` (extend the `macos` submodule)

- [ ] **Step 1: Open the file and locate the `macos` submodule**

Open `src/common/message_loop.rs` and scroll to line 287 (`#[cfg(target_os = "macos")] mod macos { … }`).

- [ ] **Step 2: Add the new extern declarations and imports inside the `macos` mod**

Inside `mod macos { … }`, immediately after the existing `unsafe extern "C" { fn class_addMethod(…); }` block (which ends at line 303), insert:

```rust
    #[cfg(feature = "debug")]
    use bevy::prelude::{AppExit, MessageWriter};

    #[cfg(feature = "debug")]
    use objc::runtime::Method;

    #[cfg(feature = "debug")]
    unsafe extern "C" {
        fn class_getInstanceMethod(cls: *const Class, sel: Sel) -> *mut Method;
        fn method_exchangeImplementations(m1: *mut Method, m2: *mut Method);
    }
```

- [ ] **Step 3: Add the atomic flag and the swizzled IMP**

Immediately after the existing `static IS_HANDLING_SEND_EVENT: AtomicBool = AtomicBool::new(false);` line (line 305), insert:

```rust
    #[cfg(feature = "debug")]
    static TERMINATE_REQUESTED: AtomicBool = AtomicBool::new(false);
```

Immediately after the existing `set_handling_send_event` function (which ends around line 313), insert:

```rust
    #[cfg(feature = "debug")]
    extern "C" fn swizzled_terminate(_: &Object, _: Sel, _sender: *mut Object) {
        // Intentionally does NOT call the original terminate:.
        // Calling it would post NSApplicationWillTerminateNotification and
        // re-trigger the winit `applicationWillTerminate:` panic.
        TERMINATE_REQUESTED.store(true, Ordering::Relaxed);
    }
```

- [ ] **Step 4: Add the `install_terminate_swizzle` function**

After the new `swizzled_terminate` function from Step 3, insert:

```rust
    #[cfg(feature = "debug")]
    unsafe fn install_terminate_swizzle() {
        let cls = Class::get("NSApplication").expect("NSApplication class not found");

        // Register our IMP under a unique placeholder selector.
        let placeholder_sel = sel!(cef_swizzled_terminate:);
        let added = class_addMethod(
            cls as *const _,
            placeholder_sel,
            swizzled_terminate as *const c_void,
            c"v@:@".as_ptr() as *const c_char,
        );
        assert!(
            added,
            "Failed to add cef_swizzled_terminate: to NSApplication"
        );

        // Fetch both Method pointers.
        let terminate_method = class_getInstanceMethod(cls as *const _, sel!(terminate:));
        assert!(
            !terminate_method.is_null(),
            "terminate: method not found on NSApplication"
        );
        let swizzled_method = class_getInstanceMethod(cls as *const _, placeholder_sel);
        assert!(
            !swizzled_method.is_null(),
            "cef_swizzled_terminate: method not found after class_addMethod"
        );

        // Swap their IMPs. After this, [NSApp terminate:] dispatches to swizzled_terminate.
        method_exchangeImplementations(terminate_method, swizzled_method);
    }
```

- [ ] **Step 5: Call `install_terminate_swizzle` from `install_cef_app_protocol`**

Locate `pub fn install_cef_app_protocol()` (around line 315). It currently ends with the `class_addMethod` call for `setHandlingSendEvent:` and closing brackets. Inside its `unsafe { … }` block, immediately before the final closing `}` of the `unsafe` block, insert:

```rust
            #[cfg(feature = "debug")]
            install_terminate_swizzle();
```

The function body should now look like (existing code abbreviated for clarity):

```rust
    pub fn install_cef_app_protocol() {
        unsafe {
            let cls = Class::get("NSApplication").expect("NSApplication クラスが見つかりません");
            // … existing isHandlingSendEvent class_addMethod call …
            // … existing setHandlingSendEvent: class_addMethod call …

            #[cfg(feature = "debug")]
            install_terminate_swizzle();
        }
    }
```

- [ ] **Step 6: Build with the debug feature to confirm compile**

Run:
```bash
cargo build --features debug --example simple
```

Expected: clean compile, no warnings about the new items (because `observe_terminate_request` doesn't exist yet but `TERMINATE_REQUESTED` is used by `swizzled_terminate`, and `swizzled_terminate` is used by `install_terminate_swizzle`, which is called from `install_cef_app_protocol`). If there is a "dead_code" warning about `TERMINATE_REQUESTED`, that's expected — it'll be resolved in Task 3.

- [ ] **Step 7: Build without the debug feature to confirm cfg gating**

Run:
```bash
cargo build --example simple
```

Expected: clean compile. The swizzle items must be entirely absent from the binary (no extern symbol resolution attempts for `class_getInstanceMethod` etc.). If the build fails with "unresolved import" or "cannot find function in this scope", the cfg gating on the new items is wrong — re-check that every new item carries `#[cfg(feature = "debug")]`.

- [ ] **Step 8: Commit**

```bash
git add src/common/message_loop.rs
git commit -m "fix(macos): swizzle NSApplication.terminate: to set shutdown flag

Adds the macOS-only, debug-only NSApplication.terminate: swizzle that
sets a module-level AtomicBool instead of letting AppKit post
NSApplicationWillTerminateNotification. The notification path triggers
a re-entrancy panic in winit's applicationWillTerminate: observer and
aborts with SIGABRT.

Spec: docs/superpowers/specs/2026-05-28-macos-terminate-swizzle-design.md

The Bevy system that observes the flag and the plugin wiring follow
in the next commits."
```

---

## Task 3: Add the `observe_terminate_request` Bevy system

**Goal:** Add the `observe_terminate_request` system that reads `TERMINATE_REQUESTED` and emits `AppExit::from_code(130)` exactly once per shutdown. Still gated behind `#[cfg(feature = "debug")]`. No plugin wiring yet — that's Task 4.

**Files:**
- Modify: `src/common/message_loop.rs` (extend the `macos` submodule with one more function)

- [ ] **Step 1: Add the system function at the end of the `macos` mod**

Immediately before the closing `}` of `mod macos { … }`, insert:

```rust
    #[cfg(feature = "debug")]
    pub(super) fn observe_terminate_request(mut writer: MessageWriter<AppExit>) {
        // `swap(false, Relaxed)` atomically reads-and-clears the flag, so AppExit
        // is emitted exactly once even if this system runs again before shutdown
        // completes.
        if TERMINATE_REQUESTED.swap(false, Ordering::Relaxed) {
            log::info!("Termination intercepted, requesting AppExit");
            writer.write(AppExit::from_code(130));
        }
    }
```

- [ ] **Step 2: Build with the debug feature to confirm compile**

Run:
```bash
cargo build --features debug --example simple
```

Expected: clean compile. There may still be a "dead_code" warning on `observe_terminate_request` because nothing calls it yet — that will be resolved in Task 4.

- [ ] **Step 3: Build without the debug feature to confirm cfg gating**

Run:
```bash
cargo build --example simple
```

Expected: clean compile. The new function must not appear in the non-debug binary.

- [ ] **Step 4: Commit**

```bash
git add src/common/message_loop.rs
git commit -m "fix(macos): add observe_terminate_request Bevy system

Reads the TERMINATE_REQUESTED flag set by the terminate: swizzle and
emits AppExit::from_code(130), which the existing cef_shutdown system
already handles in Update. swap(false, Relaxed) ensures the flag is
consumed exactly once.

Spec: docs/superpowers/specs/2026-05-28-macos-terminate-swizzle-design.md"
```

---

## Task 4: Wire `observe_terminate_request` into `MessageLoopPlugin`

**Goal:** Add `observe_terminate_request` to the `Main` schedule with `.before(cef_do_message_loop_work)` so the observer runs first in each frame. After this task, Ctrl+C should produce a clean exit (verification is Task 6).

**Files:**
- Modify: `src/common/message_loop.rs:87-92` (add one `add_systems` call)

- [ ] **Step 1: Locate the existing `add_systems(Main, cef_do_message_loop_work)` call**

Open `src/common/message_loop.rs` and find the block at lines 87-92:

```rust
        // On non-Windows platforms, use the external message pump.
        #[cfg(not(target_os = "windows"))]
        {
            app.insert_non_send_resource(MessageLoopWorkingReceiver(rx));
            app.add_systems(Main, cef_do_message_loop_work);
        }
```

- [ ] **Step 2: Add the new `add_systems` call**

Modify the block to:

```rust
        // On non-Windows platforms, use the external message pump.
        #[cfg(not(target_os = "windows"))]
        {
            app.insert_non_send_resource(MessageLoopWorkingReceiver(rx));
            app.add_systems(Main, cef_do_message_loop_work);

            #[cfg(all(target_os = "macos", feature = "debug"))]
            app.add_systems(
                Main,
                macos::observe_terminate_request.before(cef_do_message_loop_work),
            );
        }
```

- [ ] **Step 3: Build with the debug feature to confirm compile**

Run:
```bash
cargo build --features debug --example simple
```

Expected: clean compile, no "dead_code" warnings on `observe_terminate_request`.

- [ ] **Step 4: Build without the debug feature to confirm cfg gating**

Run:
```bash
cargo build --example simple
```

Expected: clean compile. The new `add_systems` line must be entirely absent from the non-debug build.

- [ ] **Step 5: Commit**

```bash
git add src/common/message_loop.rs
git commit -m "fix(macos): wire observe_terminate_request before CEF pump

Adds the observer system to the Main schedule with an explicit
.before(cef_do_message_loop_work) ordering so AppExit is queued before
the next CEF pump tick. Completes the swizzle-based Ctrl+C crash fix
for macOS debug builds.

Spec: docs/superpowers/specs/2026-05-28-macos-terminate-swizzle-design.md"
```

---

## Task 5: Add the regression-guard comment

**Goal:** Add a brief comment inside `cef_do_message_loop_work` documenting the swizzle relationship, so a future maintainer who considers removing the `debug`-feature gate understands the release-build implications (per spec § Testing).

**Files:**
- Modify: `src/common/message_loop.rs:217-244` (the `cef_do_message_loop_work` function body)

- [ ] **Step 1: Locate the function**

Open `src/common/message_loop.rs` and find the `cef_do_message_loop_work` definition at line 217:

```rust
#[cfg(not(target_os = "windows"))]
fn cef_do_message_loop_work(
    receiver: NonSend<MessageLoopWorkingReceiver>,
    mut timer: Local<Option<MessageLoopTimer>>,
    mut max_delay_timer: Local<MessageLoopWorkingMaxDelayTimer>,
    mut last_execution: Local<Option<std::time::Instant>>,
) {
    while let Ok(t) = receiver.try_recv() {
```

- [ ] **Step 2: Insert the comment**

Immediately after the opening `{` of the function body (before `while let Ok(t)`), insert:

```rust
    // macOS+debug: `macos::observe_terminate_request` is ordered to run before this
    // system so AppExit is queued before the next CEF pump tick. The NSApplication
    // terminate: swizzle in `macos::install_terminate_swizzle` prevents
    // NSApplicationWillTerminateNotification from firing and crashing winit. Release
    // builds skip the swizzle and remain vulnerable to the crash on Cmd-Q / Ctrl+C.
```

- [ ] **Step 3: Build to confirm no syntax errors**

Run:
```bash
cargo build --features debug --example simple
```

Expected: clean compile.

- [ ] **Step 4: Commit**

```bash
git add src/common/message_loop.rs
git commit -m "docs(macos): document terminate: swizzle dependency in pump system

A future maintainer who removes the debug-feature gate exposes release
builds to the SIGINT/Cmd-Q crash that the swizzle prevents. This comment
makes the dependency explicit at the point most likely to be touched.

Spec: docs/superpowers/specs/2026-05-28-macos-terminate-swizzle-design.md"
```

---

## Task 6: Manual verification

**Goal:** Confirm the fix works end-to-end on macOS with the debug feature. This is the only "test" — there are no automated tests in this repo.

**Files:** None modified.

- [ ] **Step 1: Confirm a clean build state**

Run:
```bash
cargo build --features debug --example simple
```

Expected: clean compile, no warnings, no errors.

- [ ] **Step 2: Snapshot the DiagnosticReports directory**

Run:
```bash
ls -1 ~/Library/Logs/DiagnosticReports/ 2>/dev/null | grep -c '^simple' || echo 0
```

Record the count printed. This is the baseline — any new crash report from the next run will add to this count.

- [ ] **Step 3: Run the example with logging enabled**

Run in a terminal where you can press Ctrl+C:
```bash
RUST_LOG=info cargo run --example simple --features debug
```

Wait for the window to appear and the webview to load.

- [ ] **Step 4: Press Ctrl+C in the terminal**

Expected behavior — ALL of these must hold:
1. The terminal prints a line like `INFO ...: Termination intercepted, requesting AppExit` exactly once.
2. The process exits with status code 130 (check by running `echo $?` immediately after).
3. No macOS crash-report dialog appears.
4. No new `simple-*.ips` file appears in `~/Library/Logs/DiagnosticReports/`. Re-run the command from Step 2 — the count must not have increased.

If any of these fail, stop and investigate. Do not proceed to Step 5.

- [ ] **Step 5: Verify Cmd-Q (if winit's default menu is present)**

Re-run:
```bash
RUST_LOG=info cargo run --example simple --features debug
```

Once the window appears and focus is on the app, press Cmd-Q.

Expected: same behavior as Ctrl+C — `Termination intercepted, requesting AppExit` log line, clean exit, no crash dialog.

Note: if Cmd-Q does not produce the `terminate:` selector for this app (some Bevy configurations do not install the default menu), this step may be a no-op — the window just stays open. That is acceptable; the Ctrl+C path is the primary fix target.

- [ ] **Step 6: Confirm the regression in release builds is *expected*, not new**

This step verifies that the cfg gating is correct — release builds intentionally remain vulnerable per the spec's documented scope.

Run:
```bash
cargo run --example simple
```

Press Ctrl+C. Expected: the crash STILL occurs (this is the documented limitation; not a regression introduced by this change). If the release build now exits cleanly, the cfg gating is wrong — the swizzle is leaking into release builds, which is a scope violation.

After confirming the release-build behavior matches the documented scope, kill any lingering process:
```bash
pkill -f "target/.*simple" 2>/dev/null || true
```

- [ ] **Step 7: No commit (verification task)**

If all of Steps 4–6 pass, the implementation is complete.

---

## Self-Review

**1. Spec coverage**

| Spec section | Implementing task |
|---|---|
| Component 1: `TERMINATE_REQUESTED` static | Task 2, Step 3 |
| Component 2: `swizzled_terminate` IMP | Task 2, Step 3 |
| Component 3: `install_terminate_swizzle` function (+ extern decls + call site) | Task 2, Steps 2, 4, 5 |
| Component 4: `observe_terminate_request` Bevy system | Task 3, Step 1 |
| Component 5: Plugin wiring (`Main` schedule, `.before(cef_do_message_loop_work)`) | Task 4, Step 2 |
| Schedule ordering rationale (Data Flow section) | Task 4 design, mirrored in Task 5 comment |
| Pre-implementation check on `objc 0.2.7` exports | Task 1 |
| Error handling: install-time asserts | Task 2, Step 4 (three asserts) |
| No runtime errors, no Drop/uninstall | Spec-prescribed; Tasks 2-3 follow it |
| No logging from inside swizzled IMP | Task 2, Step 3 (IMP is store-only); Task 3, Step 1 (log only in Bevy system) |
| Regression-guard comment in `cef_do_message_loop_work` | Task 5, Step 2 |
| Manual verification: Ctrl+C, Cmd-Q, RUST_LOG, exit code 130, DiagnosticReports check, release-build regression check | Task 6, Steps 2-6 |

All spec components are covered.

**2. Placeholder scan**

- No "TBD" / "TODO" / "implement later".
- Every code step shows the full code to insert.
- Commit commands include the full commit message body.
- The release-build regression check (Task 6 Step 6) intentionally expects the crash to still occur in non-debug builds — this is the documented spec scope, not a placeholder.

**3. Type consistency**

- `TERMINATE_REQUESTED: AtomicBool` — used by `swizzled_terminate` (`.store(true, Ordering::Relaxed)`) and `observe_terminate_request` (`.swap(false, Ordering::Relaxed)`). Types match.
- `swizzled_terminate` signature `(&Object, Sel, *mut Object)` matches the type encoding `"v@:@"` (void return; self id, _cmd sel, sender id).
- `class_getInstanceMethod(*const Class, Sel) -> *mut Method` and `method_exchangeImplementations(*mut Method, *mut Method)` — used consistently in `install_terminate_swizzle`.
- `observe_terminate_request(mut writer: MessageWriter<AppExit>)` — wired via `add_systems(Main, …)`; Bevy will accept this system signature.
- `.before(cef_do_message_loop_work)` — `cef_do_message_loop_work` is the function name, not a typed handle; Bevy's system-ordering API accepts function identifiers directly.
- `AppExit::from_code(130)` — `u8` literal, type-correct per Bevy 0.18.1 docs.rs.

No mismatches detected.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-28-macos-terminate-swizzle.md`. Two execution options:

1. **Subagent-Driven (recommended)** — A fresh subagent is dispatched per task with two-stage review between tasks. Best for catching context-loss errors and for a clean audit trail.

2. **Inline Execution** — Tasks run in the current session via `superpowers:executing-plans` with checkpoints between tasks. Faster but less isolation.

Which approach?
