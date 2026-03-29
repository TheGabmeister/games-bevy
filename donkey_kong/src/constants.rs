use bevy::prelude::*;

// Window
pub const WINDOW_WIDTH: f32 = 672.0;
pub const WINDOW_HEIGHT: f32 = 768.0;
pub const WINDOW_TITLE: &str = "Donkey Kong";

// Playfield (logical coordinates)
pub const PLAYFIELD_WIDTH: f32 = 224.0;
pub const PLAYFIELD_HEIGHT: f32 = 256.0;
pub const PLAYFIELD_X_MIN: f32 = -100.0;
pub const PLAYFIELD_X_MAX: f32 = 100.0;

// Colors
pub const BG_COLOR: Color = Color::srgb(0.039, 0.039, 0.180);
pub const GIRDER_COLOR: Color = Color::srgb(0.816, 0.188, 0.188);
pub const LADDER_COLOR: Color = Color::srgb(0.0, 0.910, 0.910);
pub const LADDER_BROKEN_COLOR: Color = Color::srgb(0.0, 0.55, 0.55);
pub const PLAYER_COLOR: Color = Color::srgb(0.878, 0.251, 0.251);
pub const PLAYER_HAMMER_COLOR: Color = Color::srgb(1.0, 0.843, 0.0);
pub const DK_COLOR: Color = Color::srgb(0.545, 0.271, 0.075);
pub const PAULINE_COLOR: Color = Color::srgb(1.0, 0.412, 0.706);
pub const BARREL_COLOR: Color = Color::srgb(0.804, 0.522, 0.247);
pub const BLUE_BARREL_COLOR: Color = Color::srgb(0.255, 0.412, 0.882);
pub const FIREBALL_COLOR: Color = Color::srgb(1.0, 0.271, 0.0);
pub const OIL_DRUM_COLOR: Color = Color::srgb(0.251, 0.251, 0.251);
pub const HAMMER_PICKUP_COLOR: Color = Color::srgb(1.0, 0.843, 0.0);
pub const BONUS_ITEM_COLOR: Color = Color::srgb(0.0, 1.0, 0.5);
pub const TEXT_COLOR: Color = Color::WHITE;

// Player
pub const PLAYER_WIDTH: f32 = 16.0;
pub const PLAYER_HEIGHT: f32 = 22.0;
pub const PLAYER_WALK_SPEED: f32 = 60.0;
pub const PLAYER_CLIMB_SPEED: f32 = 40.0;
pub const PLAYER_GRAVITY: f32 = 600.0;
pub const PLAYER_JUMP_IMPULSE: f32 = 220.0;
pub const PLAYER_FALL_DEATH_THRESHOLD: f32 = 36.0;

// Girder & Ladder
pub const GIRDER_THICKNESS: f32 = 8.0;
pub const LADDER_WIDTH: f32 = 16.0;
pub const LADDER_GRAB_HALF_WIDTH: f32 = 8.0;
pub const GIRDER_SUPPORT_TOLERANCE: f32 = 4.0;

// Barrel
pub const BARREL_RADIUS: f32 = 7.0;
pub const BARREL_BASE_SPEED: f32 = 80.0;
pub const BARREL_FALL_SPEED: f32 = 200.0;
pub const BARREL_LADDER_CHANCE: f32 = 0.30;
pub const WILD_BARREL_CHANCE: f32 = 0.10;
pub const WILD_BOUNCE_AMPLITUDE: f32 = 4.0;
pub const WILD_BOUNCE_FREQ: f32 = 3.0;

// Fireball
pub const FIREBALL_RADIUS: f32 = 5.0;
pub const FIREBALL_BASE_SPEED: f32 = 40.0;
pub const FIREBALL_CLIMB_SPEED: f32 = 30.0;
pub const FIREBALL_SPAWN_CHANCE: f32 = 0.50;
pub const FIREBALL_PURSUIT_BIAS: f32 = 0.70;
pub const FIREBALL_LADDER_BIAS: f32 = 0.70;

// DK
pub const DK_WIDTH: f32 = 40.0;
pub const DK_HEIGHT: f32 = 36.0;
pub const DK_WINDUP_DURATION: f32 = 0.5;
pub const DK_THROW_DURATION: f32 = 0.3;

// Pauline
pub const PAULINE_WIDTH: f32 = 12.0;
pub const PAULINE_HEIGHT: f32 = 20.0;

// Oil Drum
pub const OIL_DRUM_WIDTH: f32 = 20.0;
pub const OIL_DRUM_HEIGHT: f32 = 24.0;

// Hammer
pub const HAMMER_PICKUP_SIZE: f32 = 12.0;
pub const HAMMER_DURATION: f32 = 10.0;
pub const HAMMER_FLASH_TIME: f32 = 3.0;
pub const HAMMER_HIT_FORWARD: f32 = 20.0;
pub const HAMMER_HIT_UP: f32 = 12.0;

// Goal Zone
pub const GOAL_ZONE_WIDTH: f32 = 24.0;
pub const GOAL_ZONE_HEIGHT: f32 = 28.0;

// Bonus Item
pub const BONUS_ITEM_SIZE: f32 = 10.0;

// Scoring
pub const SCORE_JUMP_ONE: u32 = 100;
pub const SCORE_JUMP_MULTI: u32 = 300;
pub const SCORE_SMASH_BARREL: u32 = 300;
pub const SCORE_SMASH_FIREBALL: u32 = 500;
pub const JUMP_SCORE_HALF_WIDTH: f32 = 16.0;

// Bonus Timer
pub const BONUS_TIMER_START: i32 = 5000;
pub const BONUS_TIMER_DECREASE: i32 = 100;
pub const BONUS_TIMER_INTERVAL: f32 = 2.0;

// Lives
pub const STARTING_LIVES: u8 = 3;
pub const EXTRA_LIFE_SCORE: u32 = 10000;
pub const MAX_LIVES: u8 = 5;

// Death sequence
pub const DEATH_FLASH_DURATION: f32 = 1.0;
pub const DEATH_HOLD_DURATION: f32 = 1.0;

// Bonus items
pub const BONUS_ITEM_TIMES: [f32; 3] = [20.0, 40.0, 60.0];
pub const BONUS_ITEM_VALUES: [u32; 3] = [300, 500, 800];
pub const BONUS_ITEM_DURATION: f32 = 15.0;

// UI
pub const FONT_SIZE_TITLE: f32 = 40.0;
pub const FONT_SIZE_BODY: f32 = 24.0;
pub const FONT_SIZE_HUD: f32 = 16.0;

// Wave configs
#[derive(Clone, Copy)]
pub struct WaveConfigData {
    pub throw_interval: f32,
    pub blue_barrel_every: u32,
    pub barrel_speed_mult: f32,
    pub max_fireballs: u32,
    pub fireball_speed_mult: f32,
}

pub const WAVE_CONFIGS: [WaveConfigData; 5] = [
    WaveConfigData { throw_interval: 3.0, blue_barrel_every: 5, barrel_speed_mult: 1.00, max_fireballs: 2, fireball_speed_mult: 1.00 },
    WaveConfigData { throw_interval: 2.5, blue_barrel_every: 4, barrel_speed_mult: 1.10, max_fireballs: 3, fireball_speed_mult: 1.00 },
    WaveConfigData { throw_interval: 2.0, blue_barrel_every: 4, barrel_speed_mult: 1.20, max_fireballs: 4, fireball_speed_mult: 1.10 },
    WaveConfigData { throw_interval: 1.8, blue_barrel_every: 3, barrel_speed_mult: 1.30, max_fireballs: 5, fireball_speed_mult: 1.15 },
    WaveConfigData { throw_interval: 1.5, blue_barrel_every: 3, barrel_speed_mult: 1.40, max_fireballs: 5, fireball_speed_mult: 1.25 },
];
