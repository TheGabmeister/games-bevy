use bevy::prelude::*;
use std::collections::HashMap;

// ── Game state data ──────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct Score {
    pub value: u32,
}

#[derive(Resource)]
pub struct Lives(pub u32);

impl Default for Lives {
    fn default() -> Self {
        Lives(crate::constants::INITIAL_LIVES)
    }
}

#[derive(Resource, Default)]
pub struct Wave(pub u32);

/// Auto-incrementing ID for centipede chains (so splits get unique IDs)
#[derive(Resource, Default)]
pub struct NextChainId(pub u32);

impl NextChainId {
    pub fn next(&mut self) -> u32 {
        let id = self.0;
        self.0 += 1;
        id
    }
}

// ── Centipede chain tracking ─────────────────────────────────────────────────

/// Maps chain_id → ordered Vec<Entity> (index 0 = head)
#[derive(Resource, Default)]
pub struct CentipedeChains(pub HashMap<u32, Vec<Entity>>);

// ── Mushroom grid lookup ─────────────────────────────────────────────────────

/// Maps (col, row) → mushroom Entity for fast collision lookups
#[derive(Resource, Default)]
pub struct MushroomGrid(pub HashMap<(i32, i32), Entity>);

// ── Timers ───────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct CentipedeTimer(pub Timer);

impl CentipedeTimer {
    pub fn new(interval: f32) -> Self {
        CentipedeTimer(Timer::from_seconds(interval, TimerMode::Repeating))
    }
}

#[derive(Resource)]
pub struct FleaSpawnTimer(pub Timer);

impl Default for FleaSpawnTimer {
    fn default() -> Self {
        FleaSpawnTimer(Timer::from_seconds(
            crate::constants::FLEA_SPAWN_INTERVAL,
            TimerMode::Repeating,
        ))
    }
}

#[derive(Resource)]
pub struct SpiderSpawnTimer(pub Timer);

impl Default for SpiderSpawnTimer {
    fn default() -> Self {
        SpiderSpawnTimer(Timer::from_seconds(
            crate::constants::SPIDER_SPAWN_INTERVAL,
            TimerMode::Repeating,
        ))
    }
}

#[derive(Resource)]
pub struct ScorpionSpawnTimer(pub Timer);

impl Default for ScorpionSpawnTimer {
    fn default() -> Self {
        ScorpionSpawnTimer(Timer::from_seconds(
            crate::constants::SCORPION_SPAWN_INTERVAL,
            TimerMode::Repeating,
        ))
    }
}

/// When Some, the player is dead and waiting to respawn
#[derive(Resource, Default)]
pub struct RespawnTimer(pub Option<Timer>);

#[derive(Resource, Default)]
pub struct NextExtraLifeScore(pub u32);
