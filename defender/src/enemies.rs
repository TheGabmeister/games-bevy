use bevy::prelude::*;

use crate::camera::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::scheduling::GameplaySet;
use crate::spawning::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (lander_ai, mutant_ai, bomber_ai, swarmer_ai, baiter_ai).in_set(GameplaySet::Input),
        )
        .add_systems(Update, enemy_movement.in_set(GameplaySet::Movement));
    }
}

pub fn lander_ai(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameplayAssets>,
    player_q: Query<(&WorldPosition, &Transform), (With<Player>, Without<Lander>, Without<Human>)>,
    mut landers: Query<
        (
            Entity,
            &WorldPosition,
            &Transform,
            &mut Velocity,
            &mut LanderState,
            &mut LanderTarget,
            &mut EnemyShootTimer,
        ),
        (With<Lander>, Without<Human>, Without<Player>),
    >,
    humans: Query<
        (Entity, &WorldPosition),
        (
            With<Human>,
            Without<GrabbedBy>,
            Without<HumanFalling>,
            Without<CaughtByPlayer>,
            Without<Lander>,
            Without<Player>,
        ),
    >,
    mut human_transforms: Query<&mut Transform, (With<Human>, Without<Lander>, Without<Player>)>,
    terrain: Res<TerrainData>,
) {
    let Ok((player_wp, player_tf)) = player_q.single() else {
        return;
    };

    for (entity, wp, tf, mut vel, mut state, mut target, mut shoot_timer) in &mut landers {
        // Enemy shooting
        shoot_timer.0.tick(time.delta());
        if shoot_timer.0.just_finished() {
            let dx = world_dx(player_wp.0, wp.0);
            let dy = player_tf.translation.y - tf.translation.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 600.0 && dist > 0.0 {
                let dir = Vec2::new(dx, dy).normalize() * ENEMY_PROJECTILE_SPEED;
                spawn_enemy_projectile(&mut commands, &assets, wp.0, tf.translation.y, dir);
            }
        }

        match *state {
            LanderState::Descending => {
                let terrain_y = crate::terrain::get_terrain_y_at(&terrain, wp.0);
                // Add sine wobble to horizontal movement
                let wobble = (time.elapsed_secs() * 2.0 + wp.0 * 0.01).sin() * 30.0;
                vel.0.x = LANDER_HORIZONTAL_SPEED * vel.0.x.signum() + wobble;

                // Near ground? Try to grab a human
                if tf.translation.y < terrain_y + 40.0 {
                    vel.0.y = 0.0;

                    // Find nearest human
                    let mut best: Option<(Entity, f32)> = None;
                    for (h_entity, h_wp) in &humans {
                        let dist = world_distance(wp.0, tf.translation.y, h_wp.0, terrain_y);
                        if dist < LANDER_GRAB_RANGE && (best.is_none() || dist < best.unwrap().1) {
                            best = Some((h_entity, dist));
                        }
                    }

                    if let Some((h_entity, _)) = best {
                        commands.entity(h_entity).insert(GrabbedBy(entity));
                        *state = LanderState::Ascending(h_entity);
                        vel.0.y = LANDER_ASCENT_SPEED;
                    } else if target.0.is_none() {
                        // No human nearby, pick one to go towards
                        let mut nearest: Option<(Entity, f32)> = None;
                        for (h_entity, h_wp) in &humans {
                            let dist = world_dx(wp.0, h_wp.0).abs();
                            if nearest.is_none() || dist < nearest.unwrap().1 {
                                nearest = Some((h_entity, dist));
                            }
                        }
                        target.0 = nearest.map(|(e, _)| e);
                    }

                    // Move toward target human
                    if let Some(target_entity) = target.0 {
                        if let Ok((_, h_wp)) = humans.get(target_entity) {
                            let dx = world_dx(h_wp.0, wp.0);
                            vel.0.x = dx.signum() * LANDER_HORIZONTAL_SPEED;
                        } else {
                            target.0 = None;
                        }
                    }
                } else {
                    vel.0.y = -LANDER_DESCENT_SPEED;
                }
            }
            LanderState::Ascending(human_entity) => {
                vel.0.y = LANDER_ASCENT_SPEED;
                vel.0.x *= 0.95; // slow horizontal drift

                // Update human position to follow lander
                if let Ok(mut h_tf) = human_transforms.get_mut(human_entity) {
                    h_tf.translation.y = tf.translation.y - 12.0;
                }

                // Reached top? Transform into mutant
                if tf.translation.y > CEILING_Y - 10.0 {
                    let wx = wp.0;
                    let y = tf.translation.y;
                    commands.entity(entity).despawn();
                    commands.entity(human_entity).despawn();
                    spawn_mutant(&mut commands, &assets, wx, y);
                }
            }
        }
    }
}

pub fn mutant_ai(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameplayAssets>,
    player_q: Query<(&WorldPosition, &Transform), (With<Player>, Without<Mutant>)>,
    mut mutants: Query<
        (
            &WorldPosition,
            &Transform,
            &mut Velocity,
            &mut EnemyShootTimer,
        ),
        (With<Mutant>, Without<Player>),
    >,
) {
    let Ok((player_wp, player_tf)) = player_q.single() else {
        return;
    };

    for (wp, tf, mut vel, mut shoot_timer) in &mut mutants {
        let dx = world_dx(player_wp.0, wp.0);
        let dy = player_tf.translation.y - tf.translation.y;

        // Aggressively chase player with jitter
        let jitter_x = (time.elapsed_secs() * 5.0 + wp.0).sin() * 100.0;
        let jitter_y = (time.elapsed_secs() * 7.0 + wp.0).cos() * 80.0;

        let target_vx = dx.signum() * MUTANT_SPEED + jitter_x;
        let target_vy = dy.signum() * MUTANT_SPEED * 0.5 + jitter_y;

        vel.0.x += (target_vx - vel.0.x) * 3.0 * time.delta_secs();
        vel.0.y += (target_vy - vel.0.y) * 3.0 * time.delta_secs();

        // Shoot frequently
        shoot_timer.0.tick(time.delta());
        if shoot_timer.0.just_finished() {
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 0.0 {
                let dir = Vec2::new(dx, dy).normalize() * ENEMY_PROJECTILE_SPEED;
                spawn_enemy_projectile(&mut commands, &assets, wp.0, tf.translation.y, dir);
            }
        }
    }
}

pub fn bomber_ai(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameplayAssets>,
    mut bombers: Query<(&WorldPosition, &Transform, &mut BomberDropTimer), With<Bomber>>,
) {
    for (wp, tf, mut timer) in &mut bombers {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            spawn_mine(&mut commands, &assets, wp.0, tf.translation.y);
        }
    }
}

pub fn swarmer_ai(
    time: Res<Time>,
    player_q: Query<(&WorldPosition, &Transform), (With<Player>, Without<Swarmer>)>,
    mut swarmers: Query<
        (&WorldPosition, &Transform, &mut Velocity),
        (With<Swarmer>, Without<Player>),
    >,
) {
    let Ok((player_wp, player_tf)) = player_q.single() else {
        return;
    };

    for (wp, tf, mut vel) in &mut swarmers {
        let dx = world_dx(player_wp.0, wp.0);
        let dy = player_tf.translation.y - tf.translation.y;

        let jitter_x = (time.elapsed_secs() * 8.0 + wp.0 * 0.1).sin() * 60.0;
        let jitter_y = (time.elapsed_secs() * 6.0 + wp.0 * 0.1).cos() * 40.0;

        let target_vx = dx.signum() * SWARMER_SPEED + jitter_x;
        let target_vy = dy.signum() * SWARMER_SPEED * 0.4 + jitter_y;

        vel.0.x += (target_vx - vel.0.x) * 4.0 * time.delta_secs();
        vel.0.y += (target_vy - vel.0.y) * 4.0 * time.delta_secs();
    }
}

pub fn baiter_ai(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameplayAssets>,
    player_q: Query<(&WorldPosition, &Transform), (With<Player>, Without<Baiter>)>,
    mut baiters: Query<
        (
            &WorldPosition,
            &Transform,
            &mut Velocity,
            &mut EnemyShootTimer,
        ),
        (With<Baiter>, Without<Player>),
    >,
) {
    let Ok((player_wp, player_tf)) = player_q.single() else {
        return;
    };

    for (wp, tf, mut vel, mut shoot_timer) in &mut baiters {
        let dx = world_dx(player_wp.0, wp.0);
        let dy = player_tf.translation.y - tf.translation.y;

        // Direct fast chase
        let target_vx = dx.signum() * BAITER_SPEED;
        let target_vy = dy.signum() * BAITER_SPEED * 0.6;

        vel.0.x += (target_vx - vel.0.x) * 2.0 * time.delta_secs();
        vel.0.y += (target_vy - vel.0.y) * 2.0 * time.delta_secs();

        // Shoot
        shoot_timer.0.tick(time.delta());
        if shoot_timer.0.just_finished() {
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 0.0 {
                let dir = Vec2::new(dx, dy).normalize() * ENEMY_PROJECTILE_SPEED;
                spawn_enemy_projectile(&mut commands, &assets, wp.0, tf.translation.y, dir);
            }
        }
    }
}

pub fn enemy_movement(
    time: Res<Time>,
    mut query: Query<
        (&Velocity, &mut WorldPosition, &mut Transform),
        (With<Enemy>, Without<Player>),
    >,
) {
    for (vel, mut wp, mut tf) in &mut query {
        wp.0 += vel.0.x * time.delta_secs();
        tf.translation.y += vel.0.y * time.delta_secs();
        tf.translation.y = tf.translation.y.clamp(TERRAIN_BOTTOM_Y, CEILING_Y + 50.0);
    }
}
