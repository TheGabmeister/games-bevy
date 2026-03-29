use bevy::prelude::*;

use crate::constants::*;

#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub high_score: u32,
    pub lives: u32,
    pub level: u32,
    pub filled_bays: [bool; 5],
    pub max_row_this_life: i32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            high_score: 0,
            lives: STARTING_LIVES,
            level: 1,
            filled_bays: [false; 5],
            max_row_this_life: 0,
        }
    }
}

#[derive(Resource)]
pub struct FrogTimer {
    pub remaining_secs: f32,
}

impl Default for FrogTimer {
    fn default() -> Self {
        Self {
            remaining_secs: LIFE_TIMER_SECS,
        }
    }
}

#[derive(Resource)]
pub struct LevelState {
    pub speed_multiplier: f32,
    pub celebrating: bool,
}

impl Default for LevelState {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            celebrating: false,
        }
    }
}

#[derive(Resource, Default, PartialEq, Clone, Copy)]
pub enum FrogEvent {
    #[default]
    None,
    Death,
    BayFilled,
}
