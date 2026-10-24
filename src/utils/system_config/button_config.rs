use bevy::ecs::error::BevyError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::ParsedConfigs;
use crate::prelude::*;

#[derive(Resource)]
pub struct ButtonConfig {
    pub bindings: Vec<(PlayerAction, KeyCode)>,
}

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

// A `const` slice rather than a `HashMap`: std's `HashMap` has no `const`
// constructor (it allocates), and this is only ever walked once per config
// key during startup parsing, so the O(n) linear scan in `keycode_from_str`
// below costs nothing in practice.
const KEYCODES: &[(&str, KeyCode)] = &keycode_map![
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
];

fn keycode_from_str(name: &str) -> Option<KeyCode> {
    KEYCODES.iter().find(|(key_name, _)| *key_name == name).map(|(_, key)| *key)
}

// Pure parsing/validation, kept separate from the readers below so it's
// testable without a filesystem. Unknown actions/keys are skipped with a
// warning (soft failure, config just has stale entries); malformed RON is a
// hard `Err` (the file isn't usable at all).
fn parse_button_config(contents: &str) -> Result<ButtonConfig, BevyError> {
    let parsed: ButtonConfigFile =
        ron::from_str(contents).map_err(|e| format!("failed to parse button config: {e}"))?;

    let mut bindings = Vec::new();
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
            bindings.push((action, key));
        }
    }
    Ok(ButtonConfig { bindings })
}

// `pub(crate)`, not `pub`: only `utils::constants` (building `CONFIG_READERS`)
// needs to name this directly. Everyone else goes through `system_config_reader`.
pub(crate) fn button_config_reader(path: &Path, parsed: &mut ParsedConfigs) -> Result<(), BevyError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read button config at `{}`: {e}", path.display()))?;
    let button_config = parse_button_config(&contents)
        .map_err(|e| format!("{e} (in `{}`)", path.display()))?;
    parsed.button = Some(button_config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_button_config() {
        let ron = r#"(
            bindings: {
                "Confirm": ["Enter"],
                "Pause": ["Escape"],
            }
        )"#;

        let mut bindings = parse_button_config(ron).expect("valid config should parse").bindings;
        bindings.sort_by_key(|(action, _)| format!("{action:?}"));

        assert_eq!(
            bindings,
            vec![
                (PlayerAction::Confirm, KeyCode::Enter),
                (PlayerAction::Pause, KeyCode::Escape),
            ]
        );
    }

    #[test]
    fn malformed_button_ron_is_an_error_not_a_panic() {
        assert!(parse_button_config("not valid ron at all {{{").is_err());
    }

    #[test]
    fn unknown_action_is_skipped_not_an_error() {
        let ron = r#"(
            bindings: {
                "NotARealAction": ["Enter"],
            }
        )"#;

        let bindings = parse_button_config(ron)
            .expect("unknown action should be skipped, not fail parsing")
            .bindings;
        assert!(bindings.is_empty());
    }

    #[test]
    fn unknown_key_is_skipped_not_an_error() {
        let ron = r#"(
            bindings: {
                "Confirm": ["NotARealKey"],
            }
        )"#;

        let bindings = parse_button_config(ron)
            .expect("unknown key should be skipped, not fail parsing")
            .bindings;
        assert!(bindings.is_empty());
    }
}
