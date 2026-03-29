use bevy::prelude::*;

use crate::constants::STARTING_LIVES;

#[derive(Resource)]
pub struct GameState {
    pub score: u32,
    pub lives: u32,
    pub current_level: usize,
    pub total_gold: u32,
    pub exit_unlocked: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: STARTING_LIVES,
            current_level: 0,
            total_gold: 0,
            exit_unlocked: false,
        }
    }
}

/// Death timer resource (counts down before level reset).
#[derive(Resource)]
pub struct DeathTimer(pub Timer);
