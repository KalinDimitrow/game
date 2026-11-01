//! Bevy `Resource`s and the code that loads, initializes, holds and releases
//! them. One file per resource, or per group of resources sharing a single
//! lifecycle; this `mod.rs` carries only what applies to the module as a
//! whole.
//!
//! Each file owns a resource's *whole* lifecycle, not just its type - the
//! `Startup` system that kicks off the load, the system that turns loaded
//! assets into a usable resource, and the type they produce all sit together,
//! because changing one almost always means changing the others. What stays
//! out is scheduling: the domain plugins (`CorePlugin`,
//! `CharacterControllerPlugin`) still decide *when* these run, so a resource's
//! load order remains visible next to the rest of its owner's wiring. This
//! module defines; the plugins wire.
//!
//! Three things that look like they belong here and don't:
//!
//! - `systems::core::apply_button_config` reads [`ButtonConfig`] but writes
//!   `InputMap` components onto entities. Getting a config loaded and held is
//!   this module's job; acting on it is a system's.
//! - `terrain::spawn_test_world` tracks the world handle in [`PendingAssets`],
//!   but what it's really doing is spawning the `DynamicWorldRoot` entity that
//!   owns it. The load is incidental to the spawn, and no resource comes out
//!   of it.
//! - `utils::system_config::GraphicsConfig` is a `Resource`, but it's read
//!   with `std::fs` *before the `App` exists* (window creation needs the
//!   resolution up front) and only becomes a resource at `insert_resource` in
//!   `main`. None of its loading is Bevy's, so it stays with the rest of the
//!   pre-`App` config readers.
//!
//! Imports in here are explicit (`bevy::prelude` plus named
//! `crate::components` / `crate::utils` items) rather than `crate::prelude::*`:
//! the crate prelude re-exports this module, so staying a leaf keeps that
//! dependency one-directional.

pub mod button_config;
pub mod character_animations;
pub mod pending_assets;

pub use self::button_config::*;
pub use self::character_animations::*;
pub use self::pending_assets::*;
