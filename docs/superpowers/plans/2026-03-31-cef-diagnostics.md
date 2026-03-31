# CefDiagnosticsPlugin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add runtime performance diagnostics to bevy_cef using Bevy's `DiagnosticsStore`, exposed as `CefDiagnosticsPlugin`.

**Architecture:** A single `CefDiagnosticsPlugin` registers 5 custom diagnostics. Existing systems write timing/count data to intermediate resources. A dedicated collection system reads those resources and feeds `Diagnostics::add_measurement()`. The plugin is opt-in — users add it explicitly.

**Tech Stack:** Bevy 0.18 (`bevy_diagnostic`), `std::time::Instant`

**Spec:** `docs/superpowers/specs/2026-03-31-cef-diagnostics-design.md`

---

## File Structure

| File | Responsibility |
|---|---|
| `src/diagnostics.rs` (new) | `CefDiagnosticsPlugin`, collection system, intermediate resource types, `update_webview_count` system |
| `src/lib.rs` | Add `mod diagnostics`, re-export in prelude |
| `crates/bevy_cef_core/src/browser_process/renderer_handler.rs` | Add `created_at: Instant` to `RenderTextureMessage` |
| `crates/bevy_cef_core/src/browser_process/browsers.rs` | Add `Browsers::len()` method |
| `src/common/message_loop.rs` | Add `CefMessageLoopDuration` resource, timing in `cef_do_message_loop_work` |
| `src/webview/mesh/webview_material.rs` | Add texture diagnostics recording in `send_render_textures` |
| `src/common/ipc/js_emit.rs` | Add IPC diagnostics recording in `receive_events` |

---

### Task 1: Add `created_at: Instant` to `RenderTextureMessage`

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/renderer_handler.rs:14-26` (struct definition)
- Modify: `crates/bevy_cef_core/src/browser_process/renderer_handler.rs:117-125` (on_paint construction)

- [ ] **Step 1: Add `created_at` field to `RenderTextureMessage` struct**

In `renderer_handler.rs`, add `use std::time::Instant;` to imports and add the field:

```rust
// renderer_handler.rs:1 — add import
use std::time::Instant;

// renderer_handler.rs:14-26 — update struct
#[derive(Debug, Clone, PartialEq, Message)]
pub struct RenderTextureMessage {
    pub webview: Entity,
    pub ty: RenderPaintElementType,
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u8>,
    /// Timestamp when the texture was created in the CEF `on_paint` callback.
    pub created_at: Instant,
}
```

Note: `PartialEq` derive will fail because `Instant` implements `PartialEq`. Actually `Instant` does implement `PartialEq`, so the derive is fine.

- [ ] **Step 2: Set `created_at` in `on_paint`**

In `renderer_handler.rs:117-125`, add `Instant::now()`:

```rust
let texture = RenderTextureMessage {
    webview: self.webview,
    ty,
    width: width as u32,
    height: height as u32,
    buffer: unsafe {
        std::slice::from_raw_parts(buffer, (width * height * 4) as usize).to_vec()
    },
    created_at: Instant::now(),
};
```

- [ ] **Step 3: Verify workspace compiles**

Run: `cargo check --workspace --all-features`
Expected: PASS (no other code reads `RenderTextureMessage` by field construction — all consumers use it via `MessageReader` which accesses fields individually)

- [ ] **Step 4: Commit**

```bash
git add crates/bevy_cef_core/src/browser_process/renderer_handler.rs
git commit -m "feat(diagnostics): add created_at timestamp to RenderTextureMessage"
```

---

### Task 2: Add `Browsers::len()` method

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/browsers.rs:53` (impl block)

- [ ] **Step 1: Add `len()` method to `Browsers`**

In `browsers.rs`, add inside the `impl Browsers` block (after line 51, before `create_browser`):

```rust
/// Returns the number of active browser instances.
#[inline]
pub fn len(&self) -> usize {
    self.browsers.len()
}

/// Returns `true` if there are no active browser instances.
#[inline]
pub fn is_empty(&self) -> bool {
    self.browsers.is_empty()
}
```

- [ ] **Step 2: Verify workspace compiles**

Run: `cargo check --workspace --all-features`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/bevy_cef_core/src/browser_process/browsers.rs
git commit -m "feat(diagnostics): expose Browsers::len() and is_empty()"
```

---

### Task 3: Add message loop timing

**Files:**
- Modify: `src/common/message_loop.rs:1-5` (imports)
- Modify: `src/common/message_loop.rs:154-167` (`cef_do_message_loop_work` function)

- [ ] **Step 1: Add `CefMessageLoopDuration` resource and update `cef_do_message_loop_work`**

In `message_loop.rs`, add the resource definition after the imports:

```rust
use std::time::Instant;

/// Records the wall-clock duration of the last `cef_do_message_loop_work()` call.
///
/// Only written when `CefDiagnosticsPlugin` is added.
/// Read and reset by `cef_diagnostics_system`.
#[derive(Resource, Default)]
pub struct CefMessageLoopDuration(pub Option<std::time::Duration>);
```

Update `cef_do_message_loop_work` to accept the optional resource and time the CEF call:

```rust
fn cef_do_message_loop_work(
    receiver: NonSend<MessageLoopWorkingReceiver>,
    mut timer: Local<Option<MessageLoopTimer>>,
    mut max_delay_timer: Local<MessageLoopWorkingMaxDelayTimer>,
    mut duration: Option<ResMut<CefMessageLoopDuration>>,
) {
    while let Ok(t) = receiver.try_recv() {
        timer.replace(t);
    }
    if timer.as_ref().map(|t| t.is_finished()).unwrap_or(false) || max_delay_timer.is_finished() {
        let start = Instant::now();
        cef::do_message_loop_work();
        if let Some(ref mut d) = duration {
            d.0 = Some(start.elapsed());
        }
        *max_delay_timer = MessageLoopWorkingMaxDelayTimer::default();
        timer.take();
    }
}
```

- [ ] **Step 2: Export `CefMessageLoopDuration` from `common` module**

Check what `src/common/mod.rs` exports and ensure `CefMessageLoopDuration` is publicly accessible. Add `pub use message_loop::CefMessageLoopDuration;` to `src/common/mod.rs`.

- [ ] **Step 3: Verify workspace compiles**

Run: `cargo check --workspace --all-features`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/common/message_loop.rs src/common/mod.rs
git commit -m "feat(diagnostics): add timing to cef_do_message_loop_work"
```

---

### Task 4: Add texture diagnostics recording in `send_render_textures`

**Files:**
- Modify: `src/webview/mesh/webview_material.rs:37-41` (`send_render_textures` function)

- [ ] **Step 1: Create `CefTextureDiagnostics` resource (will live in `diagnostics.rs`, but referenced here)**

For now, define the resource in a new file `src/diagnostics.rs` (create it minimally — full plugin comes in Task 6):

```rust
use bevy::prelude::*;
use std::time::Duration;

/// Intermediate resource for texture transfer diagnostics.
///
/// Written by `send_render_textures`, read and reset by `cef_diagnostics_system`.
#[derive(Resource, Default)]
pub struct CefTextureDiagnostics {
    pub last_transfer_time: Option<Duration>,
    pub total_buffer_bytes: u64,
}

/// Intermediate resource for IPC processing diagnostics.
///
/// Written by `receive_events`, read and reset by `cef_diagnostics_system`.
#[derive(Resource, Default)]
pub struct CefIpcDiagnostics {
    pub last_processing_time: Option<Duration>,
}

/// Cached webview count for diagnostics (bridges NonSend Browsers → Send resource).
#[derive(Resource, Default)]
pub struct CefWebviewCount(pub usize);
```

Add `pub mod diagnostics;` to `src/lib.rs` (after the existing mod declarations, before `use` statements).

- [ ] **Step 2: Update `send_render_textures` to record diagnostics**

In `webview_material.rs`, update the function:

```rust
use crate::diagnostics::CefTextureDiagnostics;

fn send_render_textures(
    mut ew: MessageWriter<RenderTextureMessage>,
    browsers: NonSend<Browsers>,
    mut diagnostics: Option<ResMut<CefTextureDiagnostics>>,
) {
    if let Some(ref mut diag) = diagnostics {
        diag.total_buffer_bytes = 0;
    }
    while let Ok(texture) = browsers.try_receive_texture() {
        if let Some(ref mut diag) = diagnostics {
            diag.last_transfer_time = Some(texture.created_at.elapsed());
            diag.total_buffer_bytes += texture.buffer.len() as u64;
        }
        ew.write(texture);
    }
}
```

- [ ] **Step 3: Verify workspace compiles**

Run: `cargo check --workspace --all-features`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/diagnostics.rs src/lib.rs src/webview/mesh/webview_material.rs
git commit -m "feat(diagnostics): add texture transfer timing and buffer size recording"
```

---

### Task 5: Add IPC diagnostics recording in `receive_events`

**Files:**
- Modify: `src/common/ipc/js_emit.rs:49-61` (`receive_events` function)

- [ ] **Step 1: Update `receive_events` to record IPC processing time**

In `js_emit.rs`:

```rust
use crate::diagnostics::CefIpcDiagnostics;
use std::time::Instant;

fn receive_events<E: DeserializeOwned + Send + Sync + 'static>(
    mut commands: Commands,
    receiver: ResMut<IpcEventRawReceiver>,
    mut diagnostics: Option<ResMut<CefIpcDiagnostics>>,
) {
    while let Ok(event) = receiver.0.try_recv() {
        let start = Instant::now();
        if let Ok(payload) = serde_json::from_str::<E>(&event.payload) {
            commands.trigger(Receive {
                webview: event.webview,
                payload,
            });
        }
        if let Some(ref mut diag) = diagnostics {
            diag.last_processing_time = Some(start.elapsed());
        }
    }
}
```

- [ ] **Step 2: Verify workspace compiles**

Run: `cargo check --workspace --all-features`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/common/ipc/js_emit.rs
git commit -m "feat(diagnostics): add IPC processing time recording"
```

---

### Task 6: Implement `CefDiagnosticsPlugin`

**Files:**
- Modify: `src/diagnostics.rs` (add plugin, collection system, webview count system)
- Modify: `src/lib.rs` (add prelude re-export)

- [ ] **Step 1: Implement the full plugin in `src/diagnostics.rs`**

Replace `src/diagnostics.rs` with the full implementation:

```rust
use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::prelude::*;
use bevy_cef_core::prelude::Browsers;
use std::time::Duration;

use crate::common::CefMessageLoopDuration;

/// Intermediate resource for texture transfer diagnostics.
///
/// Written by `send_render_textures`, read and reset by `cef_diagnostics_system`.
#[derive(Resource, Default)]
pub struct CefTextureDiagnostics {
    pub last_transfer_time: Option<Duration>,
    pub total_buffer_bytes: u64,
}

/// Intermediate resource for IPC processing diagnostics.
///
/// Written by `receive_events`, read and reset by `cef_diagnostics_system`.
#[derive(Resource, Default)]
pub struct CefIpcDiagnostics {
    pub last_processing_time: Option<Duration>,
}

/// Cached webview count for diagnostics (bridges NonSend `Browsers` → Send resource).
#[derive(Resource, Default)]
pub struct CefWebviewCount(pub usize);

/// Registers CEF performance diagnostics into Bevy's `DiagnosticsStore`.
///
/// This plugin does NOT include FPS measurement — add
/// `FrameTimeDiagnosticsPlugin` separately if needed.
///
/// # Usage
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
/// # use bevy_cef::prelude::*;
/// # use bevy_cef::diagnostics::CefDiagnosticsPlugin;
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(CefPlugin::default())
///     .add_plugins(CefDiagnosticsPlugin)
///     .add_plugins(FrameTimeDiagnosticsPlugin::default())
///     .add_plugins(LogDiagnosticsPlugin::default());
/// ```
pub struct CefDiagnosticsPlugin;

impl CefDiagnosticsPlugin {
    /// Wall-clock duration of one `cef_do_message_loop_work()` call (ms).
    pub const MESSAGE_LOOP_TIME: DiagnosticPath =
        DiagnosticPath::const_new("cef/message_loop_time");

    /// Elapsed time from CEF `on_paint` to channel receive in `send_render_textures` (ms).
    pub const TEXTURE_TRANSFER_TIME: DiagnosticPath =
        DiagnosticPath::const_new("cef/texture_transfer_time");

    /// Bevy-side IPC receive → EntityEvent trigger processing time (ms).
    pub const IPC_PROCESSING_TIME: DiagnosticPath =
        DiagnosticPath::const_new("cef/ipc_processing_time");

    /// Sum of all texture buffer bytes received from channel in the frame.
    pub const TEXTURE_BUFFER_MEMORY: DiagnosticPath =
        DiagnosticPath::const_new("cef/texture_buffer_memory");

    /// Number of active browser instances.
    pub const WEBVIEW_COUNT: DiagnosticPath = DiagnosticPath::const_new("cef/webview_count");
}

impl Plugin for CefDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(
            Diagnostic::new(Self::MESSAGE_LOOP_TIME).with_suffix("ms"),
        )
        .register_diagnostic(
            Diagnostic::new(Self::TEXTURE_TRANSFER_TIME).with_suffix("ms"),
        )
        .register_diagnostic(
            Diagnostic::new(Self::IPC_PROCESSING_TIME).with_suffix("ms"),
        )
        .register_diagnostic(
            Diagnostic::new(Self::TEXTURE_BUFFER_MEMORY).with_suffix("bytes"),
        )
        .register_diagnostic(Diagnostic::new(Self::WEBVIEW_COUNT))
        .init_resource::<CefMessageLoopDuration>()
        .init_resource::<CefTextureDiagnostics>()
        .init_resource::<CefIpcDiagnostics>()
        .init_resource::<CefWebviewCount>()
        .add_systems(
            Update,
            (update_webview_count, cef_diagnostics_system),
        );
    }
}

fn update_webview_count(browsers: NonSend<Browsers>, mut count: ResMut<CefWebviewCount>) {
    count.0 = browsers.len();
}

fn cef_diagnostics_system(
    mut diagnostics: Diagnostics,
    mut message_loop: ResMut<CefMessageLoopDuration>,
    mut texture: ResMut<CefTextureDiagnostics>,
    mut ipc: ResMut<CefIpcDiagnostics>,
    webview_count: Res<CefWebviewCount>,
) {
    if let Some(duration) = message_loop.0.take() {
        diagnostics.add_measurement(&CefDiagnosticsPlugin::MESSAGE_LOOP_TIME, || {
            duration.as_secs_f64() * 1000.0
        });
    }

    if let Some(duration) = texture.last_transfer_time.take() {
        diagnostics.add_measurement(&CefDiagnosticsPlugin::TEXTURE_TRANSFER_TIME, || {
            duration.as_secs_f64() * 1000.0
        });
    }

    if texture.total_buffer_bytes > 0 {
        let bytes = texture.total_buffer_bytes;
        texture.total_buffer_bytes = 0;
        diagnostics
            .add_measurement(&CefDiagnosticsPlugin::TEXTURE_BUFFER_MEMORY, || bytes as f64);
    }

    if let Some(duration) = ipc.last_processing_time.take() {
        diagnostics.add_measurement(&CefDiagnosticsPlugin::IPC_PROCESSING_TIME, || {
            duration.as_secs_f64() * 1000.0
        });
    }

    diagnostics
        .add_measurement(&CefDiagnosticsPlugin::WEBVIEW_COUNT, || webview_count.0 as f64);
}
```

- [ ] **Step 2: Add module declaration and prelude export in `src/lib.rs`**

In `src/lib.rs`, the `pub mod diagnostics;` was added in Task 4. Now add the prelude re-export:

```rust
pub mod prelude {
    pub use crate::{CefPlugin, RunOnMainThread, common::*, navigation::*, webview::prelude::*};
    pub use crate::diagnostics::CefDiagnosticsPlugin;
    pub use bevy_cef_core::prelude::{CefExtensions, CommandLineConfig};
}
```

- [ ] **Step 3: Verify workspace compiles**

Run: `cargo check --workspace --all-features`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/diagnostics.rs src/lib.rs
git commit -m "feat(diagnostics): implement CefDiagnosticsPlugin with 5 metrics"
```

---

### Task 7: Verify full build and lint

- [ ] **Step 1: Run clippy**

Run: `cargo clippy --workspace --all-features`
Expected: No new warnings from our changes

- [ ] **Step 2: Run fmt**

Run: `cargo fmt --all`

- [ ] **Step 3: Run tests**

Run: `cargo test --workspace --all-features`
Expected: PASS (existing tests still pass)

- [ ] **Step 4: Fix any issues found in steps 1-3**

- [ ] **Step 5: Commit if there were formatting/lint fixes**

```bash
git add -A
git commit -m "chore: fix lint and formatting"
```
