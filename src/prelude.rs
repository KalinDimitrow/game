//! Base crate prelude: `bevy`/`leafwing-input-manager`'s preludes, this
//! crate's components, and path constants. Imports nothing from `systems/`
//! or `systems_prelude`, so it's a leaf every `systems/*` file can import
//! directly without risking a cycle. For the list of implemented plugins,
//! see `systems_prelude` instead - the two are independent on purpose.

pub use crate::components::*;
pub use crate::utils::constants::*;
pub use bevy::prelude::*;
pub use leafwing_input_manager::prelude::*;
