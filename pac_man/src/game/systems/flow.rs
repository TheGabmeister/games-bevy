use bevy::prelude::*;

use crate::game::{
    components::{Ghost, GridMover, Player},
    constants::{ACTOR_Z, GHOST_SPEED, PLAYER_SPEED},
    level::LevelLayout,
    resources::{GameSession, RoundState},
};

pub fn enter_ready_phase(
    layout: Res<LevelLayout>,
    mut session: ResMut<GameSession>,
    player: Single<(&mut Transform, &mut GridMover), With<Player>>,
    mut ghosts: Query<(&mut Ghost, &mut Transform, &mut GridMover)>,
) {
    session.prepare_ready_phase();

    let (mut player_transform, mut player_mover) = player.into_inner();
    reset_mover_position(&mut player_transform, &mut player_mover, &layout);
    player_mover.speed = PLAYER_SPEED;

    for (mut ghost, mut transform, mut mover) in &mut ghosts {
        ghost.returning_home = false;
        reset_mover_position(&mut transform, &mut mover, &layout);
        mover.speed = GHOST_SPEED;
    }
}

pub fn clear_round_effects(mut session: ResMut<GameSession>) {
    session.clear_round_effects();
}

pub fn advance_ready_timer(
    time: Res<Time<Fixed>>,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<RoundState>>,
) {
    session.ready_timer.tick(time.delta());
    if session.ready_timer.is_finished() {
        next_state.set(RoundState::Playing);
    }
}

pub fn advance_playing_timers(time: Res<Time<Fixed>>, mut session: ResMut<GameSession>) {
    let was_frightened = session.frightened_active();
    session.frightened_seconds = (session.frightened_seconds - time.delta_secs()).max(0.0);

    if was_frightened && !session.frightened_active() {
        session.ghost_combo = 0;
    }

    session.advance_mode_cycle(time.delta_secs());
}

fn reset_mover_position(transform: &mut Transform, mover: &mut GridMover, layout: &LevelLayout) {
    transform.translation = layout.tile_to_world(mover.spawn_tile).extend(ACTOR_Z);
    if let Some(direction) = mover.spawn_direction {
        transform.rotation = direction.rotation();
    }
    mover.current = mover.spawn_direction;
    mover.desired = mover.spawn_direction;
}
