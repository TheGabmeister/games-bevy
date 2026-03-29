use bevy::prelude::*;

// Window
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const WINDOW_TITLE: &str = "Bevy 2D Template";

// Player
pub const PLAYER_SPEED: f32 = 300.0;
pub const PLAYER_SIZE: f32 = 40.0;

// Enemy
pub const ENEMY_SIZE: f32 = 40.0;
pub const ENEMY_COUNT: usize = 3;
pub const ENEMY_Y: f32 = WINDOW_HEIGHT / 2.0 - 80.0;
pub const ENEMY_SCORE: u32 = 100;

// Laser
pub const LASER_SPEED: f32 = 500.0;
pub const LASER_SIZE: f32 = 16.0;

// UI
pub const FONT_SIZE_TITLE: f32 = 60.0;
pub const FONT_SIZE_BODY: f32 = 30.0;
pub const FONT_SIZE_HUD: f32 = 24.0;
pub const TEXT_COLOR: Color = Color::WHITE;
