use bevy::prelude::*;

use crate::components::PlayerSize;
use crate::constants::{COINS_PER_LIFE, TIMER_START};

// ── Game Messages ──

/// Emitted when points are awarded (stomp, powerup, fireball, etc.).
/// Consumed by `apply_game_events` which updates `GameData.score`.
#[derive(Message)]
pub struct ScoreEvent {
    pub points: u32,
}

/// Emitted when a coin is collected (block or floating).
/// Consumed by `apply_game_events` which increments coins and awards extra lives.
#[derive(Message)]
pub struct CoinEvent;

// ── Resources ──

#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub coins: u32,
    pub lives: u32,
    pub world_name: String,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            coins: 0,
            lives: 3,
            world_name: "1-1".to_string(),
        }
    }
}

#[derive(Resource)]
pub struct GameTimer {
    pub time: f32,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self { time: TIMER_START }
    }
}

#[derive(Resource, Default)]
pub struct SpawnPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Resource)]
pub struct DeathAnimation {
    pub phase: DeathPhase,
    pub timer: Timer,
    pub pit_death: bool,
}

pub enum DeathPhase {
    Pause,
    Bouncing,
}

impl GameData {
    pub fn add_coin(&mut self) {
        self.coins += 1;
        if self.coins >= COINS_PER_LIFE {
            self.coins -= COINS_PER_LIFE;
            self.lives += 1;
        }
    }
}

// Pending block hit from player head collision
pub struct BlockHitInfo {
    pub col: i32,
    pub row: i32,
    pub player_size: PlayerSize,
}

#[derive(Resource, Default)]
pub struct PendingBlockHit {
    pub hit: Option<BlockHitInfo>,
}

// Level complete animation state
#[derive(Resource)]
pub struct LevelCompleteAnimation {
    pub phase: LevelCompletePhase,
    pub pole_x: f32,
    pub pole_base_y: f32,
    pub castle_x: f32,
    pub done_timer: Timer,
    pub flagpole_score: u32,
}

pub enum LevelCompletePhase {
    SlideDown,
    WalkToCastle,
    TimeTally,
    Done,
}

// Level list for multi-level progression
#[derive(Resource)]
pub struct LevelList {
    pub paths: Vec<String>,
    pub current: usize,
}

impl Default for LevelList {
    fn default() -> Self {
        Self {
            paths: vec![
                "levels/1-1.level.ron".to_string(),
                "levels/1-2.level.ron".to_string(),
            ],
            current: 0,
        }
    }
}

impl LevelList {
    pub fn current_path(&self) -> String {
        self.paths[self.current].clone()
    }

    pub fn advance(&mut self) {
        self.current = (self.current + 1) % self.paths.len();
    }
}

// Level transition timer
#[derive(Resource)]
pub struct LevelTransitionTimer(pub Timer);
