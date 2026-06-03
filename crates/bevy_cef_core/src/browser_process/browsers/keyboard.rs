//! ## Reference
//!
//! - [`cef_key_event_t`](https://cef-builds.spotifycdn.com/docs/106.1/structcef__key__event__t.html)
//! - [KeyboardCodes](https://chromium.googlesource.com/external/Webkit/+/safari-4-branch/WebCore/platform/KeyboardCodes.h)

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{ButtonInput, KeyCode};
use cef_dll_sys::{cef_event_flags_t, cef_key_event_t, cef_key_event_type_t};

#[allow(clippy::unnecessary_cast)]
pub fn keyboard_modifiers(input: &ButtonInput<KeyCode>) -> u32 {
    let mut flags = 0u32;

    if input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight) {
        flags |= cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0 as u32;
    }
    if input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight) {
        flags |= cef_event_flags_t::EVENTFLAG_ALT_DOWN.0 as u32;
    }
    if input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight) {
        flags |= cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0 as u32;
    }
    if input.pressed(KeyCode::SuperLeft) || input.pressed(KeyCode::SuperRight) {
        flags |= cef_event_flags_t::EVENTFLAG_COMMAND_DOWN.0 as u32;
    }
    if input.pressed(KeyCode::CapsLock) {
        flags |= cef_event_flags_t::EVENTFLAG_CAPS_LOCK_ON.0 as u32;
    }
    if input.pressed(KeyCode::NumLock) {
        flags |= cef_event_flags_t::EVENTFLAG_NUM_LOCK_ON.0 as u32;
    }

    flags
}

/// Converts a Bevy `KeyboardInput` into one or more CEF key events.
///
/// A key press emits a `RAWKEYDOWN` — which drives the DOM `keydown` event — on
/// every platform; a character-producing key additionally emits a following
/// `CHAR` (which drives text input), mirroring the native WM_KEYDOWN → WM_CHAR
/// sequence. A release emits a single `KEYUP`. Emitting only `CHAR` (the prior
/// non-Windows behavior) delivered text input but never a DOM `keydown`, so
/// in-page `keydown` handlers and keyboard shortcuts never fired off Windows.
pub fn create_cef_key_events(
    modifiers: u32,
    _input: &ButtonInput<KeyCode>,
    key_event: &KeyboardInput,
) -> Vec<cef::KeyEvent> {
    let native_key_code = to_native_key_code(&key_event.key_code) as _;
    let vk_code = keycode_to_windows_vk(key_event.key_code);

    // Shared skeleton for every emitted event. `type_` and the character fields
    // are the only things that vary between KEYUP / RAWKEYDOWN / CHAR, so each
    // event below is built by overriding just those on top of `base`.
    let base = cef_key_event_t {
        size: core::mem::size_of::<cef_key_event_t>(),
        type_: cef_key_event_type_t::KEYEVENT_RAWKEYDOWN,
        modifiers,
        windows_key_code: vk_code,
        native_key_code,
        character: 0,
        unmodified_character: 0,
        is_system_key: false as _,
        focus_on_editable_field: false as _,
    };

    if key_event.state == ButtonState::Released {
        return vec![cef::KeyEvent::from(cef_key_event_t {
            type_: cef_key_event_type_t::KEYEVENT_KEYUP,
            ..base
        })];
    }

    let character = key_event
        .text
        .as_ref()
        .and_then(|text| text.chars().next())
        .unwrap_or('\0') as u16;

    // NOTE: macOS builds the native key event from native_key_code + character
    // and ignores windows_key_code; a RAWKEYDOWN whose character AND
    // unmodified_character are both 0 is reclassified as NSFlagsChanged (a
    // modifier-key change) and never dispatches a DOM `keydown`. So the macOS
    // key-down MUST carry the character. Windows derives the down event from
    // windows_key_code and keeps its character at 0 (the following CHAR carries
    // it, mirroring the native WM_KEYDOWN → WM_CHAR sequence).
    let key_down_character = if cfg!(target_os = "macos") {
        character
    } else {
        0
    };

    let raw_key_down = cef_key_event_t {
        character: key_down_character,
        unmodified_character: key_down_character,
        ..base
    };

    if is_not_character_key_code(&key_event.key_code) || character == 0 {
        return vec![cef::KeyEvent::from(raw_key_down)];
    }

    // NOTE: the CHAR event keeps the prior per-platform windows_key_code
    // (character on Windows, vk_code elsewhere) so existing text input is
    // byte-for-byte unchanged; only the preceding RAWKEYDOWN is new.
    let char_event = cef_key_event_t {
        type_: cef_key_event_type_t::KEYEVENT_CHAR,
        windows_key_code: if cfg!(target_os = "windows") {
            character as i32
        } else {
            vk_code
        },
        character,
        unmodified_character: character,
        ..base
    };
    vec![
        cef::KeyEvent::from(raw_key_down),
        cef::KeyEvent::from(char_event),
    ]
}

fn is_not_character_key_code(keycode: &KeyCode) -> bool {
    match keycode {
        // Function keys are not character keys
        KeyCode::F1
        | KeyCode::F2
        | KeyCode::F3
        | KeyCode::F4
        | KeyCode::F5
        | KeyCode::F6
        | KeyCode::F7
        | KeyCode::F8
        | KeyCode::F9
        | KeyCode::F10
        | KeyCode::F11
        | KeyCode::F12 => true,

        // Navigation keys are not character keys
        KeyCode::ArrowLeft
        | KeyCode::ArrowUp
        | KeyCode::ArrowRight
        | KeyCode::ArrowDown
        | KeyCode::Home
        | KeyCode::End
        | KeyCode::PageUp
        | KeyCode::PageDown => true,

        // Modifier keys are not character keys
        KeyCode::ShiftLeft
        | KeyCode::ShiftRight
        | KeyCode::ControlLeft
        | KeyCode::ControlRight
        | KeyCode::AltLeft
        | KeyCode::AltRight
        | KeyCode::SuperLeft
        | KeyCode::SuperRight => true,

        // Lock keys are not character keys
        KeyCode::CapsLock | KeyCode::NumLock | KeyCode::ScrollLock => true,

        // Special control keys are not character keys
        KeyCode::Escape
        | KeyCode::Tab
        | KeyCode::Enter
        | KeyCode::Backspace
        | KeyCode::Delete
        | KeyCode::Insert => true,

        // All other keys (letters, numbers, punctuation, space, numpad) are character keys
        _ => false,
    }
}

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

/// Returns a platform-specific native key code.
///
/// - **macOS**: Returns the Carbon virtual key code (used directly by CEF).
/// - **Linux**: Returns the XKB keycode (evdev scancode + 8).
/// - **Windows**: Returns the Chromium-format scan code. Regular keys use the raw scan code
///   (e.g., 0x1E for KeyA). Extended keys use a 0xE0 prefix (e.g., 0xE053 for Delete).
///   CEF's `NativeKeycodeToDomCode()` uses this to derive `KeyboardEvent.code`.
fn to_native_key_code(keycode: &KeyCode) -> u32 {
    if cfg!(target_os = "macos") {
        to_macos_key_code(keycode)
    } else if cfg!(target_os = "linux") {
        to_linux_native_key_code(keycode)
    } else {
        to_windows_native_key_code(keycode)
    }
}

/// Returns the macOS Carbon virtual key code for the given key.
fn to_macos_key_code(keycode: &KeyCode) -> u32 {
    match keycode {
        // Letters
        KeyCode::KeyA => 0x00,
        KeyCode::KeyB => 0x0B,
        KeyCode::KeyC => 0x08,
        KeyCode::KeyD => 0x02,
        KeyCode::KeyE => 0x0E,
        KeyCode::KeyF => 0x03,
        KeyCode::KeyG => 0x05,
        KeyCode::KeyH => 0x04,
        KeyCode::KeyI => 0x22,
        KeyCode::KeyJ => 0x26,
        KeyCode::KeyK => 0x28,
        KeyCode::KeyL => 0x25,
        KeyCode::KeyM => 0x2E,
        KeyCode::KeyN => 0x2D,
        KeyCode::KeyO => 0x1F,
        KeyCode::KeyP => 0x23,
        KeyCode::KeyQ => 0x0C,
        KeyCode::KeyR => 0x0F,
        KeyCode::KeyS => 0x01,
        KeyCode::KeyT => 0x11,
        KeyCode::KeyU => 0x20,
        KeyCode::KeyV => 0x09,
        KeyCode::KeyW => 0x0D,
        KeyCode::KeyX => 0x07,
        KeyCode::KeyY => 0x10,
        KeyCode::KeyZ => 0x06,
        // Digits
        KeyCode::Digit0 => 0x1D,
        KeyCode::Digit1 => 0x12,
        KeyCode::Digit2 => 0x13,
        KeyCode::Digit3 => 0x14,
        KeyCode::Digit4 => 0x15,
        KeyCode::Digit5 => 0x17,
        KeyCode::Digit6 => 0x16,
        KeyCode::Digit7 => 0x1A,
        KeyCode::Digit8 => 0x1C,
        KeyCode::Digit9 => 0x19,
        // Function keys
        KeyCode::F1 => 0x7A,
        KeyCode::F2 => 0x78,
        KeyCode::F3 => 0x63,
        KeyCode::F4 => 0x76,
        KeyCode::F5 => 0x60,
        KeyCode::F6 => 0x61,
        KeyCode::F7 => 0x62,
        KeyCode::F8 => 0x64,
        KeyCode::F9 => 0x65,
        KeyCode::F10 => 0x6D,
        KeyCode::F11 => 0x67,
        KeyCode::F12 => 0x6F,
        // Special keys
        KeyCode::Enter => 0x24,
        KeyCode::Space => 0x31,
        KeyCode::Backspace => 0x33,
        KeyCode::Delete => 0x75,
        KeyCode::Tab => 0x30,
        KeyCode::Escape => 0x35,
        KeyCode::Insert => 0x72,
        KeyCode::Home => 0x73,
        KeyCode::End => 0x77,
        KeyCode::PageUp => 0x74,
        KeyCode::PageDown => 0x79,
        // Arrow keys
        KeyCode::ArrowLeft => 0x7B,
        KeyCode::ArrowUp => 0x7E,
        KeyCode::ArrowRight => 0x7C,
        KeyCode::ArrowDown => 0x7D,
        // Modifier keys
        KeyCode::ShiftLeft => 0x38,
        KeyCode::ShiftRight => 0x3C,
        KeyCode::ControlLeft => 0x3B,
        KeyCode::ControlRight => 0x3E,
        KeyCode::AltLeft => 0x3A,
        KeyCode::AltRight => 0x3D,
        KeyCode::SuperLeft => 0x37,
        KeyCode::SuperRight => 0x36,
        // Lock keys
        KeyCode::CapsLock => 0x39,
        KeyCode::NumLock => 0x47,
        KeyCode::ScrollLock => 0x6B,
        // Punctuation
        KeyCode::Semicolon => 0x29,
        KeyCode::Equal => 0x18,
        KeyCode::Comma => 0x2B,
        KeyCode::Minus => 0x1B,
        KeyCode::Period => 0x2F,
        KeyCode::Slash => 0x2C,
        KeyCode::Backquote => 0x32,
        KeyCode::BracketLeft => 0x21,
        KeyCode::Backslash => 0x2A,
        KeyCode::BracketRight => 0x1E,
        KeyCode::Quote => 0x27,
        // Numpad
        KeyCode::Numpad0 => 0x52,
        KeyCode::Numpad1 => 0x53,
        KeyCode::Numpad2 => 0x54,
        KeyCode::Numpad3 => 0x55,
        KeyCode::Numpad4 => 0x56,
        KeyCode::Numpad5 => 0x57,
        KeyCode::Numpad6 => 0x58,
        KeyCode::Numpad7 => 0x59,
        KeyCode::Numpad8 => 0x5B,
        KeyCode::Numpad9 => 0x5C,
        KeyCode::NumpadMultiply => 0x43,
        KeyCode::NumpadAdd => 0x45,
        KeyCode::NumpadSubtract => 0x4E,
        KeyCode::NumpadDecimal => 0x41,
        KeyCode::NumpadDivide => 0x4B,
        _ => 0,
    }
}

/// Evdev scancodes from `<linux/input-event-codes.h>`. Values are stable kernel ABI.
#[cfg(target_os = "linux")]
mod evdev {
    pub const KEY_ESC: u32 = 1;
    pub const KEY_1: u32 = 2;
    pub const KEY_2: u32 = 3;
    pub const KEY_3: u32 = 4;
    pub const KEY_4: u32 = 5;
    pub const KEY_5: u32 = 6;
    pub const KEY_6: u32 = 7;
    pub const KEY_7: u32 = 8;
    pub const KEY_8: u32 = 9;
    pub const KEY_9: u32 = 10;
    pub const KEY_0: u32 = 11;
    pub const KEY_MINUS: u32 = 12;
    pub const KEY_EQUAL: u32 = 13;
    pub const KEY_BACKSPACE: u32 = 14;
    pub const KEY_TAB: u32 = 15;
    pub const KEY_Q: u32 = 16;
    pub const KEY_W: u32 = 17;
    pub const KEY_E: u32 = 18;
    pub const KEY_R: u32 = 19;
    pub const KEY_T: u32 = 20;
    pub const KEY_Y: u32 = 21;
    pub const KEY_U: u32 = 22;
    pub const KEY_I: u32 = 23;
    pub const KEY_O: u32 = 24;
    pub const KEY_P: u32 = 25;
    pub const KEY_LEFTBRACE: u32 = 26;
    pub const KEY_RIGHTBRACE: u32 = 27;
    pub const KEY_ENTER: u32 = 28;
    pub const KEY_LEFTCTRL: u32 = 29;
    pub const KEY_A: u32 = 30;
    pub const KEY_S: u32 = 31;
    pub const KEY_D: u32 = 32;
    pub const KEY_F: u32 = 33;
    pub const KEY_G: u32 = 34;
    pub const KEY_H: u32 = 35;
    pub const KEY_J: u32 = 36;
    pub const KEY_K: u32 = 37;
    pub const KEY_L: u32 = 38;
    pub const KEY_SEMICOLON: u32 = 39;
    pub const KEY_APOSTROPHE: u32 = 40;
    pub const KEY_GRAVE: u32 = 41;
    pub const KEY_LEFTSHIFT: u32 = 42;
    pub const KEY_BACKSLASH: u32 = 43;
    pub const KEY_Z: u32 = 44;
    pub const KEY_X: u32 = 45;
    pub const KEY_C: u32 = 46;
    pub const KEY_V: u32 = 47;
    pub const KEY_B: u32 = 48;
    pub const KEY_N: u32 = 49;
    pub const KEY_M: u32 = 50;
    pub const KEY_COMMA: u32 = 51;
    pub const KEY_DOT: u32 = 52;
    pub const KEY_SLASH: u32 = 53;
    pub const KEY_RIGHTSHIFT: u32 = 54;
    pub const KEY_KPASTERISK: u32 = 55;
    pub const KEY_LEFTALT: u32 = 56;
    pub const KEY_SPACE: u32 = 57;
    pub const KEY_CAPSLOCK: u32 = 58;
    pub const KEY_F1: u32 = 59;
    pub const KEY_F2: u32 = 60;
    pub const KEY_F3: u32 = 61;
    pub const KEY_F4: u32 = 62;
    pub const KEY_F5: u32 = 63;
    pub const KEY_F6: u32 = 64;
    pub const KEY_F7: u32 = 65;
    pub const KEY_F8: u32 = 66;
    pub const KEY_F9: u32 = 67;
    pub const KEY_F10: u32 = 68;
    pub const KEY_NUMLOCK: u32 = 69;
    pub const KEY_SCROLLLOCK: u32 = 70;
    pub const KEY_KP7: u32 = 71;
    pub const KEY_KP8: u32 = 72;
    pub const KEY_KP9: u32 = 73;
    pub const KEY_KPMINUS: u32 = 74;
    pub const KEY_KP4: u32 = 75;
    pub const KEY_KP5: u32 = 76;
    pub const KEY_KP6: u32 = 77;
    pub const KEY_KPPLUS: u32 = 78;
    pub const KEY_KP1: u32 = 79;
    pub const KEY_KP2: u32 = 80;
    pub const KEY_KP3: u32 = 81;
    pub const KEY_KP0: u32 = 82;
    pub const KEY_KPDOT: u32 = 83;
    pub const KEY_F11: u32 = 87;
    pub const KEY_F12: u32 = 88;
    pub const KEY_KPENTER: u32 = 96;
    pub const KEY_RIGHTCTRL: u32 = 97;
    pub const KEY_KPSLASH: u32 = 98;
    pub const KEY_RIGHTALT: u32 = 100;
    pub const KEY_HOME: u32 = 102;
    pub const KEY_UP: u32 = 103;
    pub const KEY_PAGEUP: u32 = 104;
    pub const KEY_LEFT: u32 = 105;
    pub const KEY_RIGHT: u32 = 106;
    pub const KEY_END: u32 = 107;
    pub const KEY_DOWN: u32 = 108;
    pub const KEY_PAGEDOWN: u32 = 109;
    pub const KEY_INSERT: u32 = 110;
    pub const KEY_DELETE: u32 = 111;
    pub const KEY_LEFTMETA: u32 = 125;
    pub const KEY_RIGHTMETA: u32 = 126;
}

/// Returns the XKB keycode for the given key.
///
/// XKB keycodes equal the Linux evdev scancode plus 8 (X11 reserves keycodes 0-7,
/// and the Wayland keyboard protocol uses the same convention via libxkbcommon).
#[cfg(target_os = "linux")]
fn to_linux_native_key_code(keycode: &KeyCode) -> u32 {
    use evdev as ev;
    const XKB_OFFSET: u32 = 8;
    let evdev: u32 = match keycode {
        // Letters
        KeyCode::KeyA => ev::KEY_A,
        KeyCode::KeyB => ev::KEY_B,
        KeyCode::KeyC => ev::KEY_C,
        KeyCode::KeyD => ev::KEY_D,
        KeyCode::KeyE => ev::KEY_E,
        KeyCode::KeyF => ev::KEY_F,
        KeyCode::KeyG => ev::KEY_G,
        KeyCode::KeyH => ev::KEY_H,
        KeyCode::KeyI => ev::KEY_I,
        KeyCode::KeyJ => ev::KEY_J,
        KeyCode::KeyK => ev::KEY_K,
        KeyCode::KeyL => ev::KEY_L,
        KeyCode::KeyM => ev::KEY_M,
        KeyCode::KeyN => ev::KEY_N,
        KeyCode::KeyO => ev::KEY_O,
        KeyCode::KeyP => ev::KEY_P,
        KeyCode::KeyQ => ev::KEY_Q,
        KeyCode::KeyR => ev::KEY_R,
        KeyCode::KeyS => ev::KEY_S,
        KeyCode::KeyT => ev::KEY_T,
        KeyCode::KeyU => ev::KEY_U,
        KeyCode::KeyV => ev::KEY_V,
        KeyCode::KeyW => ev::KEY_W,
        KeyCode::KeyX => ev::KEY_X,
        KeyCode::KeyY => ev::KEY_Y,
        KeyCode::KeyZ => ev::KEY_Z,
        // Digits
        KeyCode::Digit0 => ev::KEY_0,
        KeyCode::Digit1 => ev::KEY_1,
        KeyCode::Digit2 => ev::KEY_2,
        KeyCode::Digit3 => ev::KEY_3,
        KeyCode::Digit4 => ev::KEY_4,
        KeyCode::Digit5 => ev::KEY_5,
        KeyCode::Digit6 => ev::KEY_6,
        KeyCode::Digit7 => ev::KEY_7,
        KeyCode::Digit8 => ev::KEY_8,
        KeyCode::Digit9 => ev::KEY_9,
        // Function keys
        KeyCode::F1 => ev::KEY_F1,
        KeyCode::F2 => ev::KEY_F2,
        KeyCode::F3 => ev::KEY_F3,
        KeyCode::F4 => ev::KEY_F4,
        KeyCode::F5 => ev::KEY_F5,
        KeyCode::F6 => ev::KEY_F6,
        KeyCode::F7 => ev::KEY_F7,
        KeyCode::F8 => ev::KEY_F8,
        KeyCode::F9 => ev::KEY_F9,
        KeyCode::F10 => ev::KEY_F10,
        KeyCode::F11 => ev::KEY_F11,
        KeyCode::F12 => ev::KEY_F12,
        // Special keys
        KeyCode::Enter => ev::KEY_ENTER,
        KeyCode::Space => ev::KEY_SPACE,
        KeyCode::Backspace => ev::KEY_BACKSPACE,
        KeyCode::Delete => ev::KEY_DELETE,
        KeyCode::Tab => ev::KEY_TAB,
        KeyCode::Escape => ev::KEY_ESC,
        KeyCode::Insert => ev::KEY_INSERT,
        KeyCode::Home => ev::KEY_HOME,
        KeyCode::End => ev::KEY_END,
        KeyCode::PageUp => ev::KEY_PAGEUP,
        KeyCode::PageDown => ev::KEY_PAGEDOWN,
        // Arrow keys
        KeyCode::ArrowLeft => ev::KEY_LEFT,
        KeyCode::ArrowUp => ev::KEY_UP,
        KeyCode::ArrowRight => ev::KEY_RIGHT,
        KeyCode::ArrowDown => ev::KEY_DOWN,
        // Modifier keys
        KeyCode::ShiftLeft => ev::KEY_LEFTSHIFT,
        KeyCode::ShiftRight => ev::KEY_RIGHTSHIFT,
        KeyCode::ControlLeft => ev::KEY_LEFTCTRL,
        KeyCode::ControlRight => ev::KEY_RIGHTCTRL,
        KeyCode::AltLeft => ev::KEY_LEFTALT,
        KeyCode::AltRight => ev::KEY_RIGHTALT,
        KeyCode::SuperLeft => ev::KEY_LEFTMETA,
        KeyCode::SuperRight => ev::KEY_RIGHTMETA,
        // Lock keys
        KeyCode::CapsLock => ev::KEY_CAPSLOCK,
        KeyCode::NumLock => ev::KEY_NUMLOCK,
        KeyCode::ScrollLock => ev::KEY_SCROLLLOCK,
        // Punctuation
        KeyCode::Semicolon => ev::KEY_SEMICOLON,
        KeyCode::Equal => ev::KEY_EQUAL,
        KeyCode::Comma => ev::KEY_COMMA,
        KeyCode::Minus => ev::KEY_MINUS,
        KeyCode::Period => ev::KEY_DOT,
        KeyCode::Slash => ev::KEY_SLASH,
        KeyCode::Backquote => ev::KEY_GRAVE,
        KeyCode::BracketLeft => ev::KEY_LEFTBRACE,
        KeyCode::Backslash => ev::KEY_BACKSLASH,
        KeyCode::BracketRight => ev::KEY_RIGHTBRACE,
        KeyCode::Quote => ev::KEY_APOSTROPHE,
        // Numpad
        KeyCode::Numpad0 => ev::KEY_KP0,
        KeyCode::Numpad1 => ev::KEY_KP1,
        KeyCode::Numpad2 => ev::KEY_KP2,
        KeyCode::Numpad3 => ev::KEY_KP3,
        KeyCode::Numpad4 => ev::KEY_KP4,
        KeyCode::Numpad5 => ev::KEY_KP5,
        KeyCode::Numpad6 => ev::KEY_KP6,
        KeyCode::Numpad7 => ev::KEY_KP7,
        KeyCode::Numpad8 => ev::KEY_KP8,
        KeyCode::Numpad9 => ev::KEY_KP9,
        KeyCode::NumpadMultiply => ev::KEY_KPASTERISK,
        KeyCode::NumpadAdd => ev::KEY_KPPLUS,
        KeyCode::NumpadSubtract => ev::KEY_KPMINUS,
        KeyCode::NumpadDecimal => ev::KEY_KPDOT,
        KeyCode::NumpadDivide => ev::KEY_KPSLASH,
        KeyCode::NumpadEnter => ev::KEY_KPENTER,
        _ => return 0,
    };
    evdev + XKB_OFFSET
}

#[cfg(not(target_os = "linux"))]
fn to_linux_native_key_code(_keycode: &KeyCode) -> u32 {
    0
}

/// Returns the Chromium-format Windows scan code for the given key.
///
/// Regular keys return their raw scan code (e.g., 0x1E for KeyA).
/// Extended keys return the scan code with a 0xE0 prefix (e.g., 0xE053 for Delete).
///
/// These values match Chromium's `dom_code_data.inc` lookup table, which CEF's
/// `NativeKeycodeToDomCode()` uses to derive `KeyboardEvent.code`.
fn to_windows_native_key_code(keycode: &KeyCode) -> u32 {
    let (scan_code, extended) = match keycode {
        // Letters (row by row on US QWERTY)
        KeyCode::KeyA => (0x1E, false),
        KeyCode::KeyB => (0x30, false),
        KeyCode::KeyC => (0x2E, false),
        KeyCode::KeyD => (0x20, false),
        KeyCode::KeyE => (0x12, false),
        KeyCode::KeyF => (0x21, false),
        KeyCode::KeyG => (0x22, false),
        KeyCode::KeyH => (0x23, false),
        KeyCode::KeyI => (0x17, false),
        KeyCode::KeyJ => (0x24, false),
        KeyCode::KeyK => (0x25, false),
        KeyCode::KeyL => (0x26, false),
        KeyCode::KeyM => (0x32, false),
        KeyCode::KeyN => (0x31, false),
        KeyCode::KeyO => (0x18, false),
        KeyCode::KeyP => (0x19, false),
        KeyCode::KeyQ => (0x10, false),
        KeyCode::KeyR => (0x13, false),
        KeyCode::KeyS => (0x1F, false),
        KeyCode::KeyT => (0x14, false),
        KeyCode::KeyU => (0x16, false),
        KeyCode::KeyV => (0x2F, false),
        KeyCode::KeyW => (0x11, false),
        KeyCode::KeyX => (0x2D, false),
        KeyCode::KeyY => (0x15, false),
        KeyCode::KeyZ => (0x2C, false),
        // Digits
        KeyCode::Digit1 => (0x02, false),
        KeyCode::Digit2 => (0x03, false),
        KeyCode::Digit3 => (0x04, false),
        KeyCode::Digit4 => (0x05, false),
        KeyCode::Digit5 => (0x06, false),
        KeyCode::Digit6 => (0x07, false),
        KeyCode::Digit7 => (0x08, false),
        KeyCode::Digit8 => (0x09, false),
        KeyCode::Digit9 => (0x0A, false),
        KeyCode::Digit0 => (0x0B, false),
        // Function keys
        KeyCode::F1 => (0x3B, false),
        KeyCode::F2 => (0x3C, false),
        KeyCode::F3 => (0x3D, false),
        KeyCode::F4 => (0x3E, false),
        KeyCode::F5 => (0x3F, false),
        KeyCode::F6 => (0x40, false),
        KeyCode::F7 => (0x41, false),
        KeyCode::F8 => (0x42, false),
        KeyCode::F9 => (0x43, false),
        KeyCode::F10 => (0x44, false),
        KeyCode::F11 => (0x57, false),
        KeyCode::F12 => (0x58, false),
        // Special keys
        KeyCode::Escape => (0x01, false),
        KeyCode::Tab => (0x0F, false),
        KeyCode::CapsLock => (0x3A, false),
        KeyCode::Space => (0x39, false),
        KeyCode::Backspace => (0x0E, false),
        KeyCode::Enter => (0x1C, false),
        KeyCode::Insert => (0x52, true),
        KeyCode::Delete => (0x53, true),
        KeyCode::Home => (0x47, true),
        KeyCode::End => (0x4F, true),
        KeyCode::PageUp => (0x49, true),
        KeyCode::PageDown => (0x51, true),
        // Arrow keys (extended)
        KeyCode::ArrowLeft => (0x4B, true),
        KeyCode::ArrowUp => (0x48, true),
        KeyCode::ArrowRight => (0x4D, true),
        KeyCode::ArrowDown => (0x50, true),
        // Modifier keys
        KeyCode::ShiftLeft => (0x2A, false),
        KeyCode::ShiftRight => (0x36, false),
        KeyCode::ControlLeft => (0x1D, false),
        KeyCode::ControlRight => (0x1D, true),
        KeyCode::AltLeft => (0x38, false),
        KeyCode::AltRight => (0x38, true),
        KeyCode::SuperLeft => (0x5B, true),
        KeyCode::SuperRight => (0x5C, true),
        // Lock keys
        KeyCode::NumLock => (0x45, true),
        KeyCode::ScrollLock => (0x46, false),
        // Punctuation
        KeyCode::Minus => (0x0C, false),
        KeyCode::Equal => (0x0D, false),
        KeyCode::BracketLeft => (0x1A, false),
        KeyCode::BracketRight => (0x1B, false),
        KeyCode::Backslash => (0x2B, false),
        KeyCode::Semicolon => (0x27, false),
        KeyCode::Quote => (0x28, false),
        KeyCode::Backquote => (0x29, false),
        KeyCode::Comma => (0x33, false),
        KeyCode::Period => (0x34, false),
        KeyCode::Slash => (0x35, false),
        // Numpad
        KeyCode::Numpad0 => (0x52, false),
        KeyCode::Numpad1 => (0x4F, false),
        KeyCode::Numpad2 => (0x50, false),
        KeyCode::Numpad3 => (0x51, false),
        KeyCode::Numpad4 => (0x4B, false),
        KeyCode::Numpad5 => (0x4C, false),
        KeyCode::Numpad6 => (0x4D, false),
        KeyCode::Numpad7 => (0x47, false),
        KeyCode::Numpad8 => (0x48, false),
        KeyCode::Numpad9 => (0x49, false),
        KeyCode::NumpadMultiply => (0x37, false),
        KeyCode::NumpadAdd => (0x4E, false),
        KeyCode::NumpadSubtract => (0x4A, false),
        KeyCode::NumpadDecimal => (0x53, false),
        KeyCode::NumpadDivide => (0x35, true),
        KeyCode::NumpadEnter => (0x1C, true),
        _ => return 0,
    };
    let extended_prefix = if extended { 0xe000u32 } else { 0 };
    scan_code | extended_prefix
}
