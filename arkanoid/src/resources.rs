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

/// The exclusive paddle power-up currently active. Catching a new paddle power-up replaces
/// the previous one; a lost life resets it to `Normal`. (Slow, Disruption, Break, and
/// Player are not paddle modes — they apply instantly or coexist.)
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PaddleMode {
    #[default]
    Normal,
    Catch,
    Laser,
    Expand,
}

/// Drives the deterministic capsule-drop schedule: a capsule is released every
/// `CAPSULE_DROP_INTERVAL` bricks (when none is already falling), cycling through `SEQUENCE`.
#[derive(Resource, Default)]
pub struct CapsuleDirector {
    /// Bricks destroyed since the last capsule was released.
    pub bricks_destroyed: u32,
    /// Index into the power-up sequence for the next drop.
    pub next: usize,
}

/// Cadence timer for enemy spawning while a round runs; reset at each round's start.
#[derive(Resource)]
pub struct EnemySpawnTimer(pub Timer);

impl Default for EnemySpawnTimer {
    fn default() -> Self {
        EnemySpawnTimer(Timer::from_seconds(
            crate::constants::ENEMY_SPAWN_INTERVAL,
            TimerMode::Repeating,
        ))
    }
}

/// Counts enemies spawned this round to deterministically rotate enemy type and spawn gate.
#[derive(Resource, Default)]
pub struct EnemyDirector {
    pub spawned: u32,
}
