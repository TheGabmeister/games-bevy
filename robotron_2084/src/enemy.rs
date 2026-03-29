#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::{GameAssets, GameState};
use crate::states::*;
use crate::waves::wave_definition;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                grunt_ai,
                hulk_ai,
                brain_ai,
                prog_ai,
                spawner_wander,
                enforcer_ai,
                tank_ai,
                spawner_spawn,
                enforcer_fire,
                tank_fire,
                homing_missile_steer,
                bounce_shell_reflect,
            )
                .in_set(GameSet::Movement),
        );
    }
}

fn grunt_ai(
    player_q: Query<&Transform, With<Player>>,
    mut grunt_q: Query<(&Transform, &mut Velocity, &GruntSteerOffset), With<Grunt>>,
    game: Res<GameState>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let speed_mult = wave_definition(game.current_wave).speed_mult;
    for (tf, mut vel, offset) in &mut grunt_q {
        let pos = tf.translation.truncate();
        let dir = (player_pos - pos).normalize_or_zero();
        let angle = dir.y.atan2(dir.x) + offset.0;
        vel.0 = Vec2::new(angle.cos(), angle.sin()) * GRUNT_BASE_SPEED * speed_mult;
    }
}

fn hulk_ai(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut hulk_q: Query<
        (
            &Transform,
            &mut Velocity,
            &mut WanderTimer,
            &mut WanderTarget,
            &mut Knockback,
        ),
        With<Hulk>,
    >,
) {
    let player_pos = player_q
        .single()
        .map(|tf| tf.translation.truncate())
        .unwrap_or(Vec2::ZERO);
    let mut rng = rand::rng();
    let dt = time.delta_secs();
    for (tf, mut vel, mut timer, mut target, mut kb) in &mut hulk_q {
        timer.0.tick(time.delta());
        if target.0 == Vec2::ZERO || timer.0.just_finished() {
            let pos = tf.translation.truncate();
            let bias = (player_pos - pos).normalize_or_zero() * 0.3;
            let random_dir = Vec2::new(
                rng.random_range(-1.0f32..1.0),
                rng.random_range(-1.0f32..1.0),
            )
            .normalize_or_zero();
            target.0 = (random_dir * 0.7 + bias).normalize_or_zero();
        }
        kb.0 *= (-HULK_KNOCKBACK_DECAY * dt).exp();
        if kb.0.length() < 1.0 {
            kb.0 = Vec2::ZERO;
        }
        vel.0 = target.0 * HULK_SPEED + kb.0;
    }
}

fn brain_ai(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_q: Query<&Transform, With<Player>>,
    human_q: Query<&Transform, With<Human>>,
    mut brain_q: Query<(&Transform, &mut Velocity, &mut FireCooldown), With<Brain>>,
) {
    let player_pos = player_q
        .single()
        .map(|tf| tf.translation.truncate())
        .unwrap_or(Vec2::ZERO);
    for (tf, mut vel, mut cooldown) in &mut brain_q {
        let pos = tf.translation.truncate();
        // Seek nearest human, fall back to player
        let target = human_q
            .iter()
            .map(|h| h.translation.truncate())
            .min_by(|a, b| {
                a.distance_squared(pos)
                    .partial_cmp(&b.distance_squared(pos))
                    .unwrap()
            })
            .unwrap_or(player_pos);
        let dir = (target - pos).normalize_or_zero();
        vel.0 = dir * BRAIN_SPEED;

        // Fire homing missiles
        cooldown.0.tick(time.delta());
        if cooldown.0.just_finished() {
            let missile_dir = (player_pos - pos).normalize_or_zero();
            commands.spawn((
                EnemyProjectile,
                DamagesPlayer,
                WaveEntity,
                Mesh2d(assets.missile_mesh.clone()),
                MeshMaterial2d(assets.missile_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 0.5),
                Velocity(missile_dir * MISSILE_SPEED),
                CollisionRadius(MISSILE_RADIUS),
                HomingMissile {
                    turn_rate: MISSILE_TURN_RATE,
                },
                Lifetime(Timer::from_seconds(MISSILE_LIFETIME, TimerMode::Once)),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

fn prog_ai(
    player_q: Query<&Transform, With<Player>>,
    mut prog_q: Query<(&Transform, &mut Velocity), With<Prog>>,
    game: Res<GameState>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let speed_mult = wave_definition(game.current_wave).speed_mult;
    for (tf, mut vel) in &mut prog_q {
        let dir = (player_pos - tf.translation.truncate()).normalize_or_zero();
        vel.0 = dir * PROG_SPEED * speed_mult;
    }
}

fn spawner_wander(
    time: Res<Time>,
    mut query: Query<
        (
            &Transform,
            &mut Velocity,
            &mut WanderTimer,
            &mut WanderTarget,
            Option<&Spheroid>,
        ),
        Or<(With<Spheroid>, With<Quark>)>,
    >,
) {
    let mut rng = rand::rng();
    for (tf, mut vel, mut timer, mut target, is_spheroid) in &mut query {
        timer.0.tick(time.delta());
        if target.0 == Vec2::ZERO || timer.0.just_finished() {
            let tx = rng.random_range(-ARENA_HALF_WIDTH * 0.8..ARENA_HALF_WIDTH * 0.8);
            let ty = rng.random_range(-ARENA_HALF_HEIGHT * 0.8..ARENA_HALF_HEIGHT * 0.8);
            target.0 = Vec2::new(tx, ty);
        }
        let pos = tf.translation.truncate();
        let dir = (target.0 - pos).normalize_or_zero();
        let speed = if is_spheroid.is_some() {
            SPHEROID_SPEED
        } else {
            QUARK_SPEED
        };
        vel.0 = dir * speed;
    }
}

fn spawner_spawn(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut spheroid_q: Query<(&Transform, &mut SpawnerState), With<Spheroid>>,
    mut quark_q: Query<(&Transform, &mut SpawnerState), (With<Quark>, Without<Spheroid>)>,
    enemy_count: Query<(), With<Enemy>>,
) {
    let active_enemies = enemy_count.iter().count() as u32;
    let mut remaining_capacity = MAX_TOTAL_ENEMIES.saturating_sub(active_enemies);

    // Spheroids spawn Enforcers
    for (tf, mut state) in &mut spheroid_q {
        if remaining_capacity == 0 {
            break;
        }
        state.cooldown.tick(time.delta());
        if state.cooldown.just_finished() && state.children_spawned < state.max_children {
            state.children_spawned += 1;
            remaining_capacity -= 1;
            let pos = tf.translation.truncate();
            commands
                .spawn((
                    Enemy,
                    Enforcer,
                    Killable,
                    DamagesPlayer,
                    Confined,
                    WaveEntity,
                    Mesh2d(assets.enforcer_mesh.clone()),
                    MeshMaterial2d(assets.enforcer_material.clone()),
                    Transform::from_xyz(pos.x, pos.y, 1.0),
                    Velocity(Vec2::ZERO),
                    CollisionRadius(ENFORCER_RADIUS),
                    PointValue(150),
                    DespawnOnExit(AppState::Playing),
                ))
                .insert((
                    FireCooldown(Timer::from_seconds(
                        ENFORCER_FIRE_COOLDOWN,
                        TimerMode::Repeating,
                    )),
                    WanderTarget(Vec2::ZERO),
                    WanderTimer(Timer::from_seconds(2.0, TimerMode::Repeating)),
                ));
        }
    }

    // Quarks spawn Tanks
    for (tf, mut state) in &mut quark_q {
        if remaining_capacity == 0 {
            break;
        }
        state.cooldown.tick(time.delta());
        if state.cooldown.just_finished() && state.children_spawned < state.max_children {
            state.children_spawned += 1;
            remaining_capacity -= 1;
            let pos = tf.translation.truncate();
            commands
                .spawn((
                    Enemy,
                    Tank,
                    Killable,
                    DamagesPlayer,
                    Confined,
                    WaveEntity,
                    Mesh2d(assets.tank_mesh.clone()),
                    MeshMaterial2d(assets.tank_material.clone()),
                    Transform::from_xyz(pos.x, pos.y, 1.0),
                    Velocity(Vec2::ZERO),
                    CollisionRadius(TANK_RADIUS),
                    PointValue(200),
                    DespawnOnExit(AppState::Playing),
                ))
                .insert((
                    FireCooldown(Timer::from_seconds(
                        TANK_FIRE_COOLDOWN,
                        TimerMode::Repeating,
                    )),
                    WanderTarget(Vec2::ZERO),
                    WanderTimer(Timer::from_seconds(3.0, TimerMode::Repeating)),
                ));
        }
    }
}

fn enforcer_ai(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut query: Query<
        (
            &Transform,
            &mut Velocity,
            &mut WanderTimer,
            &mut WanderTarget,
        ),
        With<Enforcer>,
    >,
) {
    let player_pos = player_q
        .single()
        .map(|tf| tf.translation.truncate())
        .unwrap_or(Vec2::ZERO);
    let mut rng = rand::rng();
    for (tf, mut vel, mut timer, mut target) in &mut query {
        timer.0.tick(time.delta());
        if target.0 == Vec2::ZERO || timer.0.just_finished() {
            let pos = tf.translation.truncate();
            let bias = (player_pos - pos).normalize_or_zero() * 0.3;
            let rand_dir = Vec2::new(
                rng.random_range(-1.0f32..1.0),
                rng.random_range(-1.0f32..1.0),
            )
            .normalize_or_zero();
            target.0 = (rand_dir * 0.7 + bias).normalize_or_zero();
        }
        vel.0 = target.0 * ENFORCER_SPEED;
    }
}

fn enforcer_fire(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_q: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &mut FireCooldown), With<Enforcer>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (tf, mut cooldown) in &mut query {
        cooldown.0.tick(time.delta());
        if cooldown.0.just_finished() {
            let pos = tf.translation.truncate();
            let dir = (player_pos - pos).normalize_or_zero();
            commands.spawn((
                EnemyProjectile,
                DamagesPlayer,
                WaveEntity,
                Mesh2d(assets.spark_mesh.clone()),
                MeshMaterial2d(assets.spark_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 0.5),
                Velocity(dir * SPARK_SPEED),
                CollisionRadius(SPARK_RADIUS),
                Lifetime(Timer::from_seconds(SPARK_LIFETIME, TimerMode::Once)),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

fn tank_ai(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut query: Query<
        (
            &Transform,
            &mut Velocity,
            &mut WanderTimer,
            &mut WanderTarget,
        ),
        With<Tank>,
    >,
) {
    let player_pos = player_q
        .single()
        .map(|tf| tf.translation.truncate())
        .unwrap_or(Vec2::ZERO);
    let mut rng = rand::rng();
    for (tf, mut vel, mut timer, mut target) in &mut query {
        timer.0.tick(time.delta());
        if target.0 == Vec2::ZERO || timer.0.just_finished() {
            let pos = tf.translation.truncate();
            let bias = (player_pos - pos).normalize_or_zero() * 0.2;
            let rand_dir = Vec2::new(
                rng.random_range(-1.0f32..1.0),
                rng.random_range(-1.0f32..1.0),
            )
            .normalize_or_zero();
            target.0 = (rand_dir * 0.8 + bias).normalize_or_zero();
        }
        vel.0 = target.0 * TANK_SPEED;
    }
}

fn tank_fire(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_q: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &mut FireCooldown), With<Tank>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    for (tf, mut cooldown) in &mut query {
        cooldown.0.tick(time.delta());
        if cooldown.0.just_finished() {
            let pos = tf.translation.truncate();
            let dir = (player_pos - pos).normalize_or_zero();
            commands.spawn((
                EnemyProjectile,
                DamagesPlayer,
                Confined,
                WaveEntity,
                Mesh2d(assets.shell_mesh.clone()),
                MeshMaterial2d(assets.shell_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 0.5),
                Velocity(dir * SHELL_SPEED),
                CollisionRadius(SHELL_RADIUS),
                BouncesRemaining(SHELL_MAX_BOUNCES),
                Lifetime(Timer::from_seconds(SHELL_LIFETIME, TimerMode::Once)),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

fn homing_missile_steer(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut missile_q: Query<(&Transform, &mut Velocity, &HomingMissile)>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_secs();
    for (tf, mut vel, homing) in &mut missile_q {
        let pos = tf.translation.truncate();
        let desired = (player_pos - pos).normalize_or_zero();
        let current = vel.0.normalize_or_zero();
        let cross = current.x * desired.y - current.y * desired.x;
        let turn = cross.clamp(-1.0, 1.0) * homing.turn_rate * dt;
        let speed = vel.0.length();
        let angle = current.y.atan2(current.x) + turn;
        vel.0 = Vec2::new(angle.cos(), angle.sin()) * speed;
    }
}

fn bounce_shell_reflect(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Velocity, &mut BouncesRemaining)>,
) {
    for (entity, tf, mut vel, mut bounces) in &mut query {
        let pos = tf.translation;
        let mut bounced = false;
        if pos.x <= -ARENA_HALF_WIDTH + SHELL_RADIUS || pos.x >= ARENA_HALF_WIDTH - SHELL_RADIUS {
            vel.0.x = -vel.0.x;
            bounced = true;
        }
        if pos.y <= -ARENA_HALF_HEIGHT + SHELL_RADIUS || pos.y >= ARENA_HALF_HEIGHT - SHELL_RADIUS {
            vel.0.y = -vel.0.y;
            bounced = true;
        }
        if bounced {
            if bounces.0 == 0 {
                commands.entity(entity).despawn();
            } else {
                bounces.0 -= 1;
            }
        }
    }
}
