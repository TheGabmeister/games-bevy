use bevy::prelude::*;

use crate::board::{spawn_line_flash, Board};
use crate::constants::*;
use crate::tetromino::{ActivePiece, PieceBag, RotationState, TetrominoKind};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check whether the piece would fit at an offset from its current position.
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

/// Lock the active piece into the board, clear full rows, spawn the next
/// piece, and reset gravity + lock-delay state. Returns cleared row indices
/// so the caller can spawn flash effects.
fn lock_and_spawn(
    piece: &mut ActivePiece,
    board: &mut Board,
    bag: &mut PieceBag,
    gravity: &mut GravityTimer,
    lock: &mut LockDelayState,
) -> Vec<usize> {
    let cells = piece.board_cells();
    let color = piece.kind.color();
    board.lock_cells(&cells, color);

    let cleared_rows = board.clear_full_rows();

    spawn_next_piece(piece, bag);

    gravity.elapsed = 0.0;
    lock.elapsed = 0.0;
    lock.resets = 0;
    lock.prev_col = piece.col;
    lock.prev_rotation = piece.rotation;

    cleared_rows
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
            interval: 1.0, // ~level 1
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

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DasState>()
            .init_resource::<GravityTimer>()
            .init_resource::<LockDelayState>()
            .add_systems(
                Update,
                (
                    handle_horizontal_input,
                    handle_rotation,
                    handle_hard_drop,
                    handle_gravity,
                    handle_lock_delay,
                )
                    .chain(),
            );
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn handle_horizontal_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut das: ResMut<DasState>,
    mut piece: ResMut<ActivePiece>,
    board: Res<Board>,
) {
    let left = keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
    let right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);

    let dir = match (left, right) {
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
        // New direction — move immediately, start DAS timer.
        das.direction = dir;
        das.elapsed = 0.0;
        das.charged = false;
        if can_place(&board, &piece, 0, dc) {
            piece.col += dc;
        }
    } else {
        // Same direction held — tick DAS.
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
    keys: Res<ButtonInput<KeyCode>>,
    mut piece: ResMut<ActivePiece>,
    board: Res<Board>,
) {
    // O piece doesn't rotate.
    if piece.kind == TetrominoKind::O {
        return;
    }

    let cw = keys.just_pressed(KeyCode::KeyX) || keys.just_pressed(KeyCode::KeyE);
    let ccw = keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::KeyQ);

    let target = if cw {
        piece.rotation.cw()
    } else if ccw {
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
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut piece: ResMut<ActivePiece>,
    mut board: ResMut<Board>,
    mut bag: ResMut<PieceBag>,
    mut gravity: ResMut<GravityTimer>,
    mut lock: ResMut<LockDelayState>,
) {
    if !keys.just_pressed(KeyCode::ArrowUp) && !keys.just_pressed(KeyCode::Space) {
        return;
    }

    // Drop to lowest valid row.
    while can_place(&board, &piece, -1, 0) {
        piece.row -= 1;
    }

    let cleared = lock_and_spawn(&mut piece, &mut board, &mut bag, &mut gravity, &mut lock);
    for &row in &cleared {
        spawn_line_flash(&mut commands, row);
    }
}

fn handle_gravity(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut timer: ResMut<GravityTimer>,
    mut piece: ResMut<ActivePiece>,
    board: Res<Board>,
) {
    let soft_drop = keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS);
    let interval = if soft_drop {
        timer.interval / SOFT_DROP_MULTIPLIER as f32
    } else {
        timer.interval
    };

    timer.elapsed += time.delta_secs();
    while timer.elapsed >= interval {
        timer.elapsed -= interval;
        if can_place(&board, &piece, -1, 0) {
            piece.row -= 1;
        } else {
            // Can't move down — lock delay handles the rest.
            timer.elapsed = 0.0;
            break;
        }
    }
}

fn handle_lock_delay(
    mut commands: Commands,
    time: Res<Time>,
    mut lock: ResMut<LockDelayState>,
    mut piece: ResMut<ActivePiece>,
    mut board: ResMut<Board>,
    mut bag: ResMut<PieceBag>,
    mut gravity: ResMut<GravityTimer>,
) {
    // Detect player-initiated actions (col or rotation changed).
    let player_acted = piece.col != lock.prev_col || piece.rotation != lock.prev_rotation;
    lock.prev_col = piece.col;
    lock.prev_rotation = piece.rotation;

    // Reset lock delay on player action (even if floating).
    if player_acted && lock.resets < LOCK_DELAY_MAX_RESETS {
        lock.elapsed = 0.0;
        lock.resets += 1;
    }

    // Only tick while the piece is on the ground.
    let on_ground = !can_place(&board, &piece, -1, 0);
    if !on_ground {
        return;
    }

    lock.elapsed += time.delta_secs();

    if lock.elapsed >= LOCK_DELAY_SECS {
        let cleared =
            lock_and_spawn(&mut piece, &mut board, &mut bag, &mut gravity, &mut lock);
        for &row in &cleared {
            spawn_line_flash(&mut commands, row);
        }
    }
}
