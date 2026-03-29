use bevy::prelude::*;

// Player
#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Invulnerable(pub Timer);

// Enemies
#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct FormationSlot {
    pub col: usize,
    pub row: usize,
}

#[derive(Component)]
pub struct DivingEnemy {
    pub time: f32,
    pub start_x: f32,
    pub curve_direction: f32,
    pub returning: bool,
}

// Bullets
#[derive(Component)]
pub struct PlayerBullet;

#[derive(Component)]
pub struct EnemyBullet;

// Audio
#[derive(Component)]
pub struct Music;

// UI markers
#[derive(Component)]
pub struct StartScreenUI;

#[derive(Component)]
pub struct GameHudUI;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct GameOverUI;
