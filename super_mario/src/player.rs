use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::{self, WallAction};
use crate::components::*;
use crate::constants::*;
use crate::level::{LevelGrid, tile_to_world, world_to_col, world_to_row};
use crate::resources::*;
use crate::states::*;
use crate::ui;

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
            (check_pit_death, flagpole_collision)
                .in_set(GameplaySet::Late),
        )
        .add_systems(OnEnter(PlayState::Dying), start_death_animation)
        .add_systems(
            Update,
            death_animation_system.run_if(in_state(PlayState::Dying)),
        )
        .add_systems(
            Update,
            level_complete_system.run_if(in_state(PlayState::LevelComplete)),
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
            player_size: *player_size,
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

// ── Flagpole Collision ──

fn flagpole_collision(
    mut commands: Commands,
    level: Res<LevelGrid>,
    player_query: Query<(Entity, &Transform, &CollisionSize), With<Player>>,
    mut score_events: MessageWriter<ScoreEvent>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    let Ok((player_entity, player_tf, player_coll)) = player_query.single() else { return };

    let half_w = player_coll.width / 2.0;
    let half_h = player_coll.height / 2.0;

    let col_min = world_to_col(player_tf.translation.x - half_w);
    let col_max = world_to_col(player_tf.translation.x + half_w);
    let row_min = world_to_row(player_tf.translation.y + half_h);
    let row_max = world_to_row(player_tf.translation.y - half_h);

    for row in row_min..=row_max {
        for col in col_min..=col_max {
            if level.get_char(col, row) != 'F' {
                continue;
            }

            // Mario touched the flagpole!
            let contact_row = world_to_row(player_tf.translation.y)
                .clamp(FLAGPOLE_TOP_ROW as i32, FLAGPOLE_BOTTOM_ROW as i32)
                as usize;

            let score_range = FLAGPOLE_TOP_SCORE - FLAGPOLE_BOTTOM_SCORE;
            let row_range = (FLAGPOLE_BOTTOM_ROW - FLAGPOLE_TOP_ROW) as u32;
            let rows_from_bottom = (FLAGPOLE_BOTTOM_ROW - contact_row) as u32;
            let flagpole_score =
                FLAGPOLE_BOTTOM_SCORE + score_range * rows_from_bottom / row_range;

            score_events.write(ScoreEvent { points: flagpole_score });

            // Score popup
            ui::spawn_score_popup(
                &mut commands, flagpole_score,
                player_tf.translation.x + 15.0,
                player_tf.translation.y,
            );

            // Compute animation data
            let pole_col = col;
            let (pole_x, _) = tile_to_world(pole_col as usize, FLAGPOLE_TOP_ROW);
            let (_, ground_y) = tile_to_world(pole_col as usize, 13);
            let pole_base_y = ground_y + TILE_SIZE / 2.0 + player_coll.height / 2.0;

            let castle_col = pole_col as usize + CASTLE_OFFSET_TILES;
            let (castle_x, _) = tile_to_world(castle_col, 12);

            commands.insert_resource(LevelCompleteAnimation {
                phase: LevelCompletePhase::SlideDown,
                pole_x,
                pole_base_y,
                castle_x,
                done_timer: Timer::from_seconds(LEVEL_COMPLETE_DONE_DELAY, TimerMode::Once),
                flagpole_score,
            });

            // Clear invincibility so the player is fully visible
            commands.entity(player_entity).remove::<Invincible>();
            commands.entity(player_entity).insert(Visibility::Inherited);

            next_play_state.set(PlayState::LevelComplete);
            return;
        }
    }
}

// ── Level Complete Animation ──

fn level_complete_system(
    time: Res<Time>,
    mut commands: Commands,
    mut anim: ResMut<LevelCompleteAnimation>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    mut flag_query: Query<&mut Transform, (With<FlagpoleFlag>, Without<Player>)>,
    mut game_timer: ResMut<GameTimer>,
    mut score_events: MessageWriter<ScoreEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let Ok((mut player_tf, mut vel)) = player_query.single_mut() else { return };
    let dt = time.delta_secs();

    vel.x = 0.0;
    vel.y = 0.0;

    match anim.phase {
        LevelCompletePhase::SlideDown => {
            player_tf.translation.x = anim.pole_x;
            player_tf.translation.y -= FLAGPOLE_SLIDE_SPEED * dt;

            // Slide the flag down too
            if let Ok(mut flag_tf) = flag_query.single_mut() {
                flag_tf.translation.y = player_tf.translation.y;
            }

            if player_tf.translation.y <= anim.pole_base_y {
                player_tf.translation.y = anim.pole_base_y;
                anim.phase = LevelCompletePhase::WalkToCastle;
            }
        }
        LevelCompletePhase::WalkToCastle => {
            player_tf.translation.x += FLAGPOLE_WALK_SPEED * dt;

            if player_tf.translation.x >= anim.castle_x {
                player_tf.translation.x = anim.castle_x;
                anim.phase = LevelCompletePhase::TimeTally;
            }
        }
        LevelCompletePhase::TimeTally => {
            if game_timer.time > 0.0 {
                let ticks = (TIME_TALLY_RATE * dt).ceil().min(game_timer.time) as u32;
                game_timer.time -= ticks as f32;
                score_events.write(ScoreEvent { points: ticks * TIME_BONUS_PER_TICK });
            } else {
                game_timer.time = 0.0;
                anim.phase = LevelCompletePhase::Done;
            }
        }
        LevelCompletePhase::Done => {
            anim.done_timer.tick(time.delta());
            if anim.done_timer.is_finished() {
                commands.remove_resource::<LevelCompleteAnimation>();
                next_app_state.set(AppState::StartScreen);
            }
        }
    }
}

// ── Death ──

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
        (
            &mut Velocity,
            &mut Transform,
            &mut Grounded,
            &mut CollisionSize,
            &mut PlayerSize,
            &mut Mesh2d,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        With<Player>,
    >,
    mut game_data: ResMut<GameData>,
    mut game_timer: ResMut<GameTimer>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    spawn_point: Res<SpawnPoint>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    assets: Res<GameAssets>,
) {
    death_anim.timer.tick(time.delta());

    let Ok((mut vel, mut player_tf, mut grounded, mut coll_size, mut player_size, mut mesh, mut mat)) =
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
            game_timer.time = TIMER_START;

            // Reset to small Mario with normal appearance
            *player_size = PlayerSize::Small;
            coll_size.height = PLAYER_SMALL_HEIGHT;
            mesh.0 = assets.player.small_mesh.clone();
            mat.0 = assets.player.normal_mat.clone();

            if let Ok(mut camera_tf) = camera_query.single_mut() {
                camera_tf.translation.x = CAMERA_MIN_X;
            }

            next_play_state.set(PlayState::Running);
        }
    }
}
