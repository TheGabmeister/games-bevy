use bevy::prelude::*;

use crate::collision::{self, WallAction, aabb_overlap};
use crate::components::*;
use crate::constants::*;
use crate::level::LevelGrid;
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
            (mario_enemy_collision, shell_enemy_collision, enemy_despawn_out_of_bounds)
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
        (&mut Velocity, &mut Transform, &mut Grounded, &mut EnemyWalker, &CollisionSize),
        (With<EnemyActive>, Without<Squished>),
    >,
) {
    for (mut vel, mut transform, mut grounded, mut walker, coll_size) in &mut query {
        let result = collision::resolve_tile_collisions(
            &level,
            &mut transform.translation,
            &mut vel,
            coll_size.width / 2.0,
            coll_size.height / 2.0,
            WallAction::Reverse,
            &mut walker.direction,
        );
        grounded.0 = result.grounded;
    }
}

// ── Mario ↔ Enemy Collision ──

fn mario_enemy_collision(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &mut Velocity, &Transform, &CollisionSize, &PlayerSize, Has<Invincible>),
        With<Player>,
    >,
    mut goomba_query: Query<
        (Entity, &mut Transform, &mut Velocity, &CollisionSize),
        (With<Goomba>, With<EnemyActive>, Without<Squished>, Without<Player>, Without<KoopaTroopa>, Without<Shell>),
    >,
    koopa_query: Query<
        (Entity, &Transform, &CollisionSize),
        (With<KoopaTroopa>, With<EnemyActive>, Without<Player>, Without<Goomba>, Without<Shell>),
    >,
    mut shell_query: Query<
        (Entity, &Transform, &CollisionSize, &mut Shell, &mut Velocity, &mut EnemyWalker),
        (With<Shell>, Without<Player>, Without<Goomba>, Without<KoopaTroopa>),
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

    // --- Goombas ---
    for (entity, mut enemy_tf, mut enemy_vel, enemy_coll) in &mut goomba_query {
        if aabb_overlap(
            px, py, player_coll.width / 2.0, player_coll.height / 2.0,
            enemy_tf.translation.x, enemy_tf.translation.y,
            enemy_coll.width / 2.0, enemy_coll.height / 2.0,
        ).is_none() {
            continue;
        }

        if py > enemy_tf.translation.y && pvy <= 0.0 {
            // Stomp
            player_vel.y = STOMP_BOUNCE_IMPULSE;
            game_data.score += STOMP_SCORE;

            enemy_tf.scale.y = 0.3;
            enemy_vel.x = 0.0;
            enemy_vel.y = 0.0;
            commands
                .entity(entity)
                .insert(Squished(Timer::from_seconds(SQUISH_DURATION, TimerMode::Once)))
                .remove::<EnemyWalker>();

            commands.spawn((
                ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
                Text2d::new(format!("+{STOMP_SCORE}")),
                TextFont { font_size: 8.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(
                    enemy_tf.translation.x,
                    enemy_tf.translation.y + GOOMBA_HEIGHT,
                    Z_PLAYER + 1.0,
                ),
                DespawnOnExit(AppState::Playing),
            ));

            return;
        } else {
            // Damage
            mario_take_damage(
                &mut commands,
                player_entity,
                player_size,
                &mut next_play_state,
            );
            return;
        }
    }

    // --- Koopas ---
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

            let shell_y =
                enemy_tf.translation.y - (KOOPA_HEIGHT - SHELL_HEIGHT) / 2.0;

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

    // --- Shells ---
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

fn mario_take_damage(
    commands: &mut Commands,
    player_entity: Entity,
    player_size: PlayerSize,
    next_play_state: &mut ResMut<NextState<PlayState>>,
) {
    if player_size == PlayerSize::Big {
        commands.entity(player_entity).insert(GrowthAnimation {
            timer: Timer::from_seconds(GROWTH_DURATION, TimerMode::Once),
            flash_timer: Timer::from_seconds(GROWTH_FLASH_INTERVAL, TimerMode::Repeating),
            growing: false,
        });
        next_play_state.set(PlayState::Growing);
    } else {
        next_play_state.set(PlayState::Dying);
    }
}

// ── Shell ↔ Enemy Collision ──

fn shell_enemy_collision(
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
                    ScorePopup(Timer::from_seconds(
                        SCORE_POPUP_DURATION,
                        TimerMode::Once,
                    )),
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
                    ScorePopup(Timer::from_seconds(
                        SCORE_POPUP_DURATION,
                        TimerMode::Once,
                    )),
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
