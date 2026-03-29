use bevy::prelude::*;

// Entity markers
#[derive(Component)]
pub struct Frog;

#[derive(Component)]
pub struct Vehicle;

#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct HomeBay {
    pub index: usize,
}

#[derive(Component)]
pub struct LaneObject;

#[derive(Component)]
pub struct GameplayEntity;

// Data components
#[derive(Component)]
pub struct GridPosition {
    pub col: i32,
    pub row: i32,
}

#[derive(Component, Default)]
pub struct HopState {
    pub active: bool,
    pub from: Vec2,
    pub to: Vec2,
    pub progress: f32,
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct ObjectWidth(pub f32);

// UI markers
#[derive(Component)]
pub struct StartScreenUI;

#[derive(Component)]
pub struct GameHudUI;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct HighScoreText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct TimerBar;

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct GameOverUI;
