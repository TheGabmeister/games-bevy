use bevy::prelude::*;

use crate::components::*;
use crate::grid::*;

pub fn guard_ai(
    time: Res<Time>,
    grid: Res<LevelGrid>,
    hole_map: Res<HoleMap>,
    player_query: Query<&GridPosition, With<Player>>,
    mut guard_query: Query<(&GridPosition, &mut MovementState, &mut AiTimer), With<Guard>>,
) {
    let Ok(player_pos) = player_query.single() else {
        return;
    };
    let goal = player_pos.0;

    for (guard_pos, mut movement, mut ai_timer) in &mut guard_query {
        ai_timer.0.tick(time.delta());

        // Only decide when idle
        if !matches!(*movement, MovementState::Idle) {
            continue;
        }
        if !ai_timer.0.just_finished() {
            continue;
        }

        let pos = guard_pos.0;

        // If unsupported, gravity will handle it
        if !grid.is_supported(pos.x, pos.y, &hole_map) {
            continue;
        }

        if let Some(first_step) = bfs_next_step(&grid, &hole_map, pos, goal) {
            let delta = first_step - pos;
            if delta.y > 0 {
                *movement = MovementState::Climbing {
                    from: pos,
                    to: first_step,
                    progress: 0.0,
                };
            } else if delta.y < 0 {
                let target_tile = grid.effective_tile(first_step.x, first_step.y, &hole_map);
                if target_tile == Tile::Ladder
                    || grid.effective_tile(pos.x, pos.y, &hole_map) == Tile::Ladder
                {
                    *movement = MovementState::Climbing {
                        from: pos,
                        to: first_step,
                        progress: 0.0,
                    };
                } else {
                    *movement = MovementState::Falling {
                        from: pos,
                        to: first_step,
                        progress: 0.0,
                    };
                }
            } else {
                *movement = MovementState::Moving {
                    from: pos,
                    to: first_step,
                    progress: 0.0,
                };
            }
        }
    }
}
