use bevy::prelude::*;

use crate::constants::TIMER_START;

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
