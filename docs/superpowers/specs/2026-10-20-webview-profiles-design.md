# Webview Profiles — Design

Date: 2026-10-20
Status: Approved in brainstorming; builder check done; revised after a parallel Codex + Claude spec review; pending user review
Target version: 0.15.0 (breaking: a new `CefPlugin` field, and a failed `cef_initialize` no longer panics)

## Goal

Let an app give a webview a **named, disk-backed profile**, so the page's login state
(cookies, localStorage, IndexedDB) survives an app restart and stays separate per name.
Webviews without a profile keep today's in-memory behavior.

The motivating consumer is orzma's browser app, which wants
`orzbrowser {URL} --profile NAME`. The bevy_cef feature itself is generic: it knows
nothing about orzma, CLIs or protocols.

## Background

### What bevy_cef does today (0.14.0)

- **Every browser gets a fresh in-memory context.** `Browsers::request_context()`
  (`crates/bevy_cef_core/src/browser_process/browsers.rs:608`) calls
  `request_context_create_context` with `RequestContextSettings::default()` (empty
  `cache_path`), then registers the `cef://localhost` scheme handler factory and the custom
  scheme factories on that context. If creation returns `None`, the `None` is passed on to
  browser creation, and CEF then uses the global context.
- **Browser creation is synchronous.** `create_webview` (`src/webview.rs:230`) runs on
  `Added<ResolvedWebviewUri>` and calls `Browsers::create_browser`, which calls
  `browser_host_create_browser_sync(...).expect("Failed to create browser")`
  (`browsers.rs:112`–`159`) and stores the browser in the entity map.
- **State sync is change-driven.** Size, zoom, mute, DPR, focus and navigation are pushed
  to CEF by systems reacting to `Changed<…>`, and do nothing when the entity has no
  browser (for example `src/zoom.rs:17`, `src/mute.rs:16`, `src/common/dpi.rs:70`,
  `src/focus.rs:52`, `src/webview.rs:283`).
- **Despawn closes only created browsers** (`src/webview.rs:166`–`170`).
- **`cef_initialize` is asserted to return 1** (`src/common/message_loop.rs:127`, `:168`),
  inside `MessageLoopPlugin::build`. Creating `root_cache_path` also panics on failure
  (`message_loop.rs:107`–`109`, `:149`–`151`).
- **`shutdown()` runs from `Update` on `AppExit`** (`message_loop.rs:83`, `:208`), without
  closing browsers first, and it does not run at all when a window is closed: Bevy 0.20
  writes that `AppExit` from `Last` (`bevy_window-0.20.0/src/lib.rs:141`), and the winit
  runner then exits and clears the world without another frame
  (`bevy_winit-0.20.0/src/state.rs:760`, `:521`). bevy_cef has no `LifeSpanHandler`.
- **`on_already_running_app_relaunch` is not implemented** in the browser process handler.
- **Inline HTML** is served at `cef://localhost/__inline__/{id}` from a per-process counter
  that starts at 0 (`src/common/localhost/responser.rs:8`), without a `Cache-Control`
  header (`crates/bevy_cef_core/src/browser_process/localhost/headers_responser.rs`).

### CEF rules (cef 154 bindings)

- A request context's `cache_path` must be absolute and equal to or a child of
  `root_cache_path`; an empty `cache_path` means in-memory ("incognito") storage.
  `persist_session_cookies` is ignored for an empty `cache_path`.
- `cef_initialize` returns 0 when initialization fails **or when early exit is desired**,
  for example because of process-singleton relaunch. The application must then exit
  immediately without calling any other CEF function except `cef_get_exit_code`.
- A NULL request context passed to browser creation means the global context.

### Measured on Windows (CEF 154, standalone offscreen probe, 2026-10-20)

The probe set an HttpOnly persistent cookie, an HttpOnly session cookie, two JS cookies and
a localStorage item from a local HTTP page, then started a new process and read them back.

| Case | Result |
|---|---|
| `cache_path = <root>/<name>` | Everything survived the restart. |
| A different name in the same root | Saw nothing: profiles are isolated. |
| `cache_path = <root>/profiles/<name>` | CEF logged `Cannot create profile at path …`, ran in memory, kept nothing. Browser creation succeeded, so the failure is silent. |
| `persist_session_cookies = 0` | Session cookies were lost; persistent ones kept. |
| Exit through `shutdown()` with the browser still open (today's bevy_cef path) | Everything survived. |
| Exit without `shutdown()` (crash-like) | **All cookies lost**; localStorage survived. |
| `create_browser_sync` right after creating a disk context | **Returned null**, for a new and for an existing profile. Async creation worked. |
| `create_browser_sync` inside `on_request_context_initialized` | Worked. |
| Two contexts with the same `cache_path` in one process | `is_sharing_with` returned 1: they share storage. |
| A second process on the same root | `cef_initialize` returned 0, exit code 24. The **first** process logged `Creating a chrome browser without a client`, opened a default Chrome-style window, and did not shut down cleanly afterwards. |
| Two browsers on one host in one profile context; `set_zoom_level(2.0)` on the first | **The second also reported 2.0.** |
| Restart with the same profile, same host | **The new browser started at 2.0**, read from `partition.per_host_zoom_levels` in the profile's `Preferences`. |
| Restart with a different profile | Started at 0.0. |
| Two browsers on one host sharing one in-memory context | Shared the zoom as well: sharing comes from the shared context, not from the disk. |

macOS and Linux were not measured. IndexedDB was not measured.

### Why zoom is shared

CEF's `CefBrowserHostBase::SetZoomLevel` calls Chromium's `ZoomController::SetZoomLevel`.
In the default zoom mode, Chromium's `ZoomController::SetZoomLevelByClient` stores the level
with `HostZoomMap::SetZoomLevelForHost(host, level)`: per hostname (no port or scheme), in
the map of the context's storage partition, which every browser of that context uses. A
disk-backed Chrome profile persists that map through `ChromeZoomLevelPrefs`. Chromium's
per-tab `ZOOM_MODE_ISOLATED` exists, but CEF 154 exposes no API to select it, and
re-applying a level after each load cannot give two webviews on one host different zooms.
Today every webview has its own context, which is why `ZoomLevel` currently behaves per
webview.

## Goals

1. A webview can be given a named profile; its cookies (including HttpOnly and session
   cookies), localStorage and IndexedDB persist across app restarts.
2. Different profile names are isolated; webviews in one process that name the same
   profile share storage.
3. Webviews without a profile behave exactly as today.
4. A crash or kill loses only the cookie changes made since the last completed flush, a few
   seconds at most.
5. A failed `cef_initialize` (including "another process holds this root") no longer
   panics, is reported to the app, and is followed by no further CEF call.
6. A second process on the same root never makes the first process open a window.

## Non-goals

- Choosing where the root lives, and locking the root across processes. The app does this
  before adding `CefPlugin` (orzma will).
- Changing a webview's profile after the webview is created.
- Profile management: listing, deleting or clearing profiles. An app can delete a profile
  directory while no process uses it. *(Claude's call.)*
- Changing popup or DevTools behavior.
- Changing cookie encryption. On macOS this depends on `use-mock-keychain`, which bevy_cef's
  default `CommandLineConfig` sets only in debug builds
  (`crates/bevy_cef_core/src/browser_process/command_line_config.rs:40`) and which apps may
  set themselves; the docs state this, the code does not change it. The docs also state the
  consequences: a debug build and a release build sharing one root encrypt cookies with
  different keys and lose each other's cookies, and release builds show the "Chromium Safe
  Storage" keychain prompt.

## Anti-goals

These make the feature a failure even if persistence works:

- A webview that asked for a named profile running in memory, in the global context, or in
  another profile's storage, without the app being told, for any cause bevy_cef can detect
  (see "Detecting a profile that did not load").
- Any CEF call after `cef_initialize` returned 0.
- A visible Chrome window appearing because a second process started.
- A regression for webviews without a profile.

## Design

### Public API

- **`WebviewProfile` component.** Added next to `WebviewSource` on the webview entity.
  - Built with `WebviewProfile::new(name) -> Result<Self, …>`, which validates the name.
  - It derives `Reflect` and `Serialize`/`Deserialize` like the other webview components.
    The name is a validated type that deserializes with `#[serde(try_from = "String")]`, so
    serde input is checked where it enters. Reflect can still bypass it, so the name is
    validated again when the browser is created.
  - It is not `#[require]`d by `WebviewSource`; no component means in-memory.
  - It is read once, when the webview's browser is requested. Later changes, insertion
    after that point, and removal have no effect; the docs say so.
- **Name rules.** Characters `a`–`z`, `0`–`9`, `-` and `_`; length 1 to 64. Anything else,
  uppercase included, is rejected, never rewritten. This sidesteps case-insensitive file
  systems and keeps names portable.
- **Directory.** A profile named `work` lives at `<root_cache_path>/profile-work`. It must
  be a direct child of the root (measured above); the `profile-` prefix keeps names clear of
  the directories Chromium itself creates in the root (`Default`, `ShaderCache`, …) and of
  Windows reserved device names. *(Claude's call: the prefix.)*
- **Root requirement.** A named profile requires `CefPlugin::root_cache_path` to be a
  non-empty absolute path. Otherwise the webview fails with the error below.
- **Context settings.** Profile contexts use `persist_session_cookies = 1`.
- **Failure event.** A new `EntityEvent` on the webview entity (name suggestion:
  `WebviewProfileError`) carrying a reason: invalid name, no usable root, profile directory
  could not be created or written, the profile did not load as a disk profile, context
  creation failed, browser creation failed. The webview is then left without a browser;
  there is no fallback to in-memory storage.
- **Cookie flush interval.** A new `CefPlugin` field (name suggestion:
  `cookie_flush_interval: Duration`) with a default of a few seconds.
- **Startup status.** A resource (name suggestion: `CefStatus`) that says whether CEF is
  running or failed to start, with the exit code on failure. Exit codes 24
  (`CEF_RESULT_CODE_NORMAL_EXIT_PROCESS_NOTIFIED`) and 21 (`CEF_RESULT_CODE_PROFILE_IN_USE`,
  also used for lock errors) both mean another process holds or locks the root. bevy_cef
  does not exit the app; the app reads the resource and decides.

### Detecting a profile that did not load

CEF hands back a request context before it knows whether the profile loaded, and a profile
that fails to load runs in memory while browser creation still succeeds (measured with a
nested path). So bevy_cef checks the two layers it can reach:

- **Before creating the context:** the path is `profile-<name>` joined onto the exact
  `root_cache_path` string; neither side is canonicalized (`std::fs::canonicalize` would
  produce a `\\?\` path on Windows, where CEF and Chromium compare paths textually).
  bevy_cef creates the directory before creating the context, because on POSIX CEF resolves
  both paths with `realpath` only when they exist (a macOS root under `/var/folders`
  resolves to `/private/var/...`). It creates it with mode 0700 on Unix and lets it inherit
  the root's ACL on Windows, then confirms it can write a file there. Any failure is
  reported and no context is created.
- **Right after creating the context:** `context.cache_path()` must be non-empty. CEF
  validates the path synchronously inside `request_context_create_context`
  (`NormalizeCachePathAndSet`) and replaces an invalid one with an empty string, logging
  "Defaulting to in-memory storage". An empty result is reported with the "did not load"
  reason and the profile's webviews get no browser.
- **Not detected:** a failure inside Chromium's profile creation after these checks falls
  back to an in-memory profile without clearing the cache path, and
  `on_request_context_initialized` carries no result, so neither it nor its timing is a
  signal. The docs state this residual case.

### Contexts

- **One context per profile directory per process**, shared by every webview naming that
  profile. It is created on first use and shared even while it is still initializing; it is
  kept until shutdown. *(Claude's call: no release when the last webview closes.)*
- Webviews without a profile keep getting their own in-memory context, as today. They must
  not end up sharing one context because they all have an empty path.
- **Per-context setup** (the `cef://localhost` factory and the custom scheme factories)
  happens once per context and completes before any browser on it navigates.
- **Per-browser state** stays per webview: the client and its handlers, the IPC entity,
  paint slots, and `extra_info` with the preload scripts.

### Browser creation

- A webview with a profile is recorded as pending when it is added. When its context
  reports `on_request_context_initialized`, the callback only sends a "profile ready"
  message; a Bevy system then creates every pending, still-alive webview of that profile
  with the existing `create_browser_sync`, reading the components at that moment. No
  browser is ever in flight for an entity. Sync creation on a frame after initialization
  follows from CEF's `VerifyBrowserContext` check but was not measured; the plan verifies
  it first.
- Webviews without a profile keep today's creation path unchanged.
- While a webview is waiting:
  - When the browser is created, it uses the **current** values of the URL, size, DPR,
    mute and focus, not the values at request time, so changes made while waiting are not
    lost. Zoom follows the rule in "Zoom in profile webviews".
  - `HostEmitEvent` sent before the page is ready is not guaranteed to arrive; this is the
    same as today.
- A `RequestNavigate` that arrives while waiting replaces the URL the browser is created
  with (the latest wins). Back, forward and reload requests while waiting are dropped.
- **Cancellation.** Despawning a waiting entity drops its pending entry.
- On macOS the parent `NSView` is resolved when the browser is created, so a window closed
  in between is not used.

### Zoom in profile webviews

Decided with the user after measuring and a Codex review: profile webviews follow Chrome's
behavior instead of trying to stay per webview.

- **Zoom is per site within a profile.** Webviews of one profile showing the same host share
  one zoom level, and the profile remembers it across restarts.
- **The initial zoom is applied at the first main-frame commit.** Chromium ignores a zoom
  set before the main frame has a live renderer and a committed URL
  (`ZoomController::SetZoomLevelByClient` returns early), so a value sent at creation is
  lost. A non-default `ZoomLevel` present at creation, or changed before the first commit,
  is held per browser and applied once in `LoadHandler::on_load_start` for the main frame.
  A default `0.0` is not sent, so the site's remembered level applies. The change-sync
  system (`src/zoom.rs`) does not send the component's initial value, which `Changed`
  includes; after the first commit every change is sent, as today. The same early return
  drops a spawn-time zoom today, so this applies to every webview.
- **`ZoomLevel` is a request, not a mirror.** For profile webviews the component can differ
  from the zoom actually shown, because another webview or a remembered level changed it.
  No API for reading the effective zoom is added. *(Claude's call.)*
- **Webviews without a profile are unchanged**, and keep per-webview zoom because each still
  has its own context.
- The `ZoomLevel` docs and the zoom guide (`docs/website/docs/guides/zoom-and-audio.md`,
  which promises "per-webview zoom") are updated to say this.

### Saving cookies

- bevy_cef flushes the cookie store of every disk-backed context (`CookieManager::flush_store`)
  on the configured interval, and after each main-frame load end in a profile webview.
  In-memory contexts are not flushed.
- Flushes are coalesced per context: at most one is in flight, and requests that arrive
  meanwhile become one follow-up flush. The interval uses wall-clock time
  (`on_real_timer`), so an app that pauses or scales virtual `Time` keeps flushing.
- This narrows the crash loss to cookie changes since the last flush. localStorage already
  survived a kill in the measurement; IndexedDB is checked by the tests below.

### Shutdown

bevy_cef runs the shutdown sequence in a system that sees every `AppExit` of the frame: in
`Main` after `Main::run_main`, and after the macOS terminate-request writer. Because the
runner exits without another frame, the whole sequence runs as one blocking call:

1. Stop creating browsers and drop pending requests.
2. Close all browsers and pump `do_message_loop_work` until the live-browser count is 0 or
   a time limit passes; on the limit, log how many are still open and continue.
3. Release every CEF reference bevy_cef holds (the browser map, contexts, pending
   requests, cookie managers), since `world.clear_all()` would otherwise drop them after
   `shutdown()`.
4. Call `shutdown()`, which persisted everything in the measurement even with a browser
   still open. Message-loop work never runs after it.

No explicit cookie flush runs at shutdown: `flush_store` completes asynchronously, and
`shutdown()` alone persisted everything.

The `LifeSpanHandler` only counts live browsers (`on_after_created` adds one,
`on_before_close` removes one). Popups share the client and are counted; DevTools has its
own client. Its callbacks never touch the entity map.

### Startup failure and a second process

- The browser process handler implements `on_already_running_app_relaunch` and returns 1,
  so CEF opens no window in the first process.
- Returning 1 helps only while the first process's UI thread keeps pumping. On Windows a
  second process whose notification goes unanswered for about 5 s shows a "kill hung
  browser" prompt and terminates the first. An app that locks the root itself before CEF
  starts (as orzma will) never reaches this; the docs state it.
- When `cef_initialize` returns 0, `CefPlugin::build` records the exit code in the startup
  status resource and does not add the sub-plugins that call CEF (message loop, webview
  creation, IPC, input, …), so nothing can call CEF afterwards, including
  `Browsers::emit_event`, which calls `process_message_create` before looking up a browser
  (`browsers.rs:338`). No `shutdown()` follows.
- Failing to create `root_cache_path`, and on macOS failing to load the CEF library
  (`message_loop.rs:93`–`94`), are reported the same way instead of panicking.

### Local scheme responses

- Responses served from `cef://localhost` (inline HTML and local assets) carry
  `Cache-Control: no-store`, so a disk cache never serves a previous run's inline page whose
  counter-based URL is reused. *(Claude's call: covering local assets as well as inline
  HTML.)*
- The docs state that `cef://localhost` pages in the same profile share one origin, and
  therefore share its storage.

### Popups and DevTools

Unchanged. Popups inherit the opener's context (CEF default), so a popup from a profile
webview uses the same profile.

## Platform scope

- **Windows and macOS** are in scope and verified by hand: Windows by the builder, macOS by
  the user on a Mac, with a release build.
- **Linux** must compile and uses the same code path; it is not verified by hand.
  *(Claude's call.)*

## Testing

- **Unit tests:** name validation (accepted and rejected names), name-to-directory mapping,
  the root requirement.
- **Process-level tests.** CEF can be initialized only once per process, so persistence is
  checked by starting a child process twice against the same root, with an integration test
  that sets `harness = false`, calls `early_exit_if_subprocess()` first, and re-executes
  itself with a role variable:
  - cookies (HttpOnly, session), localStorage and IndexedDB survive a restart;
  - two names are isolated; two webviews with one name share;
  - a webview without a profile keeps nothing;
  - a cookie set before a completed flush survives a kill;
  - closing the window, not only an `AppExit` from a system, still runs `shutdown()`;
  - a non-default zoom set while waiting is applied once the browser exists, and a default
    `ZoomLevel` leaves a remembered site zoom in place;
  - mute and size changed while waiting are applied once the browser exists;
  - despawning while waiting leaves no browser behind;
  - a second process on the same root reports failure, with exit code 24, and the first
    process opens no window and still shuts down cleanly.
- **Example:** a `profile` example showing a webview with a named profile. *(Claude's call.)*
- **Manual macOS check** by the user, with a release build: log in to a site in a profile webview, restart, still
  logged in; a second profile is not.

## Notes for the consumer (orzma)

These are outside bevy_cef but shape its API:

- orzma passes a persistent absolute `root_cache_path` and takes its own cross-process lock
  on it before adding `CefPlugin`, so CEF's process singleton never fires. When the lock is
  held elsewhere, orzma chooses what to do (for example an ephemeral per-PID root with
  profile requests refused).
- orzma maps `orzbrowser --profile NAME` to `WebviewProfile::new(NAME)` and reports
  `WebviewProfileError` back to the client.
- orzma reads the startup status resource.

## Left to the builder

- Exact type, field and variant names, and the error type.
- The default flush interval ("a few seconds") and the shutdown time limit.
- How pending requests and shared contexts are stored.

## Calls made in brainstorming

The user approved the overall design. These specific points were Claude's calls and are
open to correction:

- The `profile-` directory prefix.
- No profile management API.
- Contexts are kept until shutdown, not released when their last webview closes.
- `no-store` on all `cef://localhost` responses, not only inline HTML.
- Linux is compile-only.
- A `profile` example is added.
- A Chromium-layer profile failure that leaves no signal is documented, not detected.
- No API for reading a profile webview's effective zoom.
