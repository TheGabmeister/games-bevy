use bevy::prelude::*;

pub const CELL_SIZE: f32 = 40.0;
pub const GRID_WIDTH: usize = 28;
pub const GRID_HEIGHT: usize = 16;
pub const WINDOW_WIDTH: f32 = CELL_SIZE * GRID_WIDTH as f32;
pub const WINDOW_HEIGHT: f32 = CELL_SIZE * GRID_HEIGHT as f32;

// Speeds (cells per second)
pub const PLAYER_MOVE_SPEED: f32 = 6.0;
pub const PLAYER_CLIMB_SPEED: f32 = 4.0;
pub const PLAYER_FALL_SPEED: f32 = 10.0;

pub const GUARD_MOVE_SPEED: f32 = 4.5;
pub const GUARD_CLIMB_SPEED: f32 = 3.5;
pub const GUARD_FALL_SPEED: f32 = 10.0;

// Timers
pub const DIG_DURATION: f32 = 0.25;
pub const HOLE_OPEN_DURATION: f32 = 4.5;
pub const HOLE_CLOSE_DURATION: f32 = 0.5;
pub const GUARD_TRAP_DURATION: f32 = 3.0;

// Colors
pub const COLOR_BRICK: Color = Color::srgb(0.6, 0.3, 0.1);
pub const COLOR_CONCRETE: Color = Color::srgb(0.4, 0.4, 0.4);
pub const COLOR_LADDER: Color = Color::srgb(0.9, 0.8, 0.2);
pub const COLOR_BAR: Color = Color::srgb(0.3, 0.7, 0.9);
pub const COLOR_GOLD: Color = Color::srgb(1.0, 0.85, 0.0);
pub const COLOR_PLAYER: Color = Color::srgb(0.2, 0.9, 0.2);
pub const COLOR_GUARD: Color = Color::srgb(0.9, 0.2, 0.2);
pub const COLOR_HIDDEN_LADDER: Color = Color::srgb(0.5, 0.9, 0.3);
pub const COLOR_BACKGROUND: Color = Color::srgb(0.05, 0.05, 0.1);

pub const STARTING_LIVES: u32 = 5;
pub const GOLD_SCORE: u32 = 100;
pub const LEVEL_COMPLETE_SCORE: u32 = 500;
