use bevy::prelude::*;

// Entity markers
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Human;

#[derive(Component)]
pub struct Lander;

#[derive(Component)]
pub struct Mutant;

#[derive(Component)]
pub struct Bomber;

#[derive(Component)]
pub struct Pod;

#[derive(Component)]
pub struct Swarmer;

#[derive(Component)]
pub struct Baiter;

#[derive(Component)]
pub struct PlayerProjectile;

#[derive(Component)]
pub struct EnemyProjectile;

#[derive(Component)]
pub struct Mine;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct TerrainChunk;

// Data components
#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct FacingDirection(pub f32); // 1.0 = right, -1.0 = left

#[derive(Component)]
pub struct WorldPosition(pub f32);

#[derive(Component)]
pub struct CollisionRadius(pub f32);

#[derive(Component)]
pub struct Lifetime(pub Timer);

#[derive(Component)]
pub struct FireCooldown(pub Timer);

// Lander state machine
#[derive(Component)]
pub enum LanderState {
    Descending,
    Ascending(Entity),
}

#[derive(Component)]
pub struct GrabbedBy(pub Entity);

#[derive(Component)]
pub struct HumanFalling;

#[derive(Component)]
pub struct CaughtByPlayer;

#[derive(Component)]
pub struct BomberDropTimer(pub Timer);

#[derive(Component)]
pub struct EnemyShootTimer(pub Timer);

#[derive(Component)]
pub struct LanderTarget(pub Option<Entity>);

// Scanner
#[derive(Component)]
pub struct ScannerDot(pub Entity);

#[derive(Component)]
pub struct ScannerBar;

// UI markers
#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct SmartBombText;

#[derive(Component)]
pub struct WaveBanner;

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct StartScreen;

// Explosion effect
#[derive(Component)]
pub struct Explosion(pub Timer);
