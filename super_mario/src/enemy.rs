use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::level::{LevelGrid, tile_to_world, world_to_col, world_to_row};
use crate::resources::GameData;
use crate::states::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            enemy_activation.in_set(GameplaySet::Input),
        )
        .add_systems(
            Update,
            (enemy_walk, enemy_gravity, enemy_apply_velocity, enemy_tile_collision)
                .chain()
                .in_set(GameplaySet::Physics),
        )
        .add_systems(
            Update,
            (mario_enemy_collision, enemy_despawn_out_of_bounds)
                .in_set(GameplaySet::Late),
        )
        .add_systems(
            Update,
            (squish_timer, score_popup_system)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

// ── Activation ──

fn enemy_activation(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera2d>>,
    query: Query<(Entity, &Transform), (With<EnemyWalker>, Without<EnemyActive>)>,
) {
    let Ok(camera_tf) = camera_query.single() else { return };
    let activate_x = camera_tf.translation.x + CAMERA_VISIBLE_WIDTH / 2.0 + TILE_SIZE;

    for (entity, transform) in &query {
        if transform.translation.x <= activate_x {
            commands.entity(entity).insert(EnemyActive);
        }
    }
}

// ── Enemy Physics ──

fn enemy_walk(
    mut query: Query<
        (&mut Velocity, &EnemyWalker),
        (With<EnemyActive>, Without<Squished>),
    >,
) {
    for (mut vel, walker) in &mut query {
        vel.x = walker.speed * walker.direction;
    }
}

fn enemy_gravity(
    time: Res<Time>,
    mut query: Query<
        (&mut Velocity, &Grounded),
        (With<EnemyActive>, Without<Squished>, Without<Player>),
    >,
) {
    for (mut vel, grounded) in &mut query {
        if grounded.0 {
            continue;
        }
        vel.y -= GRAVITY_DESCENDING * time.delta_secs();
        vel.y = vel.y.max(-TERMINAL_VELOCITY);
    }
}

fn enemy_apply_velocity(
    time: Res<Time>,
    mut query: Query<
        (&Velocity, &mut Transform),
        (With<EnemyActive>, Without<Squished>, Without<Player>),
    >,
) {
    for (vel, mut transform) in &mut query {
        transform.translation.x += vel.x * time.delta_secs();
        transform.translation.y += vel.y * time.delta_secs();
    }
}

fn enemy_tile_collision(
    level: Res<LevelGrid>,
    mut query: Query<
        (&mut Velocity, &mut Transform, &mut Grounded, &mut EnemyWalker),
        (With<EnemyActive>, Without<Squished>),
    >,
) {
    let half_w = GOOMBA_WIDTH / 2.0;
    let half_h = GOOMBA_HEIGHT / 2.0;
    let tile_half = TILE_SIZE / 2.0;

    for (mut vel, mut transform, mut grounded, mut walker) in &mut query {
        let col_min = world_to_col(transform.translation.x - half_w) - 1;
        let col_max = world_to_col(transform.translation.x + half_w) + 1;
        let row_min = world_to_row(transform.translation.y + half_h) - 1;
        let row_max = world_to_row(transform.translation.y - half_h) + 1;

        for row in row_min..=row_max {
            for col in col_min..=col_max {
                if !level.is_solid(col, row) {
                    continue;
                }

                let (tile_cx, tile_cy) = tile_to_world(col as usize, row as usize);
                let px = transform.translation.x;
                let py = transform.translation.y;

                let overlap_x = (half_w + tile_half) - (px - tile_cx).abs();
                let overlap_y = (half_h + tile_half) - (py - tile_cy).abs();

                if overlap_x <= 0.0 || overlap_y <= 0.0 {
                    continue;
                }

                if overlap_y < overlap_x {
                    if py > tile_cy {
                        transform.translation.y += overlap_y;
                        if vel.y < 0.0 {
                            vel.y = 0.0;
                        }
                    } else {
                        transform.translation.y -= overlap_y;
                        if vel.y > 0.0 {
                            vel.y = 0.0;
                        }
                    }
                } else {
                    walker.direction = -walker.direction;
                    if px > tile_cx {
                        transform.translation.x += overlap_x;
                    } else {
                        transform.translation.x -= overlap_x;
                    }
                    vel.x = 0.0;
                }
            }
        }

        // Grounded probe
        let probe_y = transform.translation.y - half_h - 1.0;
        let probe_row = world_to_row(probe_y);
        let left_col = world_to_col(transform.translation.x - half_w + 1.0);
        let right_col = world_to_col(transform.translation.x + half_w - 1.0);
        grounded.0 = level.is_solid(left_col, probe_row) || level.is_solid(right_col, probe_row);
    }
}

// ── Mario ↔ Enemy Collision ──

fn mario_enemy_collision(
    mut commands: Commands,
    mut player_query: Query<(&mut Velocity, &Transform), With<Player>>,
    mut enemy_query: Query<
        (Entity, &mut Transform, &mut Velocity),
        (With<Goomba>, With<EnemyActive>, Without<Squished>, Without<Player>),
    >,
    mut game_data: ResMut<GameData>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    let Ok((mut player_vel, player_tf)) = player_query.single_mut() else {
        return;
    };

    let p_half_w = PLAYER_WIDTH / 2.0;
    let p_half_h = PLAYER_SMALL_HEIGHT / 2.0;
    let g_half_w = GOOMBA_WIDTH / 2.0;
    let g_half_h = GOOMBA_HEIGHT / 2.0;

    for (entity, mut enemy_tf, mut enemy_vel) in &mut enemy_query {
        let overlap_x =
            (p_half_w + g_half_w) - (player_tf.translation.x - enemy_tf.translation.x).abs();
        let overlap_y =
            (p_half_h + g_half_h) - (player_tf.translation.y - enemy_tf.translation.y).abs();

        if overlap_x <= 0.0 || overlap_y <= 0.0 {
            continue;
        }

        let is_stomp =
            player_tf.translation.y > enemy_tf.translation.y && player_vel.y <= 0.0;

        if is_stomp {
            // Mario bounces
            player_vel.y = STOMP_BOUNCE_IMPULSE;
            game_data.score += STOMP_SCORE;

            // Squish the Goomba
            enemy_tf.scale.y = 0.3;
            enemy_vel.x = 0.0;
            enemy_vel.y = 0.0;
            commands
                .entity(entity)
                .insert(Squished(Timer::from_seconds(
                    SQUISH_DURATION,
                    TimerMode::Once,
                )))
                .remove::<EnemyWalker>();

            // Score popup
            let popup_pos = enemy_tf.translation;
            commands.spawn((
                ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
                Text2d::new(format!("+{STOMP_SCORE}")),
                TextFont {
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(popup_pos.x, popup_pos.y + GOOMBA_HEIGHT, Z_PLAYER + 1.0),
                DespawnOnExit(AppState::Playing),
            ));

            break;
        } else {
            // Mario dies
            next_play_state.set(PlayState::Dying);
            return;
        }
    }
}

// ── Cleanup ──

fn enemy_despawn_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<EnemyActive>, Without<Player>)>,
) {
    for (entity, transform) in &query {
        if transform.translation.y < DEATH_Y {
            commands.entity(entity).despawn();
        }
    }
}

fn squish_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Squished)>,
) {
    for (entity, mut squished) in &mut query {
        squished.0.tick(time.delta());
        if squished.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn score_popup_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScorePopup, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut popup, mut transform, mut color) in &mut query {
        popup.0.tick(time.delta());
        transform.translation.y += SCORE_POPUP_RISE_SPEED * time.delta_secs();

        let alpha = 1.0 - popup.0.fraction();
        color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);

        if popup.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
