#![allow(dead_code)]

use crate::constants::*;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Goomba,
    Koopa,
}

#[derive(Clone)]
pub struct EnemySpawnData {
    pub tile_x: usize,
    pub tile_y: usize,
    pub kind: EnemyKind,
}

#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub coins: u32,
    pub lives: u32,
    pub timer: u32,
    pub world_label: String,
}

#[derive(Resource, Default)]
pub struct LevelState {
    pub width_tiles: usize,
    pub height_tiles: usize,
    pub player_start: Vec2,
    pub camera_min_x: f32,
    pub camera_max_x: f32,
    pub camera_y: f32,
    pub enemy_spawns: Vec<EnemySpawnData>,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            coins: 0,
            lives: INITIAL_LIVES,
            timer: INITIAL_TIMER,
            world_label: WORLD_LABEL_1_1.to_string(),
        }
    }
}

impl GameData {
    pub fn add_coin(&mut self) {
        self.coins += 1;
        self.score += SCORE_COIN;
        if self.coins >= COINS_FOR_1UP {
            self.coins -= COINS_FOR_1UP;
            self.lives += 1;
        }
    }

    pub fn reset_for_new_game(&mut self) {
        self.score = 0;
        self.coins = 0;
        self.lives = INITIAL_LIVES;
        self.timer = INITIAL_TIMER;
        self.world_label = WORLD_LABEL_1_1.to_string();
    }

    pub fn reset_timer(&mut self) {
        self.timer = INITIAL_TIMER;
    }
}
