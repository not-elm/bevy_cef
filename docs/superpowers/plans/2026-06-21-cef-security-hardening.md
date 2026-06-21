# CEF Security Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `bevy_cef` secure-by-default by removing hardcoded security-relaxing CEF switches, let users opt back in (and have it actually reach the renderer/network process), and make the OS sandbox configurable without changing current defaults.

**Architecture:** Pure, unit-tested helpers do the logic (`resolve_no_sandbox` in the root crate; `effective_command_line_config` + a `switches` constants module in `bevy_cef_core`). `CefPlugin::build` resolves the sandbox + computes one "effective" command-line config, logs warnings, and threads both down through `MessageLoopPlugin` to `cef_initialize`. User switches are forwarded to every CEF child process via the existing `on_before_child_process_launch` hook.

**Tech Stack:** Rust (edition 2024), Bevy 0.18, the `cef` crate (145.6.1), `cargo test` with inline `#[cfg(test)] mod tests`.

## Global Constraints

- **Rust edition 2024.** Workspace crate versions are `0.11.0-dev`.
- **No new dependencies.** Use only what the workspace already pulls in.
- **`resolve_no_sandbox` MUST live in the root `bevy_cef` crate** (`src/`), so `cfg!(feature = "debug")` observes the same `debug` feature as the historical `#[cfg(feature = "debug")]` gate in `message_loop.rs`. Do not move it to `bevy_cef_core`.
- **Switch-name convention is WITHOUT a leading `--`** (CEF's `append_switch` adds dashes). The `--` stripping in `effective_command_line_config` is defensive only.
- **Preserve `SandboxMode::PlatformDefault` parity exactly:** macOS release → sandbox on (`no_sandbox=false`); macOS `debug` feature → off; Windows/Linux → off.
- **macOS builds/runs of examples require `--features debug`** (auto-links the local CEF framework). `cargo build`/`cargo test` of the libraries work without it.
- **Every git commit message must end with these two trailer lines** (blank line before them):
  ```
  Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01GszAQk55h9MmGtzRYmD2p4
  ```
- Follow existing code patterns. Source of truth: `docs/superpowers/specs/2026-06-21-cef-security-hardening-design.md`.

---

## File Structure

| File | Responsibility | Tasks |
|---|---|---|
| `crates/bevy_cef_core/src/browser_process/command_line_config.rs` | `CommandLineConfig`, the `switches` constants module + `risky_present`, `effective_command_line_config`, their tests | 1, 3, 5 |
| `crates/bevy_cef_core/src/browser_process/browser_process_handler.rs` | Forward user switches to children; drop hardcoded risky switches | 2, 3 |
| `crates/bevy_cef_core/src/browser_process/app.rs` | Pass config into the handler builder | 2 |
| `src/common/sandbox.rs` (new) | `SandboxMode` + `resolve_no_sandbox` (+ tests) | 4 |
| `src/common.rs` | Register the `sandbox` module | 4 |
| `src/common/message_loop.rs` | Thread `no_sandbox: bool` into `Settings` | 6 |
| `src/lib.rs` | `CefPlugin.sandbox` field; resolve + effective-config + warnings; wiring; prelude exports | 6 |
| `docs/website/docs/reference/plugin-configuration.md`, `CLAUDE.md`, `CHANGELOG.md` | Docs + migration | 7 |

---

## Task 1: `switches` constants module (bevy_cef_core)

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/command_line_config.rs` (append module + new test module)

**Interfaces:**
- Produces: `bevy_cef_core::prelude::switches` containing `const DISABLE_WEB_SECURITY/ALLOW_RUNNING_INSECURE_CONTENT/IGNORE_CERTIFICATE_ERRORS/IGNORE_SSL_ERRORS: &str`, `const RISKY_SWITCHES: &[&str]`, and `fn risky_present<'a>(switches: &[&'a str]) -> Vec<&'a str>`. (Re-exported automatically via `pub use command_line_config::*` in `browser_process.rs` → core prelude.)

- [ ] **Step 1: Write the failing test**

Append to `crates/bevy_cef_core/src/browser_process/command_line_config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risky_switches_contains_disable_web_security() {
        assert!(switches::RISKY_SWITCHES.contains(&switches::DISABLE_WEB_SECURITY));
    }

    #[test]
    fn risky_present_filters_to_risky_only() {
        let input = [
            switches::DISABLE_WEB_SECURITY,
            "disable-gpu",
            switches::IGNORE_SSL_ERRORS,
        ];
        assert_eq!(
            switches::risky_present(&input),
            vec![switches::DISABLE_WEB_SECURITY, switches::IGNORE_SSL_ERRORS],
        );
    }

    #[test]
    fn risky_present_empty_when_none_risky() {
        let input = ["disable-gpu", "no-zygote"];
        assert!(switches::risky_present(&input).is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p bevy_cef_core --lib command_line_config`
Expected: FAIL — compile error, `cannot find ... switches` / `failed to resolve: use of undeclared ... switches`.

- [ ] **Step 3: Write minimal implementation**

Append to the same file (above the `#[cfg(test)] mod tests`):

```rust
/// Typo-safe constants for common security-relaxing CEF switch names, plus helpers
/// to detect them.
///
/// Use these with [`CommandLineConfig::with_switch`] instead of raw strings:
///
/// ```no_run
/// use bevy_cef_core::prelude::*;
///
/// let config = CommandLineConfig::default().with_switch(switches::DISABLE_WEB_SECURITY);
/// ```
pub mod switches {
    /// Disables the Same-Origin Policy (SOP/CORS). **Insecure** — trusted content only.
    pub const DISABLE_WEB_SECURITY: &str = "disable-web-security";
    /// Allows mixed (HTTP) content on HTTPS pages. **Insecure.**
    pub const ALLOW_RUNNING_INSECURE_CONTENT: &str = "allow-running-insecure-content";
    /// Disables TLS certificate validation. **Insecure** (enables MITM).
    pub const IGNORE_CERTIFICATE_ERRORS: &str = "ignore-certificate-errors";
    /// Disables SSL error handling. **Insecure** (enables MITM).
    pub const IGNORE_SSL_ERRORS: &str = "ignore-ssl-errors";

    /// Switches that relax browser security; their presence triggers a startup warning.
    pub const RISKY_SWITCHES: &[&str] = &[
        DISABLE_WEB_SECURITY,
        ALLOW_RUNNING_INSECURE_CONTENT,
        IGNORE_CERTIFICATE_ERRORS,
        IGNORE_SSL_ERRORS,
    ];

    /// Returns the subset of `switches` present in [`RISKY_SWITCHES`], preserving
    /// order. Names are compared verbatim — normalize a leading `--` first if needed.
    pub fn risky_present<'a>(switches: &[&'a str]) -> Vec<&'a str> {
        switches
            .iter()
            .copied()
            .filter(|s| RISKY_SWITCHES.iter().any(|&r| r == *s))
            .collect()
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p bevy_cef_core --lib command_line_config`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/bevy_cef_core/src/browser_process/command_line_config.rs
git commit -m "feat(security): add switches constants + risky_present helper"
```

---

## Task 2: Forward user switches to CEF child processes (bevy_cef_core)

This wires `CommandLineConfig` into the child-process hook so user switches reach the renderer/network process. It does NOT yet remove the hardcoded risky switches (Task 3) — so this task is behavior-preserving plus additive.

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/browser_process_handler.rs`
- Modify: `crates/bevy_cef_core/src/browser_process/app.rs:107-112` (`browser_process_handler()`)

**Interfaces:**
- Consumes: `CommandLineConfig` (Task 1's file).
- Produces: `BrowserProcessHandlerBuilder::build(requester: Sender<MessageLoopTimer>, config: CommandLineConfig, extensions: CefExtensions) -> BrowserProcessHandler` (new `config` parameter, 2nd position).

- [ ] **Step 1: Add the `config` field + import**

In `browser_process_handler.rs`, change the import line:

```rust
use crate::prelude::{CefExtensions, CommandLineConfig, EXTENSIONS_SWITCH, MessageLoopTimer};
```

Add the field to `BrowserProcessHandlerBuilder`:

```rust
pub struct BrowserProcessHandlerBuilder {
    object: *mut RcImpl<cef_dll_sys::cef_browser_process_handler_t, Self>,
    message_loop_working_requester: Sender<MessageLoopTimer>,
    config: CommandLineConfig,
    extensions: CefExtensions,
}
```

- [ ] **Step 2: Thread `config` through `build` and `Clone`**

Update `build`:

```rust
    pub fn build(
        message_loop_working_requester: Sender<MessageLoopTimer>,
        config: CommandLineConfig,
        extensions: CefExtensions,
    ) -> BrowserProcessHandler {
        BrowserProcessHandler::new(Self {
            object: core::ptr::null_mut(),
            message_loop_working_requester,
            config,
            extensions,
        })
    }
```

Update the `Clone` impl body:

```rust
        Self {
            object,
            message_loop_working_requester: self.message_loop_working_requester.clone(),
            config: self.config.clone(),
            extensions: self.extensions.clone(),
        }
```

- [ ] **Step 3: Forward switches inside `on_before_child_process_launch`**

In `on_before_child_process_launch`, immediately after the `let Some(command_line) = command_line else { return; };` guard and before the existing `append_switch` calls, insert:

```rust
        // Forward user-configured switches to every child process. Chromium enforces
        // CORS / web-security in the network (utility) process under NetworkService,
        // so forwarding to all children — not just the renderer — is required for an
        // opt-in like `disable-web-security` to take effect.
        for switch in &self.config.switches {
            command_line.append_switch(Some(&(*switch).into()));
        }
```

- [ ] **Step 4: Pass `config` from `app.rs`**

In `crates/bevy_cef_core/src/browser_process/app.rs`, update `browser_process_handler()`:

```rust
    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
        Some(BrowserProcessHandlerBuilder::build(
            self.message_loop_working_requester.clone(),
            self.config.clone(),
            self.extensions.clone(),
        ))
    }
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build -p bevy_cef_core --features browser`
Expected: builds with no errors.

- [ ] **Step 6: Manual smoke check (macOS)**

Run: `cargo run --example simple --features debug`
Expected: the webview renders as before (no behavior change yet). Close the window.

- [ ] **Step 7: Commit**

```bash
git add crates/bevy_cef_core/src/browser_process/browser_process_handler.rs crates/bevy_cef_core/src/browser_process/app.rs
git commit -m "feat(security): forward user command-line switches to CEF child processes"
```

---

## Task 3: Secure the defaults (bevy_cef_core)

Remove the hardcoded risky switches from the child hook and relocate the harmless `disable-session-crashed-bubble` into the visible `CommandLineConfig::default()` (now correctly forwarded by Task 2). Drop the malformed `enable-logging=stderr` entirely.

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/browser_process_handler.rs` (remove 7 lines)
- Modify: `crates/bevy_cef_core/src/browser_process/command_line_config.rs` (`Default` + a test)

**Interfaces:**
- Consumes: `switches::RISKY_SWITCHES` (Task 1).

- [ ] **Step 1: Write the failing test**

Add this test to the `#[cfg(test)] mod tests` block in `command_line_config.rs`:

```rust
    #[test]
    fn default_is_secure_and_keeps_session_bubble() {
        let cfg = CommandLineConfig::default();
        assert!(
            cfg.switches.contains(&"disable-session-crashed-bubble"),
            "default should keep the session-crashed bubble suppressed"
        );
        for risky in switches::RISKY_SWITCHES {
            assert!(
                !cfg.switches.contains(risky),
                "default must not enable risky switch: {risky}"
            );
        }
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p bevy_cef_core --lib default_is_secure`
Expected: FAIL — assertion: `default should keep the session-crashed bubble suppressed` (not yet in defaults).

- [ ] **Step 3: Add the bubble to `Default`**

In `command_line_config.rs`, update the `Default` impl's `switches` vec to include the bubble as the first entry:

```rust
            switches: vec![
                // Suppress Chromium's "restore pages?" bubble. Harmless UX, not a
                // security relaxation; visible here so users can override it.
                "disable-session-crashed-bubble",
                #[cfg(all(target_os = "macos", debug_assertions))]
                "use-mock-keychain",
                // Without this Chromium tries to launch a zygote process on Linux even
                // with `no_sandbox: true`, which fails with "No such file or directory"
                // in `ZygoteHostImpl` (see issue #9).
                #[cfg(target_os = "linux")]
                "no-zygote",
            ],
```

- [ ] **Step 4: Remove the hardcoded switches from the child hook**

In `browser_process_handler.rs`, delete these seven lines from `on_before_child_process_launch` (the entire hardcoded block — keep the Task-2 forward loop, the extensions block, and the custom-schemes block):

```rust
        command_line.append_switch(Some(&"disable-web-security".into()));
        command_line.append_switch(Some(&"allow-running-insecure-content".into()));
        command_line.append_switch(Some(&"disable-session-crashed-bubble".into()));
        command_line.append_switch(Some(&"ignore-certificate-errors".into()));
        command_line.append_switch(Some(&"ignore-ssl-errors".into()));
        command_line.append_switch(Some(&"enable-logging=stderr".into()));
        command_line.append_switch(Some(&"disable-web-security".into()));
```

- [ ] **Step 5: Run test + verify the switches are gone**

Run: `cargo test -p bevy_cef_core --lib command_line_config`
Expected: PASS (all tests, including `default_is_secure_and_keeps_session_bubble`).

Run: `rg -n "disable-web-security|allow-running-insecure-content|ignore-certificate-errors|ignore-ssl-errors|enable-logging=stderr" crates/bevy_cef_core/src/browser_process/browser_process_handler.rs`
Expected: NO matches (empty output).

- [ ] **Step 6: Manual smoke check (macOS)**

Run: `cargo run --example simple --features debug`
Expected: the webview still renders correctly without the risky switches (proves BRP/IPC + localhost CORS don't need SOP disabled). Close the window.

- [ ] **Step 7: Commit**

```bash
git add crates/bevy_cef_core/src/browser_process/browser_process_handler.rs crates/bevy_cef_core/src/browser_process/command_line_config.rs
git commit -m "feat(security)!: stop enabling disable-web-security and other risky switches by default"
```

---

## Task 4: `SandboxMode` + `resolve_no_sandbox` (root crate)

**Files:**
- Create: `src/common/sandbox.rs`
- Modify: `src/common.rs`

**Interfaces:**
- Produces: `crate::common::SandboxMode` (enum: `PlatformDefault` (default) / `Enabled` / `Disabled`, derives `Clone, Copy, Debug, Default, PartialEq, Eq`) and `crate::common::resolve_no_sandbox(mode: SandboxMode) -> bool`. Re-exported into the root prelude via `common::*`.

- [ ] **Step 1: Create the module with a deliberately-wrong stub + tests**

Create `src/common/sandbox.rs`:

```rust
//! Configuration for Chromium's OS-level process sandbox.

/// Controls Chromium's OS-level sandbox (`Settings.no_sandbox`).
///
/// The default, [`SandboxMode::PlatformDefault`], reproduces bevy_cef's existing
/// per-platform behavior: sandbox **on** for macOS release builds; **off** for
/// macOS debug builds, Windows, and Linux.
///
/// Enabling the sandbox is best-effort and platform-specific (Linux needs a SUID
/// `chrome-sandbox` helper; macOS needs a `cef_sandbox`-linked render process).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SandboxMode {
    /// Preserve bevy_cef's current per-platform default.
    #[default]
    PlatformDefault,
    /// Force the Chromium sandbox ON (`no_sandbox = false`).
    Enabled,
    /// Force the Chromium sandbox OFF (`no_sandbox = true`).
    Disabled,
}

/// Resolves a [`SandboxMode`] to the `no_sandbox` boolean passed to CEF `Settings`.
///
/// Lives in the root `bevy_cef` crate so `cfg!(feature = "debug")` observes the same
/// `debug` feature value as the historical `#[cfg(feature = "debug")]` gate in
/// `message_loop.rs`.
pub fn resolve_no_sandbox(mode: SandboxMode) -> bool {
    // INTENTIONALLY WRONG STUB — replaced in Step 3.
    let _ = mode;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_means_sandbox_on() {
        assert!(!resolve_no_sandbox(SandboxMode::Enabled));
    }

    #[test]
    fn disabled_means_no_sandbox() {
        assert!(resolve_no_sandbox(SandboxMode::Disabled));
    }

    #[test]
    fn platform_default_matches_current_matrix() {
        let expected = {
            #[cfg(target_os = "macos")]
            {
                cfg!(feature = "debug")
            }
            #[cfg(not(target_os = "macos"))]
            {
                true
            }
        };
        assert_eq!(resolve_no_sandbox(SandboxMode::PlatformDefault), expected);
    }
}
```

Register the module in `src/common.rs` — add `mod sandbox;` (after `mod message_loop;`) and `pub use sandbox::*;` (after `pub use message_loop::*;`):

```rust
mod components;
mod dpi;
mod ipc;
pub(crate) mod localhost;
mod message_loop;
mod sandbox;

pub use components::*;
pub use dpi::WebviewDpiPlugin;
pub use ipc::*;
pub(crate) use localhost::*;
pub use message_loop::*;
pub use sandbox::*;
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p bevy_cef --lib sandbox`
Expected: FAIL — `enabled_means_sandbox_on` fails (stub returns `true`, expected `false`).

- [ ] **Step 3: Replace the stub with the real implementation**

In `src/common/sandbox.rs`, replace the `resolve_no_sandbox` body:

```rust
pub fn resolve_no_sandbox(mode: SandboxMode) -> bool {
    match mode {
        SandboxMode::Enabled => false,
        SandboxMode::Disabled => true,
        SandboxMode::PlatformDefault => {
            #[cfg(target_os = "macos")]
            {
                cfg!(feature = "debug")
            }
            #[cfg(not(target_os = "macos"))]
            {
                true
            }
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p bevy_cef --lib sandbox`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add src/common/sandbox.rs src/common.rs
git commit -m "feat(security): add SandboxMode + resolve_no_sandbox"
```

---

## Task 5: `effective_command_line_config` (bevy_cef_core)

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/command_line_config.rs` (add fn + tests to existing test module)

**Interfaces:**
- Produces: `bevy_cef_core::prelude::effective_command_line_config(config: &CommandLineConfig, strip_no_zygote: bool) -> CommandLineConfig`. Normalizes (`--` strip), de-duplicates switches, optionally drops `no-zygote`; passes `switch_values` through unchanged.

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)] mod tests` block in `command_line_config.rs`:

```rust
    #[test]
    fn effective_dedups_and_normalizes() {
        let cfg = CommandLineConfig {
            switches: vec!["--disable-gpu", "disable-gpu", "no-zygote"],
            switch_values: vec![("remote-debugging-port", "9222")],
        };
        let eff = effective_command_line_config(&cfg, false);
        assert_eq!(eff.switches, vec!["disable-gpu", "no-zygote"]);
        assert_eq!(eff.switch_values, vec![("remote-debugging-port", "9222")]);
    }

    #[test]
    fn effective_strips_no_zygote_when_requested() {
        let cfg = CommandLineConfig {
            switches: vec!["no-zygote", "disable-gpu"],
            switch_values: vec![],
        };
        let eff = effective_command_line_config(&cfg, true);
        assert_eq!(eff.switches, vec!["disable-gpu"]);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p bevy_cef_core --lib effective`
Expected: FAIL — compile error, `cannot find function effective_command_line_config`.

- [ ] **Step 3: Write the implementation**

Add to `command_line_config.rs` (e.g. directly after the `impl CommandLineConfig` block, above the `switches` module):

```rust
/// Produces the *effective* command-line config handed to CEF.
///
/// - Strips a single leading `--` from each switch name.
/// - De-duplicates switches, preserving first-seen order.
/// - When `strip_no_zygote` is true, drops the `no-zygote` switch (used on Linux
///   when the sandbox is enabled, since the zygote is part of the sandbox model).
///
/// `switch_values` are passed through unchanged.
pub fn effective_command_line_config(
    config: &CommandLineConfig,
    strip_no_zygote: bool,
) -> CommandLineConfig {
    let mut switches: Vec<&'static str> = Vec::new();
    for &switch in &config.switches {
        let normalized = switch.strip_prefix("--").unwrap_or(switch);
        if strip_no_zygote && normalized == "no-zygote" {
            continue;
        }
        if !switches.contains(&normalized) {
            switches.push(normalized);
        }
    }
    CommandLineConfig {
        switches,
        switch_values: config.switch_values.clone(),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p bevy_cef_core --lib command_line_config`
Expected: PASS (all tests in the module).

- [ ] **Step 5: Commit**

```bash
git add crates/bevy_cef_core/src/browser_process/command_line_config.rs
git commit -m "feat(security): add effective_command_line_config (dedup/normalize/strip no-zygote)"
```

---

## Task 6: Wire sandbox + effective config + warnings into the plugin

**Files:**
- Modify: `src/lib.rs` (imports, `CefPlugin` struct, `build`, prelude)
- Modify: `src/common/message_loop.rs` (`MessageLoopPlugin` struct + both `cef_initialize` fns)

**Interfaces:**
- Consumes: `resolve_no_sandbox`, `SandboxMode` (Task 4); `effective_command_line_config`, `switches` (Tasks 1, 5).
- Produces: `CefPlugin.sandbox: SandboxMode` (public field); `SandboxMode` + `switches` in the root prelude.

- [ ] **Step 1: Add `no_sandbox` to `MessageLoopPlugin` and `cef_initialize`**

In `src/common/message_loop.rs`, add the field to the struct:

```rust
pub struct MessageLoopPlugin {
    pub config: CommandLineConfig,
    pub extensions: CefExtensions,
    pub root_cache_path: Option<String>,
    pub no_sandbox: bool,
}
```

Update the **macOS** `cef_initialize` signature and its `Settings`:

```rust
#[cfg(target_os = "macos")]
fn cef_initialize(
    args: &Args,
    cef_app: &mut cef::App,
    root_cache_path: Option<&str>,
    no_sandbox: bool,
) {
```

…and in that function's `Settings { .. }`, replace the line
`#[cfg(feature = "debug")]` / `no_sandbox: true as _,` with (no cfg gate):

```rust
        no_sandbox: no_sandbox as _,
```

Update the **non-macOS** `cef_initialize` signature and its `Settings`:

```rust
#[cfg(not(target_os = "macos"))]
fn cef_initialize(
    args: &Args,
    cef_app: &mut cef::App,
    root_cache_path: Option<&str>,
    render_process_binary: Option<&std::path::Path>,
    no_sandbox: bool,
) {
```

…and replace `no_sandbox: true as _,` with:

```rust
        no_sandbox: no_sandbox as _,
```

- [ ] **Step 2: Pass `self.no_sandbox` at the two call sites**

In `MessageLoopPlugin::build`, update the macOS call:

```rust
        #[cfg(target_os = "macos")]
        cef_initialize(&args, &mut cef_app, self.root_cache_path.as_deref(), self.no_sandbox);
```

and the non-macOS call:

```rust
        #[cfg(not(target_os = "macos"))]
        cef_initialize(
            &args,
            &mut cef_app,
            self.root_cache_path.as_deref(),
            render_process_binary.as_deref(),
            self.no_sandbox,
        );
```

- [ ] **Step 3: Update `src/lib.rs` imports**

Replace the `bevy_cef_core::prelude` import (currently line 29):

```rust
use bevy_cef_core::prelude::{
    CefCustomScheme, CefExtensions, CommandLineConfig, effective_command_line_config, switches,
};
```

Add `SandboxMode` and `resolve_no_sandbox` to the `crate::common` import:

```rust
use crate::common::{
    LocalHostPlugin, MessageLoopPlugin, SandboxMode, WebviewCoreComponentsPlugin, WebviewDpiPlugin,
    resolve_no_sandbox,
};
```

- [ ] **Step 4: Add the `sandbox` field to `CefPlugin`**

```rust
#[derive(Default)]
pub struct CefPlugin {
    pub command_line_config: CommandLineConfig,
    pub extensions: CefExtensions,
    /// Root directory for CEF runtime data (cache, profiles, etc.).
    /// If empty, defaults to the executable's directory.
    /// Should be set to a user-writable path (e.g. `~/.myapp/cef_data`).
    pub root_cache_path: Option<String>,
    /// Custom URL schemes to register in addition to the built-in
    /// `cef://localhost/`. Each carries a handler that services requests.
    pub custom_schemes: Vec<CefCustomScheme>,
    /// Controls Chromium's OS-level sandbox. Defaults to the current per-platform
    /// behavior; see [`SandboxMode`].
    pub sandbox: SandboxMode,
}
```

- [ ] **Step 5: Resolve, compute effective config, warn, and wire in `build`**

In `CefPlugin::build`, replace the body from `init_registered_schemes(...)` up to (but not including) `app.add_plugins((` with:

```rust
        // NOTE: Must run before MessageLoopPlugin::build, which calls cef_initialize.
        // CEF's OnRegisterCustomSchemes fires during initialize; schemes registered
        // afterward are silently ignored.
        bevy_cef_core::prelude::init_registered_schemes(self.custom_schemes.clone());

        // Resolve the sandbox decision and compute the effective command line once.
        let no_sandbox = resolve_no_sandbox(self.sandbox);
        let strip_no_zygote = cfg!(target_os = "linux") && !no_sandbox;
        let effective_config =
            effective_command_line_config(&self.command_line_config, strip_no_zygote);

        // Warn when any security-relaxing switch is active (based on the EFFECTIVE set).
        let risky = switches::risky_present(&effective_config.switches);
        if !risky.is_empty() {
            warn!(
                "bevy_cef: web-security relaxations active: {risky:?}. \
                 Only enable these when loading fully trusted content."
            );
        }

        // Warn when the sandbox is requested but its prerequisites are absent on macOS
        // (the render process is not linked against cef_sandbox).
        #[cfg(target_os = "macos")]
        if self.sandbox == SandboxMode::Enabled {
            warn!(
                "bevy_cef: SandboxMode::Enabled requested on macOS, but the render \
                 process is not linked against cef_sandbox / does not call \
                 cef_sandbox_initialize(); the sandbox will not function and may abort. \
                 See the plugin-configuration docs."
            );
        }
```

Then update the `MessageLoopPlugin` entry inside `app.add_plugins((` to use the effective config + resolved `no_sandbox`:

```rust
            MessageLoopPlugin {
                config: effective_config,
                extensions: self.extensions.clone(),
                root_cache_path: self.root_cache_path.clone(),
                no_sandbox,
            },
```

(Leave all other sub-plugins and the trailing `RemotePlugin` block unchanged.)

- [ ] **Step 6: Export `switches` from the root prelude**

In `src/lib.rs`'s `pub mod prelude`, add `switches` to the `bevy_cef_core::prelude` re-export (`SandboxMode` already arrives via `common::*`):

```rust
    pub use bevy_cef_core::prelude::{
        CefCustomScheme, CefExtensions, CefSchemeBody, CefSchemeHandler, CefSchemeOptions,
        CefSchemeRequest, CefSchemeResponse, CommandLineConfig, switches,
    };
```

- [ ] **Step 7: Verify it compiles (both feature paths)**

Run: `cargo build --features debug`
Expected: builds with no errors.

Run: `cargo build`
Expected: builds with no errors (release library-loader path).

- [ ] **Step 8: Manual parity + warning check (macOS)**

Run: `cargo run --example simple --features debug`
Expected: renders exactly as before (PlatformDefault parity). No "web-security relaxations" warning appears (default config is clean).

Temporarily edit `examples/simple.rs` to set
`command_line_config: CommandLineConfig::default().with_switch(switches::DISABLE_WEB_SECURITY),`
in the `CefPlugin { .. }`, run again, and confirm the startup `WARN bevy_cef: web-security relaxations active: ["disable-web-security"]...` line appears. **Revert the example edit afterward.**

- [ ] **Step 9: Commit**

```bash
git add src/lib.rs src/common/message_loop.rs
git commit -m "feat(security): make sandbox configurable + warn on risky switches via CefPlugin"
```

---

## Task 7: Documentation, migration, CHANGELOG

**Files:**
- Modify: `docs/website/docs/reference/plugin-configuration.md`
- Modify: `CLAUDE.md`
- Modify: `CHANGELOG.md`

**Interfaces:** none (docs only).

- [ ] **Step 1: Fix and extend `plugin-configuration.md`**

Replace the stale `CefPlugin` struct block (it currently lists only 3 fields and omits `custom_schemes`/`sandbox`):

```rust
pub struct CefPlugin {
    pub command_line_config: CommandLineConfig,
    pub extensions: CefExtensions,
    pub root_cache_path: Option<String>,
    pub custom_schemes: Vec<CefCustomScheme>,
    pub sandbox: SandboxMode,
}
```

Replace the stale `command_line_config` example (it uses the nonexistent `CommandLineConfig::new()`/`.arg(...)`) with the real API:

```rust
use bevy_cef::prelude::*;

let plugin = CefPlugin {
    command_line_config: CommandLineConfig::default()
        .with_switch("disable-gpu")
        .with_switch_value("remote-debugging-port", "9222"),
    ..default()
};
```

Add a new subsection after `### command_line_config`:

```markdown
### Security: relaxing web security

By default bevy_cef enables **no** security-relaxing switches. If you load fully
trusted content and need to disable the Same-Origin Policy (or ignore TLS errors),
opt in explicitly using the typed constants in `switches`:

​```rust
use bevy_cef::prelude::*;

let plugin = CefPlugin {
    command_line_config: CommandLineConfig::default()
        .with_switch(switches::DISABLE_WEB_SECURITY),
    ..default()
};
​```

Opt-in switches are forwarded to every CEF child process (including the network
process, where CORS is enforced under Chromium's NetworkService). When any switch in
`switches::RISKY_SWITCHES` is active, bevy_cef logs a one-time warning at startup.
Only enable these for trusted content — disabling web security exposes your app to
cross-origin attacks.
```

Add a new subsection after `### root_cache_path`:

```markdown
### sandbox

`sandbox: SandboxMode` controls Chromium's OS-level sandbox. The default,
`SandboxMode::PlatformDefault`, preserves bevy_cef's existing behavior: sandbox on
for macOS release builds; off for macOS debug builds, Windows, and Linux.

​```rust
use bevy_cef::prelude::*;

let plugin = CefPlugin {
    sandbox: SandboxMode::Disabled, // or ::Enabled, ::PlatformDefault
    ..default()
};
​```

Enabling the sandbox is **best-effort and platform-specific**: Linux needs a
SUID-root `chrome-sandbox` helper (bevy_cef drops its default `no-zygote` switch when
the sandbox is on); macOS additionally requires the render process to link
`cef_sandbox` and call `cef_sandbox_initialize()`, which bevy_cef does not yet do —
so `SandboxMode::Enabled` on macOS currently logs a warning and will not produce a
working sandbox.
```

(The `​```rust` fences above use a zero-width char only to render in this plan — write normal ```` ```rust ```` fences in the actual file.)

- [ ] **Step 2: Add a CHANGELOG entry**

First read `CHANGELOG.md` to match its existing heading style, then add (at the top, under the current unreleased/dev section):

```markdown
### Security
- **Breaking:** bevy_cef no longer enables `disable-web-security`,
  `ignore-certificate-errors`, `ignore-ssl-errors`, or
  `allow-running-insecure-content` by default. Apps that relied on these must opt in
  explicitly, e.g. `CommandLineConfig::default().with_switch(switches::DISABLE_WEB_SECURITY)`.
  User-supplied switches are now forwarded to CEF child processes (so the opt-in
  reaches the renderer/network process), and a startup warning is logged whenever a
  security-relaxing switch is active.
- Added `CefPlugin::sandbox: SandboxMode` to control Chromium's OS-level sandbox.
  `SandboxMode::PlatformDefault` (the default) preserves current per-platform behavior.
```

- [ ] **Step 3: Update `CLAUDE.md`**

Add a bullet under the `### Key Non-Obvious Patterns` section:

```markdown
- **Secure-by-default switches**: no security-relaxing CEF switches are enabled by default. Users opt into `disable-web-security` etc. via `CommandLineConfig::default().with_switch(switches::DISABLE_WEB_SECURITY)`; opt-in switches are forwarded to all CEF child processes (CORS is enforced in the network process). `CefPlugin::sandbox: SandboxMode` controls the OS sandbox (`PlatformDefault` preserves per-platform behavior; enabling is best-effort and needs platform setup).
```

- [ ] **Step 4: Verify the docs no longer reference the dead API**

Run: `rg -n "CommandLineConfig::new\(\)|\.arg\(" docs/website/docs/reference/plugin-configuration.md`
Expected: NO matches.

- [ ] **Step 5: Commit**

```bash
git add docs/website/docs/reference/plugin-configuration.md CLAUDE.md CHANGELOG.md
git commit -m "docs(security): document secure-by-default switches, opt-in, and SandboxMode"
```

---

## Self-Review

**Spec coverage** (against `2026-06-21-cef-security-hardening-design.md`):
- §4.1 remove risky switches + relocate bubble + drop enable-logging → Task 3 ✓
- §4.2 forward `switches` to children, `switch_values` browser-only → Task 2 (forward loop) ✓; `switch_values` untouched in `effective_command_line_config` and only applied browser-side in `app.rs` (unchanged) ✓
- §4.3 `switches` constants + `RISKY_SWITCHES` + startup warning → Tasks 1, 6 ✓
- §4.3 effective config (dedup/normalize/strip no-zygote) → Tasks 5, 6 ✓
- §4.4 `SandboxMode` + `resolve_no_sandbox` + threading + Linux no-zygote strip + macOS prerequisite guard → Tasks 4, 6 ✓
- §5 public API (SandboxMode, sandbox field, switches in prelude) → Task 6 ✓
- §6 migration + §7 docs → Task 7 ✓

**Placeholder scan:** every code/step shows concrete content; commands have expected output. No TBD/TODO. ✓

**Type consistency:** `effective_command_line_config(&CommandLineConfig, bool) -> CommandLineConfig`, `resolve_no_sandbox(SandboxMode) -> bool`, `switches::risky_present(&[&str]) -> Vec<&str>`, `BrowserProcessHandlerBuilder::build(Sender, CommandLineConfig, CefExtensions)`, `MessageLoopPlugin { config, extensions, root_cache_path, no_sandbox }` — names/signatures match across Tasks 1–7. ✓
