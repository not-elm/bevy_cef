# Windows Support MVP

## Overview

Add Windows support to `bevy_cef` so CEF webviews work on Windows. Use the same `external_message_pump` architecture as macOS for consistency. Leverage the existing `export-cef-dir` tool from cef-rs for CEF binary setup. Target: getting the `simple` example running end-to-end on Windows.

## Goals

- CEF initializes and creates an offscreen browser on Windows
- Webview content renders to a Bevy texture on a 3D quad
- Keyboard and mouse input forwarded to CEF
- The `simple` example runs end-to-end
- Developer setup documented using `export-cef-dir`

## Non-Goals

- Feature parity with macOS (IPC, DevTools, local assets can come later)
- CI that exercises actual CEF functionality on Windows
- Release packaging/distribution tooling (future work)
- Linux support (separate effort)

## Developer Setup Workflow (Windows)

### Makefile target

```makefile
setup-windows:
	cargo install export-cef-dir --version 144.4.0 --force
	export-cef-dir --force $(USERPROFILE)/.local/share/cef
```

Users run `make setup-windows`, then configure their shell environment.

### Full workflow

1. `make setup-windows`
2. Add to shell profile:
   ```powershell
   $env:CEF_PATH = "$env:USERPROFILE/.local/share/cef"
   $env:PATH += ";$env:CEF_PATH"
   ```
3. `cargo run --example simple`

### Key files needed at runtime

Resolved via PATH or next to exe:
- `libcef.dll`, `chrome_elf.dll`, `icudtl.dat`, `v8_context_snapshot.bin`
- `locales/*.pak`, `chrome_100_percent.pak`, `chrome_200_percent.pak`, `resources.pak`

Note: `cef-dll-sys` build script can also download CEF to its `OUT_DIR` if `CEF_PATH` isn't set, but this is slower for repeated builds.

## Technical Approach

### Message Loop

Use `external_message_pump: true` on all platforms (same as macOS). Call `CefDoMessageLoopWork()` each frame. This avoids threading complications with NonSend resources and keeps the architecture consistent.

### Library Loading

On macOS, `LibraryLoader`/`DebugLibraryLoader` explicitly loads the CEF framework. On Windows, no explicit loading is needed — `libcef.dll` is found via standard DLL search order (PATH). The library loader code in `message_loop.rs` is gated to macOS only.

### Subprocess (Render Process)

On Windows, use the same executable as both browser and render process. Call `execute_process()` early — if it returns >= 0, we're a subprocess and should exit. If it returns -1, we're the browser process and continue normally. No separate render process binary needed.

### Window Info

The `#[cfg(target_os = "macos")]` gate on `parent_view` in `browsers.rs` needs to be expanded to include Windows. The Win32 handle path already exists in the match arm — it just needs to compile on Windows.

### Settings (message_loop.rs)

- `no_sandbox: true` — already gated for Windows
- `multi_threaded_message_loop: false` — keep as-is (all platforms)
- `external_message_pump: true` — keep as-is (all platforms)
- Skip `framework_dir_path` and `browser_subprocess_path` on Windows (already macOS+debug only)

## Key File Changes

### `src/common/message_loop.rs` — Heaviest changes
- Gate library loader to macOS only (mostly done already)
- Add early `execute_process()` call for Windows subprocess detection
- Verify `CefDoMessageLoopWork()` works on Windows with `external_message_pump`
- NSApplication hooks already macOS-gated — no change needed

### `crates/bevy_cef_core/src/browser_process/browsers.rs` — Window info
- Change `#[cfg(target_os = "macos")]` on `parent_view` to include Windows, or remove the gate entirely
- The `Win32(handle)` match arm already exists

### `crates/bevy_cef_core/src/browser_process/display_handler.rs` — Cursor
- Windows cursor stub (`HICON__`) already exists — leave as no-op for MVP

### `crates/bevy_cef_core/src/render_process.rs` — Render process entry
- Library loading gated to macOS only (already correct)

### `crates/bevy_cef_core/src/lib.rs` — Module gates
- `debug` module stays macOS-only (appropriate)
- Verify all other modules compile on Windows

### `Makefile` — Add `setup-windows` target

No new crates needed. No new dependencies expected beyond what's already in the workspace.

## Implementation Order

1. **Makefile** — Add `setup-windows` target (quick win, enables testing everything else)
2. **`browsers.rs`** — Remove/expand the macOS-only gate on `parent_view` so WindowInfo compiles on Windows
3. **`message_loop.rs`** — Add subprocess detection via `execute_process()` on Windows; verify settings work
4. **Compile check** — `cargo check --target x86_64-pc-windows-msvc` to catch remaining platform gates
5. **Run `simple` example** — End-to-end test on Windows
6. **Documentation** — Update README and installation docs with Windows setup instructions

## Open Questions

- Does `cef-dll-sys` with the `sandbox` feature link correctly on Windows? (If not, gate the feature to macOS only)
- Does the manifest file (`cefsimple.exe.manifest`) need to be present, or is that only for CEF's own sample apps?
- Will `external_message_pump` + `CefDoMessageLoopWork()` perform well enough on Windows, or will we need to revisit `multi_threaded_message_loop` later?

---
*Generated via /brainstorm on 2026-03-04, refined 2026-03-05*
