use bevy::prelude::*;

use crate::components::*;
use crate::resources::{GameAssets, GameData};
use crate::spawn::{asteroid_radius, asteroid_score, spawn_fragments, spawn_ship};
use crate::state::AppState;
use crate::{BULLET_RADIUS, GameSet, SHIP_RADIUS};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                bullet_asteroid_collision_system,
                ship_asteroid_collision_system,
            )
                .chain()
                .in_set(GameSet::Collision)
                .run_if(in_state(AppState::Playing)),
        )
        .add_observer(on_asteroid_destroyed_score)
        .add_observer(on_asteroid_destroyed_fragment);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Circle-circle collision between bullets and asteroids.
/// Collisions are collected first, then processed, to avoid borrow conflicts.
fn bullet_asteroid_collision_system(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    asteroids: Query<(Entity, &Transform, &Asteroid, &Velocity)>,
) {
    // (bullet_entity, asteroid_entity, asteroid_world_pos, asteroid_vel, asteroid_size)
    let mut hits: Vec<(Entity, Entity, Vec3, Vec2, AsteroidSize)> = Vec::new();

    for (bullet_entity, bullet_tf) in &bullets {
        for (asteroid_entity, asteroid_tf, asteroid, asteroid_vel) in &asteroids {
            let dist = bullet_tf.translation.distance(asteroid_tf.translation);
            if dist < BULLET_RADIUS + asteroid_radius(asteroid.size) {
                hits.push((
                    bullet_entity,
                    asteroid_entity,
                    asteroid_tf.translation,
                    asteroid_vel.0,
                    asteroid.size,
                ));
            }
        }
    }

    // Guard against the same bullet or asteroid being processed twice
    // (e.g. one bullet hitting two asteroids at once)
    let mut used_bullets = std::collections::HashSet::new();
    let mut used_asteroids = std::collections::HashSet::new();

    for (bullet_entity, asteroid_entity, pos, vel, size) in hits {
        if used_bullets.contains(&bullet_entity) || used_asteroids.contains(&asteroid_entity) {
            continue;
        }
        used_bullets.insert(bullet_entity);
        used_asteroids.insert(asteroid_entity);

        commands.entity(bullet_entity).despawn();
        commands.entity(asteroid_entity).despawn();
        commands.trigger(AsteroidDestroyed {
            position: pos,
            velocity: vel,
            size,
        });
    }
}

// ── Observers ─────────────────────────────────────────────────────────────────

/// Awards points when an asteroid is destroyed.
fn on_asteroid_destroyed_score(trigger: On<AsteroidDestroyed>, mut game_data: ResMut<GameData>) {
    game_data.score += asteroid_score(trigger.event().size);
}

/// Spawns fragment asteroids when a non-Small asteroid is destroyed.
fn on_asteroid_destroyed_fragment(
    trigger: On<AsteroidDestroyed>,
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    let event = trigger.event();
    spawn_fragments(
        &mut commands,
        &assets,
        event.position,
        event.velocity,
        event.size,
    );
}

/// Circle-circle collision between the ship and asteroids.
/// Only runs when the ship does NOT have the Invincible component.
fn ship_asteroid_collision_system(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<AppState>>,
    // Without<Invincible> means invincible ships are never matched
    ship_query: Query<(Entity, &Transform), (With<Ship>, Without<Invincible>)>,
    asteroid_query: Query<(&Transform, &Asteroid)>,
) {
    let Ok((ship_entity, ship_tf)) = ship_query.single() else {
        return;
    };
    for (asteroid_tf, asteroid) in &asteroid_query {
        let dist = ship_tf.translation.distance(asteroid_tf.translation);
        if dist < SHIP_RADIUS + asteroid_radius(asteroid.size) {
            commands.entity(ship_entity).despawn();
            game_data.lives = game_data.lives.saturating_sub(1);
            if game_data.lives == 0 {
                next_state.set(AppState::GameOver);
            } else {
                // Respawn at center with temporary invincibility
                spawn_ship(&mut commands, &assets);
            }
            return; // handle only one collision per frame
        }
    }
}
