use crate::prelude::*;

// Drives `Loading` -> `Initialize` off `AssetServer`'s own load state, folded
// over every handle any plugin registered in `PendingAssets` during `Startup`.
// One system for the whole app, not one readiness check per plugin.
//
// Fallible on purpose. A failed load returns `Err`, which routes through
// bevy's error handler with the offending path in the message. The two
// alternatives are both worse: a `GameState::LoadFailed` variant is a state
// with nothing to render, and the `AssetBarrier` version this replaced just
// advanced anyway and left the next state to `.expect()` on an invariant the
// barrier never actually provided (see `PendingAssets`'s doc comment).
pub fn check_pending_assets(
    pending: Res<PendingAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result<(), BevyError> {
    match pending.progress(&asset_server) {
        AssetLoadProgress::Loading => {}
        AssetLoadProgress::Ready => next_state.set(GameState::Initialize),
        AssetLoadProgress::Failed(failures) => {
            return Err(format!(
                "{} asset(s) failed to load:\n  {}",
                failures.len(),
                failures.join("\n  ")
            )
            .into());
        }
    }

    Ok(())
}
