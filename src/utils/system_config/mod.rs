//! Unifies what used to be `graphics_config`'s and `systems::core::button_config`'s
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
use std::path::Path;

pub mod button_config;
pub mod graphics_config;
pub use self::button_config::*;
pub use self::graphics_config::*;

/// Output of `system_config_reader` - purely a transient carrier for the
/// parsed configs on their way from "just read off disk" to "inserted as
/// their own resource" in `main`. Not a `Resource` itself: `ButtonConfig`
/// and `GraphicsConfig` are each inserted separately, so systems depend on
/// only the one config they actually use. No `Option` here: by the time
/// this exists, `system_config_reader` has already confirmed both were read
/// (or returned `Err` instead), so there's nothing left to re-check.
pub struct SystemConfig {
    pub button: ButtonConfig,
    pub graphics: GraphicsConfig,
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
    graphics: Option<GraphicsConfig>,
}

/// Shared by every `*_config_reader` - each reads/parses its own file and
/// fills in the one field of `parsed` it owns. The uniform signature is what
/// makes them storable as plain data in `CONFIG_READERS`, so calling one is
/// just `reader(path, parsed)`, not a `match` on some tag.
pub type ConfigReader = fn(&Path, &mut ParsedConfigs) -> Result<(), BevyError>;

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
        graphics: parsed.graphics.expect("CONFIG_READERS always includes a graphics reader"),
    })
}
