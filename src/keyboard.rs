use crate::common::CefWebviewUri;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_cef_core::prelude::{Browsers, create_cef_key_event, keyboard_modifiers};
use serde::{Deserialize, Serialize};

/// The plugin to handle keyboard inputs.
///
/// To use IME, you need to set [`Window::ime_enabled`](bevy::prelude::Window) to `true`.
pub(super) struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsImeCommiting>().add_systems(
            Update,
            (
                ime_event.run_if(on_message::<Ime>),
                send_key_event.run_if(on_message::<KeyboardInput>),
            )
                .chain(),
        );
    }
}

#[derive(Resource, Default, Serialize, Deserialize, Reflect)]
#[reflect(Default, Serialize, Deserialize)]
struct IsImeCommiting(bool);

fn send_key_event(
    mut er: MessageReader<KeyboardInput>,
    mut is_ime_commiting: ResMut<IsImeCommiting>,
    input: Res<ButtonInput<KeyCode>>,
    browsers: NonSend<Browsers>,
    webviews: Query<Entity, With<CefWebviewUri>>,
) {
    let modifiers = keyboard_modifiers(&input);
    for event in er.read() {
        if event.key_code == KeyCode::Enter && is_ime_commiting.0 {
            // If the IME is committing, we don't want to send the Enter key event.
            // This is to prevent sending the Enter key event when the IME is committing.
            is_ime_commiting.0 = false;
            continue;
        }
        let Some(key_event) = create_cef_key_event(modifiers, &input, event) else {
            continue;
        };
        for webview in webviews.iter() {
            browsers.send_key(&webview, key_event.clone());
        }
    }
}

fn ime_event(
    mut er: MessageReader<Ime>,
    mut is_ime_commiting: ResMut<IsImeCommiting>,
    browsers: NonSend<Browsers>,
) {
    for event in er.read() {
        match event {
            Ime::Preedit { value, cursor, .. } => {
                browsers.set_ime_composition(value, cursor.map(|(_, e)| e as u32))
            }
            Ime::Commit { value, .. } => {
                browsers.set_ime_commit_text(value);
                is_ime_commiting.0 = true;
            }
            Ime::Disabled { .. } => {
                browsers.ime_finish_composition(false);
            }
            _ => {}
        }
    }
}
