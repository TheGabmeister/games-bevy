use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::aabb_overlap;
use crate::components::*;
use crate::constants::*;
use crate::resources::ScoreEvent;
use crate::states::*;
use crate::ui;

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
    mut score_events: MessageWriter<ScoreEvent>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    assets: Res<GameAssets>,
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
            score_events.write(ScoreEvent { points: STOMP_SCORE });

            let shell_y = enemy_tf.translation.y - (KOOPA_HEIGHT - SHELL_HEIGHT) / 2.0;

            // Despawn Koopa (recursive, removes children too)
            commands.entity(entity).despawn();

            // Spawn Shell
            assets.shell.spawn(&mut commands, enemy_tf.translation.x, shell_y);

            ui::spawn_score_popup(
                &mut commands, STOMP_SCORE,
                enemy_tf.translation.x,
                enemy_tf.translation.y + KOOPA_HEIGHT / 2.0,
            );

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
    mut score_events: MessageWriter<ScoreEvent>,
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
                score_events.write(ScoreEvent { points: score });

                ui::spawn_score_popup(
                    &mut commands, score,
                    enemy_tf.translation.x,
                    enemy_tf.translation.y + 10.0,
                );
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
                score_events.write(ScoreEvent { points: score });

                ui::spawn_score_popup(
                    &mut commands, score,
                    enemy_tf.translation.x,
                    enemy_tf.translation.y + 10.0,
                );
            }
        }
    }
}
