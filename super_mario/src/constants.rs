#![allow(dead_code)]

use bevy::prelude::*;

// Window
pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

// Tile
pub const TILE_SIZE: f32 = 32.0;
pub const LEVEL_HEIGHT_TILES: usize = 15;
pub const GROUND_TILE_ROWS: usize = 2;
pub const CAMERA_FOLLOW_LERP: f32 = 8.0;

// Physics
pub const GRAVITY: f32 = -1800.0;
pub const MAX_FALL_SPEED: f32 = -600.0;

// Player movement
pub const PLAYER_ACCELERATION: f32 = 1200.0;
pub const PLAYER_DECELERATION: f32 = 1800.0;
pub const PLAYER_MAX_SPEED: f32 = 250.0;
pub const PLAYER_JUMP_FORCE: f32 = 600.0;
pub const PLAYER_JUMP_CUT_MULTIPLIER: f32 = 0.4;

// Player dimensions (small Mario)
pub const PLAYER_WIDTH: f32 = 24.0;
pub const PLAYER_HEIGHT: f32 = 30.0;

// Player dimensions (big Mario)
pub const PLAYER_BIG_WIDTH: f32 = 24.0;
pub const PLAYER_BIG_HEIGHT: f32 = 56.0;

// Scoring
pub const INITIAL_LIVES: u32 = 3;
pub const INITIAL_TIMER: u32 = 400;
pub const COINS_FOR_1UP: u32 = 100;
pub const WORLD_LABEL_1_1: &str = "1-1";

// Score values
pub const SCORE_COIN: u32 = 200;
pub const SCORE_GOOMBA: u32 = 100;
pub const SCORE_KOOPA: u32 = 200;
pub const SCORE_BRICK: u32 = 50;

// Enemy
pub const GOOMBA_SPEED: f32 = 60.0;
pub const KOOPA_SPEED: f32 = 50.0;
pub const SHELL_SPEED: f32 = 300.0;

// Mushroom
pub const MUSHROOM_SPEED: f32 = 80.0;
pub const MUSHROOM_RISE_SPEED: f32 = 60.0;

// Z-layers
pub const Z_BACKGROUND: f32 = 0.0;
pub const Z_TILES: f32 = 10.0;
pub const Z_ITEMS: f32 = 20.0;
pub const Z_ENEMIES: f32 = 20.0;
pub const Z_PLAYER: f32 = 30.0;
pub const Z_PARTICLES: f32 = 40.0;

// Death threshold (Y position below which player dies)
pub const DEATH_Y: f32 = -200.0;

// Color palette
pub const COLOR_SKY: Color = Color::srgb(0.37, 0.60, 0.97);
pub const COLOR_GROUND: Color = Color::srgb(0.55, 0.33, 0.16);
pub const COLOR_BRICK: Color = Color::srgb(0.72, 0.42, 0.18);
pub const COLOR_QUESTION_BLOCK: Color = Color::srgb(0.95, 0.80, 0.15);
pub const COLOR_QUESTION_BLOCK_SPENT: Color = Color::srgb(0.45, 0.38, 0.25);
pub const COLOR_HARD_BLOCK: Color = Color::srgb(0.40, 0.40, 0.42);
pub const COLOR_PIPE_GREEN: Color = Color::srgb(0.18, 0.65, 0.22);
pub const COLOR_PIPE_GREEN_DARK: Color = Color::srgb(0.12, 0.48, 0.15);
pub const COLOR_MARIO_RED: Color = Color::srgb(0.90, 0.15, 0.12);
pub const COLOR_MARIO_SKIN: Color = Color::srgb(0.95, 0.75, 0.55);
pub const COLOR_MARIO_BROWN: Color = Color::srgb(0.55, 0.30, 0.10);
pub const COLOR_MARIO_BLUE: Color = Color::srgb(0.18, 0.34, 0.78);
pub const COLOR_GOOMBA: Color = Color::srgb(0.60, 0.35, 0.15);
pub const COLOR_KOOPA_GREEN: Color = Color::srgb(0.20, 0.72, 0.25);
pub const COLOR_COIN_GOLD: Color = Color::srgb(1.0, 0.85, 0.15);
pub const COLOR_MUSHROOM_RED: Color = Color::srgb(0.90, 0.15, 0.12);
pub const COLOR_MUSHROOM_SPOTS: Color = Color::srgb(1.0, 1.0, 1.0);
pub const COLOR_FLAGPOLE: Color = Color::srgb(0.30, 0.30, 0.30);
pub const COLOR_FLAG: Color = Color::srgb(0.10, 0.65, 0.15);
pub const COLOR_CASTLE: Color = Color::srgb(0.55, 0.50, 0.45);
