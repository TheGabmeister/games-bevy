use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::level::*;
use crate::resources::*;
use crate::states::AppState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (player_input, player_movement.after(player_input), player_hammer_tick.after(player_movement))
                .run_if(in_state(AppState::Playing)),
        );
    }
}

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    stage: Res<StageData>,
    mut query: Query<(&mut PlayerState, &mut Transform), With<Player>>,
) {
    let Ok((mut ps, mut tf)) = query.single_mut() else { return };

    if ps.locomotion == Locomotion::Dying {
        return;
    }

    let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);
    let up = keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp);
    let down = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);
    let jump = keyboard.just_pressed(KeyCode::Space);

    match ps.locomotion {
        Locomotion::Walking => {
            let dir_x = if left { -1.0 } else if right { 1.0 } else { 0.0 };
            if dir_x != 0.0 {
                ps.facing = dir_x;
            }

            // Jump (always allowed, even with hammer)
            if jump {
                ps.locomotion = Locomotion::Jumping;
                ps.vel_y = PLAYER_JUMP_IMPULSE;
                ps.jump_dx = dir_x;
                ps.current_girder = None;
                ps.jump_scored.clear();
                ps.jump_score_count = 0;
                return;
            }

            // Ladder entry (disabled if hammer active)
            if ps.hammer_timer.is_none() && (up || down) && let Some(girder_idx) = ps.current_girder {
                    for (i, ladder) in stage.ladders.iter().enumerate() {
                        if ladder.kind == LadderKind::Broken {
                            continue;
                        }
                        if (tf.translation.x - ladder.x).abs() > LADDER_GRAB_HALF_WIDTH {
                            continue;
                        }
                        if up && ladder.bottom_girder == girder_idx {
                            ps.locomotion = Locomotion::Climbing;
                            ps.current_ladder = Some(i);
                            ps.current_girder = None;
                            tf.translation.x = ladder.x;
                            return;
                        }
                        if down && ladder.top_girder == girder_idx {
                            ps.locomotion = Locomotion::Climbing;
                            ps.current_ladder = Some(i);
                            ps.current_girder = None;
                            tf.translation.x = ladder.x;
                            return;
                        }
                }
            }
        }
        Locomotion::Climbing => {
            // handled in movement
        }
        _ => {}
    }
}

fn player_movement(
    time: Res<Time>,
    stage: Res<StageData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerState, &mut Transform), With<Player>>,
) {
    let Ok((mut ps, mut tf)) = query.single_mut() else { return };
    let dt = time.delta_secs();

    match ps.locomotion {
        Locomotion::Walking => {
            let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
            let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);
            let dir_x = if left { -1.0 } else if right { 1.0 } else { 0.0 };

            if let Some(gi) = ps.current_girder {
                let girder = &stage.girders[gi];
                let new_x = tf.translation.x + dir_x * PLAYER_WALK_SPEED * dt;

                // Check if walked off girder edge
                if new_x < girder.left.x || new_x > girder.right.x {
                    // Walk off edge — start falling
                    tf.translation.x = new_x.clamp(PLAYFIELD_X_MIN, PLAYFIELD_X_MAX);
                    ps.locomotion = Locomotion::Falling;
                    ps.vel_y = 0.0;
                    ps.current_girder = None;
                } else {
                    tf.translation.x = new_x;
                    let surface = girder_surface_y(girder, new_x);
                    tf.translation.y = surface + PLAYER_HEIGHT / 2.0;
                    ps.last_supported_y = tf.translation.y;
                }
            }
        }
        Locomotion::Jumping | Locomotion::Falling => {
            tf.translation.x += ps.jump_dx * PLAYER_WALK_SPEED * dt;
            tf.translation.x = tf.translation.x.clamp(PLAYFIELD_X_MIN, PLAYFIELD_X_MAX);

            ps.vel_y -= PLAYER_GRAVITY * dt;
            tf.translation.y += ps.vel_y * dt;

            if ps.vel_y < 0.0 && ps.locomotion == Locomotion::Jumping {
                ps.locomotion = Locomotion::Falling;
            }

            // Check for girder landing when falling
            if ps.vel_y <= 0.0 {
                let feet_y = tf.translation.y - PLAYER_HEIGHT / 2.0;
                if let Some(gi) = find_supporting_girder(&stage, tf.translation.x, feet_y) {
                    let surface = girder_surface_y(&stage.girders[gi], tf.translation.x);
                    tf.translation.y = surface + PLAYER_HEIGHT / 2.0;

                    let fall_dist = ps.last_supported_y - tf.translation.y;
                    if fall_dist > PLAYER_FALL_DEATH_THRESHOLD {
                        ps.locomotion = Locomotion::Dying;
                    } else {
                        ps.locomotion = Locomotion::Walking;
                        ps.current_girder = Some(gi);
                        ps.vel_y = 0.0;
                        ps.last_supported_y = tf.translation.y;
                    }
                }
            }

            // Fell off the bottom
            if tf.translation.y < -PLAYFIELD_HEIGHT / 2.0 {
                ps.locomotion = Locomotion::Dying;
            }
        }
        Locomotion::Climbing => {
            let up = keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp);
            let down = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);

            if let Some(li) = ps.current_ladder {
                let ladder = &stage.ladders[li];
                let bot_y = girder_surface_y(&stage.girders[ladder.bottom_girder], ladder.x);
                let top_y = girder_surface_y(&stage.girders[ladder.top_girder], ladder.x);

                if up {
                    tf.translation.y += PLAYER_CLIMB_SPEED * dt;
                    if tf.translation.y - PLAYER_HEIGHT / 2.0 >= top_y {
                        tf.translation.y = top_y + PLAYER_HEIGHT / 2.0;
                        ps.locomotion = Locomotion::Walking;
                        ps.current_girder = Some(ladder.top_girder);
                        ps.current_ladder = None;
                        ps.last_supported_y = tf.translation.y;
                    }
                }
                if down {
                    tf.translation.y -= PLAYER_CLIMB_SPEED * dt;
                    if tf.translation.y - PLAYER_HEIGHT / 2.0 <= bot_y {
                        tf.translation.y = bot_y + PLAYER_HEIGHT / 2.0;
                        ps.locomotion = Locomotion::Walking;
                        ps.current_girder = Some(ladder.bottom_girder);
                        ps.current_ladder = None;
                        ps.last_supported_y = tf.translation.y;
                    }
                }
            }
        }
        Locomotion::Dying => {}
    }
}

fn player_hammer_tick(
    time: Res<Time>,
    game_mats: Res<GameMaterials>,
    mut query: Query<(&mut PlayerState, &mut MeshMaterial2d<ColorMaterial>), With<Player>>,
) {
    let Ok((mut ps, mut mat)) = query.single_mut() else { return };

    if let Some(ref mut timer) = ps.hammer_timer {
        *timer -= time.delta_secs();
        if *timer <= 0.0 {
            ps.hammer_timer = None;
            mat.0 = game_mats.player_normal.clone();
        } else if *timer < HAMMER_FLASH_TIME {
            let flash = (*timer * 8.0).sin() > 0.0;
            mat.0 = if flash {
                game_mats.player_hammer.clone()
            } else {
                game_mats.player_normal.clone()
            };
        }
    }
}
