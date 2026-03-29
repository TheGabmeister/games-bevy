use bevy::prelude::*;

// Window
pub const WINDOW_WIDTH: f32 = 720.0;
pub const WINDOW_HEIGHT: f32 = 960.0;
pub const WINDOW_TITLE: &str = "Frogger";

// Grid
pub const CELL_SIZE: f32 = 48.0;
pub const GRID_COLS: i32 = 13;
pub const PLAYFIELD_WIDTH: f32 = 624.0; // GRID_COLS * CELL_SIZE

// Frog
pub const HOP_DURATION: f32 = 0.13;
pub const FROG_SPAWN_COL: i32 = 6;
pub const FROG_SPAWN_ROW: i32 = 0;
pub const FROG_SIZE: f32 = 40.0;

// Game rules
pub const STARTING_LIVES: u32 = 3;
pub const LIFE_TIMER_SECS: f32 = 30.0;
pub const MAX_SPEED_MULTIPLIER: f32 = 2.5;
pub const SPEED_INCREMENT: f32 = 0.15;

// Scoring
pub const SCORE_FORWARD_HOP: u32 = 10;
pub const SCORE_HOME_BAY: u32 = 50;
pub const SCORE_TIME_BONUS_PER_SEC: u32 = 10;
pub const SCORE_LEVEL_CLEAR: u32 = 1000;

// Row assignments
pub const RIVER_ROW_START: i32 = 7;
pub const RIVER_ROW_END: i32 = 11;
pub const HOME_ROW: i32 = 12;

// Home bays
pub const HOME_BAY_COLS: [i32; 5] = [1, 4, 6, 8, 11];

// Lane wrapping
pub const WRAP_MARGIN: f32 = 240.0;

// Colors
pub const COLOR_BACKGROUND: Color = Color::srgb(0.05, 0.05, 0.15);
pub const COLOR_SAFE_ZONE: Color = Color::srgb(0.15, 0.4, 0.15);
pub const COLOR_ROAD: Color = Color::srgb(0.2, 0.2, 0.2);
pub const COLOR_RIVER: Color = Color::srgb(0.05, 0.1, 0.5);
pub const COLOR_FROG: Color = Color::srgb(0.1, 0.85, 0.1);
pub const COLOR_LOG: Color = Color::srgb(0.55, 0.3, 0.1);
pub const COLOR_HOME_WALL: Color = Color::srgb(0.15, 0.4, 0.15);
pub const COLOR_HOME_BAY_OPEN: Color = Color::srgb(0.05, 0.05, 0.15);
pub const COLOR_FILLED_BAY: Color = Color::srgb(0.1, 0.85, 0.1);
pub const VEHICLE_COLORS: [Color; 5] = [
    Color::srgb(0.9, 0.2, 0.2),
    Color::srgb(0.9, 0.9, 0.2),
    Color::srgb(0.2, 0.5, 0.9),
    Color::srgb(0.9, 0.5, 0.1),
    Color::srgb(0.9, 0.2, 0.9),
];

// UI
pub const FONT_SIZE_TITLE: f32 = 60.0;
pub const FONT_SIZE_BODY: f32 = 30.0;
pub const FONT_SIZE_HUD: f32 = 24.0;
pub const TEXT_COLOR: Color = Color::WHITE;

// --- Helpers ---

pub fn grid_to_world(col: i32, row: i32) -> Vec2 {
    Vec2::new(
        (col as f32 - 6.0) * CELL_SIZE,
        (row as f32 - 6.0) * CELL_SIZE,
    )
}

pub fn row_to_world_y(row: i32) -> f32 {
    (row as f32 - 6.0) * CELL_SIZE
}

pub fn world_x_to_col(x: f32) -> i32 {
    (x / CELL_SIZE + 6.0).round().clamp(0.0, 12.0) as i32
}
