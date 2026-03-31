use bevy::prelude::*;

use crate::collision::aabb_overlap;
use crate::components::*;
use crate::constants::*;
use crate::resources::GameData;
use crate::states::*;

use super::mario_take_damage;

// ── Mario ↔ Koopa Collision ──

pub fn mario_koopa_collision(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &mut Velocity, &Transform, &CollisionSize, &PlayerSize, Has<Invincible>),
        With<Player>,
    >,
    koopa_query: Query<
        (Entity, &Transform, &CollisionSize),
        (With<KoopaTroopa>, With<EnemyActive>, Without<Player>, Without<Goomba>, Without<Shell>),
    >,
    mut game_data: ResMut<GameData>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok((player_entity, mut player_vel, player_tf, player_coll, &player_size, is_invincible)) =
        player_query.single_mut()
    else {
        return;
    };

    if is_invincible {
        return;
    }

    let px = player_tf.translation.x;
    let py = player_tf.translation.y;
    let pvy = player_vel.y;

    for (entity, enemy_tf, enemy_coll) in &koopa_query {
        if aabb_overlap(
            px, py, player_coll.width / 2.0, player_coll.height / 2.0,
            enemy_tf.translation.x, enemy_tf.translation.y,
            enemy_coll.width / 2.0, enemy_coll.height / 2.0,
        ).is_none() {
            continue;
        }

        if py > enemy_tf.translation.y && pvy <= 0.0 {
            // Stomp Koopa → Shell
            player_vel.y = STOMP_BOUNCE_IMPULSE;
            game_data.score += STOMP_SCORE;

            let shell_y = enemy_tf.translation.y - (KOOPA_HEIGHT - SHELL_HEIGHT) / 2.0;

            // Despawn Koopa (recursive, removes children too)
            commands.entity(entity).despawn();

            // Spawn Shell
            let shell_mesh = meshes.add(Rectangle::new(SHELL_WIDTH, SHELL_HEIGHT));
            let shell_mat =
                materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.65, 0.2)));

            commands.spawn((
                Shell {
                    state: ShellState::Stationary,
                    chain_kills: 0,
                },
                EnemyWalker {
                    speed: 0.0,
                    direction: 1.0,
                },
                CollisionSize {
                    width: SHELL_WIDTH,
                    height: SHELL_HEIGHT,
                },
                Velocity::default(),
                Grounded(true),
                EnemyActive,
                Mesh2d(shell_mesh),
                MeshMaterial2d(shell_mat),
                Transform::from_xyz(enemy_tf.translation.x, shell_y, Z_ENEMY),
                DespawnOnExit(AppState::Playing),
            ));

            commands.spawn((
                ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
                Text2d::new(format!("+{STOMP_SCORE}")),
                TextFont { font_size: 8.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(
                    enemy_tf.translation.x,
                    enemy_tf.translation.y + KOOPA_HEIGHT / 2.0,
                    Z_PLAYER + 1.0,
                ),
                DespawnOnExit(AppState::Playing),
            ));

            return;
        } else {
            mario_take_damage(
                &mut commands,
                player_entity,
                player_size,
                &mut next_play_state,
            );
            return;
        }
    }
}

// ── Mario ↔ Shell Collision ──

pub fn mario_shell_collision(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &mut Velocity, &Transform, &CollisionSize, &PlayerSize, Has<Invincible>),
        With<Player>,
    >,
    mut shell_query: Query<
        (Entity, &Transform, &CollisionSize, &mut Shell, &mut Velocity, &mut EnemyWalker),
        (With<Shell>, Without<Player>, Without<Goomba>, Without<KoopaTroopa>),
    >,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    let Ok((player_entity, mut player_vel, player_tf, player_coll, &player_size, is_invincible)) =
        player_query.single_mut()
    else {
        return;
    };

    if is_invincible {
        return;
    }

    let px = player_tf.translation.x;
    let py = player_tf.translation.y;
    let pvy = player_vel.y;

    for (_entity, shell_tf, shell_coll, mut shell, mut shell_vel, mut walker) in &mut shell_query {
        if aabb_overlap(
            px, py, player_coll.width / 2.0, player_coll.height / 2.0,
            shell_tf.translation.x, shell_tf.translation.y,
            shell_coll.width / 2.0, shell_coll.height / 2.0,
        ).is_none() {
            continue;
        }

        match shell.state {
            ShellState::Stationary => {
                // Kick the shell
                let kick_dir = if px < shell_tf.translation.x {
                    1.0
                } else {
                    -1.0
                };
                shell.state = ShellState::Moving;
                shell.chain_kills = 0;
                walker.speed = SHELL_SPEED;
                walker.direction = kick_dir;
                return;
            }
            ShellState::Moving => {
                if py > shell_tf.translation.y && pvy <= 0.0 {
                    // Stomp moving shell → stop it
                    player_vel.y = STOMP_BOUNCE_IMPULSE;
                    shell.state = ShellState::Stationary;
                    shell.chain_kills = 0;
                    shell_vel.x = 0.0;
                    walker.speed = 0.0;
                    return;
                } else {
                    // Moving shell hits Mario
                    mario_take_damage(
                        &mut commands,
                        player_entity,
                        player_size,
                        &mut next_play_state,
                    );
                    return;
                }
            }
        }
    }
}

// ── Shell ↔ Enemy Collision ──

pub fn shell_enemy_collision(
    mut commands: Commands,
    mut shell_query: Query<
        (&Transform, &CollisionSize, &mut Shell),
        (With<Shell>, Without<Goomba>, Without<KoopaTroopa>),
    >,
    goomba_query: Query<
        (Entity, &Transform, &CollisionSize),
        (With<Goomba>, With<EnemyActive>, Without<Squished>),
    >,
    koopa_query: Query<
        (Entity, &Transform, &CollisionSize),
        (With<KoopaTroopa>, With<EnemyActive>),
    >,
    mut game_data: ResMut<GameData>,
) {
    for (shell_tf, shell_coll, mut shell) in &mut shell_query {
        if shell.state != ShellState::Moving {
            continue;
        }

        // Kill Goombas
        for (entity, enemy_tf, enemy_coll) in &goomba_query {
            if aabb_overlap(
                shell_tf.translation.x, shell_tf.translation.y,
                shell_coll.width / 2.0, shell_coll.height / 2.0,
                enemy_tf.translation.x, enemy_tf.translation.y,
                enemy_coll.width / 2.0, enemy_coll.height / 2.0,
            ).is_some() {
                commands.entity(entity).despawn();

                shell.chain_kills += 1;
                let score =
                    SHELL_BASE_SCORE * (1 << (shell.chain_kills - 1).min(5));
                game_data.score += score;

                commands.spawn((
                    ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
                    Text2d::new(format!("+{score}")),
                    TextFont { font_size: 8.0, ..default() },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(
                        enemy_tf.translation.x,
                        enemy_tf.translation.y + 10.0,
                        Z_PLAYER + 1.0,
                    ),
                    DespawnOnExit(AppState::Playing),
                ));
            }
        }

        // Kill Koopas
        for (entity, enemy_tf, enemy_coll) in &koopa_query {
            if aabb_overlap(
                shell_tf.translation.x, shell_tf.translation.y,
                shell_coll.width / 2.0, shell_coll.height / 2.0,
                enemy_tf.translation.x, enemy_tf.translation.y,
                enemy_coll.width / 2.0, enemy_coll.height / 2.0,
            ).is_some() {
                commands.entity(entity).despawn();

                shell.chain_kills += 1;
                let score =
                    SHELL_BASE_SCORE * (1 << (shell.chain_kills - 1).min(5));
                game_data.score += score;

                commands.spawn((
                    ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
                    Text2d::new(format!("+{score}")),
                    TextFont { font_size: 8.0, ..default() },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(
                        enemy_tf.translation.x,
                        enemy_tf.translation.y + 10.0,
                        Z_PLAYER + 1.0,
                    ),
                    DespawnOnExit(AppState::Playing),
                ));
            }
        }
    }
}
