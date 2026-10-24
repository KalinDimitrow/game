use crate::prelude::*;
use crate::utils::system_config::ButtonConfig;

// `ButtonConfig` was already read/parsed and inserted as a resource by
// `main`, before the `App` existed - this system just turns it into an
// `InputMap`.
pub fn load_button_config(mut commands: Commands, button_config: Res<ButtonConfig>) {
    let mut input_map = InputMap::<PlayerAction>::default();
    for (action, key) in &button_config.bindings {
        input_map.insert(*action, *key);
    }

    // `ActionState<PlayerAction>` is added automatically as a required component of `InputMap`.
    commands.spawn(input_map);
}
