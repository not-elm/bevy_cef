use crate::common::WebviewSource;
use crate::focus::FocusedWebview;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
#[cfg(not(target_os = "windows"))]
use bevy_cef_core::prelude::Browsers;
#[cfg(target_os = "windows")]
use bevy_cef_core::prelude::BrowsersProxy;
use bevy_cef_core::prelude::{create_cef_key_events, keyboard_modifiers};
use serde::{Deserialize, Serialize};

/// The plugin to handle keyboard inputs.
///
/// Keyboard and IME input is delivered to the webview that currently holds
/// focus ([`FocusedWebview`]), which is set when a webview is clicked. A webview
/// therefore receives keyboard input only after it has been clicked at least
/// once; while no webview is focused, input is dropped.
///
/// To use IME, you need to set [`Window::ime_enabled`](bevy::prelude::Window) to `true`.
pub(super) struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsImeCommiting>()
            .init_resource::<IsImeComposing>();

        #[cfg(not(target_os = "windows"))]
        app.add_systems(
            Update,
            (
                // Workaround for bevy_winit not calling `set_ime_allowed` on initial window
                // creation when `Window::ime_enabled` is `true` from the start.
                activate_ime,
                ime_event.run_if(on_message::<Ime>),
                send_key_event.run_if(on_message::<KeyboardInput>),
            )
                .chain(),
        );

        #[cfg(target_os = "windows")]
        app.add_systems(
            Update,
            (
                activate_ime,
                ime_event_win.run_if(on_message::<Ime>),
                send_key_event_win.run_if(on_message::<KeyboardInput>),
            )
                .chain(),
        );
    }
}

/// Workaround: bevy_winit does not call `winit::Window::set_ime_allowed()` during initial window
/// creation when `Window::ime_enabled` is `true`. This means `Ime` events are never generated.
///
/// To trigger bevy_winit's own `changed_windows` system, we temporarily toggle `ime_enabled` off
/// then back on over two frames, which causes the change detection to fire and call
/// `set_ime_allowed(true)` internally.
fn activate_ime(mut windows: Query<&mut Window>, mut state: Local<ImeActivationState>) {
    match *state {
        ImeActivationState::Pending => {
            for mut window in windows.iter_mut() {
                if window.ime_enabled {
                    window.ime_enabled = false;
                    *state = ImeActivationState::Toggled;
                }
            }
        }
        ImeActivationState::Toggled => {
            for mut window in windows.iter_mut() {
                if !window.ime_enabled {
                    window.ime_enabled = true;
                    *state = ImeActivationState::Done;
                }
            }
        }
        ImeActivationState::Done => {}
    }
}

#[derive(Default)]
enum ImeActivationState {
    #[default]
    Pending,
    Toggled,
    Done,
}

#[derive(Resource, Default, Serialize, Deserialize, Reflect)]
#[reflect(Default, Serialize, Deserialize)]
struct IsImeCommiting(bool);

/// Tracks whether CEF has an active IME composition.
///
/// Set to `true` when `ImeSetComposition(non-empty)` is called, cleared only on
/// `ImeCancelComposition()` or `ImeCommitText()`. Critically, empty `Preedit` does NOT clear this
/// flag — this avoids the same-frame ordering problem where `ime_event` processes `Preedit { "" }`
/// before `send_key_event` processes the BackSpace that caused it.
#[derive(Resource, Default, Serialize, Deserialize, Reflect)]
#[reflect(Default, Serialize, Deserialize)]
struct IsImeComposing(bool);

#[cfg(not(target_os = "windows"))]
fn send_key_event(
    mut er: MessageReader<KeyboardInput>,
    mut is_ime_commiting: ResMut<IsImeCommiting>,
    mut is_ime_composing: ResMut<IsImeComposing>,
    input: Res<ButtonInput<KeyCode>>,
    browsers: NonSend<Browsers>,
    focused: Res<FocusedWebview>,
    webviews: Query<Entity, With<WebviewSource>>,
) {
    let modifiers = keyboard_modifiers(&input);
    let target = focused.0.filter(|e| webviews.get(*e).is_ok());
    for event in er.read() {
        if (event.key_code == KeyCode::Enter || event.key_code == KeyCode::Backspace)
            && is_ime_commiting.0
        {
            is_ime_commiting.0 = false;
            continue;
        }
        if event.key_code == KeyCode::Backspace && is_ime_composing.0 {
            is_ime_composing.0 = false;
            continue;
        }
        // Deliver only to the explicitly-focused webview. When nothing is
        // focused — before the first click, or focus is on a non-webview surface
        // (e.g. a terminal pane in an embedder) — no webview receives keys.
        // Broadcasting to all webviews here leaked keystrokes to a
        // previously-focused webview: `send_key` is gated on CEF's
        // `focused_frame()`, which survives `set_focus(false)`, so the blurred
        // webview kept receiving input.
        let Some(webview) = target else {
            continue;
        };
        for key_event in create_cef_key_events(modifiers, &input, event) {
            browsers.send_key(&webview, key_event);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn ime_event(
    mut er: MessageReader<Ime>,
    mut is_ime_commiting: ResMut<IsImeCommiting>,
    mut is_ime_composing: ResMut<IsImeComposing>,
    browsers: NonSend<Browsers>,
    focused: Res<FocusedWebview>,
    webviews: Query<Entity, With<WebviewSource>>,
) {
    let has_target = focused.0.filter(|e| webviews.get(*e).is_ok()).is_some();
    if !has_target {
        // No webview is focused (focus is on a non-webview surface, e.g. a
        // terminal pane in an embedder). Cancel any composition still live on
        // the previously-focused webview — CEF keeps its focused frame after
        // `set_focus(false)`, so the cancel reaches it — and clear the shared
        // IME flags so a later keystroke on a re-focused webview is not wrongly
        // suppressed by stale composition state.
        if is_ime_composing.0 {
            browsers.ime_cancel_composition();
        }
        is_ime_composing.0 = false;
        is_ime_commiting.0 = false;
    }
    for event in er.read() {
        // Drive CEF IME only when a webview is focused. With focus on a
        // non-webview surface (e.g. a terminal pane in an embedder), routing IME
        // here would leak composition to a previously-focused webview whose CEF
        // `focused_frame()` survives `set_focus(false)` — the same leak as keys.
        if !has_target {
            continue;
        }
        match event {
            Ime::Preedit { value, cursor, .. } => {
                if value.is_empty() {
                    browsers.ime_cancel_composition();
                } else {
                    browsers.set_ime_composition(value, cursor.map(|(_, e)| e as u32));
                    is_ime_composing.0 = true;
                }
            }
            Ime::Commit { value, .. } => {
                browsers.set_ime_commit_text(value);
                is_ime_commiting.0 = true;
                is_ime_composing.0 = false;
            }
            Ime::Disabled { .. } => {
                browsers.ime_cancel_composition();
                is_ime_composing.0 = false;
            }
            _ => {}
        }
    }
}

#[cfg(target_os = "windows")]
fn send_key_event_win(
    mut er: MessageReader<KeyboardInput>,
    mut is_ime_commiting: ResMut<IsImeCommiting>,
    mut is_ime_composing: ResMut<IsImeComposing>,
    input: Res<ButtonInput<KeyCode>>,
    proxy: Res<BrowsersProxy>,
    focused: Res<FocusedWebview>,
    webviews: Query<Entity, With<WebviewSource>>,
) {
    let modifiers = keyboard_modifiers(&input);
    let target = focused.0.filter(|e| webviews.get(*e).is_ok());
    for event in er.read() {
        if (event.key_code == KeyCode::Enter || event.key_code == KeyCode::Backspace)
            && is_ime_commiting.0
        {
            is_ime_commiting.0 = false;
            continue;
        }
        if event.key_code == KeyCode::Backspace && is_ime_composing.0 {
            is_ime_composing.0 = false;
            continue;
        }
        // Deliver only to the explicitly-focused webview. See the non-Windows
        // variant for why broadcasting on `None` leaks keys to a blurred webview,
        // and why a webview receives keys only after it is first clicked.
        let Some(webview) = target else {
            continue;
        };
        for key_event in create_cef_key_events(modifiers, &input, event) {
            proxy.send_key(&webview, key_event);
        }
    }
}

#[cfg(target_os = "windows")]
fn ime_event_win(
    mut er: MessageReader<Ime>,
    mut is_ime_commiting: ResMut<IsImeCommiting>,
    mut is_ime_composing: ResMut<IsImeComposing>,
    proxy: Res<BrowsersProxy>,
    focused: Res<FocusedWebview>,
    webviews: Query<Entity, With<WebviewSource>>,
) {
    let has_target = focused.0.filter(|e| webviews.get(*e).is_ok()).is_some();
    if !has_target {
        // See `ime_event`: finalize any composition on the now-blurred webview
        // and clear the shared IME flags when no webview is focused.
        if is_ime_composing.0 {
            proxy.ime_cancel_composition();
        }
        is_ime_composing.0 = false;
        is_ime_commiting.0 = false;
    }
    for event in er.read() {
        // See `ime_event`: drive CEF IME only when a webview is focused.
        if !has_target {
            continue;
        }
        match event {
            Ime::Preedit { value, cursor, .. } => {
                if value.is_empty() {
                    proxy.ime_cancel_composition();
                } else {
                    proxy.set_ime_composition(value, cursor.map(|(_, e)| e as u32));
                    is_ime_composing.0 = true;
                }
            }
            Ime::Commit { value, .. } => {
                proxy.set_ime_commit_text(value);
                is_ime_commiting.0 = true;
                is_ime_composing.0 = false;
            }
            Ime::Disabled { .. } => {
                proxy.ime_cancel_composition();
                is_ime_composing.0 = false;
            }
            _ => {}
        }
    }
}
