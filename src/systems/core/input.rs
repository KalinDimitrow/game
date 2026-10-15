use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::components::PlayerAction;

// Reacts to `PlayerAction`s rather than hardcoded keys - whatever key the
// button config binds to `Confirm`/`Pause` ends up here unchanged.
pub fn player_action_system(action_state: Single<&ActionState<PlayerAction>>) {
    if action_state.just_pressed(&PlayerAction::Confirm) {
        info!("Confirm action triggered");
    }
    if action_state.just_pressed(&PlayerAction::Pause) {
        info!("Pause action triggered");
    }
}
