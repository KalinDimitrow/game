use bevy::ecs::error::BevyError;
use bevy::window::WindowResolution;
use serde::Deserialize;

pub const DISPLAY_CONFIG_PATH: &str = "data/config/display.ron";

// Duplicates `button_config`'s read/parse/wrap-errors shape - fine for two
// config files, but see the cleanup backlog's "unify config loading" entry
// once a third one shows up.
//
// Placeholder config format: just the window's starting size, in physical
// pixels (matches `WindowResolution::new`'s units). Read directly in
// `main()`, before the `App` exists - window creation happens as part of
// adding `DefaultPlugins`, so this has to be known before any plugin/system
// runs, unlike `button_config`, which can load as an ordinary Startup system.
#[derive(Deserialize)]
struct DisplayConfigFile {
    width: u32,
    height: u32,
}

fn parse_display_config(contents: &str) -> Result<(u32, u32), BevyError> {
    let parsed: DisplayConfigFile =
        ron::from_str(contents).map_err(|e| format!("failed to parse display config: {e}"))?;

    if parsed.width == 0 || parsed.height == 0 {
        return Err(format!(
            "display config width/height must be non-zero, got {}x{}",
            parsed.width, parsed.height
        )
        .into());
    }

    Ok((parsed.width, parsed.height))
}

pub fn load_display_config() -> Result<WindowResolution, BevyError> {
    let contents = std::fs::read_to_string(DISPLAY_CONFIG_PATH)
        .map_err(|e| format!("failed to read display config at `{DISPLAY_CONFIG_PATH}`: {e}"))?;
    let (width, height) = parse_display_config(&contents)
        .map_err(|e| format!("{e} (in `{DISPLAY_CONFIG_PATH}`)"))?;

    Ok(WindowResolution::new(width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_config() {
        let ron = "(width: 1920, height: 1080)";
        assert_eq!(parse_display_config(ron).unwrap(), (1920, 1080));
    }

    #[test]
    fn malformed_ron_is_an_error_not_a_panic() {
        assert!(parse_display_config("not valid ron at all {{{").is_err());
    }

    #[test]
    fn zero_size_is_rejected() {
        assert!(parse_display_config("(width: 0, height: 720)").is_err());
        assert!(parse_display_config("(width: 1280, height: 0)").is_err());
    }
}
