use bevy::prelude::*;
use crate::constants::*;

#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub coins: u32,
    pub lives: u32,
    pub timer: u32,
    pub world_label: String,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            coins: 0,
            lives: INITIAL_LIVES,
            timer: INITIAL_TIMER,
            world_label: "1-1".to_string(),
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
    }

    pub fn reset_timer(&mut self) {
        self.timer = INITIAL_TIMER;
    }
}
