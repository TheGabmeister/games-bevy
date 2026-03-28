use bevy::prelude::*;

/// Marks the player ship entity.
#[derive(Component)]
pub struct Ship;

/// Velocity shared by ship, bullets, and asteroids.
#[derive(Component)]
pub struct Velocity(pub Vec2);

/// Size of an asteroid, used for splitting and scoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsteroidSize {
    Large,
    Medium,
    Small,
}

/// Marks an asteroid entity.
#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
}

/// Marks a bullet entity.
#[derive(Component)]
pub struct Bullet;

/// Seconds until a bullet is despawned.
#[derive(Component)]
pub struct Lifetime(pub Timer);

/// Post-respawn invincibility; removed when it reaches zero.
#[derive(Component)]
pub struct Invincible(pub Timer);

// ── UI marker components ──────────────────────────────────────────────────────

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct GameOverText;
