# CefDiagnosticsPlugin Design Spec

## Goal

Add runtime performance diagnostics to bevy_cef using Bevy's standard `DiagnosticsStore` system, exposed as an independent `CefDiagnosticsPlugin` that users explicitly add to their app.

## Non-Goals

- FPS measurement (delegated to Bevy's `FrameTimeDiagnosticsPlugin`)
- Continuous benchmarks / `cargo bench` infrastructure (future work)
- Screen overlay / UI rendering of diagnostics
- Feature flag gating (plugin addition is the opt-in mechanism)

## Plugin API

```rust
pub struct CefDiagnosticsPlugin;

impl CefDiagnosticsPlugin {
    pub const MESSAGE_LOOP_TIME: DiagnosticPath = DiagnosticPath::const_new("cef/message_loop_time");
    pub const TEXTURE_TRANSFER_TIME: DiagnosticPath = DiagnosticPath::const_new("cef/texture_transfer_time");
    pub const IPC_PROCESSING_TIME: DiagnosticPath = DiagnosticPath::const_new("cef/ipc_processing_time");
    pub const TEXTURE_BUFFER_MEMORY: DiagnosticPath = DiagnosticPath::const_new("cef/texture_buffer_memory");
    pub const WEBVIEW_COUNT: DiagnosticPath = DiagnosticPath::const_new("cef/webview_count");
}
```

### Usage

```rust
app.add_plugins((
    FrameTimeDiagnosticsPlugin::default(), // FPS (Bevy standard, user adds separately)
    CefDiagnosticsPlugin,
    LogDiagnosticsPlugin::default(),       // Console output (optional)
));
```

`CefDiagnosticsPlugin` does NOT automatically add `FrameTimeDiagnosticsPlugin`. Users add it themselves if they want FPS metrics. This follows Bevy ecosystem conventions.

## Metrics

| Path | Suffix | Definition | Aggregation (per frame) |
|---|---|---|---|
| `cef/message_loop_time` | `"ms"` | Wall-clock duration of one `cef_do_message_loop_work()` call | Single value per frame (overwrite) |
| `cef/texture_transfer_time` | `"ms"` | Elapsed time from `on_paint` timestamp to channel receive in `send_render_textures()` | Last received value |
| `cef/ipc_processing_time` | `"ms"` | Bevy-side IPC receive → EntityEvent trigger processing time | Last value |
| `cef/texture_buffer_memory` | `"bytes"` | Sum of all texture buffer bytes received from channel in that frame | Sum, reset each frame |
| `cef/webview_count` | (none) | Number of active browser instances in `Browsers` resource | Current value |

### Diagnostic Registration

All diagnostics are registered in `CefDiagnosticsPlugin::build()`:

```rust
impl Plugin for CefDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::MESSAGE_LOOP_TIME).with_suffix("ms"))
           .register_diagnostic(Diagnostic::new(Self::TEXTURE_TRANSFER_TIME).with_suffix("ms"))
           .register_diagnostic(Diagnostic::new(Self::IPC_PROCESSING_TIME).with_suffix("ms"))
           .register_diagnostic(Diagnostic::new(Self::TEXTURE_BUFFER_MEMORY).with_suffix("bytes"))
           .register_diagnostic(Diagnostic::new(Self::WEBVIEW_COUNT))
           .init_resource::<CefTextureDiagnostics>()
           .init_resource::<CefIpcDiagnostics>()
           .init_resource::<CefWebviewCount>()
           .add_systems(Update, (
               update_webview_count,
               cef_diagnostics_system.after(send_render_textures),
           ));
    }
}
```

## Architecture

### Data Flow

```
[Measurement Points]                          [Diagnostics Collection]

on_paint (CEF callback)
  → Instant::now() stamped on
    RenderTextureMessage
  → async_channel::send()
                                              send_render_textures() [Update]
                                                → channel.try_recv()
                                                → elapsed = msg.created_at.elapsed()
                                                → CefTextureDiagnostics {
                                                    last_transfer_time,
                                                    total_buffer_bytes += msg.buffer.len()
                                                  }

cef_do_message_loop_work() [Main]
  → Instant::now() before
  → cef::do_message_loop_work()
  → CefMessageLoopDuration = elapsed

receive_events<E>() [Update]
  → Instant::now() before processing
  → trigger Receive<E> EntityEvent
  → CefIpcDiagnostics {
      last_processing_time = elapsed
    }

update_webview_count [Update, NonSend]
  → Browsers::len()
  → CefWebviewCount

                                              cef_diagnostics_system [Update, after send_render_textures]
                                                → read all intermediate resources
                                                → Diagnostics::add_measurement() for each
                                                → reset intermediate resources
```

### System Schedule

| Schedule | System | Role |
|---|---|---|
| Main | `cef_do_message_loop_work()` (existing, add measurement) | Write loop time to `CefMessageLoopDuration` |
| Update | `send_render_textures()` (existing, add measurement) | Write transfer time and buffer bytes to `CefTextureDiagnostics` |
| Update | `receive_events<E>()` (existing, add measurement) | Write processing time to `CefIpcDiagnostics` |
| Update | `update_webview_count` (new, NonSend) | Read `Browsers::len()` → `CefWebviewCount` |
| Update | `cef_diagnostics_system` (new, after `send_render_textures`) | Read all resources → `Diagnostics::add_measurement()` → reset |

### Intermediate Resources

```rust
// src/common/message_loop.rs
#[derive(Resource, Default)]
pub struct CefMessageLoopDuration(pub Option<Duration>);

// src/diagnostics.rs
#[derive(Resource, Default)]
pub struct CefTextureDiagnostics {
    pub last_transfer_time: Option<Duration>,
    pub total_buffer_bytes: u64,
}

#[derive(Resource, Default)]
pub struct CefIpcDiagnostics {
    pub last_processing_time: Option<Duration>,
}

#[derive(Resource, Default)]
pub struct CefWebviewCount(pub usize);
```

### Reset and Staleness Behavior

After `cef_diagnostics_system` reads each resource:
- `CefMessageLoopDuration.0` → set to `None`
- `CefTextureDiagnostics` → `last_transfer_time = None`, `total_buffer_bytes = 0`
- `CefIpcDiagnostics.last_processing_time` → set to `None`
- `CefWebviewCount` → not reset (always current)

When a value is `None`, `cef_diagnostics_system` skips `add_measurement()` for that metric. This prevents stale or zero values from polluting the diagnostics history.

### NonSend Handling

`Browsers` is a `NonSend` resource. To keep `cef_diagnostics_system` as a regular Send system:

- `update_webview_count` is a separate NonSend system that reads `Browsers::len()` and writes to `CefWebviewCount` (a regular Send `Resource`)
- `cef_diagnostics_system` reads `CefWebviewCount` without touching `Browsers`

### Overhead When Plugin Not Added

Measurement points in existing systems use `Option<ResMut<...>>` to check for resource existence. When `CefDiagnosticsPlugin` is not added, no intermediate resources exist, and the Option resolves to `None` (skipped).

The one unavoidable cost is `Instant::now()` on `RenderTextureMessage` creation (~25ns per call). This is a minimal overhead acceptable for the measurement capability it enables.

## File Changes

| File | Change |
|---|---|
| `src/diagnostics.rs` (new) | `CefDiagnosticsPlugin`, `cef_diagnostics_system`, `update_webview_count`, intermediate resource types |
| `src/lib.rs` | `pub mod diagnostics;`, re-export `CefDiagnosticsPlugin` in prelude |
| `src/common/message_loop.rs` | Add `CefMessageLoopDuration` resource, wrap `cef_do_message_loop_work()` with timing |
| `src/webview/` (send_render_textures) | Add `Option<ResMut<CefTextureDiagnostics>>` param, record transfer time and buffer bytes |
| `src/common/ipc/js_emit.rs` | Add `Option<ResMut<CefIpcDiagnostics>>` param, record processing time |
| `crates/bevy_cef_core/` | Add `created_at: Instant` to `RenderTextureMessage`, expose `Browsers::len()` |

## Deferred Semantics Note

Bevy's `Diagnostics` SystemParam uses a `Deferred` buffer. Measurements recorded in `cef_diagnostics_system` are applied to `DiagnosticsStore` after the system completes. This means values appear in `DiagnosticsStore` with up to one frame delay. Since `LogDiagnosticsPlugin` outputs at 1-second intervals, this delay is not observable in practice.
