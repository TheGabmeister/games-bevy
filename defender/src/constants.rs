use bevy::prelude::*;

// Window
pub const SCREEN_WIDTH: u32 = 1280;
pub const SCREEN_HEIGHT: u32 = 720;
pub const HALF_SCREEN_W: f32 = SCREEN_WIDTH as f32 / 2.0;

// World
pub const WORLD_WIDTH: f32 = 8000.0;
pub const GROUND_Y: f32 = -280.0;
pub const CEILING_Y: f32 = 300.0;

// Player
pub const PLAYER_THRUST: f32 = 800.0;
pub const PLAYER_MAX_SPEED: f32 = 600.0;
pub const PLAYER_FRICTION: f32 = 0.96;
pub const PLAYER_VERTICAL_SPEED: f32 = 400.0;
pub const FIRE_COOLDOWN: f32 = 0.12;
pub const PROJECTILE_SPEED: f32 = 1200.0;
pub const PROJECTILE_LIFETIME: f32 = 0.7;
pub const STARTING_LIVES: u32 = 3;
pub const STARTING_SMART_BOMBS: u32 = 3;
pub const EXTRA_LIFE_INTERVAL: u32 = 10000;
pub const HYPERSPACE_DEATH_CHANCE: f32 = 0.15;

// Enemies
pub const LANDER_DESCENT_SPEED: f32 = 60.0;
pub const LANDER_HORIZONTAL_SPEED: f32 = 80.0;
pub const LANDER_ASCENT_SPEED: f32 = 80.0;
pub const LANDER_GRAB_RANGE: f32 = 40.0;
pub const MUTANT_SPEED: f32 = 300.0;
pub const BOMBER_SPEED: f32 = 100.0;
pub const BOMBER_DROP_INTERVAL: f32 = 3.0;
pub const SWARMER_SPEED: f32 = 350.0;
pub const BAITER_SPEED: f32 = 500.0;
pub const ENEMY_PROJECTILE_SPEED: f32 = 400.0;
pub const ENEMY_SHOOT_INTERVAL: f32 = 2.0;
pub const MINE_LIFETIME: f32 = 8.0;
pub const BAITER_SPAWN_DELAY: f32 = 30.0;
pub const BAITER_SPAWN_INTERVAL: f32 = 10.0;
pub const ENEMY_SPAWN_INTERVAL: f32 = 0.8;

// Scoring
pub const SCORE_LANDER: u32 = 150;
pub const SCORE_MUTANT: u32 = 150;
pub const SCORE_BOMBER: u32 = 250;
pub const SCORE_POD: u32 = 1000;
pub const SCORE_SWARMER: u32 = 200;
pub const SCORE_BAITER: u32 = 200;
pub const SCORE_HUMAN_SAVED: u32 = 500;

// Scanner
pub const SCANNER_WIDTH: f32 = 500.0;
pub const SCANNER_HEIGHT: f32 = 30.0;

// Humans
pub const HUMANS_PER_WAVE: u32 = 10;
pub const HUMAN_WALK_SPEED: f32 = 20.0;
pub const HUMAN_FALL_GRAVITY: f32 = 300.0;

// Terrain
pub const TERRAIN_SEGMENTS: usize = 200;
pub const TERRAIN_CHUNKS: usize = 8;
pub const TERRAIN_AMPLITUDE_1: f32 = 40.0;
pub const TERRAIN_AMPLITUDE_2: f32 = 20.0;
pub const TERRAIN_AMPLITUDE_3: f32 = 10.0;
pub const TERRAIN_BOTTOM_Y: f32 = -360.0;

// Collision radii
pub const PLAYER_RADIUS: f32 = 10.0;
pub const LANDER_RADIUS: f32 = 8.0;
pub const MUTANT_RADIUS: f32 = 10.0;
pub const BOMBER_RADIUS: f32 = 9.0;
pub const POD_RADIUS: f32 = 8.0;
pub const SWARMER_RADIUS: f32 = 5.0;
pub const BAITER_RADIUS: f32 = 8.0;
pub const HUMAN_RADIUS: f32 = 5.0;
pub const PROJECTILE_RADIUS: f32 = 4.0;
pub const MINE_RADIUS: f32 = 4.0;

// Colors
pub const COLOR_PLAYER: Color = Color::srgb(0.0, 1.0, 0.5);
pub const COLOR_LANDER: Color = Color::srgb(0.0, 1.0, 0.0);
pub const COLOR_MUTANT: Color = Color::srgb(1.0, 0.0, 1.0);
pub const COLOR_BOMBER: Color = Color::srgb(1.0, 0.0, 0.0);
pub const COLOR_POD: Color = Color::srgb(1.0, 0.0, 0.0);
pub const COLOR_SWARMER: Color = Color::srgb(1.0, 0.5, 0.0);
pub const COLOR_BAITER: Color = Color::srgb(1.0, 1.0, 0.0);
pub const COLOR_HUMAN: Color = Color::srgb(0.2, 0.8, 0.2);
pub const COLOR_PLAYER_PROJECTILE: Color = Color::srgb(1.0, 1.0, 1.0);
pub const COLOR_ENEMY_PROJECTILE: Color = Color::srgb(1.0, 0.3, 0.3);
pub const COLOR_MINE: Color = Color::srgb(1.0, 1.0, 0.0);
pub const COLOR_TERRAIN: Color = Color::srgb(0.4, 0.25, 0.1);
pub const COLOR_SCANNER_BG: Color = Color::srgba(0.1, 0.1, 0.2, 0.8);
