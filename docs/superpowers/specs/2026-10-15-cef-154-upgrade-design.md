# CEF 154 Upgrade — Design

Date: 2026-10-15
Status: Approved in brainstorming (design sections 1–3); revised after a parallel Codex + Claude spec review; pending user review
Target version: 0.14.0 (breaking; bundled with the unreleased Bevy 0.20 support)

## Goal

Move bevy_cef from CEF `152.3.0+152.0.6` to `154.5.0+154.0.34` (Chromium 154.0.8037.98, the
newest `cef` / `cef-dll-sys` / `export-cef-dir` release, published 2026-10-07) and ship it in
0.14.0 together with the Bevy 0.20 support that is already on `main` but not yet published.

Two public contracts stay unchanged:

- The JavaScript API `window.cef.brp` / `window.cef.emit` / `window.cef.listen`.
- The Rust API `CefExtensions::new().add(name, code)` and the `CefPlugin::extensions` field.

## Background

### What changes between cef 152.3.0 and 154.5.0

A diff of the published crate sources shows that only the generated files differ
(`src/bindings/*.rs`, `src/resources/*.rs`). The hand-written wrapper code (`rc.rs`,
`string.rs`, `osr_texture_import/`, …) is byte-identical.

Inside the generated bindings (`aarch64_apple_darwin.rs`, same picture on the Windows and
Linux targets):

- **Added:** about 24 `ColorId` constants, `CEF_API_VERSION_15400`, and
  `CEF_API_VERSION_LAST` moves from 15200 to 15400.
- **Removed:** `cef::register_extension` and the underlying `cef_dll_sys::cef_register_extension`.

The remaining differences do not touch bevy_cef: a new
`ChromePageActionIconType::WALLET_REMINDER_NOTICE`, renumbered or removed `IDR_*` / `IDC_*` /
`IDS_*` resource constants, and revised download-handling documentation. No other `pub fn`
is removed, and all struct size and offset assertions in `cef-dll-sys` are identical, so the
structs and callbacks bevy_cef uses for offscreen rendering (`OnPaint`,
`OnAcceleratedPaint`, the accelerated paint info) are unchanged and no other compile errors
are expected.

### Why `register_extension` is gone

CEF commit `33624e6bc2` ("Update to Chromium version 154.0.8037.0", 2026-09-02) deprecates
`CefRegisterExtension`, citing https://crbug.com/40347662, and the CEF JavaScript integration
docs now state "Extension support is deprecated in M154." The current `include/cef_v8.h`
wraps the function in `#if CEF_API_REMOVED(15400)` with the annotation
`/*--cef(optional_param=handler,removed=15400)--*/`.

`cef-dll-sys` compiles against `CEF_API_VERSION_LAST` (its `build.rs` reads
`CEF_API_VERSION_LAST` from `include/cef_api_versions.h` and defines
`CEF_API_VERSION=<that>`), so cef-rs offers no way to build against an older API version
where the function still exists.

The feature is gone underneath as well. Chromium deleted the V8 extension registration
infrastructure (commit `c8c5a6a954`, "Delete V8 extensions registration infrastructure",
2026-07-30), and CEF 154's `CefRegisterExtension` is a stub —
`CEF_API_REQUIRE_REMOVED(15400); NOTIMPLEMENTED(); return false;`
(`libcef/renderer/v8_impl.cc`, 8037 branch). `CEF_API_REQUIRE_REMOVED` expands to
`LOG_IF(FATAL, …)` when the configured API version is 15400 or newer, which is what
bevy_cef configures through `api_hash(sys::CEF_API_VERSION_LAST, 0)`.

CEF names no successor. The remaining documented way to expose native functions is the
"window binding" pattern: set values on `context.global()` in
`CefRenderProcessHandler::OnContextCreated` (CEF `docs/javascript_integration.md`).

### Where bevy_cef depends on it

Both call sites are in `crates/bevy_cef_core/src/render_process/render_process_handler.rs`
and run from `on_web_kit_initialized` (lines 88–91):

1. `register_cef_api_extension` (lines 198–204) registers the extension `v8/bevy-cef-api`,
   whose code declares `native function __cef_brp/__cef_emit/__cef_listen` and assigns them
   to a global `cef` object. `CefApiHandler` (`cef_api_handler.rs`) implements the three
   native functions. **All IPC from JavaScript is built on this.**
2. `register_extensions_from_command_line` (lines 283–295) reads the
   `bevy-cef-extensions` switch (JSON `{name: code}`) and registers each entry as
   `v8/{name}` with no handler. This is the whole implementation of `CefExtensions`.

Bumping the version alone fails to compile; removing the two calls to make it compile would
leave every webview without `window.cef`.

### History

Before #17 (`519aa2b`, "feat: add custom CEF V8 extensions support"), `window.cef` was
bound in `on_context_created` with `v8_value_create_function`, the same technique this
design uses. It ran **after** `inject_initialize_scripts`, so a `PreloadScripts` entry could
not call `window.cef` at its top level. #17 moved the API into a V8 extension, which made it
"available earlier in the page lifecycle". This design returns to the binding but runs it
first, which keeps that guarantee.

## Decisions

| # | Decision | Alternatives considered |
|---|----------|-------------------------|
| D1 | Ship in 0.14.0 together with Bevy 0.20 support | A separate 0.15.0. Rejected: users would migrate twice in a short span, each time reinstalling CEF |
| D2 | Bind `window.cef` per context in `on_context_created` | Declaring the still-exported `cef_register_extension` symbol by hand. Rejected: under API version 15400 the CEF 154 stub is a fatal assertion that kills the render process, and Chromium has already deleted the feature underneath |
| D3 | Keep `CefExtensions` and reimplement it with `V8Context::eval` | Removing it and pointing users to `PreloadScripts`. Rejected: `PreloadScripts` is per webview, so users would lose global injection for no gain of theirs |
| D4 | Pin `cef` and `cef-dll-sys` with `=154.5.0+154.0.34` | A caret requirement. Rejected for the reason 0.14.0 already pins: the Rust bindings must match the runtime `export-cef-dir` installs. 154.0.34 also carries the fix for CEF #4291 (sustained idle CPU from a missing component path provider, `c6f60d143f`), so do not move back to an earlier 154.x without checking for it |

## Design

### 1. Render-process JavaScript injection

Scope: `crates/bevy_cef_core/src/render_process/` and the docs of
`crates/bevy_cef_core/src/browser_process/extensions.rs`. The browser process is unchanged:
it keeps passing `CefExtensions` to the render process as JSON in the `bevy-cef-extensions`
switch (`browser_process_handler.rs:77-80`).

#### `render_process_handler.rs`

- The `on_web_kit_initialized` override is removed. The extensions switch is parsed lazily
  on first use into `static EXTENSIONS: LazyLock<BTreeMap<String, String>>` with the
  existing `read_switch_json`; an absent or invalid switch yields an empty map. The
  `BTreeMap` makes extensions run in name order in every render process, where `HashMap`'s
  random state could give each process (including out-of-process iframes) its own order.
  The browser-side `CefExtensions` keeps its `HashMap` field.
- `on_context_created` runs three steps in this order. CEF already enters the V8 context
  before calling `OnContextCreated` (CEF `docs/javascript_integration.md`: "V8 will be
  inside a context if … OnContextCreated()/OnContextReleased() callback higher in the call
  stack"), so no `enter()` / `exit()` calls are made, and the existing pair inside
  `inject_initialize_scripts` is removed:

  ```text
  on_context_created(browser, frame, context):
      if context is None: return
      bind_cef_api(context)                    # window.cef = { brp, emit, listen }
      eval_extensions(context)                 # CefExtensions
      if browser and frame are Some:
          inject_initialize_scripts(browser, context, frame)  # PreloadScripts
  ```

  1. **`bind_cef_api`** creates one `V8Handler::new(CefApiHandler::default())`, then three
     functions with `v8_value_create_function` named `brp`, `emit` and `listen`, all backed
     by that handler. It sets them on a plain object from
     `v8_value_create_object(None, None)` and sets that object on `context.global()` under
     the key `cef`. Every `set_value_bykey` uses `V8Propertyattribute::default()`, which is
     `V8_PROPERTY_ATTRIBUTE_NONE`, so `window.cef` stays writable, as the old `var cef` was.
     The cef-rs wrappers add a reference for every handler and value they hand to CEF, so no
     manual `add_ref` / `release` is needed.
  2. **`eval_extensions`** evaluates each non-empty entry of `EXTENSIONS` with
     `context.eval`, using `cef://localhost/__extensions__/{name}.js` as the script URL.
     The URL must be absolute: CEF passes it to `blink::KURL` without a base URL, so a
     relative string such as `v8/{name}` would be dropped. Empty code is skipped because
     `CefV8Context::Eval` rejects it. The URL is metadata only; nothing is fetched.
  3. **`inject_initialize_scripts`** keeps its current behaviour (the browser's
     `PreloadScripts`, keyed by browser id, joined with `";"` and evaluated as one script
     with `frame.url()` as the script URL), minus the `enter()` / `exit()` calls.

- A shared helper evaluates one script with `context.eval` and, on failure, calls
  `cef_error!` with the script URL, the exception message, line number and start column
  (the format `inject_initialize_scripts` already uses). Both extensions and preload
  scripts go through it. It also calls the context's `console.error` with
  `<message> (<url>:<line>:<column>)`, because an exception caught by `CefV8Context::Eval`
  during `OnContextCreated` never reaches the page console on its own. §3 describes where
  such an error becomes visible.
- Removed: the `on_web_kit_initialized` override, `CEF_API_EXTENSION_NAME`,
  `CEF_API_EXTENSION_CODE`, `register_cef_api_extension`,
  `register_extensions_from_command_line`, and the `register_extension` import.

#### `cef_api_handler.rs`

- `ImplV8Handler::execute` matches `"brp"`, `"emit"` and `"listen"` instead of
  `"__cef_brp"`, `"__cef_emit"` and `"__cef_listen"`. The three `execute_*` bodies are
  unchanged: they already find their frame through `v8_context_get_current_context()`.
- The type's doc comment changes from "registered as a CEF extension during
  `on_web_kit_initialized`" to describing the per-context `window.cef` binding, and lists
  the functions by their new names.

#### `browser_process/extensions.rs`

- The `CefExtensions` doc states that each extension's code is evaluated in every frame's
  V8 context, in name order, after `window.cef` is defined and before `PreloadScripts` and
  page scripts, and that it applies to all webviews.
- The `add` doc states that `name` becomes the script URL
  `cef://localhost/__extensions__/{name}.js` shown in console errors and DevTools, instead
  of "will be prefixed with `v8/` internally".

#### Behaviour that stays the same

- `on_context_created` fires for every frame and every script world, so iframes
  (including out-of-process iframes, whose render processes receive the same switch) keep
  receiving `window.cef` and the extensions. Dedicated workers get neither, as before.
- The code runs before any page script: Blink dispatches `DidCreateScriptContext`
  synchronously while it initialises the window proxy, right after creating the context.
- `V8Context::eval` runs the code as a classic script (`blink::ClassicScript`) in the
  global scope, so top-level `var` and function declarations become `window` properties.

#### Behaviour that changes

- Extensions run in name order instead of an unspecified order.
- `window.cef` can now be used at the top level of extension code, not only inside
  functions called later.
- `document` exists when extension code runs. APIs that Blink installs conditionally after
  context creation may still be missing at an extension's top level; `PreloadScripts`
  already has this limitation.
- Top-level `let`, `const` and `class` declarations in extension code live in the global
  lexical scope shared with page scripts, so a page that declares the same name with
  `let` / `const` fails with a `SyntaxError`.
- Extension code can no longer declare `native function`. No supported usage breaks:
  `CefExtensions` always registered extensions without a handler, so such a declaration
  could never be called.
- `window.cef` is now a configurable property: `delete window.cef` succeeds, and a page can
  shadow it with a top-level `let cef`. Both failed against the old non-configurable
  `var cef`.
- An exception or syntax error in an extension no longer goes unnoticed (see §3), and the
  remaining extensions, the preload scripts and the page keep running.

### 2. Dependencies, documentation and CHANGELOG

#### Dependencies

- `Cargo.toml`: `cef = { version = "=154.5.0+154.0.34" }` and
  `cef-dll-sys = { version = "=154.5.0+154.0.34", features = ["sandbox"] }`.
- `Cargo.lock`: `cargo update -p cef -p cef-dll-sys --precise 154.5.0+154.0.34`. Never run a
  bare `cargo update`, which would also move Bevy and the other dependencies.
- `Makefile`: the three `cargo install --locked export-cef-dir@152.3.0+152.0.6` lines
  (`setup-macos`, `setup-windows`, `setup-linux`) become `@154.5.0+154.0.34`.

#### Repository docs

- `README.md`: the `export-cef-dir@` install line, and the compatibility table row
  `| 0.20 | 0.14.0 | 154.5.0+154.0.34 |`. The `0.19 | 0.13.0` row stays at 152.
- `CLAUDE.md`: the `export-cef-dir --version` line under "Debug Tools Setup (macOS)", the
  same compatibility table row, and the `CefExtensions` bullet under "Core Components"
  (it currently says "via `register_extension`").
- `examples/extensions.rs` (module doc, line 1) and `assets/extensions.html` (line 13) say
  extensions are registered via `register_extension`; both are reworded.

#### Website (`docs/website/docs/`)

Only the passages that describe the mechanism this work changes:

- `guides/extensions.md`: the "Global Scope" paragraph says extensions are registered via
  `register_extension`; it becomes "evaluated in every frame's V8 context". The closing
  caution ("syntax errors … will prevent the render process from initializing correctly")
  becomes "an error in an extension is reported to the console, and from there to the Bevy
  log; the other extensions and the page keep running". The page also states that
  extensions run in name order.
- `reference/plugin-configuration.md`: the `extensions` example calls a nonexistent
  `CefExtensions::register_extension` method; it becomes `.add(...)`. The paragraph saying
  extensions "run in V8's extension context … and cannot access the DOM directly" is
  replaced by the run order: `window.cef` → extensions → `PreloadScripts` → page scripts,
  and by the fact that `document` already exists when extension code runs.
- `reference/javascript-api.md`: the sentence saying `window.cef` "is not available during
  CEF extension execution" becomes the run order above, which makes `window.cef` available
  to extensions.

#### CHANGELOG (`## v0.14.0`)

- **Breaking Changes**, new entry: "Update CEF to 154.5.0+154.0.34 (Chromium
  154.0.8037.98). Reinstall the CEF framework and the render process
  (`bevy_cef_render_process`, or `bevy_cef_debug_render_process` on macOS) with the
  Makefile setup command."
- **Fixed**: the entry "`cef` and `cef-dll-sys` are pinned to `=152.3.0+152.0.6` …" is
  rewritten for `=154.5.0+154.0.34`, because 0.14.0 never ships with 152.
- **Changed**, new entry: CEF 154 removed `CefRegisterExtension`, so `window.cef` and
  `CefExtensions` are injected in `on_context_created`; the public API is unchanged;
  `window.cef` is available at the top level of extensions and preload scripts; extensions
  run in name order; an error in an extension is reported to the page console, and from
  there to the Bevy log, instead of being silently ignored.

### 3. Error handling

| Situation | Behaviour |
|-----------|-----------|
| `on_context_created` receives no context | Return; there is nothing to bind |
| `context.global()` or a `v8_value_create_*` call returns `None` | `cef_error!`, skip the `window.cef` binding, still run extensions and preload scripts |
| An extension throws or fails to compile | `cef_error!` with script URL, message, line and column; continue with the next extension |
| A preload script throws or fails to compile | Unchanged: the preload scripts are one joined script, so the rest of them do not run; the error is now reported to the console as well |
| An extension's code is empty | Skipped without evaluation |
| The extensions switch is absent | Empty `EXTENSIONS`, nothing logged |
| The extensions switch holds invalid JSON | `cef_error!` from `read_switch_json`; empty `EXTENSIONS` |

`cef_error!` reaches a log only when `bevy_cef_core`'s `log` feature is on and a subscriber
is installed. The release render process enables neither and the debug render process
installs no subscriber, so these lines are silent in practice — the same as every existing
`cef_error!` in the render process, which this work does not change. Script errors become
visible through another path: the eval helper calls the context's `console.error` with
`<message> (<url>:<line>:<column>)`, and the browser process's
`DisplayHandler::on_console_message` logs console errors with Bevy's `error!`. Without that
call the error stays invisible: an exception caught by `CefV8Context::Eval` during
`OnContextCreated` is not reported to the page console, although `Eval` uses a verbose
`TryCatch` (verified on CEF 152.0.8, where `console.error` calls from the same code do
arrive). The absolute extension URL names the extension in that message. No crate-specific error type is introduced.

### 4. Verification

#### Automated (the same commands CI runs)

- `cargo clippy --workspace --all-targets --all-features -- -Dwarnings`
- `cargo fmt --all -- --check`
- `cargo test --workspace --no-default-features --tests`
- `cargo test --doc`

No new unit tests are added. The changed code needs a live V8 context inside a CEF render
process, and the repository has no harness that provides one.

#### Local environment (macOS)

This overwrites the CEF 152 installation in `$HOME/.local/share/cef`:

1. `cargo install --locked export-cef-dir@154.5.0+154.0.34 --force`
2. `export-cef-dir --force "$HOME/.local/share/cef"`
3. Reinstall the debug render process from the workspace with `--locked` and copy it into
   the framework's `Libraries/` directory (as `make setup-macos` does).

#### Manual checks on macOS (`--features debug`)

- **IPC:** in `js_emit`, `host_emit` and `brp`, events and BRP promises round-trip as on 152.
- **Extensions:** in `extensions`, `myGame.sendScore()` reaches the Bevy log and
  `myGame.version` is displayed.
- **Throwaway checks, not committed:**
  - A preload script that calls `window.cef.emit` at its top level reaches Bevy.
  - Code inside an `<iframe>` can call `window.cef`.
  - An extension with a syntax error appears in the Bevy log as a console error whose
    source is `cef://localhost/__extensions__/<name>.js`, and a second, valid extension and
    the page still work.
  - A page with a transparent background stays transparent, and clicks on its transparent
    pixels still pass through to what is behind it.
- **CEF 154 runtime regressions:** run each remaining example for about 20 seconds —
  `simple`, `inline_html`, `navigation`, `zoom_level`, `sprite`, `devtool`,
  `custom_material`, `custom_scheme`, `headless_texture`, `hidpi`, `resize`,
  `toolbar_drag`, `ui_webview` — and check for crashes, blank textures and error logs.
  In `resize` and `hidpi`, resize repeatedly and, if a second display is available, move
  the window between displays. In `navigation`, go back and forward after a resize.

#### Not verifiable in this environment

Windows and Linux CI covers build and tests only. Runtime checks on those platforms are
listed in the PR description for the maintainer to run.

## Out of scope

- The outdated version numbers in `docs/website/docs/installation.md` (`bevy_cef = "0.4.0"`).
- Changing how `PreloadScripts` are joined: they are concatenated with `";"` and evaluated
  as one script, so one failing entry stops the rest, and a trailing `//` comment in one
  entry swallows the next. This is existing behaviour.
- Any CEF 154 feature adoption beyond what the upgrade requires.

## Risks

- **Runtime-only Chromium 154 changes** that compile cleanly. The CEF commits between
  152.0.6 and 154.0.34 that touch offscreen rendering:
  - `33624e6bc2`: the OSR root layer changed from `ui::LayerSolidColor` to
    `ui::LayerSurface` with a fallback background colour. bevy_cef relies on transparent
    painting and alpha hit-testing.
  - `52d82d5326` (#4258): macOS OSR now handles `DidUpdateVisualProperties`, which changes
    surface synchronisation on resize, device-scale changes and display moves.
  - `87b9ff5d5f` (#3826): frame refresh after a resize changed in the Viz capturer.
  - `85595822f7` (#3822): pages restored from the back/forward cache re-sync their bounds.

  The macOS checks in §4 target each of these. Windows and Linux stay unverified until the
  maintainer runs them.
- **Known CEF bugs still present in 154.0.34.** #4301: `<select>` popups stop opening after
  a cross-site navigation in OSR (the Windows/Linux popup path; macOS does not render popups
  yet). #4298: a renderer crash for `<iframe srcdoc>` under an opaque-origin parent, such as
  a user `data:` URL (`cef://localhost` is a standard scheme and unaffected). Their fixes
  (`dff7cc2c12`, `6497b876b1`) are on CEF master only. Before release, check whether a newer
  154.x cef-rs release carries them, and re-pin if so.
- **Context-creation timing.** `on_context_created` runs before the page's own scripts;
  `PreloadScripts` already depends on this, and the throwaway preload check confirms it for
  `window.cef`.
