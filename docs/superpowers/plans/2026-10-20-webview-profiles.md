# Webview Profiles Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a webview carry a named, disk-backed profile whose cookies, localStorage and IndexedDB survive a restart, and fix the startup, shutdown and zoom paths that profile persistence depends on.

**Architecture:** `bevy_cef_core::Browsers` gains a registry of one `RequestContext` per profile directory, a live-browser counter fed by a new `LifeSpanHandler`, per-browser initial-zoom cells applied at the first main-frame commit, and coalesced cookie flushing. The `bevy_cef` crate gains a `profile` module (name validation, the `WebviewProfile` component, deferred creation systems, the failure event), a startup path that reports `CefStatus` instead of panicking, and a shutdown system in `Main` after `Main::run_main` that closes, waits, releases and calls `shutdown()` within the exiting frame.

**Tech Stack:** Rust 2024 (rust-version 1.97.1), Bevy 0.20, `cef = "=154.5.0+154.0.34"`, `async-channel`, `serde`.

**Spec:** `docs/superpowers/specs/2026-10-20-webview-profiles-design.md` (commit `2b81b3a`). Read it before starting any task; this plan argues from it and does not repeat its rationale.

## Global Constraints

- Base: bevy_cef 0.14.0 on branch `webview-profiles` in `wt/webview-profiles`. Do not bump the crate version; the release does that (target 0.15.0).
- A profile named `NAME` lives at `<root_cache_path>/profile-NAME`, joined onto the exact `root_cache_path` string. Never canonicalize either side.
- Profile names: characters `a`–`z`, `0`–`9`, `-`, `_`; length 1 to 64. Anything else is rejected, never rewritten.
- A named profile requires `CefPlugin::root_cache_path` to be a non-empty absolute path.
- Profile contexts use `persist_session_cookies = 1`.
- Webviews without `WebviewProfile` keep today's creation path. The only behavior changes for them are the first-commit zoom (Task 4) and `Cache-Control: no-store` (Task 7).
- After `cef_initialize` returns 0, bevy_cef calls no CEF function except `cef::get_exit_code`.
- Exit codes 21 and 24 both mean "another process holds or locks the root".
- The release render process must not gain a Bevy dependency (CI checks it): every change in `bevy_cef_core` goes under `browser_process/`, which is compiled only with the `browser` feature.
- No `.expect`/`.unwrap`/`assert!` added on a CEF result in non-test code; failures become `Result`s, events or `CefStatus`.
- Commits: Conventional style as in the history (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`). No `Co-Authored-By` or other attribution trailer; after each commit, `git log -1 --format=%B | grep -i co-authored-by` must print nothing.
- Build environment on the Windows box: export `CEF_PATH=$HOME/.cache/orzma/cef` for every cargo command, and prefix `target/debug` on `PATH` (`PATH="$PWD/target/debug:$PATH"`) for any test binary that links CEF. `~/.local/share/cef` already holds CEF 154.0.34 and `bevy_cef_render_process` 0.14.0.
- CI commands that must stay green (`.github/workflows/ci.yml`): `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -Dwarnings`, `cargo clippy -p bevy_cef_core --no-default-features -- -Dwarnings`, `cargo check -p bevy_cef_render_process`, the render-process Bevy-dependency check (`cargo tree -p bevy_cef_render_process -e normal --target all`), `cargo test --workspace --no-default-features --tests` (CI runs it on macOS too), and `cargo test --doc`.

## Review Focus

1. A file (not a directory) sits where the profile directory should be → `WebviewProfileError` with the directory reason, no browser, no context. Test in Task 1 (`prepare_profile_dir_rejects_a_file_in_the_way`) and Task 5 (`a_file_at_the_profile_path_reports_a_directory_error`).
2. `ZoomLevel` is set to `0.0` after a non-default value but before the first commit → nothing is applied at commit, so a remembered site zoom stays. Test in Task 4 (`resetting_to_default_before_commit_applies_nothing`).
3. `WebviewSource` changes after a `RequestNavigate` while the webview is waiting → the source change wins (latest wins). Test in Task 5 (`a_source_change_after_request_navigate_wins`).
4. `AppExit` arrives while a profile webview is still waiting → the process shuts down within the time limit and exits 0. Test in Task 8 (scenario `exit_while_pending`).
5. A waiting profile webview is despawned and a new webview with the same profile is spawned → the new one gets a browser and the old one never does. Test in Task 8 (scenario `despawn_then_respawn`).

---

### Task 1: Profile names, the component and the profile directory

**Files:**
- Create: `src/profile.rs`
- Modify: `src/lib.rs` (declare `mod profile;`, export the public items in `prelude`)
- Test: unit tests in `src/profile.rs`

**Interfaces:**
- Produces (in `bevy_cef::prelude`):
  - `pub struct ProfileName(String)` with `impl TryFrom<String> for ProfileName { type Error = ProfileNameError }`, `pub fn as_str(&self) -> &str`, `pub const MAX_LEN: usize = 64`. Serde: `#[serde(try_from = "String", into = "String")]`, with `impl From<ProfileName> for String`. Derives `Reflect, Debug, Clone, PartialEq, Eq, Hash`, with `#[reflect(opaque, Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]`, so reflection and BRP writes go through serde's validation.
  - `pub enum ProfileNameError { Empty, TooLong { len: usize }, InvalidChar { ch: char } }`, implementing `Display` and `std::error::Error` by hand (the crate has no `thiserror`).
  - `#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)] #[reflect(Component, Debug, Serialize, Deserialize)] pub struct WebviewProfile(ProfileName)` with `pub fn new(name: impl Into<String>) -> Result<Self, ProfileNameError>` and `pub fn name(&self) -> &ProfileName`.
- Produces (crate-private, used by Task 5):
  - `ProfileName::revalidate(&self) -> Result<(), ProfileNameError>` (Reflect can write an invalid inner string).
  - `pub(crate) fn profile_dir(root: Option<&str>, name: &ProfileName) -> Result<PathBuf, ProfileRootError>` where `pub(crate) enum ProfileRootError { Missing, NotAbsolute }`.
  - `pub(crate) fn prepare_profile_dir(dir: &Path) -> std::io::Result<()>`.

- [ ] **Step 1: Write the failing tests** in `src/profile.rs`:
  - `accepts_lowercase_digits_dash_underscore`: `ProfileName::try_from("work_2-a".to_owned())` is `Ok`; `"a".repeat(64)` is `Ok`.
  - `rejects_invalid_names`: `""` → `Err(Empty)`; `"a".repeat(65)` → `Err(TooLong { len: 65 })`; `"Work"` → `Err(InvalidChar { ch: 'W' })`; `"a/b"`, `"a\\b"`, `".."`, `"a b"`, `"é"` → `Err(InvalidChar { .. })`.
  - `deserializing_an_invalid_name_fails`: `serde_json::from_str::<WebviewProfile>("\"Bad\"")` is `Err`; `"\"ok\""` round-trips through `serde_json::to_string`.
  - `profile_dir_joins_the_exact_root`: with an absolute root built from `std::env::temp_dir()`, `profile_dir(Some(root), &name("work"))` equals `Path::new(root).join("profile-work")`; the same root with a trailing separator yields the same path.
  - `profile_dir_requires_an_absolute_root`: `None`, `Some("")`, `Some("   ")` → `Err(Missing)`; `Some("relative/root")` → `Err(NotAbsolute)`.
  - `prepare_profile_dir_creates_a_writable_dir`: on a fresh path under `std::env::temp_dir()` (unique name with the process id and a counter), the call returns `Ok`, the directory exists, and no probe file is left behind. On Unix, the mode is `0o700`.
  - `prepare_profile_dir_rejects_a_file_in_the_way`: a regular file at the target path → `Err`.

- [ ] **Step 2: Run them to see them fail**

Run: `CEF_PATH=$HOME/.cache/orzma/cef PATH="$PWD/target/debug:$PATH" cargo test -p bevy_cef --lib profile`
Expected: compile errors (the items do not exist yet).

- [ ] **Step 3: Implement the items listed under Interfaces in `src/profile.rs`.** `prepare_profile_dir` uses `std::fs::DirBuilder` with `recursive(true)`, plus `std::os::unix::fs::DirBuilderExt::mode(0o700)` under `cfg(unix)`. On Windows it relies on ACL inheritance from the root. It then creates and removes a `.bevy_cef_write_check` file inside the directory. Register `WebviewProfile` with `register_type` in `WebviewCoreComponentsPlugin` (`src/common/components.rs:12`). `WebviewProfile`'s rustdoc states that the component is read once, when the webview's browser is requested, and that later changes, insertion after that point, and removal have no effect.

- [ ] **Step 4: Run the tests to see them pass** (same command). Expected: all 7 pass.

- [ ] **Step 5: Commit** — `feat: add profile names and the WebviewProfile component`

---

### Task 2: Startup status instead of panics, and no relaunch window

**Files:**
- Modify: `src/common/message_loop.rs` (initialization returns a `Result`; `MessageLoopPlugin::build` adds systems only)
- Modify: `src/lib.rs` (`CefPlugin::build` branches on the result; inserts `CefStatus` and `CefRootCachePath`)
- Modify: `crates/bevy_cef_core/src/browser_process/browser_process_handler.rs` (`on_already_running_app_relaunch`)
- Test: unit tests in `src/common/message_loop.rs`

**Interfaces:**
- Produces (in `bevy_cef::prelude`):
  - `#[derive(Debug, Clone)] pub enum CefStartupError { RootCacheDir { path: String, error: String }, LibraryLoad, Initialize { exit_code: i32 } }` with `pub fn is_root_in_use(&self) -> bool` (true exactly for an `Initialize` exit code equal to `cef_resultcode_t::CEF_RESULT_CODE_PROFILE_IN_USE` (21) or `CEF_RESULT_CODE_NORMAL_EXIT_PROCESS_NOTIFIED` (24), compared through the bindings' constants rather than literals), plus `Display` and `Error`.
  - `#[derive(Resource, Debug, Clone)] pub enum CefStatus { Running, Failed(CefStartupError) }`.
- Produces (crate-private):
  - `impl MessageLoopPlugin { pub(crate) fn start(&self, app: &mut App) -> Result<(), CefStartupError> }`. It does today's `build` work up to and including `cef_initialize`, then inserts the `cef::App` and `MessageLoopWorkingReceiver` non-send resources.
  - `#[derive(Resource)] pub(crate) struct CefRootCachePath(pub(crate) Option<String>)`, inserted by `CefPlugin::build` from `self.root_cache_path`. Task 5 reads it.
- Changes: both `cef_initialize` functions (`message_loop.rs:99`, `:140`) return `Result<(), CefStartupError>`. On a 0 result they read `cef::get_exit_code()` and call nothing else in CEF. The `create_dir_all` panics (`:107`, `:149`) become `RootCacheDir`. On macOS, `load_cef_library` (`:88`–`95`) returns `LibraryLoad` instead of `unwrap`/`assert!`.
- `CefPlugin::build`, on `Ok`: insert `CefStatus::Running` and add every sub-plugin as today (`MessageLoopPlugin` now only adds systems and `RunOnMainThread`). On `Err(e)`: log with `error!`, insert `CefStatus::Failed(e)` and `CefRootCachePath`, and add only what calls no CEF function but what user-added bevy_cef plugins read: `WebviewCoreComponentsPlugin`, `IpcRawEventPlugin` (the IPC channel resources `JsEmitEventPlugin` reads), `add_message::<RenderTextureMessage>()` (read by the material plugins) and `init_resource::<FocusedWebview>()`. No plugin that calls CEF and no `RemotePlugin` is added.
- `BrowserProcessHandlerBuilder::on_already_running_app_relaunch(&self, _: Option<&mut CommandLine>, _: Option<&CefString>) -> c_int` returns `1`.

- [ ] **Step 1: Write the failing tests** in `message_loop.rs` `mod tests`:
  - `root_in_use_covers_codes_21_and_24`: `Initialize { exit_code: 21 }` and `{ 24 }` → `is_root_in_use()` true; `{ 1 }`, `LibraryLoad`, `RootCacheDir { .. }` → false.
  - `a_failed_start_adds_no_cef_systems`: build `App::new()` with `MinimalPlugins`. Call the crate-private `add_failed_startup(app: &mut App, error: CefStartupError, root: Option<String>)`, which is the `Err` branch of `CefPlugin::build`, extracted so it is testable without CEF. Also add `JsEmitEventPlugin::<serde_json::Value>::default()`, as an app would. Assert that `CefStatus::Failed(_)` exists, that `world.contains_non_send::<Browsers>()` is false, and that one `app.update()` completes without a panic.

- [ ] **Step 2: Run them to see them fail.** Run: `CEF_PATH=$HOME/.cache/orzma/cef PATH="$PWD/target/debug:$PATH" cargo test -p bevy_cef --lib message_loop`. Expected: compile errors.

- [ ] **Step 3: Implement the interfaces above**, routing `CefPlugin::build` through `MessageLoopPlugin::start` and `add_failed_startup`.

- [ ] **Step 4: Run the tests to see them pass**, then run `cargo run --example simple` on Windows and confirm the webview still renders (the `Ok` path is unchanged).

- [ ] **Step 5: Commit** — `feat: report CEF startup failures through CefStatus instead of panicking`

---

### Task 3: Orderly shutdown in the exiting frame

**Files:**
- Create: `crates/bevy_cef_core/src/browser_process/life_span_handler.rs` (declare in `browser_process.rs`, `pub use` it)
- Modify: `crates/bevy_cef_core/src/browser_process/client_handler.rs` (`with_life_span_handler`, `ImplClient::life_span_handler`)
- Modify: `crates/bevy_cef_core/src/browser_process/browsers.rs` (live-browser list, `close_all`, `release_all`, `live_browser_count`)
- Modify: `src/common/message_loop.rs` (replace `cef_shutdown` in `Update` with `shutdown_cef_on_exit` in `Main`)
- Test: unit test in `src/common/message_loop.rs`; behavior is covered end to end by Task 8

**Interfaces:**
- Produces in `bevy_cef_core`:
  - `pub type SharedLiveBrowsers = std::rc::Rc<std::cell::RefCell<Vec<cef::Browser>>>`.
  - `LifeSpanHandlerBuilder::build(live: SharedLiveBrowsers) -> cef::LifeSpanHandler`: `on_after_created` pushes the browser, and `on_before_close` removes the entry with the same `identifier()`. Popups share the client, so they are tracked too. It never touches the entity map.
  - On `Browsers`: field `live_browsers: SharedLiveBrowsers` (passed into every client built by `client_handler`); `pub fn live_browser_count(&self) -> usize`; `pub fn close_all(&mut self)` (`close_browser(1)` on every live browser, popups included, then clears the entity map); `pub fn release_all(&mut self)` (drops every CEF reference `Browsers` holds, the live list included; Task 5 extends it to the profile registry).
  - Pending profile requests (Task 5) need no step here: they are ECS components, and no system runs after this one before the runner exits.
- Produces in `bevy_cef` (crate-private):
  - `fn shutdown_cef_on_exit(mut browsers: NonSendMut<Browsers>, mut commands: Commands)`; `NonSendMut` already pins it to the main thread. It runs `close_all()`, then loops `cef::do_message_loop_work()` with a 5 ms sleep until `live_browser_count() == 0` or `SHUTDOWN_TIME_LIMIT` (3 s, a `const` local to the function) passes. On the limit it logs `warn!` with the remaining count and continues. It then calls `release_all()`, `cef::shutdown()`, and inserts the marker resource `CefShutDown`.
  - `pub(crate) fn register_on_app_exit<M>(app: &mut App, system: impl IntoScheduleConfigs<ScheduleSystem, M>)`. It adds the system to `Main` with `.after(Main::run_main).after(cef_do_message_loop_work).run_if(on_message::<AppExit>)`. Both the plugin and the test register through it.
  - `cef_do_message_loop_work`, `shutdown_cef_on_exit` itself, `create_webview` and (in Task 5) `create_ready_profile_browsers` gain `.run_if(not(resource_exists::<CefShutDown>))`. A runner that keeps updating after `AppExit` then never calls CEF after `shutdown()`.

- [ ] **Step 1: Write the failing test** `an_app_exit_written_in_last_reaches_the_exit_system`. In an `App::new()`, use `register_on_app_exit` to register a probe system that sets a `Resource` flag. Add a `Last` system that writes `AppExit::Success`, as Bevy's `exit_on_all_closed` does. Run `app.update()` once and assert that the flag is set in that same update.

- [ ] **Step 2: Run it to see it fail** (`cargo test -p bevy_cef --lib message_loop` with the Global Constraints env). Expected: compile error.

- [ ] **Step 3: Implement the interfaces above**, and delete the old `cef_shutdown` system and its `Update` registration (`message_loop.rs:83`, `:208`).

- [ ] **Step 4: Run the test to see it pass.** Then run `cargo run --example simple`, close the window, and confirm that the process exits within about 3 s with no panic and no `GPU process exited unexpectedly` storm in the console.

- [ ] **Step 5: Commit** — `fix: run CEF shutdown in the exiting frame, after closing every browser`

---

### Task 4: Apply the initial zoom at the first main-frame commit

**Files:**
- Create: `crates/bevy_cef_core/src/browser_process/initial_zoom.rs` (declare and `pub use`)
- Modify: `crates/bevy_cef_core/src/browser_process/load_handler.rs` (`on_load_start`)
- Modify: `crates/bevy_cef_core/src/browser_process/browsers.rs` (`WebviewBrowser.initial_zoom`, `create_browser` parameter, `set_zoom_level`)
- Modify: `src/webview.rs` (`create_webview` reads `&ZoomLevel`)
- Test: unit tests in `initial_zoom.rs`

**Interfaces:**
- Produces:
  - `#[derive(Clone, Copy, Debug, PartialEq)] pub enum InitialZoom { Waiting(Option<f64>), Committed }` and `pub type SharedInitialZoom = Rc<Cell<InitialZoom>>`.
  - `InitialZoom::waiting(level: f64) -> Self`: `0.0` becomes `Waiting(None)`.
  - `fn request(&mut self, level: f64) -> ZoomRequest`: in `Waiting`, it stores the level (`0.0` clears it to `None`) and returns `ZoomRequest::Held`; in `Committed`, it returns `ZoomRequest::SendNow`.
  - `fn commit(&mut self) -> Option<f64>`: returns the held level once and moves to `Committed`.
  - `pub enum ZoomRequest { Held, SendNow }`.
  - `Browsers::create_browser` gains `initial_zoom: f64`, and stores `Rc::new(Cell::new(InitialZoom::waiting(initial_zoom)))` in `WebviewBrowser` and in the `LoadHandlerBuilder`.
  - `LoadHandlerBuilder::build(webview, sender, initial_zoom: SharedInitialZoom)`. Its `on_load_start` runs for the main frame only: when `commit()` returns `Some(level)`, it calls `browser.host().set_zoom_level(level)`.
  - `Browsers::set_zoom_level` sends to CEF only on `ZoomRequest::SendNow`.

- [ ] **Step 1: Write the failing tests** in `initial_zoom.rs`:
  - `a_default_zoom_applies_nothing_at_commit`: `waiting(0.0).commit()` is `None`.
  - `a_non_default_zoom_applies_once`: `waiting(1.5)`; the first `commit()` returns `Some(1.5)`, the second returns `None`.
  - `a_change_before_commit_replaces_the_held_level`: `waiting(1.0)`, `request(2.0)` returns `Held`, and `commit()` returns `Some(2.0)`.
  - `resetting_to_default_before_commit_applies_nothing`: `waiting(1.0)`, `request(0.0)`, and `commit()` returns `None`.
  - `after_commit_changes_are_sent_now`: after `commit()`, `request(3.0)` returns `SendNow`.

- [ ] **Step 2: Run them to see them fail** (`cargo test -p bevy_cef_core --lib initial_zoom --features browser`). Expected: compile errors.

- [ ] **Step 3: Implement the interfaces above.** Wire `create_webview` (`src/webview.rs:230`) to pass the entity's current `ZoomLevel.0`.

- [ ] **Step 4: Run the tests to see them pass.** Then run `cargo run --example zoom_level` and confirm that wheel zoom still changes the page after it has loaded, which shows that changes after the commit are sent immediately. The initial-zoom path is checked end to end by Task 8's `zoom` scenario.

- [ ] **Step 5: Commit** — `fix: apply a webview's initial zoom at the first main-frame commit`

---

### Task 5: Profile contexts and deferred browser creation

**Files:**
- Create: `crates/bevy_cef_core/src/browser_process/profiles.rs` (profile registry types; declare and `pub use`)
- Modify: `crates/bevy_cef_core/src/browser_process/request_context_handler.rs` (a ready-flag variant)
- Modify: `crates/bevy_cef_core/src/browser_process/browsers.rs` (registry, `prepare_profile`, `is_profile_ready`, `create_browser` gains `profile`, returns `Result`)
- Modify: `src/profile.rs` (systems, `WebviewProfileError`, `ProfilePlugin`), `src/lib.rs` (add `ProfilePlugin`), `src/webview.rs` (`create_webview` excludes profile webviews; shared creation `SystemParam`), `src/navigation.rs` (`apply_request_navigate` while pending)
- Create: `examples/profile.rs`
- Test: unit tests in `src/profile.rs`; persistence is covered by Task 8

**Interfaces:**
- Produces in `bevy_cef_core`:
  - `RequestContextHandlerBuilder::build_with_ready(ready: Rc<Cell<bool>>) -> cef::RequestContextHandler`; its `on_request_context_initialized` sets `ready` to `true`.
  - `pub enum ProfileContextError { CreationFailed, DidNotLoad }`. `DidNotLoad` is returned when `context.cache_path()` is empty right after creation.
  - `pub enum CreateBrowserError { NullBrowser, UnknownProfile }`.
  - `Browsers::prepare_profile(&mut self, cache_path: &str, requester: Requester) -> Result<(), ProfileContextError>`. It gets or creates the context with `cache_path` and `persist_session_cookies: 1`. It registers the `cef://localhost` and custom scheme factories once per context (the same calls `request_context()` makes today, `browsers.rs:608`) and checks `cache_path()`. The context is keyed by `cache_path`.
  - `Browsers::is_profile_ready(&self, cache_path: &str) -> bool`. Readiness is the `Rc<Cell<bool>>` the handler sets. It stands in for the spec's "profile ready" message: the callback runs on the CEF UI thread, which is the Bevy main thread.
  - `Browsers::is_audio_muted(&self, webview: &Entity) -> Option<bool>` (wraps `BrowserHost::is_audio_muted`; Task 8 reads it).
  - `Browsers::create_browser(..., profile: Option<&str>) -> Result<(), CreateBrowserError>`. `None` keeps today's fresh in-memory context. `Some` uses the registered context and returns `UnknownProfile` if there is none. The `.expect`s at `browsers.rs:159` (null browser) and `:160` (no host) both become `NullBrowser`.
  - `release_all` (Task 3) also clears the profile registry.
- Produces in `bevy_cef` (`bevy_cef::prelude`):
  - `#[derive(Debug, EntityEvent, Clone, Reflect, Serialize, Deserialize)] pub struct WebviewProfileError { #[event_target] pub webview: Entity, pub reason: WebviewProfileErrorReason }`.
  - `#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)] pub enum WebviewProfileErrorReason { InvalidName(String), NoRoot, Directory(String), DidNotLoad, ContextCreation, BrowserCreation }`.
- Produces in `bevy_cef` (crate-private):
  - `#[derive(Component)] pub(crate) struct PendingProfileBrowser { cache_path: String, navigation: Option<(String, Tick)> }`. `navigation` is the latest `RequestNavigate` URL with the `SystemChangeTick::this_run()` of the observer that stored it.
  - `pub(crate) fn initial_url<'a>(resolved: &'a str, navigation: Option<&'a (String, Tick)>, source_changed: Tick, this_run: Tick) -> &'a str`. It returns the navigation URL unless `source_changed.is_newer_than(navigation_tick, this_run)`, in which case it returns `resolved`. Comparing against the `WebviewSource` change tick, not the later `ResolvedWebviewUri` resolution in `PreUpdate`, makes the latest of the two win whatever the frame order. If a source change is still unresolved when the browser is created, `navigate_on_source_change` navigates the new browser on the next frame.
  - `#[derive(SystemParam)] pub(crate) struct BrowserCreation<'w, 's>`. It bundles the `Requester`, the IPC/BRP/cursor/drag/load/address/title senders, the `PrimaryWindow` query and window-handle lookup that `create_webview` uses today (`src/webview.rs:230`–`282`), with `fn create(&self, browsers: &mut Browsers, entity: Entity, args: CreateArgs, profile: Option<&str>) -> Result<(), CreateBrowserError>`. Both `create_webview` and the profile systems call it.
  - `pub(crate) struct CreateArgs<'a> { uri: &'a str, size: Vec2, dpr: f32, preload: &'a [String], zoom: f64, host_window: Option<Entity> }`. `BrowserCreation::create` resolves `host_window` (or the primary window) to a raw window handle at call time, so a window closed while the webview waited is not used.
  - Systems in `ProfilePlugin`, in `WebviewSet::CreateBrowser`, in this order:
    - `request_profile_browsers`: on `Added<ResolvedWebviewUri>` with `WebviewProfile`. It runs `revalidate` → `profile_dir(CefRootCachePath)` → `prepare_profile_dir` → `browsers.prepare_profile`. It then inserts `PendingProfileBrowser`, or triggers `WebviewProfileError` with the matching reason.
    - `create_ready_profile_browsers`: gated by a run condition that is true only when some pending entity's profile `is_profile_ready`, plus `not(resource_exists::<CefShutDown>)`. A false condition skips the system without validating its `BrowserCreation` parameters. For each ready entity it creates the browser from the components current at that moment: `initial_url(...)` over `ResolvedWebviewUri` and `Ref<WebviewSource>::last_changed()`, `WebviewSize`, `WebviewDpr`, `PreloadScripts`, `ZoomLevel` and `HostWindow`. It removes `PendingProfileBrowser` on both `Ok` and `Err`. On `Ok` it applies `AudioMuted(true)` and `FocusedWebview == entity` through `Browsers`; on `Err` it triggers `WebviewProfileError` once.
  - `create_webview` gets `Without<WebviewProfile>`.
  - `apply_request_navigate` (`src/navigation.rs:198`) stores `(url, this_run)` in `PendingProfileBrowser::navigation` when the target is pending, instead of calling `Browsers::navigate`.

- [ ] **Step 0: Verify sync creation after initialization first (spec "Browser creation").** Write a throwaway `examples/profile_spike.rs` and do not commit it. In a normal bevy_cef app, create a context with `cef::request_context_create_context` (`cache_path` = `<root>/profile-spike`, with a handler that sets an `Rc<Cell<bool>>`). A few frames after the flag is set, call `cef::browser_host_create_browser_sync` with a windowless `WindowInfo`, and log whether it returned a browser. If it returns null, stop and report to the user before Step 1: the fallback (creating inside `on_request_context_initialized`, which was measured) changes this task's design. Delete the file afterwards.

- [ ] **Step 1: Write the failing tests** in `src/profile.rs` (`MinimalPlugins`, `Browsers::default()` as a non-send resource, `Requester(async_channel::unbounded().0)`, `ProfilePlugin` and `NavigationPlugin`. Every case below returns before any CEF call, and `create_ready_profile_browsers` stays skipped because no profile is ready):
  - `an_invalid_name_reports_invalid_name`: spawn a `WebviewProfile` built with a `#[cfg(test)] ProfileName::unchecked(&str)` constructor holding `"Bad"`; this stands in for a Reflect write. Expect a `WebviewProfileError` with `InvalidName` and no `PendingProfileBrowser`.
  - `a_missing_root_reports_no_root`: `CefRootCachePath(None)` → `NoRoot`.
  - `a_file_at_the_profile_path_reports_a_directory_error`: a file at `<tmp root>/profile-work` → `Directory(_)`.
  - `request_navigate_while_pending_is_kept`: insert `PendingProfileBrowser { cache_path: "x".into(), navigation: None }` on an entity by hand, trigger `RequestNavigate { url: "https://b.example" }`, and assert that `navigation` holds `"https://b.example"`.
  - `a_source_change_after_request_navigate_wins`: a unit test of `initial_url` with hand-built `Tick`s. A source change newer than the navigation returns `resolved`; a navigation newer than the source change returns the navigation URL; no navigation returns `resolved`.

- [ ] **Step 2: Run them to see them fail** (`cargo test -p bevy_cef --lib profile`). Expected: compile errors.

- [ ] **Step 3: Implement the core interfaces** (`profiles.rs`, `request_context_handler.rs`, `browsers.rs`).

- [ ] **Step 4: Implement the bevy_cef interfaces** (`BrowserCreation`, `ProfilePlugin`, navigation and `create_webview` changes).

- [ ] **Step 5: Run the tests to see them pass.**

- [ ] **Step 6: Write `examples/profile.rs`.** It sets `CefPlugin { root_cache_path: Some(<absolute dir under the OS data dir, e.g. std::env::temp_dir().join("bevy_cef_profile_example")>), ..default() }`. It spawns a webview showing `https://github.com/login` with `WebviewProfile::new(std::env::args().nth(1).unwrap_or("default".into()))`, and logs every `WebviewProfileError` and `LoadFinished`. Its doc comment tells the user to log in, close the window, run again and still be logged in.

- [ ] **Step 7: Run the example.** Run `cargo run --example profile -- work` twice. Both runs log `LoadFinished` and no `WebviewProfileError`.

- [ ] **Step 8: Commit** — `feat: create profile webviews on a shared disk-backed request context`

---

### Task 6: Coalesced cookie flushing

**Files:**
- Create: `crates/bevy_cef_core/src/browser_process/cookie_flush.rs` (declare and `pub use`)
- Modify: `crates/bevy_cef_core/src/browser_process/profiles.rs` (each profile context owns a `CookieFlusher`)
- Modify: `crates/bevy_cef_core/src/browser_process/load_handler.rs` (`on_load_end` requests a flush for profile browsers)
- Modify: `crates/bevy_cef_core/src/browser_process/browsers.rs` (`flush_profile_cookies`; pass the profile's flusher to the load handler)
- Modify: `src/lib.rs` (`CefPlugin::cookie_flush_interval`, hand-written `Default`), `src/profile.rs` (timer system)
- Test: unit tests in `cookie_flush.rs`

**Interfaces:**
- Produces:
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum FlushState { Idle, InFlight { again: bool } }`.
  - `fn request(&mut self) -> bool`: returns true when a flush should start now (Idle → InFlight). When a flush is already in flight it sets `again` and returns false.
  - `fn complete(&mut self) -> bool`: returns true when a follow-up flush should start (InFlight{again: true} → InFlight{again: false}); otherwise it moves to Idle and returns false.
  - `pub struct CookieFlusher` (Clone) holding the context and an `Rc<Cell<FlushState>>`, with `pub fn request(&self)`. When the state says start, it calls `cookie_manager(None)` and then `flush_store(Some(callback))`. The callback's `on_complete` calls `complete()` and restarts if needed. A 0 return or a missing manager resets the state to `Idle` and logs a warning.
  - `LoadHandlerBuilder::build(webview, sender, initial_zoom, cookie_flusher: Option<CookieFlusher>)`. Its `on_load_end` calls `request()` for the main frame.
  - `Browsers::flush_profile_cookies(&self)` calls `request()` on every profile context.
  - `CefPlugin { pub cookie_flush_interval: Duration, .. }`, defaulting to `Duration::from_secs(5)`. `CefPlugin`'s `#[derive(Default)]` becomes a hand-written `impl Default`.
  - `ProfilePlugin` gains `cookie_flush_interval: Duration`, passed from `CefPlugin`. Its system `flush_profile_cookies(browsers: NonSend<Browsers>)` is registered with `.run_if(on_real_timer(cookie_flush_interval))`.

- [ ] **Step 1: Write the failing tests** in `cookie_flush.rs`:
  - `an_idle_request_starts_a_flush`: `Idle.request()` returns true and the state becomes `InFlight { again: false }`.
  - `requests_during_a_flush_coalesce_into_one_follow_up`: from `InFlight { again: false }`, two `request()` calls return false and the state becomes `InFlight { again: true }`. Then `complete()` returns true, a second `complete()` returns false, and the state is `Idle`.

- [ ] **Step 2: Run them to see them fail** (`cargo test -p bevy_cef_core --lib cookie_flush --features browser`).

- [ ] **Step 3: Implement the interfaces above.**

- [ ] **Step 4: Run the tests to see them pass**, and run `cargo test --doc` to make sure the `CefPlugin` docs still compile.

- [ ] **Step 5: Commit** — `feat: flush profile cookies on a wall-clock interval and after each page load`

---

### Task 7: `Cache-Control: no-store` on local scheme responses

**Files:**
- Modify: `crates/bevy_cef_core/src/browser_process/localhost/headers_responser.rs`
- Test: the existing `mod tests` in the same file

- [ ] **Step 1: Write the failing test** `prepare_sets_no_store`: after `HeadersResponser::prepare`, `headers` contains `("Cache-Control", "no-store")`, both with and without a range.
- [ ] **Step 2: Run it to see it fail** (`cargo test -p bevy_cef_core --lib headers_responser --features browser`).
- [ ] **Step 3: Push the header in `prepare`.**
- [ ] **Step 4: Run it to see it pass.**
- [ ] **Step 5: Commit** — `fix: mark cef://localhost responses no-store`

---

### Task 8: Process-level persistence tests

**Files:**
- Create: `tests/profile_persistence.rs`
- Modify: `Cargo.toml` (`[[test]] name = "profile_persistence"`, `harness = false`)

**Interfaces:**
- Consumes: everything above, through `bevy_cef::prelude` only. The live-browser check uses `NonSend<Browsers>` from `bevy_cef_core::prelude`.
- Shape: `main()` first checks `BEVY_CEF_PROCESS_TESTS`: without `=1` it prints `skipped` and returns before any CEF call, so CI passes; on macOS it always prints `skipped` (macOS is verified by hand). Only then does it call `early_exit_if_subprocess()`, under `#[cfg(not(target_os = "macos"))]` because the function does not exist on macOS. CEF subprocesses inherit the variable, so they still reach the early exit. With `BEVY_CEF_TEST_ROLE` set, it runs one role app and exits with that role's code; without it, it runs the orchestrator.
- Orchestrator: it starts a `std::net::TcpListener` HTTP server on `127.0.0.1:0` in a thread. `GET /page?mode=…` returns the test page and, for `mode=set`, `Set-Cookie: sid=…; Max-Age=31536000; HttpOnly` and `Set-Cookie: ssid=…; HttpOnly`. `GET /whoami` echoes the request's `Cookie` header. The orchestrator creates a fresh absolute root under `std::env::temp_dir()`, runs every scenario by spawning `std::env::current_exe()` with role env vars (`ROLE`, `ROOT`, `PORT`, `PROFILE`, and the scenario options), and parses `RESULT {json}` lines from the child's stdout.
- Role app: `DefaultPlugins` with `WinitPlugin` disabled and `WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, .. }`, plus `.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))`. `DefaultPlugins` includes the runner only when `bevy_window` is off (`bevy_internal-0.20.0/src/default_plugins.rs:19`), and `set` panics on a plugin missing from the group. If a bevy_cef system turns out to need a window, use the normal winit runner with `primary_window: Some(Window { visible: false, .. })` instead, and keep writing `AppExit` from `Last`. Add `CefPlugin { root_cache_path: Some(ROOT), cookie_flush_interval: 1 s, .. }` and `JsEmitEventPlugin::<PageReport>`. The page sets JS cookies and localStorage and writes one IndexedDB record in `set` mode. In both modes it reports `{server_cookies, js_cookies, local_storage, indexed_db, device_pixel_ratio, inner_width}` through `window.cef.emit`. The role prints it as a `RESULT` line. Exits go through a `Last` system writing `AppExit`, the same path a window close takes, unless the scenario says otherwise.

- [ ] **Step 1: Write the harness and these scenarios**, each asserting on the parsed results:
  - `persist`: `set` with profile `a` and a `cookie_flush_interval` of one hour; the page also sets a `late=1` JS cookie 500 ms after `load`, after the load-end flush. Then `get` with profile `a`: all four cookies, `late`, localStorage and IndexedDB are present. Only `shutdown()` can have written `late`, so this observes the window-close exit path.
  - `isolation`: `get` with profile `b` sees none of them.
  - `shared_in_process`: one role spawns two `a` webviews. The second reads the localStorage the first wrote.
  - `no_profile_keeps_nothing`: `set`, then `get`, both without `WebviewProfile`. Nothing is present.
  - `kill_after_flush`: `set` with profile `k`, wait 3 s, `std::process::exit(0)` without `AppExit`; `get` with `k` still sees `sid`.
  - `zoom`: zoom is applied after the commit, so these checks read a second report. The page re-reports when the role sends a `report` `HostEmitEvent` (the page listens with `window.cef.listen`), which the role does 500 ms after `LoadFinished`. `set` with profile `z` and `ZoomLevel(1.0)` at spawn reports `device_pixel_ratio ≈ 1.2` (±0.01). After `LoadFinished` the role sets `ZoomLevel(2.0)`. A following `get` with `z` and the default `ZoomLevel` reports `≈ 1.44`.
  - `state_while_pending`: a role spawns an `a` webview with `WebviewSize(Vec2::new(800., 600.))`, and in the next frame (still waiting) sets `WebviewSize(Vec2::new(640., 480.))`, `AudioMuted(true)` and `ZoomLevel(1.0)`. The report has `inner_width == 640`. After `LoadFinished`, `browsers.is_audio_muted(&entity) == Some(true)`, and the second report (as in `zoom`) has `device_pixel_ratio ≈ 1.2`.
  - `exit_while_pending`: a role spawns an `a` webview and writes `AppExit` in the same frame. The child exits with code 0 within 10 s.
  - `despawn_then_respawn`: a role spawns an `a` webview and despawns it next frame, waits 2 s, and asserts `live_browser_count() == 0` and that no `LoadFinished` arrived for the despawned entity. It then spawns a new `a` webview, which reports. The child exits 0.
  - `second_process`: a holder role (`a`) prints `READY` once its page has reported and then holds for 8 s. The orchestrator starts the second role on the same root only after reading `READY`. The second reports `CefStatus::Failed(e)` with `e.is_root_in_use()`, and both exit 0.
- [ ] **Step 2: Run without the env var** (`cargo test -p bevy_cef --test profile_persistence`). Expected: `skipped`, exit 0.
- [ ] **Step 3: Run for real on Windows**: `BEVY_CEF_PROCESS_TESTS=1 CEF_PATH=$HOME/.cache/orzma/cef PATH="$PWD/target/debug:$PATH" cargo test -p bevy_cef --test profile_persistence`. Expected: every scenario prints `ok`. Fix the owning task's code for any failure; never the assertion's expected value.
- [ ] **Step 4: Run the CI commands** from Global Constraints. Expected: green.
- [ ] **Step 5: Commit** — `test: add process-level profile persistence tests`

---

### Task 9: Documentation

**Files:**
- Create: `docs/website/docs/guides/profiles.md`
- Modify: `docs/website/sidebars.ts` (add `'guides/profiles'` before `'guides/zoom-and-audio'`)
- Modify: `docs/website/docs/guides/zoom-and-audio.md` (the opening sentence's "per-webview zoom"; a "Profiles" subsection; the first-commit timing)
- Modify: `docs/website/docs/reference/plugin-configuration.md` (`cookie_flush_interval`, `CefStatus`, root requirements)
- Modify: `docs/website/docs/reference/components-and-events.md` (`WebviewProfile`, `WebviewProfileError`, `CefStatus`)
- Modify: `src/common/components.rs` (`ZoomLevel` rustdoc, `:122`–`126`: the value is a request, it is applied at the first main-frame commit, and in a profile it is shared per site and remembered)
- Modify: `CLAUDE.md` (Core Components: `WebviewProfile`; Key Non-Obvious Patterns: the shutdown system's placement and why)

- [ ] **Step 1: Write the profiles guide.** Cover what a profile keeps, that `WebviewProfile` is read once at creation, the name rules, `root_cache_path` and the `profile-NAME` directory, and that one root is usable by one process at a time (exit codes 21 and 24 through `CefStatus`; lock the root yourself before adding `CefPlugin`). Also cover sharing within a process; that `cef://localhost` pages in one profile share an origin; per-site zoom; the crash window of one flush interval; the undetected Chromium-layer failure; the macOS debug/release keychain caveat (spec "Non-goals"); and that on Windows a hung UI thread in the first process can still get it terminated by a second process (spec "Startup failure and a second process").
- [ ] **Step 2: Update the zoom guide, the references and `CLAUDE.md`** as listed above. Run `cargo test --doc` if any Rust code block changed.
- [ ] **Step 3: Commit** — `docs: document webview profiles, CefStatus and profile zoom`

---

## Manual verification after Task 9

- Windows (builder): `cargo run --example profile -- work`. Log in, close the window, run again: still logged in. Run with `-- other`: logged out.
- Windows (builder): while Task 8's `second_process` scenario runs, no Chrome-style window appears.
- macOS (user, **release build**): the same steps with `cargo run --release --example profile -- work`.
