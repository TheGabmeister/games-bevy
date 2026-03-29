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

impl GameData {
    pub fn reset_for_new_run(&mut self) {
        self.score = 0;
        self.lives = STARTING_LIVES;
        self.level = 1;
        self.filled_bays.fill(false);
        self.max_row_this_life = 0;
    }

    pub fn add_score(&mut self, points: u32) {
        self.score += points;
        self.high_score = self.high_score.max(self.score);
    }

    pub fn lose_life(&mut self) -> bool {
        self.lives = self.lives.saturating_sub(1);
        self.lives == 0
    }

    pub fn reset_life_progress(&mut self) {
        self.max_row_this_life = 0;
    }

    pub fn complete_level(&mut self) {
        self.add_score(SCORE_LEVEL_CLEAR);
        self.level += 1;
        self.filled_bays.fill(false);
        self.max_row_this_life = 0;
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

impl FrogTimer {
    pub fn reset(&mut self) {
        self.remaining_secs = LIFE_TIMER_SECS;
    }
}

#[derive(Resource)]
pub struct LevelState {
    pub speed_multiplier: f32,
    pub celebrating: bool,
    pub celebration_timer: Timer,
}

impl Default for LevelState {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            celebrating: false,
            celebration_timer: Timer::from_seconds(LEVEL_CLEAR_DELAY_SECS, TimerMode::Once),
        }
    }
}

impl LevelState {
    pub fn reset_for_new_run(&mut self) {
        self.speed_multiplier = 1.0;
        self.celebrating = false;
        self.celebration_timer = Timer::from_seconds(LEVEL_CLEAR_DELAY_SECS, TimerMode::Once);
    }

    pub fn start_level_clear(&mut self) {
        self.celebrating = true;
        self.celebration_timer = Timer::from_seconds(LEVEL_CLEAR_DELAY_SECS, TimerMode::Once);
    }

    pub fn finish_level_clear(&mut self) {
        self.celebrating = false;
    }

    pub fn advance_speed(&mut self) {
        self.speed_multiplier = (self.speed_multiplier + SPEED_INCREMENT).min(MAX_SPEED_MULTIPLIER);
    }
}

#[derive(Resource, Default, PartialEq, Clone, Copy)]
pub enum FrogEvent {
    #[default]
    None,
    Death,
    BayFilled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_score_updates_high_score_immediately() {
        let mut game_data = GameData::default();
        game_data.add_score(250);

        assert_eq!(game_data.score, 250);
        assert_eq!(game_data.high_score, 250);
    }

    #[test]
    fn level_speed_caps_at_maximum_multiplier() {
        let mut level_state = LevelState::default();
        for _ in 0..20 {
            level_state.advance_speed();
        }

        assert_eq!(level_state.speed_multiplier, MAX_SPEED_MULTIPLIER);
    }
}
