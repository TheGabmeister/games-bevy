use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::grid::*;
use crate::resources::*;
use crate::states::{AppState, PlayState};

pub fn collect_gold(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut grid: ResMut<LevelGrid>,
    player_query: Query<&GridPosition, With<Player>>,
    gold_query: Query<(Entity, &GridPosition), With<Gold>>,
    mut hidden_query: Query<&mut Visibility, With<HiddenLadderTile>>,
) {
    let Ok(player_pos) = player_query.single() else {
        return;
    };

    for (entity, gold_pos) in &gold_query {
        if gold_pos.0 == player_pos.0 {
            commands.entity(entity).despawn();
            game_state.score += GOLD_SCORE;
            game_state.total_gold = game_state.total_gold.saturating_sub(1);

            if game_state.total_gold == 0 && !game_state.exit_unlocked {
                game_state.exit_unlocked = true;
                grid.reveal_hidden_ladders();
                for mut vis in &mut hidden_query {
                    *vis = Visibility::Visible;
                }
            }
        }
    }
}

pub fn check_guard_trap(
    hole_map: Res<HoleMap>,
    mut query: Query<(&GridPosition, &mut MovementState), With<Guard>>,
) {
    for (grid_pos, mut movement) in &mut query {
        if !matches!(*movement, MovementState::Idle) {
            continue;
        }

        let pos = grid_pos.0;
        if hole_map.get(pos.x, pos.y) == Some(HolePhase::Open) {
            *movement = MovementState::Trapped {
                timer: Timer::from_seconds(GUARD_TRAP_DURATION, TimerMode::Once),
            };
        }
    }
}

pub fn check_player_death(
    player_query: Query<&GridPosition, With<Player>>,
    guard_query: Query<(&GridPosition, &MovementState), With<Guard>>,
    mut next_play: ResMut<NextState<PlayState>>,
) {
    let Ok(player_pos) = player_query.single() else {
        return;
    };

    for (guard_pos, movement) in &guard_query {
        if matches!(movement, MovementState::Trapped { .. }) {
            continue;
        }
        if guard_pos.0 == player_pos.0 {
            next_play.set(PlayState::Dying);
            return;
        }
    }
}

pub fn check_exit(
    game_state: Res<GameState>,
    player_query: Query<&GridPosition, With<Player>>,
    grid: Res<LevelGrid>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !game_state.exit_unlocked {
        return;
    }

    let Ok(player_pos) = player_query.single() else {
        return;
    };

    if player_pos.0.y >= grid.height as i32 - 1 {
        next_state.set(AppState::LevelComplete);
    }
}

pub fn start_death_sequence(mut commands: Commands, mut game_state: ResMut<GameState>) {
    game_state.lives = game_state.lives.saturating_sub(1);
    commands.insert_resource(DeathTimer(Timer::from_seconds(
        DEATH_PAUSE,
        TimerMode::Once,
    )));
}

pub fn tick_death(
    time: Res<Time>,
    mut death_timer: ResMut<DeathTimer>,
    game_state: Res<GameState>,
    mut next_app: ResMut<NextState<AppState>>,
) {
    death_timer.0.tick(time.delta());
    if !death_timer.0.is_finished() {
        return;
    }

    if game_state.lives == 0 {
        next_app.set(AppState::GameOver);
    } else {
        next_app.set(AppState::Restarting);
    }
}

pub fn restart_level(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Playing);
}
