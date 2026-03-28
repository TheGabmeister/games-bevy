use bevy::prelude::*;

// Entity markers
#[derive(Component)]
pub struct Player;

// Data components
#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Laser;

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
pub struct GameOverUI;
