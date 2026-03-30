use bevy::prelude::*;

use crate::constants::*;

// ---------------------------------------------------------------------------
// Messages: gameplay → stats
// ---------------------------------------------------------------------------

#[derive(Message)]
pub struct LineClearMsg(pub u32);

#[derive(Message)]
pub struct HardDropMsg(pub u32);

#[derive(Message)]
pub struct SoftDropMsg(pub u32);

// Message: stats → gameplay
#[derive(Message)]
pub struct LevelChangedMsg(pub u32);

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks score, level, and lines cleared.
#[derive(Resource)]
pub struct GameStats {
    pub score: u32,
    pub level: u32,
    pub lines_cleared: u32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            score: 0,
            level: STARTING_LEVEL,
            lines_cleared: 0,
        }
    }
}

impl GameStats {
    fn award_line_clear(&mut self, lines: u32) {
        let base = match lines {
            1 => SCORE_SINGLE,
            2 => SCORE_DOUBLE,
            3 => SCORE_TRIPLE,
            4 => SCORE_TETRIS,
            _ => return,
        };
        self.score += base * self.level;
        self.lines_cleared += lines;
        self.level = STARTING_LEVEL + self.lines_cleared / LINES_PER_LEVEL;
    }

    fn award_soft_drop(&mut self, rows: u32) {
        self.score += SOFT_DROP_SCORE_PER_ROW * rows;
    }

    fn award_hard_drop(&mut self, rows: u32) {
        self.score += HARD_DROP_SCORE_PER_ROW * rows;
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameStats>()
            .add_message::<LineClearMsg>()
            .add_message::<HardDropMsg>()
            .add_message::<SoftDropMsg>()
            .add_message::<LevelChangedMsg>()
            .add_systems(Update, process_scoring);
    }
}

fn process_scoring(
    mut stats: ResMut<GameStats>,
    mut line_clears: MessageReader<LineClearMsg>,
    mut hard_drops: MessageReader<HardDropMsg>,
    mut soft_drops: MessageReader<SoftDropMsg>,
    mut level_changed: MessageWriter<LevelChangedMsg>,
) {
    for msg in hard_drops.read() {
        stats.award_hard_drop(msg.0);
    }
    for msg in soft_drops.read() {
        stats.award_soft_drop(msg.0);
    }
    for msg in line_clears.read() {
        let prev_level = stats.level;
        stats.award_line_clear(msg.0);
        if stats.level != prev_level {
            level_changed.write(LevelChangedMsg(stats.level));
        }
    }
}
