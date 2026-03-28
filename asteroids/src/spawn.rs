use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

use crate::components::*;
use crate::resources::GameAssets;
use crate::{
    ASTEROID_LARGE_RADIUS, ASTEROID_LARGE_SPEED, ASTEROID_MEDIUM_RADIUS, ASTEROID_MEDIUM_SPEED,
    ASTEROID_SMALL_RADIUS, ASTEROID_SMALL_SPEED, BULLET_LIFETIME, INITIAL_ASTEROIDS,
    INVINCIBILITY_DURATION, SCORE_LARGE, SCORE_MEDIUM, SCORE_SMALL,
};

// ── AsteroidSize helpers ──────────────────────────────────────────────────────

pub fn asteroid_radius(size: AsteroidSize) -> f32 {
    match size {
        AsteroidSize::Large => ASTEROID_LARGE_RADIUS,
        AsteroidSize::Medium => ASTEROID_MEDIUM_RADIUS,
        AsteroidSize::Small => ASTEROID_SMALL_RADIUS,
    }
}

pub fn asteroid_score(size: AsteroidSize) -> u32 {
    match size {
        AsteroidSize::Large => SCORE_LARGE,
        AsteroidSize::Medium => SCORE_MEDIUM,
        AsteroidSize::Small => SCORE_SMALL,
    }
}

pub fn asteroid_spin(size: AsteroidSize) -> f32 {
    match size {
        AsteroidSize::Large => 0.5,
        AsteroidSize::Medium => 0.8,
        AsteroidSize::Small => 1.2,
    }
}

// ── Spawn functions ───────────────────────────────────────────────────────────

pub fn spawn_ship(commands: &mut Commands, assets: &GameAssets) {
    commands.spawn((
        Ship,
        Velocity(Vec2::ZERO),
        Invincible(Timer::from_seconds(INVINCIBILITY_DURATION, TimerMode::Once)),
        Mesh2d(assets.ship_mesh.clone()),
        MeshMaterial2d(assets.ship_material.clone()),
        // z = 1 so the ship renders on top of asteroids
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));
}

pub fn spawn_bullet(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    direction: Vec2,
    speed: f32,
) {
    commands.spawn((
        Bullet,
        Velocity(direction * speed),
        Lifetime(Timer::from_seconds(BULLET_LIFETIME, TimerMode::Once)),
        Mesh2d(assets.bullet_mesh.clone()),
        MeshMaterial2d(assets.bullet_material.clone()),
        Transform::from_translation(position),
    ));
}

pub fn spawn_asteroid(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    velocity: Vec2,
    size: AsteroidSize,
) {
    let mesh = match size {
        AsteroidSize::Large => assets.asteroid_large_mesh.clone(),
        AsteroidSize::Medium => assets.asteroid_medium_mesh.clone(),
        AsteroidSize::Small => assets.asteroid_small_mesh.clone(),
    };
    commands.spawn((
        Asteroid { size },
        Velocity(velocity),
        Mesh2d(mesh),
        MeshMaterial2d(assets.asteroid_material.clone()),
        Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
    ));
}

/// Deterministic spawn position: radius 250 from center, distributed by index.
/// The wave offset ensures asteroids don't always appear in the same spots.
fn asteroid_start_position(index: u32, count: u32, wave: u32) -> Vec2 {
    let angle = (index as f32 / count as f32) * TAU + wave as f32 * 0.7;
    Vec2::new(angle.cos() * 250.0, angle.sin() * 250.0)
}

/// Initial velocity roughly perpendicular to spawn direction so asteroids
/// travel across the screen rather than toward/away from the center.
fn asteroid_start_velocity(pos: Vec2, size: AsteroidSize) -> Vec2 {
    let speed = match size {
        AsteroidSize::Large => ASTEROID_LARGE_SPEED,
        AsteroidSize::Medium => ASTEROID_MEDIUM_SPEED,
        AsteroidSize::Small => ASTEROID_SMALL_SPEED,
    };
    let angle = pos.y.atan2(pos.x) + PI / 2.0;
    Vec2::new(angle.cos() * speed, angle.sin() * speed)
}

pub fn spawn_wave(commands: &mut Commands, assets: &GameAssets, wave: u32) {
    // One extra asteroid per wave, capped at 8
    let count = (INITIAL_ASTEROIDS + wave - 1).min(8);
    for i in 0..count {
        let pos = asteroid_start_position(i, count, wave);
        let vel = asteroid_start_velocity(pos, AsteroidSize::Large);
        spawn_asteroid(commands, assets, pos.extend(0.0), vel, AsteroidSize::Large);
    }
}

/// Split a destroyed asteroid into two smaller ones at ±30° from its velocity.
pub fn spawn_fragments(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    parent_vel: Vec2,
    parent_size: AsteroidSize,
) {
    let child_size = match parent_size {
        AsteroidSize::Large => AsteroidSize::Medium,
        AsteroidSize::Medium => AsteroidSize::Small,
        AsteroidSize::Small => return, // smallest size: no fragments
    };
    let speed = match child_size {
        AsteroidSize::Medium => ASTEROID_MEDIUM_SPEED,
        AsteroidSize::Small => ASTEROID_SMALL_SPEED,
        AsteroidSize::Large => unreachable!(),
    };
    // Use parent velocity direction; fall back to rightward if nearly zero
    let base_angle = if parent_vel.length_squared() > 0.01 {
        parent_vel.y.atan2(parent_vel.x)
    } else {
        0.0
    };
    for offset in [PI / 6.0, -PI / 6.0] {
        let angle = base_angle + offset;
        let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        spawn_asteroid(commands, assets, position, vel, child_size);
    }
}
