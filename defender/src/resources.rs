use bevy::prelude::*;

use crate::constants::*;

#[derive(Resource)]
pub struct GameState {
    pub score: u32,
    pub lives: u32,
    pub smart_bombs: u32,
    pub current_wave: u32,
    pub next_extra_life_score: u32,
    pub humans_alive: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: STARTING_LIVES,
            smart_bombs: STARTING_SMART_BOMBS,
            current_wave: 0,
            next_extra_life_score: EXTRA_LIFE_INTERVAL,
            humans_alive: HUMANS_PER_WAVE,
        }
    }
}

#[derive(Resource)]
pub struct WaveState {
    pub landers_to_spawn: u32,
    pub bombers_to_spawn: u32,
    pub pods_to_spawn: u32,
    pub spawn_timer: Timer,
    pub baiter_timer: Timer,
    pub baiter_interval: Timer,
    pub baiters_active: bool,
    pub wave_active: bool,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            landers_to_spawn: 0,
            bombers_to_spawn: 0,
            pods_to_spawn: 0,
            spawn_timer: Timer::from_seconds(ENEMY_SPAWN_INTERVAL, TimerMode::Repeating),
            baiter_timer: Timer::from_seconds(BAITER_SPAWN_DELAY, TimerMode::Once),
            baiter_interval: Timer::from_seconds(BAITER_SPAWN_INTERVAL, TimerMode::Repeating),
            baiters_active: false,
            wave_active: false,
        }
    }
}

#[derive(Resource)]
pub struct TerrainData {
    pub points: Vec<Vec2>,
}

#[derive(Resource)]
pub struct CameraWorldPos(pub f32);

impl Default for CameraWorldPos {
    fn default() -> Self {
        Self(WORLD_WIDTH / 2.0)
    }
}

#[derive(Resource)]
pub struct WaveIntroTimer(pub Timer);

#[derive(Resource)]
pub struct DeathTimer(pub Timer);
