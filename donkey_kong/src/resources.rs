use bevy::prelude::*;

use crate::constants::*;

// --- Session Data (persists across runs) ---

#[derive(Resource, Default)]
pub struct SessionData {
    pub high_score: u32,
}

// --- Run Data (per-run state) ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayingEntry {
    NewRun,
    RetryAfterDeath,
    NextWave,
}

#[derive(Resource)]
pub struct RunData {
    pub score: u32,
    pub lives: u8,
    pub wave: u8,
    pub extra_life_awarded: bool,
    pub next_entry: PlayingEntry,
}

impl Default for RunData {
    fn default() -> Self {
        Self {
            score: 0,
            lives: STARTING_LIVES,
            wave: 1,
            extra_life_awarded: false,
            next_entry: PlayingEntry::NewRun,
        }
    }
}

// --- Wave Runtime ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BonusItemStatus {
    Pending,
    Active,
    Collected,
    Expired,
}

#[derive(Resource)]
pub struct WaveRuntime {
    pub bonus_timer: i32,
    pub bonus_tick: f32,
    pub elapsed_wave_time: f32,
    pub bonus_items: [BonusItemStatus; 3],
    pub rng: SimpleRng,
}

impl Default for WaveRuntime {
    fn default() -> Self {
        Self {
            bonus_timer: BONUS_TIMER_START,
            bonus_tick: 0.0,
            elapsed_wave_time: 0.0,
            bonus_items: [BonusItemStatus::Pending; 3],
            rng: SimpleRng::new(12345),
        }
    }
}

// --- Death Sequence ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeathCause {
    Barrel,
    Fireball,
    Fall,
    Timer,
}

#[derive(Resource)]
pub struct DeathSequence {
    pub elapsed: f32,
    pub cause: DeathCause,
}

// --- Wave Config ---

#[derive(Resource)]
pub struct WaveConfig {
    pub throw_interval: f32,
    pub blue_barrel_every: u32,
    pub barrel_speed_mult: f32,
    pub max_fireballs: u32,
    pub fireball_speed_mult: f32,
}

impl WaveConfig {
    pub fn from_wave(wave: u8) -> Self {
        let idx = (wave as usize).saturating_sub(1).min(WAVE_CONFIGS.len() - 1);
        let cfg = &WAVE_CONFIGS[idx];
        Self {
            throw_interval: cfg.throw_interval,
            blue_barrel_every: cfg.blue_barrel_every,
            barrel_speed_mult: cfg.barrel_speed_mult,
            max_fireballs: cfg.max_fireballs,
            fireball_speed_mult: cfg.fireball_speed_mult,
        }
    }
}

// --- Shared Mesh & Material Handles ---

#[derive(Resource)]
pub struct GameMeshes {
    pub player: Handle<Mesh>,
    pub barrel: Handle<Mesh>,
    pub fireball: Handle<Mesh>,
    pub hammer_pickup: Handle<Mesh>,
    pub bonus_item: Handle<Mesh>,
}

#[derive(Resource)]
pub struct GameMaterials {
    pub player_normal: Handle<ColorMaterial>,
    pub player_hammer: Handle<ColorMaterial>,
    pub barrel: Handle<ColorMaterial>,
    pub blue_barrel: Handle<ColorMaterial>,
    pub fireball: Handle<ColorMaterial>,
    pub hammer_pickup: Handle<ColorMaterial>,
    pub bonus_item: Handle<ColorMaterial>,
}

// --- Simple RNG ---

pub struct SimpleRng {
    pub state: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        Self { state: if seed == 0 { 1 } else { seed } }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        (self.next_u32() % 10000) < (probability * 10000.0) as u32
    }
}
