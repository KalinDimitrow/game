//! Unifies what used to be `display_config`'s and `systems::core::button_config`'s
//! separate read/parse/wrap-errors logic: each `*_config_reader` function
//! reads/parses one file and fills in the one field of `ParsedConfigs` it
//! owns. All of them share the `ConfigReader` signature, which is what lets
//! `system_config_reader` walk `CONFIG_READERS` (a `(path, reader)` table)
//! and just call `reader(path, &mut parsed)` directly - no `match` on a key
//! needed to pick the right one. A missing file is a hard error - the app
//! can't run without its config - so by the time `system_config_reader`
//! turns `ParsedConfigs` into the `SystemConfig` it returns, every field is
//! known to be populated; callers never see an `Option`.

use bevy::ecs::error::BevyError;
use bevy::window::WindowResolution;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::prelude::*;

/// Output of `system_config_reader` - purely a transient carrier for the
/// parsed configs on their way from "just read off disk" to "inserted as
/// their own resource" in `main`. Not a `Resource` itself: `ButtonConfig`
/// and `DisplayConfig` are each inserted separately, so systems depend on
/// only the one config they actually use. No `Option` here: by the time
/// this exists, `system_config_reader` has already confirmed both were read
/// (or returned `Err` instead), so there's nothing left to re-check.
pub struct SystemConfig {
    pub button: ButtonConfig,
    pub display: DisplayConfig,
}

#[derive(Resource)]
pub struct ButtonConfig {
    pub bindings: Vec<(PlayerAction, KeyCode)>,
}

#[derive(Resource)]
pub struct DisplayConfig {
    pub resolution: WindowResolution,
}

/// Mutation target for `ConfigReader` while configs are still being read.
/// `Option` here means "not read yet," not "may legitimately be missing" -
/// `system_config_reader` only ever turns this into a `SystemConfig` after
/// every reader in `CONFIG_READERS` has already succeeded, so the `Option`
/// never survives past that point. Opaque outside this module: there's no
/// public way to build or inspect one other than running the readers.
#[derive(Default)]
pub struct ParsedConfigs {
    button: Option<ButtonConfig>,
    display: Option<DisplayConfig>,
}

// Placeholder config format: maps an action's name to the keys that trigger it.
// This indirection is what makes rebinding possible - systems only ever check
// `PlayerAction`, never a hardcoded `KeyCode`, so changing this file is enough
// to change which keys do what.
#[derive(Deserialize)]
struct ButtonConfigFile {
    bindings: HashMap<String, Vec<String>>,
}

// Placeholder config format: just the window's starting size, in physical
// pixels (matches `WindowResolution::new`'s units).
#[derive(Deserialize)]
struct DisplayConfigFile {
    width: u32,
    height: u32,
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

fn parse_display_config(contents: &str) -> Result<DisplayConfig, BevyError> {
    let parsed: DisplayConfigFile =
        ron::from_str(contents).map_err(|e| format!("failed to parse display config: {e}"))?;

    if parsed.width == 0 || parsed.height == 0 {
        return Err(format!(
            "display config width/height must be non-zero, got {}x{}",
            parsed.width, parsed.height
        )
        .into());
    }

    Ok(DisplayConfig { resolution: WindowResolution::new(parsed.width, parsed.height) })
}

/// Shared by every `*_config_reader` - each reads/parses its own file and
/// fills in the one field of `parsed` it owns. The uniform signature is what
/// makes them storable as plain data in `CONFIG_READERS`, so calling one is
/// just `reader(path, parsed)`, not a `match` on some tag.
pub type ConfigReader = fn(&Path, &mut ParsedConfigs) -> Result<(), BevyError>;

// `pub(crate)`, not `pub`: only `utils::constants` (building `CONFIG_READERS`)
// needs to name these directly. Everyone else goes through `system_config_reader`.
pub(crate) fn button_config_reader(path: &Path, parsed: &mut ParsedConfigs) -> Result<(), BevyError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read button config at `{}`: {e}", path.display()))?;
    let button_config = parse_button_config(&contents)
        .map_err(|e| format!("{e} (in `{}`)", path.display()))?;
    parsed.button = Some(button_config);
    Ok(())
}

pub(crate) fn display_config_reader(path: &Path, parsed: &mut ParsedConfigs) -> Result<(), BevyError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read display config at `{}`: {e}", path.display()))?;
    let display_config = parse_display_config(&contents)
        .map_err(|e| format!("{e} (in `{}`)", path.display()))?;
    parsed.display = Some(display_config);
    Ok(())
}

/// Runs every `(path, reader)` pair in `config_readers` (pass
/// `CONFIG_READERS`). A missing config file is a hard error - the app can't
/// run properly without it - not a skipped entry.
pub fn system_config_reader(config_readers: &[(&str, ConfigReader)]) -> Result<SystemConfig, BevyError> {
    let mut parsed = ParsedConfigs::default();

    for &(path, reader) in config_readers {
        if !Path::new(path).exists() {
            return Err(format!("missing config file `{path}`").into());
        }
        reader(Path::new(path), &mut parsed)?;
    }

    Ok(SystemConfig {
        button: parsed.button.expect("CONFIG_READERS always includes a button reader"),
        display: parsed.display.expect("CONFIG_READERS always includes a display reader"),
    })
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

    #[test]
    fn parses_valid_display_config() {
        let ron = "(width: 1920, height: 1080)";
        let display = parse_display_config(ron).unwrap();
        assert_eq!((display.resolution.width(), display.resolution.height()), (1920.0, 1080.0));
    }

    #[test]
    fn malformed_display_ron_is_an_error_not_a_panic() {
        assert!(parse_display_config("not valid ron at all {{{").is_err());
    }

    #[test]
    fn zero_size_is_rejected() {
        assert!(parse_display_config("(width: 0, height: 720)").is_err());
        assert!(parse_display_config("(width: 1280, height: 0)").is_err());
    }
}
