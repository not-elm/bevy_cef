# CEF Security Hardening: Secure-by-default Switches + Configurable Sandbox

- **Date:** 2026-06-21
- **Status:** Draft (pending review)
- **Crate(s):** `bevy_cef` (root), `bevy_cef_core`
- **Breaking change:** Yes (acceptable at `0.11.0-dev`, pre-1.0)

## 1. Problem Statement

`bevy_cef` unconditionally enables several security-relaxing Chromium command-line
switches, and the OS-level sandbox is disabled on most platforms. Library users
cannot control these from the outside. We want to:

1. **Eliminate insecure-by-default behavior** — stop forcing `disable-web-security`
   and related risk-relaxing switches on every embedding application.
2. **Let users opt back in** to `disable-web-security` (and the other risky
   switches) from the library-consumer side, via the existing command-line switch
   mechanism — and make that opt-in actually reach the renderer process where it
   takes effect.
3. **Make the sandbox configurable** without changing today's per-platform
   defaults.

## 2. Current Behavior (precise baseline)

### 2.1 Hardcoded command-line switches

`crates/bevy_cef_core/src/browser_process/browser_process_handler.rs`,
`ImplBrowserProcessHandler::on_before_child_process_launch` (~lines 60–94)
unconditionally appends the following switches to **every CEF child process**
(GPU, renderer, utility/Network Service):

| Switch | Risk | Notes |
| --- | --- | --- |
| `disable-web-security` | **High** — disables the Same-Origin Policy | Appended **twice** (lines 65 & 71) — redundant |
| `allow-running-insecure-content` | **Medium** — mixed HTTP content on HTTPS | |
| `ignore-certificate-errors` | **High** — disables TLS certificate validation (MITM) | |
| `ignore-ssl-errors` | **High** — disables SSL error handling (MITM) | |
| `disable-session-crashed-bubble` | None (UX) | Harmless; suppresses "restore pages" bubble |
| `enable-logging=stderr` | None | **Malformed**: uses `append_switch` for a `key=value` pair instead of `append_switch_with_value("enable-logging", "stderr")` |

The same hook also injects the extensions JSON (`EXTENSIONS_SWITCH`) and custom
schemes JSON (`CUSTOM_SCHEMES_SWITCH`); both are functional, not security
relaxations, and must remain.

### 2.2 User command-line config reaches the browser process only

`CommandLineConfig` (`crates/bevy_cef_core/src/browser_process/command_line_config.rs`)
holds `switches: Vec<&'static str>` and `switch_values: Vec<(&'static str, &'static str)>`.
It is applied in `BrowserProcessAppBuilder::on_before_command_line_processing`
(`crates/bevy_cef_core/src/browser_process/app.rs` ~lines 66–82).

In this codebase's multi-process architecture the render process is normally a
**separate executable** (`bevy_cef_render_process` / debug variant) with its own
`CefApp`. On non-macOS, when no dedicated render binary is installed, CEF instead
relaunches the *main* executable as the subprocess (`early_exit_if_subprocess`); the
forwarding design still holds because the subprocess reads switches from its own argv.
As the in-code comment notes, `on_before_command_line_processing` here only runs
for the **browser process** (`process_type` is always `None`). Therefore switches
a user adds to `CommandLineConfig` today **never reach the renderer**. A switch
like `disable-web-security`, which is read by the renderer, must instead be
forwarded via `on_before_child_process_launch`. This is the mechanism the
hardcoded block already uses.

`CommandLineConfig::default()` currently contains only:
- `use-mock-keychain` (macOS + `debug_assertions`)
- `no-zygote` (Linux — required because `no_sandbox: true`, avoids the
  `chrome-sandbox` dependency; see issue #9)

### 2.3 Sandbox (`Settings.no_sandbox`)

Set in `src/common/message_loop.rs`, two separate `cef_initialize` paths:

| Platform / build | `no_sandbox` | Sandbox |
| --- | --- | --- |
| macOS **release** | unset → `false` | **ON** |
| macOS **debug** (`feature = "debug"`) | `true` | OFF |
| Windows | `true` | OFF |
| Linux | `true` (+ `no-zygote` default) | OFF |

"Keep the current default" therefore means **preserve this exact per-platform
matrix**, not force `no_sandbox = true` everywhere.

### 2.4 Why the risky switches appear unnecessary

- **BRP/IPC does not use HTTP/`fetch`.** `window.cef.brp()` → V8 native `__cef_brp`
  → CEF process message (`PROCESS_MESSAGE_BRP`) → in-process `bevy_remote`. No
  cross-origin network request is involved, so the Same-Origin Policy is irrelevant
  to it.
- **The `cef://localhost/` scheme already sends permissive CORS headers**
  (`Access-Control-Allow-Origin: *`, plus `Allow-Methods`/`Allow-Headers: *`) from
  `crates/bevy_cef_core/src/browser_process/localhost/headers_responser.rs`
  (~lines 21–26). Local-asset fetching works without disabling web security. (Note: a
  wildcard `Access-Control-Allow-Origin: *` does not cover credentialed requests,
  cross-origin `<canvas>` pixel reads, or X-Frame-Options — it is not a full SOP
  substitute, but covers the documented flows.)
- **Custom schemes** already expose `CORS_ENABLED` / `FETCH_ENABLED` options.
- **No example or doc** performs a cross-origin `fetch`/`XMLHttpRequest` or
  documents `disable-web-security` as required.

These switches have existed since the INIT commit and appear to be inherited
offscreen-rendering boilerplate rather than a justified requirement.

## 3. Goals / Non-Goals

### Goals
- Secure-by-default: no risk-relaxing switch is enabled unless the user asks.
- A user opt-in (`disable-web-security`, etc.) via the existing
  `CommandLineConfig` switch mechanism that **actually reaches the renderer**.
- Guardrails: typo-safe named constants + a one-time startup warning when any
  risk-relaxing switch is active.
- A configurable sandbox whose default reproduces today's per-platform behavior.

### Non-Goals
- Enabling the sandbox by default (deferred; requires platform helper/entitlement
  work). We only make it *possible* to enable.
- A typed `WebSecurityConfig` struct of booleans (explicitly rejected in favor of
  the generic switch + guardrails approach).
- Changing the IPC/BRP transport or the localhost CORS behavior.

## 4. Design

### 4.1 Remove risky switches from the hardcoded block

In `on_before_child_process_launch`, delete the four risk-relaxing switches
(`disable-web-security` ×2, `allow-running-insecure-content`,
`ignore-certificate-errors`, `ignore-ssl-errors`). Keep the extensions and
custom-schemes injection unchanged.

Relocate the two harmless switches:
- `disable-session-crashed-bubble` → add to `CommandLineConfig::default()` so it
  remains active but is now visible and overridable.
- `enable-logging=stderr` → **drop from defaults** (it is malformed and noisy).
  Users can re-enable correctly with
  `with_switch_value("enable-logging", "stderr")`.

### 4.2 Forward user switches to child processes (the core fix)

`BrowserProcessHandlerBuilder` currently holds only the message-loop requester
and `extensions`. Give it the `CommandLineConfig` too:

- `BrowserProcessAppBuilder` already stores `config: CommandLineConfig`. In
  `browser_process_handler()` (`app.rs` ~lines 107–112), pass `self.config.clone()`
  into `BrowserProcessHandlerBuilder::build(...)`.
- In `on_before_child_process_launch`, after the early-return guard, append every
  flag in `self.config.switches` to the child command line:
  ```rust
  for switch in &self.config.switches {
      command_line.append_switch(Some(&(*switch).into()));
  }
  ```

Propagation rule:
- **`switches` (flags)** → applied to **both** the browser process
  (`on_before_command_line_processing`, unchanged) **and** every child process
  (`on_before_child_process_launch`, new). Under Chromium's NetworkService, the
  Same-Origin/CORS enforcement that `disable-web-security` relaxes lives in the
  **network/utility process**, not the renderer — so forwarding to *all* children is
  what makes the opt-in actually take effect. (Verified: CEF issue #3058.)
- **`switch_values`** → remain **browser-process only** (unchanged). Forwarding
  values such as `remote-debugging-port` to every child would cause
  port/path collisions.

Rationale for "all children" rather than renderer-only: it mirrors the existing
hardcoded behavior, keeps the code simple, and flag-style switches are ignored by
processes they do not apply to. (Renderer-only targeting via the child's `type`
switch is possible but adds complexity for no concrete benefit here.)

### 4.3 Guardrails

In `command_line_config.rs`, add a `switches` submodule of typo-safe constants,
exported via the prelude:

```rust
pub mod switches {
    pub const DISABLE_WEB_SECURITY: &str = "disable-web-security";
    pub const ALLOW_RUNNING_INSECURE_CONTENT: &str = "allow-running-insecure-content";
    pub const IGNORE_CERTIFICATE_ERRORS: &str = "ignore-certificate-errors";
    pub const IGNORE_SSL_ERRORS: &str = "ignore-ssl-errors";

    /// Switches that relax browser security; presence triggers a startup warning.
    pub const RISKY_SWITCHES: &[&str] = &[
        DISABLE_WEB_SECURITY,
        ALLOW_RUNNING_INSECURE_CONTENT,
        IGNORE_CERTIFICATE_ERRORS,
        IGNORE_SSL_ERRORS,
    ];
}
```

`with_switch` keeps its `&'static str` signature, so both string literals and the
constants work.

**Startup warning:** in `CefPlugin::build` (`src/lib.rs`), before adding the
plugins, scan `command_line_config.switches` for membership in `RISKY_SWITCHES`
and, if any are present, emit a single `warn!` listing them, e.g.:

> `bevy_cef: web-security relaxations active: [disable-web-security]. Only enable these when loading fully trusted content.`

Matching ignores a leading `--` if a user wrote one. The scan runs once at plugin
build, in the browser process.

**Effective config:** `CefPlugin::build` computes one *effective* switch list —
normalized (strip a leading `--`), de-duplicated (kills the double-append class of
bug), and with `no-zygote` removed on Linux when the sandbox is enabled. This single
list feeds both the browser and child hooks, and the startup warning is keyed off it
so the message reflects what actually shipped to CEF.

### 4.4 Configurable sandbox

Add a three-state enum so "unspecified" preserves the current matrix:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SandboxMode {
    /// Preserve bevy_cef's current per-platform default (Section 2.3).
    #[default]
    PlatformDefault,
    /// Force the Chromium sandbox ON (`no_sandbox = false`).
    Enabled,
    /// Force the Chromium sandbox OFF (`no_sandbox = true`).
    Disabled,
}
```

- Add `pub sandbox: SandboxMode` to `CefPlugin` (defaults to `PlatformDefault`).
- Thread it `CefPlugin → MessageLoopPlugin → cef_initialize`.
- Resolve to the `no_sandbox` boolean at runtime:
  ```rust
  fn resolve_no_sandbox(mode: SandboxMode) -> bool {
      match mode {
          SandboxMode::Enabled => false,
          SandboxMode::Disabled => true,
          SandboxMode::PlatformDefault => {
              #[cfg(target_os = "macos")]   { cfg!(feature = "debug") }
              #[cfg(not(target_os = "macos"))] { true }
          }
      }
  }
  ```
  `PlatformDefault` is byte-for-byte equivalent to the current behavior: on macOS
  it yields `true` in debug builds and `false` in release; on Windows/Linux it
  yields `true`.

> **Gating note:** `no_sandbox` is gated by the Cargo **`debug` feature**, not
> `debug_assertions` (which gates `use-mock-keychain`). `resolve_no_sandbox` must
> therefore live in the root `bevy_cef` crate so `cfg!(feature = "debug")` observes
> the same feature value as today's `#[cfg(feature = "debug")]` site.

**Linux `no-zygote` coupling:** the `no-zygote` default exists *because*
`no_sandbox = true`. When the resolved `no_sandbox` is `false` on Linux (user chose
`Enabled`), `CefPlugin::build` filters `no-zygote` out of the effective switch list
and logs that the sandbox requires a correctly-installed SUID `chrome-sandbox`
helper.

**Documented caveats (best-effort enabling):** The Chromium macOS sandbox is the
*seatbelt* sandbox, NOT the Mac App Store App-Sandbox entitlement. It requires the
render-process executable to (a) link the `cef_sandbox` static library and (b) call
`cef_sandbox_initialize()` / `CefScopedSandboxContext` at the top of `main()` before
loading the CEF framework. This codebase does **none** of that today (no `cef_sandbox`
symbol exists anywhere), so `SandboxMode::Enabled` on macOS is currently
**non-functional and would abort the helper at launch** — wiring up the render-process
sandbox bootstrap is tracked as future work (Non-Goal). Linux needs a SUID
`chrome-sandbox` helper; Windows generally works as-is. We do not guarantee a working
sandbox — we only stop forcing it off and let the user opt in.

**Prerequisite guard:** when `SandboxMode::Enabled` is requested on a platform whose
sandbox bootstrap is absent (notably macOS today), emit a `warn!`/`error!` at startup
instead of silently passing `no_sandbox=false` and crashing the helper.

### 4.5 Data flow summary

```
CefPlugin { command_line_config, sandbox, ... }
   │  (warn! if RISKY_SWITCHES present; strip no-zygote if Linux+sandbox-on)
   ├─ MessageLoopPlugin { config, sandbox } ── cef_initialize(Settings{ no_sandbox: resolve(...) })
   └─ BrowserProcessAppBuilder { config }
        ├─ on_before_command_line_processing (BROWSER):  switches + switch_values
        └─ browser_process_handler() → BrowserProcessHandlerBuilder { config, extensions }
             └─ on_before_child_process_launch (EACH CHILD): switches + extensions + custom-schemes
```

## 5. Public API Changes

| Item | Change |
| --- | --- |
| `SandboxMode` enum | **New**, exported via prelude |
| `CefPlugin.sandbox: SandboxMode` | **New** field (default `PlatformDefault`) |
| `switches` module (constants + `RISKY_SWITCHES`) | **New**, exported via prelude |
| `CommandLineConfig` | Unchanged shape; `Default` gains `disable-session-crashed-bubble`, no longer carries risky switches |
| Hardcoded child-process switches | **Removed** (risky ones) |

## 6. Migration

Apps that depended on the implicit insecure defaults must opt back in:

```rust
use bevy_cef::prelude::*;

CefPlugin {
    command_line_config: CommandLineConfig::default()
        .with_switch(switches::DISABLE_WEB_SECURITY),
    ..default()
}
```

Documented in: `CHANGELOG`, a migration note,
`docs/website/docs/reference/plugin-configuration.md`, and the CLAUDE.md security
section.

## 7. Testing / Verification

No automated test suite exists; verification is manual via examples:

1. **Secure default still works:** run `simple`, `brp`, `inline_html`, `js_emit`
   with the risky switches removed — confirms BRP-over-IPC and localhost CORS need
   no SOP bypass.
2. **Opt-in reaches the renderer:** a build using
   `with_switch(switches::DISABLE_WEB_SECURITY)` allows a cross-origin `fetch`
   from page content that fails without it — proves child-process forwarding works.
3. **Sandbox parity:** `SandboxMode::PlatformDefault` produces the same `Settings`
   as today on each platform; `Disabled` forces off; `Enabled` forces on — and on
   macOS (missing the `cef_sandbox` bootstrap) `Enabled` warns rather than crashing.
4. **Warning fires once** when a risky switch is configured, and not otherwise.
5. Existing `headers_responser` unit tests remain green.

## 8. Risks / Open Questions

- **Child-process forwarding:** appending flags in `on_before_child_process_launch`
  is the proven mechanism (same path as `EXTENSIONS_SWITCH`/`CUSTOM_SCHEMES_SWITCH`),
  and reaches the network/utility process where CORS is enforced. Do **not** narrow
  this to renderer-only targeting — that would leave CORS enforced. Still confirmed
  empirically via verification step 2.
- **Sandbox enabling is best-effort** and platform-dependent; we document rather
  than guarantee it.
- **Breaking change** for downstream apps relying on implicit `disable-web-security`
  — mitigated by the migration note.
