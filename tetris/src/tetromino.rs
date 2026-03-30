use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::board::grid_to_world;
use crate::constants::*;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TetrominoKind {
    I = 0,
    O = 1,
    T = 2,
    S = 3,
    Z = 4,
    J = 5,
    L = 6,
}

impl TetrominoKind {
    pub const ALL: [TetrominoKind; 7] = [
        TetrominoKind::I,
        TetrominoKind::O,
        TetrominoKind::T,
        TetrominoKind::S,
        TetrominoKind::Z,
        TetrominoKind::J,
        TetrominoKind::L,
    ];

    pub fn color(self) -> Color {
        match self {
            TetrominoKind::I => COLOR_I,
            TetrominoKind::O => COLOR_O,
            TetrominoKind::T => COLOR_T,
            TetrominoKind::S => COLOR_S,
            TetrominoKind::Z => COLOR_Z,
            TetrominoKind::J => COLOR_J,
            TetrominoKind::L => COLOR_L,
        }
    }

    /// Cell offsets `(row, col)` for the given rotation state.
    /// Row 0 = bottom of bounding box, col 0 = left.
    pub fn cells(self, rotation: RotationState) -> [(i32, i32); 4] {
        PIECE_CELLS[self as usize][rotation as usize]
    }

    /// The spawn row so the piece starts in the buffer rows.
    pub fn spawn_row(self) -> i32 {
        match self {
            TetrominoKind::I => GRID_TOTAL_ROWS as i32 - 4,
            _ => GRID_TOTAL_ROWS as i32 - 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum RotationState {
    #[default]
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
}

// ---------------------------------------------------------------------------
// SRS cell data — offsets (row, col) from piece position
// Row 0 = bottom of bounding box (board-up convention).
// Indexed as PIECE_CELLS[kind][rotation].
// ---------------------------------------------------------------------------

const PIECE_CELLS: [[[(i32, i32); 4]; 4]; 7] = [
    // I (4×4)
    [
        [(2, 0), (2, 1), (2, 2), (2, 3)], // R0 — horizontal bar
        [(3, 2), (2, 2), (1, 2), (0, 2)], // R1
        [(1, 0), (1, 1), (1, 2), (1, 3)], // R2
        [(3, 1), (2, 1), (1, 1), (0, 1)], // R3
    ],
    // O (3×3, occupies cols 1-2)
    [
        [(2, 1), (2, 2), (1, 1), (1, 2)],
        [(2, 1), (2, 2), (1, 1), (1, 2)],
        [(2, 1), (2, 2), (1, 1), (1, 2)],
        [(2, 1), (2, 2), (1, 1), (1, 2)],
    ],
    // T
    [
        [(2, 1), (1, 0), (1, 1), (1, 2)], // R0
        [(2, 1), (1, 1), (1, 2), (0, 1)], // R1
        [(1, 0), (1, 1), (1, 2), (0, 1)], // R2
        [(2, 1), (1, 0), (1, 1), (0, 1)], // R3
    ],
    // S
    [
        [(2, 1), (2, 2), (1, 0), (1, 1)], // R0
        [(2, 1), (1, 1), (1, 2), (0, 2)], // R1
        [(1, 1), (1, 2), (0, 0), (0, 1)], // R2
        [(2, 0), (1, 0), (1, 1), (0, 1)], // R3
    ],
    // Z
    [
        [(2, 0), (2, 1), (1, 1), (1, 2)], // R0
        [(2, 2), (1, 1), (1, 2), (0, 1)], // R1
        [(1, 0), (1, 1), (0, 1), (0, 2)], // R2
        [(2, 1), (1, 0), (1, 1), (0, 0)], // R3
    ],
    // J
    [
        [(2, 0), (1, 0), (1, 1), (1, 2)], // R0
        [(2, 1), (2, 2), (1, 1), (0, 1)], // R1
        [(1, 0), (1, 1), (1, 2), (0, 2)], // R2
        [(2, 1), (1, 1), (0, 0), (0, 1)], // R3
    ],
    // L
    [
        [(2, 2), (1, 0), (1, 1), (1, 2)], // R0
        [(2, 1), (1, 1), (0, 1), (0, 2)], // R1
        [(1, 0), (1, 1), (1, 2), (0, 0)], // R2
        [(2, 0), (2, 1), (1, 1), (0, 1)], // R3
    ],
];

// ---------------------------------------------------------------------------
// Active piece
// ---------------------------------------------------------------------------

/// The currently falling piece.
#[derive(Resource)]
pub struct ActivePiece {
    pub kind: TetrominoKind,
    pub rotation: RotationState,
    pub row: i32,
    pub col: i32,
}

impl ActivePiece {
    /// Absolute board positions `(row, col)` of the 4 cells.
    pub fn board_cells(&self) -> [(i32, i32); 4] {
        self.kind
            .cells(self.rotation)
            .map(|(r, c)| (self.row + r, self.col + c))
    }
}

// ---------------------------------------------------------------------------
// 7-bag randomizer
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct PieceBag {
    bag: Vec<TetrominoKind>,
}

impl Default for PieceBag {
    fn default() -> Self {
        let mut bag = Self { bag: Vec::new() };
        bag.refill();
        bag
    }
}

impl PieceBag {
    fn refill(&mut self) {
        self.bag = TetrominoKind::ALL.to_vec();
        self.bag.shuffle(&mut rand::rng());
    }

    pub fn draw(&mut self) -> TetrominoKind {
        if self.bag.is_empty() {
            self.refill();
        }
        self.bag.pop().unwrap()
    }

    /// Peek at the next `n` pieces without consuming them.
    pub fn peek(&self, n: usize) -> Vec<TetrominoKind> {
        self.bag.iter().rev().take(n).copied().collect()
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker + index for the 4 sprites that render the active piece.
#[derive(Component)]
pub struct ActivePieceCell(pub usize);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct TetrominoPlugin;

impl Plugin for TetrominoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PieceBag>()
            .add_systems(Startup, spawn_first_piece)
            .add_systems(Update, sync_active_piece_cells);
    }
}

fn spawn_first_piece(mut commands: Commands, mut bag: ResMut<PieceBag>) {
    let kind = bag.draw();
    commands.insert_resource(ActivePiece {
        kind,
        rotation: RotationState::R0,
        row: kind.spawn_row(),
        col: 3,
    });

    // Spawn the 4 cell sprites (initially invisible until sync runs).
    for i in 0..4 {
        commands.spawn((
            ActivePieceCell(i),
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(CELL_INNER_SIZE, CELL_INNER_SIZE)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 2.0),
            Visibility::Hidden,
        ));
    }
}

fn sync_active_piece_cells(
    piece: Option<Res<ActivePiece>>,
    mut query: Query<(&ActivePieceCell, &mut Transform, &mut Sprite, &mut Visibility)>,
) {
    let Some(piece) = piece else { return };
    let cells = piece.board_cells();
    let color = piece.kind.color();

    for (idx, mut transform, mut sprite, mut visibility) in &mut query {
        let (row, col) = cells[idx.0];
        sprite.color = color;

        if row >= 0 && (row as usize) < GRID_VISIBLE_ROWS && col >= 0 && (col as usize) < GRID_COLS
        {
            let pos = grid_to_world(row as usize, col as usize);
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
