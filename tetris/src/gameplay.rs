use bevy::prelude::*;

use crate::board::Board;
use crate::constants::*;
use crate::input::InputActions;
use crate::resources::{
    HardDropMsg, HoldPiece, LevelChangedMsg, LineClearMsg, PieceLockedMsg, SoftDropMsg,
};
use crate::states::{AppState, PlayState};
use crate::tetromino::{ActivePiece, PieceBag, RotationState, TetrominoKind};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn can_place(board: &Board, piece: &ActivePiece, dr: i32, dc: i32) -> bool {
    let cells = piece
        .kind
        .cells(piece.rotation)
        .map(|(r, c)| (piece.row + dr + r, piece.col + dc + c));
    board.is_valid_cells(&cells)
}

fn spawn_next_piece(piece: &mut ActivePiece, bag: &mut PieceBag) {
    let kind = bag.draw();
    piece.kind = kind;
    piece.rotation = RotationState::R0;
    piece.row = kind.spawn_row();
    piece.col = SPAWN_COL;
}

struct LockResult {
    cleared_rows: Vec<usize>,
    game_over: bool,
}

/// Lock the active piece, clear lines, spawn next piece, detect game over.
fn lock_and_spawn(
    piece: &mut ActivePiece,
    board: &mut Board,
    bag: &mut PieceBag,
    gravity: &mut GravityTimer,
    lock: &mut LockDelayState,
    hold: &mut HoldPiece,
) -> LockResult {
    let cells = piece.board_cells();
    board.lock_cells(&cells, piece.kind);

    // Lock out: entire piece above visible area.
    let lock_out = cells
        .iter()
        .all(|&(r, _)| r as usize >= GRID_VISIBLE_ROWS);

    let cleared_rows = board.clear_full_rows();

    spawn_next_piece(piece, bag);
    hold.can_hold = true;

    // Block out: new piece overlaps filled cells.
    let block_out = !board.is_valid_cells(&piece.board_cells());

    gravity.elapsed = 0.0;
    lock.elapsed = 0.0;
    lock.resets = 0;
    lock.prev_col = piece.col;
    lock.prev_rotation = piece.rotation;

    LockResult {
        cleared_rows,
        game_over: lock_out || block_out,
    }
}

/// Lock the active piece, emit messages, check game over, and spawn the next piece.
fn perform_lock(
    piece: &mut ActivePiece,
    board: &mut Board,
    bag: &mut PieceBag,
    gravity: &mut GravityTimer,
    lock: &mut LockDelayState,
    hold: &mut HoldPiece,
    line_clear_msgs: &mut MessageWriter<LineClearMsg>,
    piece_locked_msgs: &mut MessageWriter<PieceLockedMsg>,
    next_app_state: &mut NextState<AppState>,
) {
    let locked_cells = piece.board_cells();
    piece_locked_msgs.write(PieceLockedMsg {
        cells: locked_cells,
    });

    let result = lock_and_spawn(piece, board, bag, gravity, lock, hold);
    if !result.cleared_rows.is_empty() {
        line_clear_msgs.write(LineClearMsg {
            rows: result.cleared_rows,
        });
    }
    if result.game_over {
        next_app_state.set(AppState::GameOver);
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum HorizontalDir {
    Left,
    Right,
}

#[derive(Resource)]
struct DasState {
    direction: Option<HorizontalDir>,
    elapsed: f32,
    charged: bool,
}

impl Default for DasState {
    fn default() -> Self {
        Self {
            direction: None,
            elapsed: 0.0,
            charged: false,
        }
    }
}

#[derive(Resource)]
pub struct GravityTimer {
    pub elapsed: f32,
    pub interval: f32,
}

impl Default for GravityTimer {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            interval: 1.0,
        }
    }
}

#[derive(Resource)]
struct LockDelayState {
    elapsed: f32,
    resets: u32,
    prev_col: i32,
    prev_rotation: RotationState,
}

impl Default for LockDelayState {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            resets: 0,
            prev_col: SPAWN_COL,
            prev_rotation: RotationState::R0,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DasState>()
            .init_resource::<GravityTimer>()
            .init_resource::<LockDelayState>()
            .add_systems(OnEnter(AppState::Playing), reset_gameplay)
            .add_systems(
                Update,
                (
                    handle_hold,
                    handle_horizontal_input,
                    handle_rotation,
                    handle_hard_drop,
                    handle_gravity,
                    handle_lock_delay,
                    handle_level_change,
                )
                    .chain()
                    .run_if(in_state(PlayState::Running)),
            );
    }
}

fn reset_gameplay(
    mut gravity: ResMut<GravityTimer>,
    mut das: ResMut<DasState>,
    mut lock: ResMut<LockDelayState>,
) {
    *gravity = GravityTimer::default();
    *das = DasState::default();
    *lock = LockDelayState::default();
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn handle_hold(
    actions: Res<InputActions>,
    mut piece: ResMut<ActivePiece>,
    mut hold: ResMut<HoldPiece>,
    mut bag: ResMut<PieceBag>,
    mut gravity: ResMut<GravityTimer>,
    mut lock: ResMut<LockDelayState>,
) {
    if !actions.hold || !hold.can_hold {
        return;
    }

    let current_kind = piece.kind;

    if let Some(held_kind) = hold.piece {
        piece.kind = held_kind;
    } else {
        piece.kind = bag.draw();
    }

    hold.piece = Some(current_kind);
    hold.can_hold = false;

    piece.rotation = RotationState::R0;
    piece.row = piece.kind.spawn_row();
    piece.col = SPAWN_COL;

    gravity.elapsed = 0.0;
    lock.elapsed = 0.0;
    lock.resets = 0;
    lock.prev_col = piece.col;
    lock.prev_rotation = piece.rotation;
}

fn handle_horizontal_input(
    actions: Res<InputActions>,
    time: Res<Time>,
    mut das: ResMut<DasState>,
    mut piece: ResMut<ActivePiece>,
    board: Res<Board>,
) {
    let dir = match (actions.move_left, actions.move_right) {
        (true, false) => Some(HorizontalDir::Left),
        (false, true) => Some(HorizontalDir::Right),
        _ => None,
    };

    let dc: i32 = match dir {
        Some(HorizontalDir::Left) => -1,
        Some(HorizontalDir::Right) => 1,
        None => {
            das.direction = None;
            return;
        }
    };

    if das.direction != dir {
        das.direction = dir;
        das.elapsed = 0.0;
        das.charged = false;
        if can_place(&board, &piece, 0, dc) {
            piece.col += dc;
        }
    } else {
        das.elapsed += time.delta_secs();

        if !das.charged {
            if das.elapsed >= DAS_INITIAL_DELAY {
                das.charged = true;
                das.elapsed -= DAS_INITIAL_DELAY;
                if can_place(&board, &piece, 0, dc) {
                    piece.col += dc;
                }
            }
        }

        if das.charged {
            while das.elapsed >= DAS_REPEAT_RATE {
                das.elapsed -= DAS_REPEAT_RATE;
                if can_place(&board, &piece, 0, dc) {
                    piece.col += dc;
                } else {
                    break;
                }
            }
        }
    }
}

fn handle_rotation(
    actions: Res<InputActions>,
    mut piece: ResMut<ActivePiece>,
    board: Res<Board>,
) {
    if piece.kind == TetrominoKind::O {
        return;
    }

    let target = if actions.rotate_cw {
        piece.rotation.cw()
    } else if actions.rotate_ccw {
        piece.rotation.ccw()
    } else {
        return;
    };

    let kicks = piece.kind.kicks(piece.rotation, target);
    for &(dc, dr) in kicks {
        let new_row = piece.row + dr;
        let new_col = piece.col + dc;
        let cells = piece
            .kind
            .cells(target)
            .map(|(r, c)| (new_row + r, new_col + c));
        if board.is_valid_cells(&cells) {
            piece.rotation = target;
            piece.row = new_row;
            piece.col = new_col;
            return;
        }
    }
}

fn handle_hard_drop(
    actions: Res<InputActions>,
    mut piece: ResMut<ActivePiece>,
    mut board: ResMut<Board>,
    mut bag: ResMut<PieceBag>,
    mut gravity: ResMut<GravityTimer>,
    mut lock: ResMut<LockDelayState>,
    mut hold: ResMut<HoldPiece>,
    mut hard_drop_msgs: MessageWriter<HardDropMsg>,
    mut line_clear_msgs: MessageWriter<LineClearMsg>,
    mut piece_locked_msgs: MessageWriter<PieceLockedMsg>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if !actions.hard_drop {
        return;
    }

    let start_row = piece.row;
    while can_place(&board, &piece, -1, 0) {
        piece.row -= 1;
    }
    hard_drop_msgs.write(HardDropMsg((start_row - piece.row) as u32));

    perform_lock(
        &mut piece,
        &mut board,
        &mut bag,
        &mut gravity,
        &mut lock,
        &mut hold,
        &mut line_clear_msgs,
        &mut piece_locked_msgs,
        &mut next_app_state,
    );
}

fn handle_gravity(
    time: Res<Time>,
    actions: Res<InputActions>,
    mut timer: ResMut<GravityTimer>,
    mut piece: ResMut<ActivePiece>,
    board: Res<Board>,
    mut soft_drop_msgs: MessageWriter<SoftDropMsg>,
) {
    let interval = if actions.soft_drop {
        timer.interval / SOFT_DROP_MULTIPLIER
    } else {
        timer.interval
    };

    let mut soft_rows = 0u32;

    timer.elapsed += time.delta_secs();
    while timer.elapsed >= interval {
        timer.elapsed -= interval;
        if can_place(&board, &piece, -1, 0) {
            piece.row -= 1;
            if actions.soft_drop {
                soft_rows += 1;
            }
        } else {
            timer.elapsed = 0.0;
            break;
        }
    }

    if soft_rows > 0 {
        soft_drop_msgs.write(SoftDropMsg(soft_rows));
    }
}

fn handle_lock_delay(
    time: Res<Time>,
    mut lock: ResMut<LockDelayState>,
    mut piece: ResMut<ActivePiece>,
    mut board: ResMut<Board>,
    mut bag: ResMut<PieceBag>,
    mut gravity: ResMut<GravityTimer>,
    mut hold: ResMut<HoldPiece>,
    mut line_clear_msgs: MessageWriter<LineClearMsg>,
    mut piece_locked_msgs: MessageWriter<PieceLockedMsg>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let player_acted = piece.col != lock.prev_col || piece.rotation != lock.prev_rotation;
    lock.prev_col = piece.col;
    lock.prev_rotation = piece.rotation;

    if player_acted && lock.resets < LOCK_DELAY_MAX_RESETS {
        lock.elapsed = 0.0;
        lock.resets += 1;
    }

    let on_ground = !can_place(&board, &piece, -1, 0);
    if !on_ground {
        return;
    }

    lock.elapsed += time.delta_secs();

    if lock.elapsed >= LOCK_DELAY_SECS {
        perform_lock(
            &mut piece,
            &mut board,
            &mut bag,
            &mut gravity,
            &mut lock,
            &mut hold,
            &mut line_clear_msgs,
            &mut piece_locked_msgs,
            &mut next_app_state,
        );
    }
}

fn handle_level_change(
    mut gravity: ResMut<GravityTimer>,
    mut level_msgs: MessageReader<LevelChangedMsg>,
) {
    for msg in level_msgs.read() {
        let l = msg.0 as f32;
        gravity.interval =
            (GRAVITY_BASE - (l - 1.0) * GRAVITY_FACTOR).powf(l - 1.0).max(GRAVITY_FLOOR);
    }
}
