use bevy::prelude::*;

use crate::collision::{self, WallAction};
use crate::components::*;
use crate::constants::*;
use crate::level::LevelGrid;
use crate::resources::*;
use crate::states::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            player_input.in_set(GameplaySet::Input),
        )
        .add_systems(
            Update,
            (apply_gravity, apply_velocity, tile_collision)
                .chain()
                .in_set(GameplaySet::Physics),
        )
        .add_systems(
            Update,
            check_pit_death.in_set(GameplaySet::Late),
        )
        .add_systems(OnEnter(PlayState::Dying), start_death_animation)
        .add_systems(
            Update,
            death_animation_system.run_if(in_state(PlayState::Dying)),
        );
    }
}

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut FacingDirection, &Grounded, Has<Ducking>), With<Player>>,
) {
    let Ok((mut vel, mut facing, grounded, is_ducking)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Ducking: decelerate and block horizontal input, but allow jumping
    if is_ducking {
        let decel = PLAYER_DECELERATION * dt;
        if vel.x.abs() < decel {
            vel.x = 0.0;
        } else {
            vel.x -= decel * vel.x.signum();
        }

        // Jump cut still works
        if (keyboard.just_released(KeyCode::Space) || keyboard.just_released(KeyCode::ArrowUp))
            && vel.y > 0.0
        {
            vel.y *= JUMP_CUT_MULTIPLIER;
        }
        return;
    }

    let mut dir = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dir -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dir += 1.0;
    }

    if dir != 0.0 {
        *facing = if dir < 0.0 {
            FacingDirection::Left
        } else {
            FacingDirection::Right
        };
    }

    let running = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let max_speed = if running { PLAYER_RUN_SPEED } else { PLAYER_WALK_SPEED };
    let accel = if grounded.0 { PLAYER_ACCELERATION } else { PLAYER_AIR_ACCELERATION };

    if dir != 0.0 {
        vel.x += dir * accel * dt;
        vel.x = vel.x.clamp(-max_speed, max_speed);
    } else if grounded.0 {
        let decel = PLAYER_DECELERATION * dt;
        if vel.x.abs() < decel {
            vel.x = 0.0;
        } else {
            vel.x -= decel * vel.x.signum();
        }
    }

    if (keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::ArrowUp))
        && grounded.0
    {
        vel.y = PLAYER_JUMP_IMPULSE;
    }

    if (keyboard.just_released(KeyCode::Space) || keyboard.just_released(KeyCode::ArrowUp))
        && vel.y > 0.0
    {
        vel.y *= JUMP_CUT_MULTIPLIER;
    }
}

fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Grounded), With<Player>>,
) {
    let Ok((mut vel, grounded)) = query.single_mut() else {
        return;
    };

    if grounded.0 {
        return;
    }

    let gravity = if vel.y > 0.0 {
        GRAVITY_ASCENDING
    } else {
        GRAVITY_DESCENDING
    };

    vel.y -= gravity * time.delta_secs();
    vel.y = vel.y.max(-TERMINAL_VELOCITY);
}

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<Player>>,
) {
    let Ok((vel, mut transform)) = query.single_mut() else {
        return;
    };

    transform.translation.x += vel.x * time.delta_secs();
    transform.translation.y += vel.y * time.delta_secs();
}

fn tile_collision(
    level: Res<LevelGrid>,
    mut pending_hit: ResMut<PendingBlockHit>,
    mut query: Query<
        (&mut Velocity, &mut Transform, &mut Grounded, &CollisionSize, &PlayerSize),
        With<Player>,
    >,
    camera_query: Query<&Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok((mut vel, mut transform, mut grounded, coll_size, player_size)) = query.single_mut()
    else {
        return;
    };

    let half_w = coll_size.width / 2.0;
    let half_h = coll_size.height / 2.0;

    // Clamp to left edge of camera
    if let Ok(camera_tf) = camera_query.single() {
        let camera_left = camera_tf.translation.x - CAMERA_VISIBLE_WIDTH / 2.0;
        let min_x = camera_left + half_w;
        if transform.translation.x < min_x {
            transform.translation.x = min_x;
            if vel.x < 0.0 {
                vel.x = 0.0;
            }
        }
    }

    let mut unused_dir = 0.0;
    let result = collision::resolve_tile_collisions(
        &level,
        &mut transform.translation,
        &mut vel,
        half_w,
        half_h,
        WallAction::Stop,
        &mut unused_dir,
    );

    grounded.0 = result.grounded;

    if let Some((col, row)) = result.head_hit {
        pending_hit.hit = Some(BlockHitInfo {
            col,
            row,
            is_big: *player_size == PlayerSize::Big,
        });
    }
}

fn check_pit_death(
    player_query: Query<&Transform, With<Player>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    if player_tf.translation.y < DEATH_Y {
        next_play_state.set(PlayState::Dying);
    }
}

fn start_death_animation(
    mut commands: Commands,
    mut player_query: Query<(&mut Velocity, &Transform), With<Player>>,
) {
    let Ok((mut vel, transform)) = player_query.single_mut() else { return };
    vel.x = 0.0;
    vel.y = 0.0;

    let pit_death = transform.translation.y < DEATH_Y;

    commands.insert_resource(DeathAnimation {
        phase: DeathPhase::Pause,
        timer: Timer::from_seconds(
            if pit_death { 1.0 } else { DEATH_PAUSE_DURATION },
            TimerMode::Once,
        ),
        pit_death,
    });
}

fn death_animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut death_anim: ResMut<DeathAnimation>,
    mut player_query: Query<
        (&mut Velocity, &mut Transform, &mut Grounded, &mut CollisionSize, &mut PlayerSize, &mut Mesh2d),
        With<Player>,
    >,
    mut game_data: ResMut<GameData>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    spawn_point: Res<SpawnPoint>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player_meshes: Option<Res<PlayerMeshes>>,
) {
    death_anim.timer.tick(time.delta());

    let Ok((mut vel, mut player_tf, mut grounded, mut coll_size, mut player_size, mut mesh)) =
        player_query.single_mut()
    else {
        return;
    };

    let mut animation_complete = false;

    match death_anim.phase {
        DeathPhase::Pause => {
            if death_anim.timer.is_finished() {
                if death_anim.pit_death {
                    animation_complete = true;
                } else {
                    death_anim.phase = DeathPhase::Bouncing;
                    death_anim.timer =
                        Timer::from_seconds(DEATH_FALL_DURATION, TimerMode::Once);
                    vel.y = DEATH_BOUNCE_IMPULSE;
                }
            }
        }
        DeathPhase::Bouncing => {
            let gravity = if vel.y > 0.0 {
                GRAVITY_ASCENDING
            } else {
                GRAVITY_DESCENDING
            };
            vel.y -= gravity * time.delta_secs();
            vel.y = vel.y.max(-TERMINAL_VELOCITY);
            player_tf.translation.y += vel.y * time.delta_secs();

            if death_anim.timer.is_finished() {
                animation_complete = true;
            }
        }
    }

    if animation_complete {
        commands.remove_resource::<DeathAnimation>();
        game_data.lives = game_data.lives.saturating_sub(1);

        if game_data.lives == 0 {
            next_app_state.set(AppState::GameOver);
        } else {
            player_tf.translation.x = spawn_point.x;
            player_tf.translation.y = spawn_point.y;
            vel.x = 0.0;
            vel.y = 0.0;
            grounded.0 = false;
            game_data.timer = TIMER_START;

            // Reset to small Mario
            *player_size = PlayerSize::Small;
            coll_size.height = PLAYER_SMALL_HEIGHT;
            if let Some(meshes) = player_meshes {
                mesh.0 = meshes.small.clone();
            }

            if let Ok(mut camera_tf) = camera_query.single_mut() {
                camera_tf.translation.x = CAMERA_MIN_X;
            }

            next_play_state.set(PlayState::Running);
        }
    }
}
