use bevy::prelude::*;

// Entity markers
#[derive(Component)]
pub struct Player;

// Data components
#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

// UI markers
#[derive(Component)]
pub struct StartScreenUI;

#[derive(Component)]
pub struct GameHudUI;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct GameOverUI;
