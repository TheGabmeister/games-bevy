use bevy::prelude::*;

use crate::collision::entities_overlap;
use crate::components::*;
use crate::constants::*;
use crate::level::SpawnerRegistry;
use crate::resources::ScoreEvent;
use crate::states::*;
use crate::ui;

use super::mario_take_damage;

// ── Shell Assets (Resource — accessed by koopa.rs and powerup.rs) ──

#[derive(Resource, Clone)]
pub struct ShellAssets {
    mesh: Handle<Mesh>,
    mat: Handle<ColorMaterial>,
}

impl ShellAssets {
    pub fn spawn(&self, commands: &mut Commands, x: f32, y: f32) -> Entity {
        commands
            .spawn((
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
                Mesh2d(self.mesh.clone()),
                MeshMaterial2d(self.mat.clone()),
                Transform::from_xyz(x, y, Z_ENEMY),
                DespawnOnExit(AppState::Playing),
            ))
            .id()
    }
}

// ── Plugin ──

pub struct KoopaPlugin;

impl Plugin for KoopaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_koopa)
            .add_systems(
                Update,
                (
                    mario_koopa_collision,
                    mario_shell_collision,
                    shell_enemy_collision,
                )
                    .in_set(GameplaySet::Late),
            );
    }
}

/// Create koopa + shell mesh/material handles, register the 'K' level spawner.
fn init_koopa(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut registry: ResMut<SpawnerRegistry>,
) {
    // Shell assets — kept as a Resource for runtime spawning (stomp + fireball)
    commands.insert_resource(ShellAssets {
        mesh: meshes.add(Rectangle::new(SHELL_WIDTH, SHELL_HEIGHT)),
        mat: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.65, 0.2))),
    });

    // Koopa visual handles — captured in the spawner closure
    let body_mesh = meshes.add(Rectangle::new(KOOPA_WIDTH, 16.0));
    let body_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.7, 0.2)));
    let head_mesh = meshes.add(Circle::new(5.0));
    let head_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.3, 0.8, 0.3)));

    registry.register(
        'K',
        Box::new(move |commands, wx, wy, _, _| {
            commands
                .spawn((
                    KoopaTroopa,
                    EnemyWalker {
                        speed: KOOPA_SPEED,
                        direction: -1.0,
                    },
                    CollisionSize {
                        width: KOOPA_WIDTH,
                        height: KOOPA_HEIGHT,
                    },
                    Velocity::default(),
                    Grounded::default(),
                    Mesh2d(body_mesh.clone()),
                    MeshMaterial2d(body_mat.clone()),
                    Transform::from_xyz(wx, wy, Z_ENEMY),
                    DespawnOnExit(AppState::Playing),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh2d(head_mesh.clone()),
                        MeshMaterial2d(head_mat.clone()),
                        Transform::from_xyz(0.0, 11.0, 0.0),
                    ));
                });
        }),
    );
}

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
    shell_assets: Res<ShellAssets>,
) {
    let Ok((player_entity, mut player_vel, player_tf, player_coll, &player_size, is_invincible)) =
        player_query.single_mut()
    else {
        return;
    };

    if is_invincible {
        return;
    }

    for (entity, enemy_tf, enemy_coll) in &koopa_query {
        if !entities_overlap(player_tf, player_coll, enemy_tf, enemy_coll) {
            continue;
        }

        if player_tf.translation.y > enemy_tf.translation.y && player_vel.y <= 0.0 {
            // Stomp Koopa → Shell
            player_vel.y = STOMP_BOUNCE_IMPULSE;
            score_events.write(ScoreEvent { points: STOMP_SCORE });

            let shell_y = enemy_tf.translation.y - (KOOPA_HEIGHT - SHELL_HEIGHT) / 2.0;

            commands.entity(entity).despawn();
            shell_assets.spawn(&mut commands, enemy_tf.translation.x, shell_y);

            ui::spawn_score_popup(
                &mut commands,
                STOMP_SCORE,
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

    for (_entity, shell_tf, shell_coll, mut shell, mut shell_vel, mut walker) in &mut shell_query {
        if !entities_overlap(player_tf, player_coll, &shell_tf, &shell_coll) {
            continue;
        }

        match shell.state {
            ShellState::Stationary => {
                let kick_dir = if player_tf.translation.x < shell_tf.translation.x {
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
                if player_tf.translation.y > shell_tf.translation.y && player_vel.y <= 0.0 {
                    player_vel.y = STOMP_BOUNCE_IMPULSE;
                    shell.state = ShellState::Stationary;
                    shell.chain_kills = 0;
                    shell_vel.x = 0.0;
                    walker.speed = 0.0;
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

        for (entity, enemy_tf, enemy_coll) in &goomba_query {
            if entities_overlap(&shell_tf, &shell_coll, enemy_tf, enemy_coll) {
                commands.entity(entity).despawn();

                shell.chain_kills += 1;
                let score = SHELL_BASE_SCORE * (1 << (shell.chain_kills - 1).min(5));
                score_events.write(ScoreEvent { points: score });

                ui::spawn_score_popup(
                    &mut commands,
                    score,
                    enemy_tf.translation.x,
                    enemy_tf.translation.y + 10.0,
                );
            }
        }

        for (entity, enemy_tf, enemy_coll) in &koopa_query {
            if entities_overlap(&shell_tf, &shell_coll, enemy_tf, enemy_coll) {
                commands.entity(entity).despawn();

                shell.chain_kills += 1;
                let score = SHELL_BASE_SCORE * (1 << (shell.chain_kills - 1).min(5));
                score_events.write(ScoreEvent { points: score });

                ui::spawn_score_popup(
                    &mut commands,
                    score,
                    enemy_tf.translation.x,
                    enemy_tf.translation.y + 10.0,
                );
            }
        }
    }
}
