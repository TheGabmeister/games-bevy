use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::grid::*;

pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    grid: Res<LevelGrid>,
    hole_map: Res<HoleMap>,
    mut query: Query<(&GridPosition, &mut MovementState), With<Player>>,
) {
    let Ok((grid_pos, mut movement)) = query.single_mut() else {
        return;
    };

    if !matches!(*movement, MovementState::Idle) {
        return;
    }

    let pos = grid_pos.0;

    let dig_left = keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::KeyZ);
    let dig_right = keyboard.just_pressed(KeyCode::KeyE) || keyboard.just_pressed(KeyCode::KeyX);

    if dig_left && grid.can_dig(pos, -1, &hole_map) {
        start_dig(&mut movement, HorizontalDir::Left);
        return;
    }
    if dig_right && grid.can_dig(pos, 1, &hole_map) {
        start_dig(&mut movement, HorizontalDir::Right);
        return;
    }

    let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);
    let up = keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp);
    let down = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);

    if up && grid.can_climb_up(pos.x, pos.y, &hole_map) {
        *movement = MovementState::Climbing {
            from: pos,
            to: pos + IVec2::Y,
            progress: 0.0,
        };
    } else if down && grid.can_climb_down(pos.x, pos.y, &hole_map) {
        let target = pos - IVec2::Y;
        let here = grid.effective_tile(pos.x, pos.y, &hole_map);
        if here == Tile::Ladder
            || grid.effective_tile(target.x, target.y, &hole_map) == Tile::Ladder
        {
            *movement = MovementState::Climbing {
                from: pos,
                to: target,
                progress: 0.0,
            };
        } else {
            *movement = MovementState::Falling {
                from: pos,
                to: target,
                progress: 0.0,
            };
        }
    } else if left && grid.can_move_horizontal(pos.x, pos.y, -1, &hole_map) {
        *movement = MovementState::Moving {
            from: pos,
            to: pos - IVec2::X,
            progress: 0.0,
        };
    } else if right && grid.can_move_horizontal(pos.x, pos.y, 1, &hole_map) {
        *movement = MovementState::Moving {
            from: pos,
            to: pos + IVec2::X,
            progress: 0.0,
        };
    }
}

fn start_dig(movement: &mut MovementState, side: HorizontalDir) {
    *movement = MovementState::Digging {
        side,
        timer: Timer::from_seconds(DIG_DURATION, TimerMode::Once),
    };
}
