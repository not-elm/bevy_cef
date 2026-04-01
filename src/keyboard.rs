use crate::common::WebviewSource;
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
    webviews: Query<Entity, With<WebviewSource>>,
) {
    let modifiers = keyboard_modifiers(&input);
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
        let key_events = create_cef_key_events(modifiers, &input, event);
        for key_event in key_events {
            for webview in webviews.iter() {
                browsers.send_key(&webview, key_event.clone());
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn ime_event(
    mut er: MessageReader<Ime>,
    mut is_ime_commiting: ResMut<IsImeCommiting>,
    mut is_ime_composing: ResMut<IsImeComposing>,
    browsers: NonSend<Browsers>,
) {
    for event in er.read() {
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

// ---------------------------------------------------------------------------
// Windows: BrowsersProxy variants
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn send_key_event_win(
    mut er: MessageReader<KeyboardInput>,
    mut is_ime_commiting: ResMut<IsImeCommiting>,
    mut is_ime_composing: ResMut<IsImeComposing>,
    input: Res<ButtonInput<KeyCode>>,
    proxy: Res<BrowsersProxy>,
    webviews: Query<Entity, With<WebviewSource>>,
) {
    let modifiers = keyboard_modifiers(&input);
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
        let key_events = create_cef_key_events(modifiers, &input, event);
        for key_event in key_events {
            for webview in webviews.iter() {
                proxy.send_key(&webview, key_event.clone());
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn ime_event_win(
    mut er: MessageReader<Ime>,
    mut is_ime_commiting: ResMut<IsImeCommiting>,
    mut is_ime_composing: ResMut<IsImeComposing>,
    proxy: Res<BrowsersProxy>,
) {
    for event in er.read() {
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
