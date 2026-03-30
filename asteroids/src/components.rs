use std::time::Duration;

use bevy::prelude::*;

use crate::SHOOT_COOLDOWN;

/// Marks the player ship entity.
#[derive(Component, Reflect)]
pub struct Ship;

/// Velocity shared by ship, bullets, and asteroids.
#[derive(Component, Reflect)]
pub struct Velocity(pub Vec2);

/// Size of an asteroid, used for splitting and scoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum AsteroidSize {
    Large,
    Medium,
    Small,
}

/// Marks an asteroid entity.
#[derive(Component, Reflect)]
pub struct Asteroid {
    pub size: AsteroidSize,
}

/// Marks a bullet entity.
#[derive(Component, Reflect)]
pub struct Bullet;

/// Seconds until a bullet is despawned.
#[derive(Component, Reflect)]
pub struct Lifetime(pub Timer);

/// Post-respawn invincibility; removed when it reaches zero.
#[derive(Component, Reflect)]
pub struct Invincible(pub Timer);

/// Per-ship weapon cooldown timer.
#[derive(Component, Reflect)]
pub struct ShootCooldown(pub Timer);

impl Default for ShootCooldown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(SHOOT_COOLDOWN, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(SHOOT_COOLDOWN));
        Self(timer)
    }
}

// ── Events ───────────────────────────────────────────────────────────────────

/// Triggered when a bullet destroys an asteroid.
#[derive(Event)]
pub struct AsteroidDestroyed {
    pub position: Vec3,
    pub velocity: Vec2,
    pub size: AsteroidSize,
}

// ── UI marker components ──────────────────────────────────────────────────────

#[derive(Component, Reflect)]
pub struct ScoreText;

#[derive(Component, Reflect)]
pub struct LivesText;

#[derive(Component, Reflect)]
pub struct GameOverText;
