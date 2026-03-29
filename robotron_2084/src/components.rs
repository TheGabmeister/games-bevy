use bevy::prelude::*;

#[derive(Component)]
pub struct Player;
#[derive(Component)]
pub struct Enemy;
#[derive(Component)]
pub struct Grunt;
#[derive(Component)]
pub struct Hulk;
#[derive(Component)]
pub struct Brain;
#[derive(Component)]
pub struct Prog;
#[derive(Component)]
pub struct Spheroid;
#[derive(Component)]
pub struct Enforcer;
#[derive(Component)]
pub struct Quark;
#[derive(Component)]
pub struct Tank;
#[derive(Component)]
pub struct Human;
#[derive(Component)]
pub struct Electrode;

#[derive(Component)]
pub struct Killable;
#[derive(Component)]
pub struct DamagesPlayer;
#[derive(Component)]
pub struct PlayerBullet;
#[derive(Component)]
pub struct EnemyProjectile;
#[derive(Component)]
pub struct WaveEntity;
#[derive(Component)]
pub struct Confined;

#[derive(Component)]
pub struct Velocity(pub Vec2);
#[derive(Component)]
pub struct Facing(pub Vec2);
#[derive(Component)]
pub struct CollisionRadius(pub f32);
#[derive(Component)]
pub struct PointValue(pub u32);
#[derive(Component)]
pub struct FireCooldown(pub Timer);
#[derive(Component)]
pub struct Invincible(pub Timer);
#[derive(Component)]
pub struct Lifetime(pub Timer);
#[derive(Component)]
pub struct GruntSteerOffset(pub f32);
#[derive(Component)]
pub struct Knockback(pub Vec2);
#[derive(Component)]
pub struct WanderTarget(pub Vec2);
#[derive(Component)]
pub struct WanderTimer(pub Timer);
#[derive(Component)]
pub struct HomingMissile {
    pub turn_rate: f32,
}
#[derive(Component)]
pub struct BouncesRemaining(pub u32);

#[derive(Component)]
pub struct SpawnerState {
    pub children_spawned: u32,
    pub max_children: u32,
    pub cooldown: Timer,
}

#[derive(Component)]
pub struct Particle;
#[derive(Component)]
pub struct ScorePopup;

#[derive(Component)]
pub struct ScoreText;
#[derive(Component)]
pub struct LivesText;
#[derive(Component)]
pub struct WaveText;
#[derive(Component)]
pub struct HighScoreText;
#[derive(Component)]
pub struct PauseOverlay;
