//! Crate-local prelude: `use crate::prelude::*;` (or `use game::prelude::*;`
//! from `main.rs`) pulls in every component, plugin, and path constant in
//! the crate, plus `bevy`/`leafwing-input-manager`'s own preludes. Anything
//! used by only one or two files (avian3d, serde, `std::collections`, ...)
//! still stays explicit at the call site.

pub use crate::components::*;
pub use crate::systems::character_controller::*;
pub use crate::systems::core::*;
pub use crate::systems::terrain::*;
pub use crate::utils::constants::*;
pub use bevy::prelude::*;
pub use leafwing_input_manager::prelude::*;
