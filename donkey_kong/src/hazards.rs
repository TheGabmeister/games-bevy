use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::level::*;
use crate::resources::*;
use crate::states::AppState;

pub struct HazardsPlugin;

impl Plugin for HazardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (dk_throw, move_barrels.after(dk_throw), move_fireballs.after(move_barrels))
                .run_if(in_state(AppState::Playing)),
        );
    }
}

// --- DK Throwing ---

fn dk_throw(
    time: Res<Time>,
    stage: Res<StageData>,
    wave_cfg: Res<WaveConfig>,
    game_meshes: Res<GameMeshes>,
    game_mats: Res<GameMaterials>,
    mut commands: Commands,
    mut wave_rt: ResMut<WaveRuntime>,
    mut dk_q: Query<&mut DkState, With<DonkeyKong>>,
) {
    let Ok(mut dk) = dk_q.single_mut() else { return };
    let dt = time.delta_secs();

    match dk.anim {
        DkAnimState::Idle => {
            dk.throw_timer += dt;
            if dk.throw_timer >= wave_cfg.throw_interval {
                dk.anim = DkAnimState::WindUp;
                dk.timer = 0.0;
            }
        }
        DkAnimState::WindUp => {
            dk.timer += dt;
            if dk.timer >= DK_WINDUP_DURATION {
                dk.anim = DkAnimState::Throwing;
                dk.timer = 0.0;

                // Spawn barrel
                dk.barrels_thrown += 1;
                let is_blue = dk.barrels_thrown % wave_cfg.blue_barrel_every == 0;
                let is_wild = !is_blue && wave_rt.rng.chance(WILD_BARREL_CHANCE);

                let girder = &stage.girders[5]; // DK's platform
                let spawn_x = girder.right.x; // right edge of DK platform
                let spawn_y = girder_surface_y(girder, spawn_x) + BARREL_RADIUS;

                let mat = if is_blue {
                    game_mats.blue_barrel.clone()
                } else {
                    game_mats.barrel.clone()
                };

                commands.spawn((
                    Barrel {
                        is_blue,
                        is_wild,
                        phase: 0.0,
                        state: BarrelMoveState::Rolling,
                        direction: girder.roll_direction,
                        girder: 5,
                    },
                    Mesh2d(game_meshes.barrel.clone()),
                    MeshMaterial2d(mat),
                    Transform::from_xyz(spawn_x, spawn_y, 6.0),
                ));
            }
        }
        DkAnimState::Throwing => {
            dk.timer += dt;
            if dk.timer >= DK_THROW_DURATION {
                dk.anim = DkAnimState::Idle;
                dk.throw_timer = 0.0;
            }
        }
    }
}

// --- Barrel Movement ---

fn move_barrels(
    time: Res<Time>,
    stage: Res<StageData>,
    wave_cfg: Res<WaveConfig>,
    mut wave_rt: ResMut<WaveRuntime>,
    game_meshes: Res<GameMeshes>,
    game_mats: Res<GameMaterials>,
    fireball_q: Query<Entity, With<Fireball>>,
    mut commands: Commands,
    mut barrel_q: Query<(Entity, &mut Barrel, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let speed = BARREL_BASE_SPEED * wave_cfg.barrel_speed_mult;

    for (entity, mut barrel, mut tf) in &mut barrel_q {
        match barrel.state {
            BarrelMoveState::Rolling => {
                let girder = &stage.girders[barrel.girder];
                let prev_x = tf.translation.x;
                let new_x = prev_x + barrel.direction * speed * dt;

                // Check ladder descent
                let mut descended = false;
                for (i, ladder) in stage.ladders.iter().enumerate() {
                    if ladder.top_girder != barrel.girder || ladder.kind == LadderKind::Broken {
                        continue;
                    }
                    // Did we cross the ladder X?
                    if (prev_x - ladder.x).signum() != (new_x - ladder.x).signum()
                        || (prev_x - ladder.x).abs() < 2.0
                    {
                        let should = barrel.is_blue || wave_rt.rng.chance(BARREL_LADDER_CHANCE);
                        if should {
                            tf.translation.x = ladder.x;
                            barrel.state = BarrelMoveState::Descending { ladder_index: i };
                            descended = true;
                            break;
                        }
                    }
                }
                if descended {
                    continue;
                }

                // Check girder bounds
                if new_x <= girder.left.x || new_x >= girder.right.x {
                    let clamped = new_x.clamp(girder.left.x, girder.right.x);
                    tf.translation.x = clamped;

                    // On bottom girder (0) — check oil drum or despawn
                    if barrel.girder == 0 {
                        if tf.translation.x <= stage.oil_drum_x + OIL_DRUM_WIDTH / 2.0 {
                            // Hit oil drum — fireball check
                            let active_fb = fireball_q.iter().count() as u32;
                            if active_fb < wave_cfg.max_fireballs
                                && wave_rt.rng.chance(FIREBALL_SPAWN_CHANCE)
                            {
                                let fb_y = girder_surface_y(&stage.girders[0], stage.oil_drum_x)
                                    + FIREBALL_RADIUS;
                                commands.spawn((
                                    Fireball {
                                        state: FireballMoveState::Patrolling,
                                        direction: 1.0, // start moving right
                                        girder: 0,
                                    },
                                    Mesh2d(game_meshes.fireball.clone()),
                                    MeshMaterial2d(game_mats.fireball.clone()),
                                    Transform::from_xyz(stage.oil_drum_x, fb_y, 6.0),
                                ));
                            }
                        }
                        commands.entity(entity).despawn();
                        continue;
                    }

                    // Fall to next girder below
                    if barrel.girder > 0 {
                        barrel.state =
                            BarrelMoveState::Falling { target_girder: barrel.girder - 1 };
                    }
                } else {
                    tf.translation.x = new_x;
                    let surface = girder_surface_y(girder, new_x);
                    tf.translation.y = surface + BARREL_RADIUS;

                    // Wild barrel visual bounce
                    if barrel.is_wild {
                        barrel.phase += dt;
                        let bounce =
                            (barrel.phase * WILD_BOUNCE_FREQ * std::f32::consts::TAU).sin()
                                * WILD_BOUNCE_AMPLITUDE;
                        tf.translation.y += bounce;
                    }
                }
            }
            BarrelMoveState::Falling { target_girder } => {
                tf.translation.y -= BARREL_FALL_SPEED * dt;
                let target_surface =
                    girder_surface_y(&stage.girders[target_girder], tf.translation.x);
                if tf.translation.y - BARREL_RADIUS <= target_surface {
                    tf.translation.y = target_surface + BARREL_RADIUS;
                    barrel.girder = target_girder;
                    barrel.direction = stage.girders[target_girder].roll_direction;
                    barrel.state = BarrelMoveState::Rolling;
                }
            }
            BarrelMoveState::Descending { ladder_index } => {
                let ladder = &stage.ladders[ladder_index];
                tf.translation.y -= speed * dt;
                let bot_surface =
                    girder_surface_y(&stage.girders[ladder.bottom_girder], ladder.x);
                if tf.translation.y - BARREL_RADIUS <= bot_surface {
                    tf.translation.y = bot_surface + BARREL_RADIUS;
                    barrel.girder = ladder.bottom_girder;
                    barrel.direction = stage.girders[ladder.bottom_girder].roll_direction;
                    barrel.state = BarrelMoveState::Rolling;
                }
            }
        }
    }
}

// --- Fireball Movement ---

fn move_fireballs(
    time: Res<Time>,
    stage: Res<StageData>,
    wave_cfg: Res<WaveConfig>,
    mut wave_rt: ResMut<WaveRuntime>,
    player_q: Query<&Transform, With<Player>>,
    mut fireball_q: Query<(Entity, &mut Fireball, &mut Transform), Without<Player>>,
) {
    let dt = time.delta_secs();
    let speed = FIREBALL_BASE_SPEED * wave_cfg.fireball_speed_mult;
    let climb_speed = FIREBALL_CLIMB_SPEED * wave_cfg.fireball_speed_mult;

    let player_pos = player_q.single().map(|t| t.translation.truncate()).unwrap_or(Vec2::ZERO);

    for (_entity, mut fb, mut tf) in &mut fireball_q {
        match fb.state {
            FireballMoveState::Patrolling => {
                // Pursuit bias: occasionally face the player
                if wave_rt.rng.chance(FIREBALL_PURSUIT_BIAS * dt * 2.0) {
                    fb.direction = if player_pos.x > tf.translation.x {
                        1.0
                    } else {
                        -1.0
                    };
                }

                let girder = &stage.girders[fb.girder];
                let prev_x = tf.translation.x;
                let new_x = prev_x + fb.direction * speed * dt;

                // Check ladder climbing
                let player_girder_y = player_pos.y;
                let my_y = tf.translation.y;

                for ladder in &stage.ladders {
                    // Fireballs can use any ladder (full or broken)
                    let connects_to_my_girder =
                        ladder.bottom_girder == fb.girder || ladder.top_girder == fb.girder;
                    if !connects_to_my_girder {
                        continue;
                    }

                    if (prev_x - ladder.x).signum() != (new_x - ladder.x).signum()
                        || (prev_x - ladder.x).abs() < 2.0
                    {
                        // Decide whether to climb
                        let want_up = player_girder_y > my_y && ladder.bottom_girder == fb.girder;
                        let want_down = player_girder_y < my_y && ladder.top_girder == fb.girder;

                        if (want_up || want_down) && wave_rt.rng.chance(FIREBALL_LADDER_BIAS) {
                            tf.translation.x = ladder.x;
                            let target_girder = if want_up {
                                ladder.top_girder
                            } else {
                                ladder.bottom_girder
                            };
                            fb.state = FireballMoveState::Climbing { target_girder };
                            break;
                        }
                    }
                }

                if matches!(fb.state, FireballMoveState::Climbing { .. }) {
                    continue;
                }

                // Clamp to girder bounds, reverse at edges
                if new_x <= girder.left.x || new_x >= girder.right.x {
                    fb.direction = -fb.direction;
                    tf.translation.x = new_x.clamp(girder.left.x + 1.0, girder.right.x - 1.0);
                } else {
                    tf.translation.x = new_x;
                }

                let surface = girder_surface_y(girder, tf.translation.x);
                tf.translation.y = surface + FIREBALL_RADIUS;
            }
            FireballMoveState::Climbing { target_girder } => {
                let target_surface =
                    girder_surface_y(&stage.girders[target_girder], tf.translation.x);
                let target_y = target_surface + FIREBALL_RADIUS;

                if target_y > tf.translation.y {
                    tf.translation.y += climb_speed * dt;
                    if tf.translation.y >= target_y {
                        tf.translation.y = target_y;
                        fb.girder = target_girder;
                        fb.state = FireballMoveState::Patrolling;
                    }
                } else {
                    tf.translation.y -= climb_speed * dt;
                    if tf.translation.y <= target_y {
                        tf.translation.y = target_y;
                        fb.girder = target_girder;
                        fb.state = FireballMoveState::Patrolling;
                    }
                }
            }
        }
    }
}
