use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::grid::*;
use crate::render::RenderAssets;
use crate::states::AppState;

pub fn advance_movement(
    mut commands: Commands,
    time: Res<Time>,
    grid: Res<LevelGrid>,
    render_assets: Res<RenderAssets>,
    mut query: Query<(&mut GridPosition, &mut MovementState, Option<&Guard>)>,
    mut hole_map: ResMut<HoleMap>,
) {
    let dt = time.delta_secs();

    for (mut grid_pos, mut movement, is_guard) in &mut query {
        match &mut *movement {
            MovementState::Idle => continue,
            MovementState::Digging { timer, side } => {
                timer.tick(time.delta());
                if timer.is_finished() {
                    let dx = match side {
                        HorizontalDir::Left => -1,
                        HorizontalDir::Right => 1,
                    };
                    let target = IVec2::new(grid_pos.0.x + dx, grid_pos.0.y - 1);
                    hole_map.insert(target.x, target.y, HolePhase::Open);
                    let world_pos = grid_to_world(target, grid.width, grid.height, CELL_SIZE);
                    commands.spawn((
                        Hole {
                            cell: target,
                            phase: HolePhase::Open,
                            timer: Timer::from_seconds(HOLE_OPEN_DURATION, TimerMode::Once),
                        },
                        Mesh2d(render_assets.hole_mesh.clone()),
                        MeshMaterial2d(render_assets.hole_material.clone()),
                        Transform::from_xyz(world_pos.x, world_pos.y, 1.5),
                        DespawnOnExit(AppState::Playing),
                    ));
                    *movement = MovementState::Idle;
                }
                continue;
            }
            MovementState::Trapped { timer } => {
                timer.tick(time.delta());
                if timer.is_finished() {
                    // Escape: move up one cell
                    let pos = grid_pos.0;
                    grid_pos.0 = pos + IVec2::Y;
                    *movement = MovementState::Idle;
                }
                continue;
            }
            _ => {}
        }

        let speed = match *movement {
            MovementState::Moving { .. } => {
                if is_guard.is_some() {
                    GUARD_MOVE_SPEED
                } else {
                    PLAYER_MOVE_SPEED
                }
            }
            MovementState::Climbing { .. } => {
                if is_guard.is_some() {
                    GUARD_CLIMB_SPEED
                } else {
                    PLAYER_CLIMB_SPEED
                }
            }
            MovementState::Falling { .. } => {
                if is_guard.is_some() {
                    GUARD_FALL_SPEED
                } else {
                    PLAYER_FALL_SPEED
                }
            }
            _ => continue,
        };

        let (_from, to, progress) = match &mut *movement {
            MovementState::Moving { from, to, progress }
            | MovementState::Climbing { from, to, progress }
            | MovementState::Falling { from, to, progress } => (*from, *to, progress),
            _ => unreachable!(),
        };

        *progress += speed * dt;

        if *progress >= 1.0 {
            grid_pos.0 = to;
            *movement = MovementState::Idle;
        }
    }
}

pub fn tick_holes(
    mut commands: Commands,
    time: Res<Time>,
    mut hole_map: ResMut<HoleMap>,
    mut query: Query<(Entity, &mut Hole)>,
) {
    for (entity, mut hole) in &mut query {
        hole.timer.tick(time.delta());
        if !hole.timer.is_finished() {
            continue;
        }

        match hole.phase {
            HolePhase::Open => {
                hole.phase = HolePhase::Closing;
                hole.timer = Timer::from_seconds(HOLE_CLOSE_DURATION, TimerMode::Once);
                hole_map.insert(hole.cell.x, hole.cell.y, HolePhase::Closing);
            }
            HolePhase::Closing => {
                hole_map.remove(hole.cell.x, hole.cell.y);
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn apply_gravity(
    grid: Res<LevelGrid>,
    hole_map: Res<HoleMap>,
    mut query: Query<(&GridPosition, &mut MovementState)>,
) {
    for (grid_pos, mut movement) in &mut query {
        if !matches!(*movement, MovementState::Idle) {
            continue;
        }

        let pos = grid_pos.0;
        if !grid.is_supported(pos.x, pos.y, &hole_map) {
            *movement = MovementState::Falling {
                from: pos,
                to: pos - IVec2::Y,
                progress: 0.0,
            };
        }
    }
}

pub fn sync_transforms(
    grid: Res<LevelGrid>,
    mut query: Query<(&GridPosition, &MovementState, &mut Transform)>,
) {
    for (grid_pos, movement, mut transform) in &mut query {
        let world_pos = match movement {
            MovementState::Idle | MovementState::Digging { .. } | MovementState::Trapped { .. } => {
                grid_to_world(grid_pos.0, grid.width, grid.height, CELL_SIZE)
            }
            MovementState::Moving {
                from, to, progress, ..
            }
            | MovementState::Climbing {
                from, to, progress, ..
            }
            | MovementState::Falling {
                from, to, progress, ..
            } => {
                let a = grid_to_world(*from, grid.width, grid.height, CELL_SIZE);
                let b = grid_to_world(*to, grid.width, grid.height, CELL_SIZE);
                a.lerp(b, progress.clamp(0.0, 1.0))
            }
        };

        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}
