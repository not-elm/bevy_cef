//! ## Reference
//!
//! - [`cef_key_event_t`](https://cef-builds.spotifycdn.com/docs/106.1/structcef__key__event__t.html)
//! - [KeyboardCodes](https://chromium.googlesource.com/external/Webkit/+/safari-4-branch/WebCore/platform/KeyboardCodes.h)

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{ButtonInput, KeyCode};
use cef_dll_sys::{cef_event_flags_t, cef_key_event_t, cef_key_event_type_t};

pub fn keyboard_modifiers(input: &ButtonInput<KeyCode>) -> u32 {
    let mut flags = 0u32;

    if input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight) {
        flags |= cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0;
    }
    if input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight) {
        flags |= cef_event_flags_t::EVENTFLAG_ALT_DOWN.0;
    }
    if input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight) {
        flags |= cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0;
    }
    if input.pressed(KeyCode::SuperLeft) || input.pressed(KeyCode::SuperRight) {
        flags |= cef_event_flags_t::EVENTFLAG_COMMAND_DOWN.0;
    }
    if input.pressed(KeyCode::CapsLock) {
        flags |= cef_event_flags_t::EVENTFLAG_CAPS_LOCK_ON.0;
    }
    if input.pressed(KeyCode::NumLock) {
        flags |= cef_event_flags_t::EVENTFLAG_NUM_LOCK_ON.0;
    }

    flags
}

pub fn create_cef_key_event(
    modifiers: u32,
    _input: &ButtonInput<KeyCode>,
    key_event: &KeyboardInput,
) -> Option<cef::KeyEvent> {
    let key_type = match key_event.state {
        // ButtonState::Pressed if input.just_pressed(key_event.key_code) => {
        //     cef_key_event_type_t::KEYEVENT_RAWKEYDOWN
        // }
        ButtonState::Pressed => cef_key_event_type_t::KEYEVENT_CHAR,
        ButtonState::Released => cef_key_event_type_t::KEYEVENT_KEYUP,
    };
    let windows_key_code = keycode_to_windows_vk(key_event.key_code);

    let character = key_event
        .text
        .as_ref()
        .and_then(|text| text.chars().next())
        .unwrap_or('\0') as u16;

    Some(cef::KeyEvent::from(cef_key_event_t {
        size: core::mem::size_of::<cef_key_event_t>(),
        type_: key_type,
        modifiers,
        windows_key_code,
        native_key_code: to_native_key_code(&key_event.key_code) as _,
        character,
        unmodified_character: character,
        is_system_key: false as _,
        focus_on_editable_field: false as _,
    }))
}

// fn is_not_character_key_code(keycode: &KeyCode) -> bool {
//     match keycode {
//         // Function keys are not character keys
//         KeyCode::F1
//         | KeyCode::F2
//         | KeyCode::F3
//         | KeyCode::F4
//         | KeyCode::F5
//         | KeyCode::F6
//         | KeyCode::F7
//         | KeyCode::F8
//         | KeyCode::F9
//         | KeyCode::F10
//         | KeyCode::F11
//         | KeyCode::F12 => true,
//
//         // Navigation keys are not character keys
//         KeyCode::ArrowLeft
//         | KeyCode::ArrowUp
//         | KeyCode::ArrowRight
//         | KeyCode::ArrowDown
//         | KeyCode::Home
//         | KeyCode::End
//         | KeyCode::PageUp
//         | KeyCode::PageDown => true,
//
//         // Modifier keys are not character keys
//         KeyCode::ShiftLeft
//         | KeyCode::ShiftRight
//         | KeyCode::ControlLeft
//         | KeyCode::ControlRight
//         | KeyCode::AltLeft
//         | KeyCode::AltRight
//         | KeyCode::SuperLeft
//         | KeyCode::SuperRight => true,
//
//         // Lock keys are not character keys
//         KeyCode::CapsLock | KeyCode::NumLock | KeyCode::ScrollLock => true,
//
//         // Special control keys are not character keys
//         KeyCode::Escape
//         | KeyCode::Tab
//         | KeyCode::Enter
//         | KeyCode::Backspace
//         | KeyCode::Delete
//         | KeyCode::Insert => true,
//
//         // All other keys (letters, numbers, punctuation, space, numpad) are character keys
//         _ => false,
//     }
// }

fn keycode_to_windows_vk(keycode: KeyCode) -> i32 {
    match keycode {
        // Letters
        KeyCode::KeyA => 0x41,
        KeyCode::KeyB => 0x42,
        KeyCode::KeyC => 0x43,
        KeyCode::KeyD => 0x44,
        KeyCode::KeyE => 0x45,
        KeyCode::KeyF => 0x46,
        KeyCode::KeyG => 0x47,
        KeyCode::KeyH => 0x48,
        KeyCode::KeyI => 0x49,
        KeyCode::KeyJ => 0x4A,
        KeyCode::KeyK => 0x4B,
        KeyCode::KeyL => 0x4C,
        KeyCode::KeyM => 0x4D,
        KeyCode::KeyN => 0x4E,
        KeyCode::KeyO => 0x4F,
        KeyCode::KeyP => 0x50,
        KeyCode::KeyQ => 0x51,
        KeyCode::KeyR => 0x52,
        KeyCode::KeyS => 0x53,
        KeyCode::KeyT => 0x54,
        KeyCode::KeyU => 0x55,
        KeyCode::KeyV => 0x56,
        KeyCode::KeyW => 0x57,
        KeyCode::KeyX => 0x58,
        KeyCode::KeyY => 0x59,
        KeyCode::KeyZ => 0x5A,

        // Numbers
        KeyCode::Digit0 => 0x30,
        KeyCode::Digit1 => 0x31,
        KeyCode::Digit2 => 0x32,
        KeyCode::Digit3 => 0x33,
        KeyCode::Digit4 => 0x34,
        KeyCode::Digit5 => 0x35,
        KeyCode::Digit6 => 0x36,
        KeyCode::Digit7 => 0x37,
        KeyCode::Digit8 => 0x38,
        KeyCode::Digit9 => 0x39,

        // Function keys
        KeyCode::F1 => 0x70,
        KeyCode::F2 => 0x71,
        KeyCode::F3 => 0x72,
        KeyCode::F4 => 0x73,
        KeyCode::F5 => 0x74,
        KeyCode::F6 => 0x75,
        KeyCode::F7 => 0x76,
        KeyCode::F8 => 0x77,
        KeyCode::F9 => 0x78,
        KeyCode::F10 => 0x79,
        KeyCode::F11 => 0x7A,
        KeyCode::F12 => 0x7B,

        // Special keys
        KeyCode::Enter => 0x0D,
        KeyCode::Space => 0x20,
        KeyCode::Backspace => 0x08,
        KeyCode::Delete => 0x2E,
        KeyCode::Tab => 0x09,
        KeyCode::Escape => 0x1B,
        KeyCode::Insert => 0x2D,
        KeyCode::Home => 0x24,
        KeyCode::End => 0x23,
        KeyCode::PageUp => 0x21,
        KeyCode::PageDown => 0x22,

        // Arrow keys
        KeyCode::ArrowLeft => 0x25,
        KeyCode::ArrowUp => 0x26,
        KeyCode::ArrowRight => 0x27,
        KeyCode::ArrowDown => 0x28,

        // Modifier keys
        KeyCode::ShiftLeft | KeyCode::ShiftRight => 0x10,
        KeyCode::ControlLeft | KeyCode::ControlRight => 0x11,
        KeyCode::AltLeft | KeyCode::AltRight => 0x12,
        KeyCode::SuperLeft => 0x5B,  // Left Windows key
        KeyCode::SuperRight => 0x5C, // Right Windows key

        // Lock keys
        KeyCode::CapsLock => 0x14,
        KeyCode::NumLock => 0x90,
        KeyCode::ScrollLock => 0x91,

        // Punctuation
        KeyCode::Semicolon => 0xBA,
        KeyCode::Equal => 0xBB,
        KeyCode::Comma => 0xBC,
        KeyCode::Minus => 0xBD,
        KeyCode::Period => 0xBE,
        KeyCode::Slash => 0xBF,
        KeyCode::Backquote => 0xC0,
        KeyCode::BracketLeft => 0xDB,
        KeyCode::Backslash => 0xDC,
        KeyCode::BracketRight => 0xDD,
        KeyCode::Quote => 0xDE,

        // Numpad
        KeyCode::Numpad0 => 0x60,
        KeyCode::Numpad1 => 0x61,
        KeyCode::Numpad2 => 0x62,
        KeyCode::Numpad3 => 0x63,
        KeyCode::Numpad4 => 0x64,
        KeyCode::Numpad5 => 0x65,
        KeyCode::Numpad6 => 0x66,
        KeyCode::Numpad7 => 0x67,
        KeyCode::Numpad8 => 0x68,
        KeyCode::Numpad9 => 0x69,
        KeyCode::NumpadMultiply => 0x6A,
        KeyCode::NumpadAdd => 0x6B,
        KeyCode::NumpadSubtract => 0x6D,
        KeyCode::NumpadDecimal => 0x6E,
        KeyCode::NumpadDivide => 0x6F,

        // Default case for unhandled keys
        _ => 0,
    }
}

// fn is_special_key(keycode: &KeyCode) -> bool {
//     matches!(
//         keycode,
//         KeyCode::Enter
//             | KeyCode::Space
//             | KeyCode::Backspace
//             | KeyCode::Delete
//             | KeyCode::Tab
//             | KeyCode::Escape
//             | KeyCode::Insert
//             | KeyCode::Home
//             | KeyCode::End
//             | KeyCode::PageUp
//             | KeyCode::PageDown
//     )
// }

/// Native key codes for different platforms based on MDN documentation
/// [`Keyboard_event_key_values`](https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_key_values)
fn to_native_key_code(keycode: &KeyCode) -> u32 {
    match keycode {
        // Letters - Platform specific native codes
        KeyCode::KeyA => {
            if cfg!(target_os = "macos") {
                0x00
            } else {
                0x41
            } // Linux/default
        }
        KeyCode::KeyB => {
            if cfg!(target_os = "macos") {
                0x0B
            } else {
                0x42
            }
        }
        KeyCode::KeyC => {
            if cfg!(target_os = "macos") {
                0x08
            } else {
                0x43
            }
        }
        KeyCode::KeyD => {
            if cfg!(target_os = "macos") {
                0x02
            } else {
                0x44
            }
        }
        KeyCode::KeyE => {
            if cfg!(target_os = "macos") {
                0x0E
            } else {
                0x45
            }
        }
        KeyCode::KeyF => {
            if cfg!(target_os = "macos") {
                0x03
            } else {
                0x46
            }
        }
        KeyCode::KeyG => {
            if cfg!(target_os = "macos") {
                0x05
            } else {
                0x47
            }
        }
        KeyCode::KeyH => {
            if cfg!(target_os = "macos") {
                0x04
            } else {
                0x48
            }
        }
        KeyCode::KeyI => {
            if cfg!(target_os = "macos") {
                0x22
            } else {
                0x49
            }
        }
        KeyCode::KeyJ => {
            if cfg!(target_os = "macos") {
                0x26
            } else {
                0x4A
            }
        }
        KeyCode::KeyK => {
            if cfg!(target_os = "macos") {
                0x28
            } else {
                0x4B
            }
        }
        KeyCode::KeyL => {
            if cfg!(target_os = "macos") {
                0x25
            } else {
                0x4C
            }
        }
        KeyCode::KeyM => {
            if cfg!(target_os = "macos") {
                0x2E
            } else {
                0x4D
            }
        }
        KeyCode::KeyN => {
            if cfg!(target_os = "macos") {
                0x2D
            } else {
                0x4E
            }
        }
        KeyCode::KeyO => {
            if cfg!(target_os = "macos") {
                0x1F
            } else {
                0x4F
            }
        }
        KeyCode::KeyP => {
            if cfg!(target_os = "macos") {
                0x23
            } else {
                0x50
            }
        }
        KeyCode::KeyQ => {
            if cfg!(target_os = "macos") {
                0x0C
            } else {
                0x51
            }
        }
        KeyCode::KeyR => {
            if cfg!(target_os = "macos") {
                0x0F
            } else {
                0x52
            }
        }
        KeyCode::KeyS => {
            if cfg!(target_os = "macos") {
                0x01
            } else {
                0x53
            }
        }
        KeyCode::KeyT => {
            if cfg!(target_os = "macos") {
                0x11
            } else {
                0x54
            }
        }
        KeyCode::KeyU => {
            if cfg!(target_os = "macos") {
                0x20
            } else {
                0x55
            }
        }
        KeyCode::KeyV => {
            if cfg!(target_os = "macos") {
                0x09
            } else {
                0x56
            }
        }
        KeyCode::KeyW => {
            if cfg!(target_os = "macos") {
                0x0D
            } else {
                0x57
            }
        }
        KeyCode::KeyX => {
            if cfg!(target_os = "macos") {
                0x07
            } else {
                0x58
            }
        }
        KeyCode::KeyY => {
            if cfg!(target_os = "macos") {
                0x10
            } else {
                0x59
            }
        }
        KeyCode::KeyZ => {
            if cfg!(target_os = "macos") {
                0x06
            } else {
                0x5A
            }
        }

        // Numbers
        KeyCode::Digit0 => {
            if cfg!(target_os = "macos") {
                0x1D
            } else {
                0x30
            }
        }
        KeyCode::Digit1 => {
            if cfg!(target_os = "macos") {
                0x12
            } else {
                0x31
            }
        }
        KeyCode::Digit2 => {
            if cfg!(target_os = "macos") {
                0x13
            } else {
                0x32
            }
        }
        KeyCode::Digit3 => {
            if cfg!(target_os = "macos") {
                0x14
            } else {
                0x33
            }
        }
        KeyCode::Digit4 => {
            if cfg!(target_os = "macos") {
                0x15
            } else {
                0x34
            }
        }
        KeyCode::Digit5 => {
            if cfg!(target_os = "macos") {
                0x17
            } else {
                0x35
            }
        }
        KeyCode::Digit6 => {
            if cfg!(target_os = "macos") {
                0x16
            } else {
                0x36
            }
        }
        KeyCode::Digit7 => {
            if cfg!(target_os = "macos") {
                0x1A
            } else {
                0x37
            }
        }
        KeyCode::Digit8 => {
            if cfg!(target_os = "macos") {
                0x1C
            } else {
                0x38
            }
        }
        KeyCode::Digit9 => {
            if cfg!(target_os = "macos") {
                0x19
            } else {
                0x39
            }
        }

        // Function keys
        KeyCode::F1 => {
            if cfg!(target_os = "macos") {
                0x7A
            } else {
                0x70
            }
        }
        KeyCode::F2 => {
            if cfg!(target_os = "macos") {
                0x78
            } else {
                0x71
            }
        }
        KeyCode::F3 => {
            if cfg!(target_os = "macos") {
                0x63
            } else {
                0x72
            }
        }
        KeyCode::F4 => {
            if cfg!(target_os = "macos") {
                0x76
            } else {
                0x73
            }
        }
        KeyCode::F5 => {
            if cfg!(target_os = "macos") {
                0x60
            } else {
                0x74
            }
        }
        KeyCode::F6 => {
            if cfg!(target_os = "macos") {
                0x61
            } else {
                0x75
            }
        }
        KeyCode::F7 => {
            if cfg!(target_os = "macos") {
                0x62
            } else {
                0x76
            }
        }
        KeyCode::F8 => {
            if cfg!(target_os = "macos") {
                0x64
            } else {
                0x77
            }
        }
        KeyCode::F9 => {
            if cfg!(target_os = "macos") {
                0x65
            } else {
                0x78
            }
        }
        KeyCode::F10 => {
            if cfg!(target_os = "macos") {
                0x6D
            } else {
                0x79
            }
        }
        KeyCode::F11 => {
            if cfg!(target_os = "macos") {
                0x67
            } else {
                0x7A
            }
        }
        KeyCode::F12 => {
            if cfg!(target_os = "macos") {
                0x6F
            } else {
                0x7B
            }
        }

        // Special keys
        KeyCode::Enter => {
            if cfg!(target_os = "macos") {
                0x24
            } else {
                0x0D
            }
        }
        KeyCode::Space => {
            if cfg!(target_os = "macos") {
                0x31
            } else {
                0x20
            }
        }
        KeyCode::Backspace => {
            if cfg!(target_os = "macos") {
                0x33
            } else {
                0x08
            }
        }
        KeyCode::Delete => {
            if cfg!(target_os = "macos") {
                0x75
            } else {
                0x2E
            }
        }
        KeyCode::Tab => {
            if cfg!(target_os = "macos") {
                0x30
            } else {
                0x09
            }
        }
        KeyCode::Escape => {
            if cfg!(target_os = "macos") {
                0x35
            } else {
                0x1B
            }
        }
        KeyCode::Insert => {
            if cfg!(target_os = "macos") {
                0x72
            } else {
                0x2D
            }
        }
        KeyCode::Home => {
            if cfg!(target_os = "macos") {
                0x73
            } else {
                0x24
            }
        }
        KeyCode::End => {
            if cfg!(target_os = "macos") {
                0x77
            } else {
                0x23
            }
        }
        KeyCode::PageUp => {
            if cfg!(target_os = "macos") {
                0x74
            } else {
                0x21
            }
        }
        KeyCode::PageDown => {
            if cfg!(target_os = "macos") {
                0x79
            } else {
                0x22
            }
        }

        // Arrow keys
        KeyCode::ArrowLeft => {
            if cfg!(target_os = "macos") {
                0x7B
            } else {
                0x25
            }
        }
        KeyCode::ArrowUp => {
            if cfg!(target_os = "macos") {
                0x7E
            } else {
                0x26
            }
        }
        KeyCode::ArrowRight => {
            if cfg!(target_os = "macos") {
                0x7C
            } else {
                0x27
            }
        }
        KeyCode::ArrowDown => {
            if cfg!(target_os = "macos") {
                0x7D
            } else {
                0x28
            }
        }

        // Modifier keys
        KeyCode::ShiftLeft => {
            if cfg!(target_os = "macos") {
                0x38
            } else {
                0xA0
            }
        }
        KeyCode::ShiftRight => {
            if cfg!(target_os = "macos") {
                0x3C
            } else {
                0xA1
            }
        }
        KeyCode::ControlLeft => {
            if cfg!(target_os = "macos") {
                0x3B
            } else {
                0xA2
            }
        }
        KeyCode::ControlRight => {
            if cfg!(target_os = "macos") {
                0x3E
            } else {
                0xA3
            }
        }
        KeyCode::AltLeft => {
            if cfg!(target_os = "macos") {
                0x3A
            } else {
                0xA4
            }
        }
        KeyCode::AltRight => {
            if cfg!(target_os = "macos") {
                0x3D
            } else {
                0xA5
            }
        }
        KeyCode::SuperLeft => {
            if cfg!(target_os = "macos") {
                0x37
            } else {
                0x5B
            }
        }
        KeyCode::SuperRight => {
            if cfg!(target_os = "macos") {
                0x36
            } else {
                0x5C
            }
        }

        // Lock keys
        KeyCode::CapsLock => {
            if cfg!(target_os = "macos") {
                0x39
            } else {
                0x14
            }
        }
        KeyCode::NumLock => {
            if cfg!(target_os = "macos") {
                0x47
            } else {
                0x90
            }
        }
        KeyCode::ScrollLock => 0x91,

        // Punctuation
        KeyCode::Semicolon => {
            if cfg!(target_os = "macos") {
                0x29
            } else {
                0xBA
            }
        }
        KeyCode::Equal => {
            if cfg!(target_os = "macos") {
                0x18
            } else {
                0xBB
            }
        }
        KeyCode::Comma => {
            if cfg!(target_os = "macos") {
                0x2B
            } else {
                0xBC
            }
        }
        KeyCode::Minus => {
            if cfg!(target_os = "macos") {
                0x1B
            } else {
                0xBD
            }
        }
        KeyCode::Period => {
            if cfg!(target_os = "macos") {
                0x2F
            } else {
                0xBE
            }
        }
        KeyCode::Slash => {
            if cfg!(target_os = "macos") {
                0x2C
            } else {
                0xBF
            }
        }
        KeyCode::Backquote => {
            if cfg!(target_os = "macos") {
                0x32
            } else {
                0xC0
            }
        }
        KeyCode::BracketLeft => {
            if cfg!(target_os = "macos") {
                0x21
            } else {
                0xDB
            }
        }
        KeyCode::Backslash => {
            if cfg!(target_os = "macos") {
                0x2A
            } else {
                0xDC
            }
        }
        KeyCode::BracketRight => {
            if cfg!(target_os = "macos") {
                0x1E
            } else {
                0xDD
            }
        }
        KeyCode::Quote => {
            if cfg!(target_os = "macos") {
                0x27
            } else {
                0xDE
            }
        }

        // Numpad
        KeyCode::Numpad0 => {
            if cfg!(target_os = "macos") {
                0x52
            } else {
                0x60
            }
        }
        KeyCode::Numpad1 => {
            if cfg!(target_os = "macos") {
                0x53
            } else {
                0x61
            }
        }
        KeyCode::Numpad2 => {
            if cfg!(target_os = "macos") {
                0x54
            } else {
                0x62
            }
        }
        KeyCode::Numpad3 => {
            if cfg!(target_os = "macos") {
                0x55
            } else {
                0x63
            }
        }
        KeyCode::Numpad4 => {
            if cfg!(target_os = "macos") {
                0x56
            } else {
                0x64
            }
        }
        KeyCode::Numpad5 => {
            if cfg!(target_os = "macos") {
                0x57
            } else {
                0x65
            }
        }
        KeyCode::Numpad6 => {
            if cfg!(target_os = "macos") {
                0x58
            } else {
                0x66
            }
        }
        KeyCode::Numpad7 => {
            if cfg!(target_os = "macos") {
                0x59
            } else {
                0x67
            }
        }
        KeyCode::Numpad8 => {
            if cfg!(target_os = "macos") {
                0x5B
            } else {
                0x68
            }
        }
        KeyCode::Numpad9 => {
            if cfg!(target_os = "macos") {
                0x5C
            } else {
                0x69
            }
        }
        KeyCode::NumpadMultiply => {
            if cfg!(target_os = "macos") {
                0x43
            } else {
                0x6A
            }
        }
        KeyCode::NumpadAdd => {
            if cfg!(target_os = "macos") {
                0x45
            } else {
                0x6B
            }
        }
        KeyCode::NumpadSubtract => {
            if cfg!(target_os = "macos") {
                0x4E
            } else {
                0x6D
            }
        }
        KeyCode::NumpadDecimal => {
            if cfg!(target_os = "macos") {
                0x41
            } else {
                0x6E
            }
        }
        KeyCode::NumpadDivide => {
            if cfg!(target_os = "macos") {
                0x4B
            } else {
                0x6F
            }
        }

        // Default case for unhandled keys
        _ => 0,
    }
}
