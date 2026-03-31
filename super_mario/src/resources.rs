use bevy::prelude::*;

use crate::components::PlayerSize;
use crate::constants::{COINS_PER_LIFE, TIMER_START};

#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub coins: u32,
    pub lives: u32,
    pub world_name: String,
    pub timer: f32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            coins: 0,
            lives: 3,
            world_name: "1-1".to_string(),
            timer: TIMER_START,
        }
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
