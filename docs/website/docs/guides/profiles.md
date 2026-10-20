---
sidebar_position: 8
---

# Profiles

By default every webview starts empty: cookies, `localStorage` and `IndexedDB` live in memory and vanish when the app exits. A **profile** is a named, disk-backed store for a webview, so a login survives an app restart and stays separate from other profiles.

## What a profile keeps

A webview with a profile keeps its cookies (including `HttpOnly` and session cookies), `localStorage` and `IndexedDB` across restarts. Profiles with different names are isolated from each other. A webview without a profile behaves as before: it has its own in-memory store and keeps nothing.

## Setting up a profile

A profile needs two things: a persistent, absolute `root_cache_path` on `CefPlugin`, and a `WebviewProfile` component on the webview.

```rust
use bevy::prelude::*;
use bevy_cef::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CefPlugin {
                root_cache_path: Some("/home/me/.myapp/cef".to_string()),
                ..default()
            },
        ))
        .add_systems(Startup, spawn_webview)
        .run();
}

fn spawn_webview(mut commands: Commands) {
    commands.spawn((
        WebviewSource::new("https://github.com/login"),
        WebviewProfile::new("work").expect("a valid profile name"),
        // ... node or mesh and material
    ));
}
```

Run `cargo run --example profile -- work` to try it: log in, close the window and run it again to find the page still logged in. Use `-- other` to get a separate, logged-out profile.

### Profile names

`WebviewProfile::new(name)` accepts a name of 1 to 64 characters drawn from `a`-`z`, `0`-`9`, `-` and `_`. Anything else, uppercase letters included, is rejected with a `ProfileNameError` and never rewritten. A `ProfileName` deserialized from serde or BRP is checked the same way.

### Where a profile lives

A profile named `NAME` is stored in `<root_cache_path>/profile-NAME`. The path is joined onto the `root_cache_path` string exactly as you wrote it, and bevy_cef creates the directory when it is first needed. `root_cache_path` must be a non-empty absolute path; without one, a webview with a profile gets a `WebviewProfileError` instead of a browser.

### The component is read once

`WebviewProfile` is read once, when the webview's browser is created. Changing it afterwards, inserting it later and removing it have no effect. To use a different profile, spawn a new webview.

## Sharing within a process

Webviews in the same process that use the same profile name share one store. If you open two webviews with `"work"`, a login in one is visible to the other. Webviews without a profile each keep their own in-memory store and never share.

Pages served from `cef://localhost` in the same profile also share one origin, and therefore share its `localStorage` and cookies.

## Waiting for the profile

A profile webview's browser is created once its profile is ready, a short time after the entity is spawned. While it waits:

- The current URL, size, device pixel ratio, mute state and focus are used when the browser is created, so changes made while waiting are not lost.
- A `RequestNavigate` replaces the URL the browser is created with; the latest one wins.
- Back, forward and reload requests are dropped.
- Despawning the entity cancels the creation.

## Handling errors

When a profile webview cannot get a browser, bevy_cef triggers a `WebviewProfileError` on the webview entity and leaves the webview without a browser. It never falls back to in-memory storage, so a login is not silently lost.

```rust
fn log_profile_error(error: On<WebviewProfileError>) {
    warn!("profile webview {} failed: {:?}", error.webview, error.reason);
}
```

The `reason` is a `WebviewProfileErrorReason`:

| Reason | Meaning |
|--------|---------|
| `InvalidName` | The name is not a valid `ProfileName`. This can only happen if the component was written through reflection. |
| `NoRoot` | `root_cache_path` is unset, blank or not absolute. |
| `Directory` | The profile directory could not be created or written. |
| `DidNotLoad` | CEF rejected the profile directory and would have kept the profile in memory. |
| `ContextCreation` | CEF did not create the profile's store. |
| `BrowserCreation` | CEF did not create the browser. |

:::caution A failure that is not detected

If Chromium fails to load a profile after bevy_cef's checks, it falls back to an in-memory profile without telling anyone. bevy_cef cannot detect this case, so no `WebviewProfileError` is sent and the webview keeps nothing across restarts.

:::

## One process per root

A `root_cache_path` can be used by one process at a time. If a second process starts on a root that the first one holds, CEF fails to initialize with exit code 21 or 24. bevy_cef does not panic and does not exit your app. It records the failure in the `CefStatus` resource and adds none of its plugins that call into CEF, so no CEF function is called afterwards.

```rust
fn check_cef(status: Res<CefStatus>, mut exit: MessageWriter<AppExit>) {
    if let CefStatus::Failed(error) = &*status {
        if error.is_root_in_use() {
            eprintln!("another instance already uses this cache directory");
        }
        exit.write(AppExit::error());
    }
}
```

The first process is unaffected: it opens no window when the second one starts.

Choosing where the root lives and keeping two processes off it is your app's job. The most robust way is to take your own lock on the root (for example a lock file) **before** adding `CefPlugin`, and decide what to do when the lock is held.

:::warning Windows: keep the first process's message loop running

On Windows, a second process notifies the first one and waits for an answer. If the first process does not answer for about 5 seconds, the second shows a "kill hung browser" prompt and terminates it. The first process answers only while its thread keeps pumping a native message loop. An app with a winit window does. A windowless app, such as one on `ScheduleRunnerPlugin` with no window, pumps none, so a second process on the same root judges it hung and terminates it. Locking the root yourself before adding `CefPlugin` avoids this entirely.

:::

## Cookies and crashes

bevy_cef flushes the cookies of every profile to disk every `CefPlugin::cookie_flush_interval` (5 seconds by default, measured in wall-clock time) and after each main-frame load in a profile webview. A crash or a kill therefore loses only the cookie changes since the last completed flush. A normal exit, including closing the window, shuts CEF down cleanly and persists everything.

## Zoom is shared per site

Inside a profile, Chromium keeps one zoom level per site, shared by every webview of that profile showing the site, and remembers it across restarts. See [Zoom & Audio](./zoom-and-audio.md#profiles) for how `ZoomLevel` behaves there.

## Cookie encryption on macOS

On macOS, Chromium encrypts cookies with a key from the keychain. bevy_cef's default `CommandLineConfig` sets `use-mock-keychain` in **debug builds only**, and your app may set it itself. Two consequences:

- A debug build and a release build that share one root encrypt cookies with different keys and lose each other's cookies. Use separate roots for the two.
- Release builds show the "Chromium Safe Storage" keychain prompt.

bevy_cef does not change this behavior.

## Limits

- The profile of a webview cannot change after the webview is created.
- bevy_cef offers no API to list, delete or clear profiles. You can delete a profile directory while no process uses the root.
- Contexts of profiles are kept until the app exits, not released when their last webview closes.
- Popups open in the profile of the webview that opened them.
- Windows and macOS are tested. Linux compiles and uses the same code path, but is not verified.
