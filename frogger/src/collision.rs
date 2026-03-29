use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::AppState;

pub fn ride_platforms(
    time: Res<Time>,
    level_state: Res<LevelState>,
    mut frog_query: Query<(&GridPosition, &HopState, &mut Transform), With<Frog>>,
    platform_query: Query<(&Transform, &Velocity, &ObjectWidth), (With<Platform>, Without<Frog>)>,
) {
    let Ok((grid_pos, hop_state, mut frog_tf)) = frog_query.single_mut() else {
        return;
    };
    if hop_state.active {
        return;
    }
    if grid_pos.row < RIVER_ROW_START || grid_pos.row > RIVER_ROW_END {
        return;
    }

    let frog_x = frog_tf.translation.x;
    let frog_y = frog_tf.translation.y;

    for (plat_tf, velocity, obj_width) in &platform_query {
        if (plat_tf.translation.y - frog_y).abs() > CELL_SIZE * 0.6 {
            continue;
        }
        let half_plat = obj_width.0 / 2.0;
        let half_frog = CELL_SIZE * 0.4;
        if (frog_x - plat_tf.translation.x).abs() < half_plat + half_frog {
            frog_tf.translation.x +=
                velocity.0.x * level_state.speed_multiplier * time.delta_secs();
            return;
        }
    }
}

pub fn check_vehicle_collision(
    frog_query: Query<&Transform, With<Frog>>,
    vehicle_query: Query<(&Transform, &ObjectWidth), (With<Vehicle>, Without<Frog>)>,
    mut frog_event: ResMut<FrogEvent>,
) {
    if *frog_event != FrogEvent::None {
        return;
    }
    let Ok(frog_tf) = frog_query.single() else {
        return;
    };
    let fp = frog_tf.translation.truncate();
    let frog_half = CELL_SIZE * 0.35;

    for (veh_tf, obj_width) in &vehicle_query {
        let vp = veh_tf.translation.truncate();
        if (fp.x - vp.x).abs() < obj_width.0 / 2.0 + frog_half
            && (fp.y - vp.y).abs() < CELL_SIZE * 0.4 + frog_half
        {
            *frog_event = FrogEvent::Death;
            return;
        }
    }
}

pub fn check_water_support(
    frog_query: Query<(&GridPosition, &HopState, &Transform), With<Frog>>,
    platform_query: Query<(&Transform, &ObjectWidth), (With<Platform>, Without<Frog>)>,
    mut frog_event: ResMut<FrogEvent>,
) {
    if *frog_event != FrogEvent::None {
        return;
    }
    let Ok((grid_pos, hop_state, frog_tf)) = frog_query.single() else {
        return;
    };
    if hop_state.active {
        return;
    }
    if grid_pos.row < RIVER_ROW_START || grid_pos.row > RIVER_ROW_END {
        return;
    }

    let frog_x = frog_tf.translation.x;
    let frog_y = frog_tf.translation.y;

    for (plat_tf, obj_width) in &platform_query {
        if (plat_tf.translation.y - frog_y).abs() > CELL_SIZE * 0.6 {
            continue;
        }
        let half_plat = obj_width.0 / 2.0;
        let half_frog = CELL_SIZE * 0.4;
        if (frog_x - plat_tf.translation.x).abs() < half_plat + half_frog {
            return; // Supported
        }
    }

    *frog_event = FrogEvent::Death;
}

pub fn check_bounds(
    frog_query: Query<(&HopState, &Transform), With<Frog>>,
    mut frog_event: ResMut<FrogEvent>,
) {
    if *frog_event != FrogEvent::None {
        return;
    }
    let Ok((hop_state, frog_tf)) = frog_query.single() else {
        return;
    };
    if hop_state.active {
        return;
    }
    if frog_tf.translation.x.abs() > PLAYFIELD_WIDTH / 2.0 + CELL_SIZE {
        *frog_event = FrogEvent::Death;
    }
}

pub fn check_home_bay(
    frog_query: Query<(&GridPosition, &HopState), With<Frog>>,
    mut game_data: ResMut<GameData>,
    timer: Res<FrogTimer>,
    mut frog_event: ResMut<FrogEvent>,
) {
    if *frog_event != FrogEvent::None {
        return;
    }
    let Ok((grid_pos, hop_state)) = frog_query.single() else {
        return;
    };
    if hop_state.active || grid_pos.row != HOME_ROW {
        return;
    }

    let bay_index = HOME_BAY_COLS.iter().position(|&c| c == grid_pos.col);

    match bay_index {
        Some(idx) if !game_data.filled_bays[idx] => {
            game_data.filled_bays[idx] = true;
            let time_bonus = (timer.remaining_secs as u32) * SCORE_TIME_BONUS_PER_SEC;
            game_data.score += SCORE_HOME_BAY + time_bonus;
            *frog_event = FrogEvent::BayFilled;
        }
        _ => {
            *frog_event = FrogEvent::Death;
        }
    }
}

pub fn tick_timer(
    time: Res<Time>,
    mut timer: ResMut<FrogTimer>,
    mut frog_event: ResMut<FrogEvent>,
    level_state: Res<LevelState>,
) {
    if *frog_event != FrogEvent::None || level_state.celebrating {
        return;
    }
    timer.remaining_secs -= time.delta_secs();
    if timer.remaining_secs <= 0.0 {
        timer.remaining_secs = 0.0;
        *frog_event = FrogEvent::Death;
    }
}

pub fn handle_frog_event(
    mut frog_event: ResMut<FrogEvent>,
    mut game_data: ResMut<GameData>,
    mut timer: ResMut<FrogTimer>,
    mut next_state: ResMut<NextState<AppState>>,
    mut frog_query: Query<(&mut GridPosition, &mut HopState, &mut Transform), With<Frog>>,
) {
    let event = *frog_event;
    if event == FrogEvent::None {
        return;
    }
    *frog_event = FrogEvent::None;

    if event == FrogEvent::Death {
        game_data.lives = game_data.lives.saturating_sub(1);
        if game_data.lives == 0 {
            game_data.high_score = game_data.high_score.max(game_data.score);
            next_state.set(AppState::GameOver);
            return;
        }
    }

    // Respawn frog (both Death-with-lives and BayFilled)
    let Ok((mut gp, mut hop, mut tf)) = frog_query.single_mut() else {
        return;
    };
    gp.col = FROG_SPAWN_COL;
    gp.row = FROG_SPAWN_ROW;
    hop.active = false;
    let pos = grid_to_world(FROG_SPAWN_COL, FROG_SPAWN_ROW);
    tf.translation.x = pos.x;
    tf.translation.y = pos.y;
    timer.remaining_secs = LIFE_TIMER_SECS;
    game_data.max_row_this_life = 0;
}

pub fn check_level_clear(mut game_data: ResMut<GameData>, mut level_state: ResMut<LevelState>) {
    if !game_data.filled_bays.iter().all(|&b| b) {
        return;
    }
    game_data.score += SCORE_LEVEL_CLEAR;
    game_data.level += 1;
    game_data.filled_bays = [false; 5];
    level_state.speed_multiplier =
        (level_state.speed_multiplier + SPEED_INCREMENT).min(MAX_SPEED_MULTIPLIER);
}
