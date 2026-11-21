use crate::prelude::*;

pub fn apply_button_config(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<ButtonConfig>>,
    configs: Res<Assets<ButtonConfig>>,
    existing: Query<Entity, With<InputMap<PlayerAction>>>,
) {
    for event in events.read() {
        let (AssetEvent::Added { id } | AssetEvent::Modified { id }) = event else {
            continue;
        };
        let Some(config) = configs.get(*id) else {
            continue;
        };

        let mut input_map = InputMap::<PlayerAction>::default();
        for (action, key) in &config.bindings {
            input_map.insert(*action, *key);
        }

        if let Ok(entity) = existing.single() {
            commands.entity(entity).insert(input_map);
        } else {
            commands.spawn(input_map);
        }
    }
}
