// round.rs
use bevy::prelude::*;
use crate::core::states::RollbackState;
use crate::core::resources::RoundEndTimer;

pub fn round_end_timeout(
    mut timer: ResMut<RoundEndTimer>,
    mut state: ResMut<NextState<RollbackState>>,
    time: Res<Time>,
) {
    timer.tick(time.delta());

    if timer.just_finished() {
        state.set(RollbackState::InRound);
    }
}