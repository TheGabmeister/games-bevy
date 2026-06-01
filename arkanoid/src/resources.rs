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

/// Remaining Vaus lives. Reaching zero ends the run.
#[derive(Resource)]
pub struct Lives(pub u32);

impl Default for Lives {
    fn default() -> Self {
        Lives(crate::constants::LIVES_START)
    }
}

/// The ball's current speed and the state driving its in-round acceleration. Reset to
/// `BALL_SPEED` on every serve; ramps up over time and at brick milestones (see `ball.rs`).
#[derive(Resource)]
pub struct BallSpeed {
    /// Current speed magnitude in pixels/second.
    pub current: f32,
    /// Repeating timer for time-based speed bumps.
    pub timer: Timer,
    /// Bricks destroyed since the last serve, for milestone bumps.
    pub bricks_destroyed: u32,
}

impl Default for BallSpeed {
    fn default() -> Self {
        BallSpeed {
            current: crate::constants::BALL_SPEED,
            timer: Timer::from_seconds(
                crate::constants::BALL_SPEEDUP_INTERVAL,
                TimerMode::Repeating,
            ),
            bricks_destroyed: 0,
        }
    }
}
