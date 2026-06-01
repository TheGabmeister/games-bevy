use bevy::prelude::*;

/// Player score for the current run, plus the best score seen this session.
#[derive(Resource, Default)]
pub struct Score {
    pub current: u32,
    pub high: u32,
}

impl Score {
    /// Adds points and keeps the high score in sync.
    pub fn add(&mut self, points: u32) {
        self.current += points;
        self.high = self.high.max(self.current);
    }
}

/// The current round number (1-based). Drives which brick layout is spawned.
#[derive(Resource)]
pub struct Round(pub u32);

impl Default for Round {
    fn default() -> Self {
        Round(1)
    }
}
