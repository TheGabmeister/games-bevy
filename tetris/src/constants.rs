use bevy::prelude::*;

// Window
pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const CLEAR_COLOR: Color = Color::srgb(0.04, 0.04, 0.04); // #0A0A0A

// Grid
pub const GRID_COLS: usize = 10;
pub const GRID_VISIBLE_ROWS: usize = 20;
pub const GRID_BUFFER_ROWS: usize = 2;
pub const GRID_TOTAL_ROWS: usize = GRID_VISIBLE_ROWS + GRID_BUFFER_ROWS;

// Cell rendering
pub const CELL_SIZE: f32 = 30.0;
pub const CELL_GAP: f32 = 2.0;
pub const CELL_INNER_SIZE: f32 = CELL_SIZE - CELL_GAP;

// Playfield border
pub const BORDER_THICKNESS: f32 = 2.0;
pub const BORDER_COLOR: Color = Color::srgb(0.165, 0.165, 0.165); // #2A2A2A

// Playfield pixel dimensions (outer)
pub const PLAYFIELD_WIDTH: f32 = GRID_COLS as f32 * CELL_SIZE;
pub const PLAYFIELD_HEIGHT: f32 = GRID_VISIBLE_ROWS as f32 * CELL_SIZE;

// Playfield origin — bottom-left corner in world space (playfield centered horizontally, vertically)
pub const PLAYFIELD_LEFT: f32 = -PLAYFIELD_WIDTH / 2.0;
pub const PLAYFIELD_BOTTOM: f32 = -PLAYFIELD_HEIGHT / 2.0;

// Sidebar
pub const SIDEBAR_CELL_SCALE: f32 = 0.7;
pub const SIDEBAR_MARGIN: f32 = 20.0;

// Tetromino HDR colors (values > 1.0 for bloom)
pub const COLOR_I: Color = Color::srgb(0.0, 4.0, 4.0);
pub const COLOR_O: Color = Color::srgb(4.0, 4.0, 0.0);
pub const COLOR_T: Color = Color::srgb(3.5, 0.5, 4.0);
pub const COLOR_S: Color = Color::srgb(0.5, 4.0, 0.5);
pub const COLOR_Z: Color = Color::srgb(4.0, 0.5, 0.5);
pub const COLOR_J: Color = Color::srgb(0.5, 0.5, 4.0);
pub const COLOR_L: Color = Color::srgb(4.0, 2.0, 0.0);

// Ghost piece alpha
pub const GHOST_ALPHA: f32 = 0.25;

// Gravity & drop
pub const GRAVITY_BASE: f64 = 0.8;
pub const GRAVITY_FACTOR: f64 = 0.007;
pub const GRAVITY_FLOOR: f64 = 0.05;
pub const SOFT_DROP_MULTIPLIER: f64 = 20.0;
pub const HARD_DROP_SCORE_PER_ROW: u32 = 2;
pub const SOFT_DROP_SCORE_PER_ROW: u32 = 1;

// Lock delay
pub const LOCK_DELAY_SECS: f32 = 0.5;
pub const LOCK_DELAY_MAX_RESETS: u32 = 15;

// DAS (Delayed Auto Shift)
pub const DAS_INITIAL_DELAY: f32 = 0.17;
pub const DAS_REPEAT_RATE: f32 = 0.05;

// Leveling
pub const STARTING_LEVEL: u32 = 1;
pub const LINES_PER_LEVEL: u32 = 10;

// Scoring
pub const SCORE_SINGLE: u32 = 100;
pub const SCORE_DOUBLE: u32 = 300;
pub const SCORE_TRIPLE: u32 = 500;
pub const SCORE_TETRIS: u32 = 800;

// Spawn
pub const SPAWN_COL: i32 = 3;

// Next queue
pub const NEXT_QUEUE_SIZE: usize = 5;
