use bevy::prelude::*;

use crate::constants::*;
use crate::states::AppState;
use crate::tetromino::TetrominoKind;

// ---------------------------------------------------------------------------
// Messages: gameplay -> stats
// ---------------------------------------------------------------------------

#[derive(Message)]
pub struct LineClearMsg {
    pub rows: Vec<usize>,
}

#[derive(Message)]
pub struct HardDropMsg(pub u32);

#[derive(Message)]
pub struct SoftDropMsg(pub u32);

// Message: stats -> gameplay
#[derive(Message)]
pub struct LevelChangedMsg(pub u32);

// Message: gameplay -> effects
#[derive(Message)]
pub struct PieceLockedMsg {
    pub cells: [(i32, i32); 4],
}

// ---------------------------------------------------------------------------
// Resources
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

/// Hold slot — stores a piece kind and a one-per-lock flag.
#[derive(Resource)]
pub struct HoldPiece {
    pub piece: Option<TetrominoKind>,
    pub can_hold: bool,
}

impl Default for HoldPiece {
    fn default() -> Self {
        Self {
            piece: None,
            can_hold: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameStats>()
            .init_resource::<HoldPiece>()
            .add_message::<LineClearMsg>()
            .add_message::<HardDropMsg>()
            .add_message::<SoftDropMsg>()
            .add_message::<LevelChangedMsg>()
            .add_message::<PieceLockedMsg>()
            .add_systems(Update, process_scoring.run_if(in_state(AppState::Playing)))
            .add_systems(OnEnter(AppState::Playing), reset_stats);
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
        stats.award_line_clear(msg.rows.len() as u32);
        if stats.level != prev_level {
            level_changed.write(LevelChangedMsg(stats.level));
        }
    }
}

fn reset_stats(mut stats: ResMut<GameStats>, mut hold: ResMut<HoldPiece>) {
    *stats = GameStats::default();
    *hold = HoldPiece::default();
}
