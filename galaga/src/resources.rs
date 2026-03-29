use bevy::prelude::*;

use crate::constants::PLAYER_LIVES;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WavePhase {
    #[default]
    Spawning,
    Combat,
    Respawning,
    StageClear,
}

#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub wave: u32,
    pub lives: u32,
    pub phase: WavePhase,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            wave: 1,
            lives: PLAYER_LIVES,
            phase: WavePhase::default(),
        }
    }
}

#[derive(Resource)]
pub struct DiveTimer(pub Timer);

#[derive(Resource)]
pub struct RespawnTimer(pub Timer);

#[derive(Resource)]
pub struct StageClearTimer(pub Timer);

#[derive(Resource, Default)]
pub struct FormationSway {
    pub time: f32,
}

#[derive(Resource, Default)]
pub struct DiveSelectionCursor {
    pub next_index: usize,
}
