use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::components::PlayerAction;

pub const BUTTON_CONFIG_PATH: &str = "assets/button_config.ron";

// Placeholder config format: maps an action's name to the keys that trigger it.
// This indirection is what makes rebinding possible - systems only ever check
// `PlayerAction`, never a hardcoded `KeyCode`, so changing this file is enough
// to change which keys do what.
#[derive(Deserialize)]
struct ButtonConfigFile {
    bindings: HashMap<String, Vec<String>>,
}

fn action_from_str(name: &str) -> Option<PlayerAction> {
    match name {
        "Confirm" => Some(PlayerAction::Confirm),
        "Pause" => Some(PlayerAction::Pause),
        _ => None,
    }
}

// Every `KeyCode` variant, keyed by its identifier name. `KeyCode` represents the
// physical location of a key (e.g. `KeyY` is always where "Y" is on a QWERTY layout),
// not the character it produces, so bindings here are independent of keyboard
// layout/language - unlike `Key`, which reports the actual character typed.
macro_rules! keycode_map {
    ($($variant:ident),* $(,)?) => {
        [$((stringify!($variant), KeyCode::$variant)),*]
    };
}

static KEYCODES: LazyLock<HashMap<&'static str, KeyCode>> = LazyLock::new(|| {
    HashMap::from(keycode_map![
        Backquote, Backslash, BracketLeft, BracketRight, Comma, Digit0, Digit1, Digit2, Digit3,
        Digit4, Digit5, Digit6, Digit7, Digit8, Digit9, Equal, IntlBackslash, IntlRo, IntlYen,
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO,
        KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ, Minus, Period, Quote,
        Semicolon, Slash, AltLeft, AltRight, Backspace, CapsLock, ContextMenu, ControlLeft,
        ControlRight, Enter, SuperLeft, SuperRight, ShiftLeft, ShiftRight, Space, Tab, Convert,
        KanaMode, Lang1, Lang2, Lang3, Lang4, Lang5, NonConvert, Delete, End, Help, Home, Insert,
        PageDown, PageUp, ArrowDown, ArrowLeft, ArrowRight, ArrowUp, NumLock, Numpad0, Numpad1,
        Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9, NumpadAdd,
        NumpadBackspace, NumpadClear, NumpadClearEntry, NumpadComma, NumpadDecimal, NumpadDivide,
        NumpadEnter, NumpadEqual, NumpadHash, NumpadMemoryAdd, NumpadMemoryClear,
        NumpadMemoryRecall, NumpadMemoryStore, NumpadMemorySubtract, NumpadMultiply,
        NumpadParenLeft, NumpadParenRight, NumpadStar, NumpadSubtract, Escape, Fn, FnLock,
        PrintScreen, ScrollLock, Pause, BrowserBack, BrowserFavorites, BrowserForward,
        BrowserHome, BrowserRefresh, BrowserSearch, BrowserStop, Eject, LaunchApp1, LaunchApp2,
        LaunchMail, MediaPlayPause, MediaSelect, MediaStop, MediaTrackNext, MediaTrackPrevious,
        Power, Sleep, AudioVolumeDown, AudioVolumeMute, AudioVolumeUp, WakeUp, Meta, Hyper, Turbo,
        Abort, Resume, Suspend, Again, Copy, Cut, Find, Open, Paste, Props, Select, Undo,
        Hiragana, Katakana, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16,
        F17, F18, F19, F20, F21, F22, F23, F24, F25, F26, F27, F28, F29, F30, F31, F32, F33, F34,
        F35,
    ])
});

fn keycode_from_str(name: &str) -> Option<KeyCode> {
    KEYCODES.get(name).copied()
}

pub fn load_button_config(mut commands: Commands) {
    let contents =
        std::fs::read_to_string(BUTTON_CONFIG_PATH).expect("failed to read button config");
    let parsed: ButtonConfigFile = ron::from_str(&contents).expect("failed to parse button config");

    let mut input_map = InputMap::<PlayerAction>::default();
    for (action_name, key_names) in parsed.bindings {
        let Some(action) = action_from_str(&action_name) else {
            warn!("unknown action `{action_name}` in button config, skipping");
            continue;
        };
        for key_name in key_names {
            let Some(key) = keycode_from_str(&key_name) else {
                warn!("unknown key `{key_name}` in button config, skipping");
                continue;
            };
            input_map.insert(action, key);
        }
    }

    // `ActionState<PlayerAction>` is added automatically as a required component of `InputMap`.
    commands.spawn(input_map);
}
