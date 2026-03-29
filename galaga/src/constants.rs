use bevy::prelude::*;

// Window
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const WINDOW_TITLE: &str = "Galaga";

// Player
pub const PLAYER_SPEED: f32 = 400.0;
pub const PLAYER_WIDTH: f32 = 36.0;
pub const PLAYER_HEIGHT: f32 = 28.0;
pub const PLAYER_Y: f32 = -WINDOW_HEIGHT / 2.0 + 60.0;
pub const PLAYER_LIVES: u32 = 3;
pub const RESPAWN_DELAY: f32 = 1.5;
pub const INVULNERABLE_DURATION: f32 = 1.5;
pub const MAX_PLAYER_BULLETS: usize = 2;

// Formation
pub const FORMATION_COLS: usize = 8;
pub const FORMATION_ROWS: usize = 4;
pub const FORMATION_SPACING_X: f32 = 64.0;
pub const FORMATION_SPACING_Y: f32 = 48.0;
pub const FORMATION_TOP_Y: f32 = WINDOW_HEIGHT / 2.0 - 120.0;
pub const FORMATION_SWAY_SPEED: f32 = 0.8;
pub const FORMATION_SWAY_AMOUNT: f32 = 40.0;

// Enemy
pub const ENEMY_SIZE: f32 = 32.0;
pub const ENEMY_SCORE_ROW0: u32 = 150;
pub const ENEMY_SCORE_ROW1: u32 = 100;
pub const ENEMY_SCORE_ROW2: u32 = 80;
pub const ENEMY_SCORE_ROW3: u32 = 50;

// Dive
pub const DIVE_SPEED: f32 = 280.0;
pub const DIVE_CURVE_AMPLITUDE: f32 = 100.0;
pub const DIVE_CURVE_FREQUENCY: f32 = 2.0;
pub const DIVE_INTERVAL_BASE: f32 = 2.5;
pub const DIVE_INTERVAL_MIN: f32 = 1.0;
pub const DIVE_INTERVAL_REDUCTION: f32 = 0.15;
pub const MAX_DIVERS_BASE: usize = 2;

// Enemy bullets
pub const ENEMY_BULLET_SPEED: f32 = 350.0;
pub const ENEMY_BULLET_SIZE: f32 = 8.0;

// Player bullet (laser)
pub const LASER_SPEED: f32 = 600.0;
pub const LASER_WIDTH: f32 = 4.0;
pub const LASER_HEIGHT: f32 = 16.0;

// Collision radii
pub const PLAYER_COLLISION_RADIUS: f32 = 14.0;
pub const ENEMY_COLLISION_RADIUS: f32 = 14.0;
pub const LASER_COLLISION_RADIUS: f32 = 6.0;
pub const ENEMY_BULLET_COLLISION_RADIUS: f32 = 4.0;

// Timers
pub const STAGE_CLEAR_DELAY: f32 = 2.0;

// UI
pub const FONT_SIZE_TITLE: f32 = 60.0;
pub const FONT_SIZE_BODY: f32 = 30.0;
pub const FONT_SIZE_HUD: f32 = 24.0;
pub const TEXT_COLOR: Color = Color::WHITE;
