use bevy::ecs::error::BevyError;
use bevy::window::WindowResolution;
use serde::Deserialize;
use std::path::Path;

use super::ParsedConfigs;
use crate::prelude::*;

#[derive(Resource)]
pub struct GraphicsConfig {
    pub resolution: WindowResolution,
}

// Placeholder config format: just the window's starting size, in physical
// pixels (matches `WindowResolution::new`'s units).
#[derive(Deserialize)]
struct GraphicsConfigFile {
    width: u32,
    height: u32,
}

fn parse_graphics_config(contents: &str) -> Result<GraphicsConfig, BevyError> {
    let parsed: GraphicsConfigFile =
        ron::from_str(contents).map_err(|e| format!("failed to parse graphics config: {e}"))?;

    if parsed.width == 0 || parsed.height == 0 {
        return Err(format!(
            "graphics config width/height must be non-zero, got {}x{}",
            parsed.width, parsed.height
        )
        .into());
    }

    Ok(GraphicsConfig { resolution: WindowResolution::new(parsed.width, parsed.height) })
}

// `pub(crate)`, not `pub`: only `utils::constants` (building `CONFIG_READERS`)
// needs to name this directly. Everyone else goes through `system_config_reader`.
pub(crate) fn graphics_config_reader(path: &Path, parsed: &mut ParsedConfigs) -> Result<(), BevyError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read graphics config at `{}`: {e}", path.display()))?;
    let graphics_config = parse_graphics_config(&contents)
        .map_err(|e| format!("{e} (in `{}`)", path.display()))?;
    parsed.graphics = Some(graphics_config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_graphics_config() {
        let ron = "(width: 1920, height: 1080)";
        let graphics = parse_graphics_config(ron).unwrap();
        assert_eq!((graphics.resolution.width(), graphics.resolution.height()), (1920.0, 1080.0));
    }

    #[test]
    fn malformed_graphics_ron_is_an_error_not_a_panic() {
        assert!(parse_graphics_config("not valid ron at all {{{").is_err());
    }

    #[test]
    fn zero_size_is_rejected() {
        assert!(parse_graphics_config("(width: 0, height: 720)").is_err());
        assert!(parse_graphics_config("(width: 1280, height: 0)").is_err());
    }
}
