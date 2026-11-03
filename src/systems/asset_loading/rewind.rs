use crate::prelude::*;

use super::LoadStage;

#[derive(Resource, Default)]
pub struct RewindRequest(Option<LoadStage>);

impl RewindRequest {
    pub fn to(&mut self, stage: LoadStage) {
        self.0 = Some(match self.0 {
            Some(current) => current.earliest(stage),
            None => stage,
        });
    }

    pub fn requested(&self) -> Option<LoadStage> {
        self.0
    }
}

pub fn rewind_on_change<A: Asset>(
    stage: LoadStage,
) -> impl FnMut(MessageReader<AssetEvent<A>>, ResMut<RewindRequest>) + Clone {
    move |mut events, mut request| {
        if events.read().any(|event| matches!(event, AssetEvent::Modified { .. })) {
            request.to(stage);
        }
    }
}

pub fn apply_rewind(
    mut request: ResMut<RewindRequest>,
    current: Res<State<LoadStage>>,
    mut next: ResMut<NextState<LoadStage>>,
) {
    let Some(target) = request.0.take() else {
        return;
    };

    if target.position() < current.get().position() {
        info!("asset changed: rewinding load stage to {target:?}");
        next.set(target);
    }
}

pub fn despawn_on_replay<M: Component>(mut commands: Commands, entities: Query<Entity, With<M>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
