//! Every implemented plugin, in one import, for wiring up the `App` in
//! `main.rs`. Deliberately doesn't re-export `crate::prelude` - the two are
//! independent, not tiered, so neither can end up importing the other.

pub use crate::systems::character_controller::*;
pub use crate::systems::core::*;
pub use crate::systems::terrain::*;
