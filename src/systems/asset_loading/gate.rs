use std::collections::HashSet;

use bevy::asset::UntypedHandle;

use crate::prelude::*;

use super::LoadStage;

#[derive(Resource, Default)]
pub struct StageGate {
    handles: PendingAssets,
    blockers: HashSet<&'static str>,
    assets_ready: bool,
}

impl StageGate {
    pub fn track(&mut self, handle: impl Into<UntypedHandle>) {
        self.handles.track(handle);
    }

    pub fn block(&mut self, reason: &'static str) {
        self.blockers.insert(reason);
    }

    pub fn release(&mut self, reason: &'static str) {
        self.blockers.remove(reason);
    }

    pub fn is_ready(&self) -> bool {
        self.assets_ready
    }

    pub fn waiting_on(&self) -> Vec<&'static str> {
        let mut reasons: Vec<_> = self.blockers.iter().copied().collect();
        reasons.sort_unstable();
        reasons
    }
}

pub fn poll_gate(
    mut gate: ResMut<StageGate>,
    asset_server: Res<AssetServer>,
    current: Res<State<LoadStage>>,
) -> Result<(), BevyError> {
    let progress = gate.handles.progress(&asset_server);
    match progress {
        AssetLoadProgress::Loading => gate.assets_ready = false,
        AssetLoadProgress::Ready => gate.assets_ready = true,
        AssetLoadProgress::Failed(failures) => {
            return Err(format!(
                "load stage {:?}: {} asset(s) failed to load:\n  {}",
                current.get(),
                failures.len(),
                failures.join("\n  ")
            )
            .into());
        }
    }

    Ok(())
}

pub fn advance_stage(
    current: Res<State<LoadStage>>,
    gate: Res<StageGate>,
    mut next: ResMut<NextState<LoadStage>>,
) {
    let stage = *current.get();
    if stage == LoadStage::Settled {
        return;
    }

    if !gate.is_ready() {
        return;
    }

    if gate.blockers.is_empty() {
        next.set(stage.next());
    }
}

pub fn log_stage_transitions(
    mut transitions: MessageReader<StateTransitionEvent<LoadStage>>,
    gate: Res<StageGate>,
) {
    for transition in transitions.read() {
        let Some(entered) = transition.entered else {
            continue;
        };
        let waiting = gate.waiting_on();
        if waiting.is_empty() {
            debug!("load stage -> {entered:?}");
        } else {
            debug!("load stage -> {entered:?}, waiting on {waiting:?}");
        }
    }
}

pub fn clear_gate(mut gate: ResMut<StageGate>) {
    *gate = StageGate::default();
}
