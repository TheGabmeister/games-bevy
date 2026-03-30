use bevy::prelude::*;

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
    pub is_big: bool,
}

#[derive(Resource, Default)]
pub struct PendingBlockHit {
    pub hit: Option<BlockHitInfo>,
}

// Player mesh handles for size switching
#[derive(Resource)]
pub struct PlayerMeshes {
    pub small: Handle<Mesh>,
    pub big: Handle<Mesh>,
}
