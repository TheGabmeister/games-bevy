use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::board::{grid_to_world, Board};
use crate::constants::*;
use crate::states::AppState;

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

    pub fn cells(self, rotation: RotationState) -> [(i32, i32); 4] {
        PIECE_CELLS[self as usize][rotation as usize]
    }

    pub fn kicks(self, from: RotationState, to: RotationState) -> &'static [(i32, i32); 5] {
        let table = match self {
            TetrominoKind::I => &I_KICKS,
            _ => &JLSTZ_KICKS,
        };
        &table[kick_index(from, to)]
    }

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

impl RotationState {
    pub fn cw(self) -> Self {
        match self {
            Self::R0 => Self::R1,
            Self::R1 => Self::R2,
            Self::R2 => Self::R3,
            Self::R3 => Self::R0,
        }
    }

    pub fn ccw(self) -> Self {
        match self {
            Self::R0 => Self::R3,
            Self::R1 => Self::R0,
            Self::R2 => Self::R1,
            Self::R3 => Self::R2,
        }
    }
}

// ---------------------------------------------------------------------------
// SRS cell data
// ---------------------------------------------------------------------------

const PIECE_CELLS: [[[(i32, i32); 4]; 4]; 7] = [
    // I (4x4)
    [
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(3, 2), (2, 2), (1, 2), (0, 2)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
        [(3, 1), (2, 1), (1, 1), (0, 1)],
    ],
    // O (3x3)
    [
        [(2, 1), (2, 2), (1, 1), (1, 2)],
        [(2, 1), (2, 2), (1, 1), (1, 2)],
        [(2, 1), (2, 2), (1, 1), (1, 2)],
        [(2, 1), (2, 2), (1, 1), (1, 2)],
    ],
    // T
    [
        [(2, 1), (1, 0), (1, 1), (1, 2)],
        [(2, 1), (1, 1), (1, 2), (0, 1)],
        [(1, 0), (1, 1), (1, 2), (0, 1)],
        [(2, 1), (1, 0), (1, 1), (0, 1)],
    ],
    // S
    [
        [(2, 1), (2, 2), (1, 0), (1, 1)],
        [(2, 1), (1, 1), (1, 2), (0, 2)],
        [(1, 1), (1, 2), (0, 0), (0, 1)],
        [(2, 0), (1, 0), (1, 1), (0, 1)],
    ],
    // Z
    [
        [(2, 0), (2, 1), (1, 1), (1, 2)],
        [(2, 2), (1, 1), (1, 2), (0, 1)],
        [(1, 0), (1, 1), (0, 1), (0, 2)],
        [(2, 1), (1, 0), (1, 1), (0, 0)],
    ],
    // J
    [
        [(2, 0), (1, 0), (1, 1), (1, 2)],
        [(2, 1), (2, 2), (1, 1), (0, 1)],
        [(1, 0), (1, 1), (1, 2), (0, 2)],
        [(2, 1), (1, 1), (0, 0), (0, 1)],
    ],
    // L
    [
        [(2, 2), (1, 0), (1, 1), (1, 2)],
        [(2, 1), (1, 1), (0, 1), (0, 2)],
        [(1, 0), (1, 1), (1, 2), (0, 0)],
        [(2, 0), (2, 1), (1, 1), (0, 1)],
    ],
];

// ---------------------------------------------------------------------------
// SRS wall-kick tables
// ---------------------------------------------------------------------------

fn kick_index(from: RotationState, to: RotationState) -> usize {
    use RotationState::*;
    match (from, to) {
        (R0, R1) => 0,
        (R1, R2) => 1,
        (R2, R3) => 2,
        (R3, R0) => 3,
        (R1, R0) => 4,
        (R2, R1) => 5,
        (R3, R2) => 6,
        (R0, R3) => 7,
        _ => unreachable!(),
    }
}

const JLSTZ_KICKS: [[(i32, i32); 5]; 8] = [
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
];

const I_KICKS: [[(i32, i32); 5]; 8] = [
    [(1, 0), (-1, 0), (2, 0), (-1, 1), (2, -2)],
    [(0, 1), (-1, 1), (2, 1), (-1, -1), (2, 2)],
    [(-1, 0), (1, 0), (-2, 0), (1, -1), (-2, 2)],
    [(0, -1), (1, -1), (-2, -1), (1, 1), (-2, -2)],
    [(-1, 0), (1, 0), (-2, 0), (1, -1), (-2, 2)],
    [(0, -1), (1, -1), (-2, -1), (1, 1), (-2, -2)],
    [(1, 0), (-1, 0), (2, 0), (-1, 1), (2, -2)],
    [(0, 1), (-1, 1), (2, 1), (-1, -1), (2, 2)],
];

// ---------------------------------------------------------------------------
// Active piece
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct ActivePiece {
    pub kind: TetrominoKind,
    pub rotation: RotationState,
    pub row: i32,
    pub col: i32,
}

impl ActivePiece {
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
        let piece = self.bag.pop().unwrap();
        // Ensure enough pieces remain for the next-queue preview.
        if self.bag.len() < NEXT_QUEUE_SIZE {
            let mut new_bag = TetrominoKind::ALL.to_vec();
            new_bag.shuffle(&mut rand::rng());
            let mut combined = new_bag;
            combined.append(&mut self.bag);
            self.bag = combined;
        }
        piece
    }

    pub fn peek(&self, n: usize) -> Vec<TetrominoKind> {
        self.bag.iter().rev().take(n).copied().collect()
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct ActivePieceCell(pub usize);

#[derive(Component)]
pub struct GhostPieceCell(pub usize);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct TetrominoPlugin;

impl Plugin for TetrominoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PieceBag>()
            .add_systems(Startup, spawn_piece_cells)
            .add_systems(OnEnter(AppState::Playing), reset_tetromino)
            .add_systems(
                Update,
                (sync_active_piece_cells, sync_ghost_piece_cells)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn spawn_piece_cells(mut commands: Commands) {
    // Active piece sprites
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
    // Ghost piece sprites
    for i in 0..4 {
        commands.spawn((
            GhostPieceCell(i),
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, GHOST_ALPHA),
                custom_size: Some(Vec2::new(CELL_INNER_SIZE, CELL_INNER_SIZE)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.5),
            Visibility::Hidden,
        ));
    }
}

fn reset_tetromino(mut commands: Commands, mut bag: ResMut<PieceBag>) {
    *bag = PieceBag::default();
    let kind = bag.draw();
    commands.insert_resource(ActivePiece {
        kind,
        rotation: RotationState::R0,
        row: kind.spawn_row(),
        col: SPAWN_COL,
    });
}

fn sync_active_piece_cells(
    piece: Option<Res<ActivePiece>>,
    mut query: Query<(
        &ActivePieceCell,
        &mut Transform,
        &mut Sprite,
        &mut Visibility,
    )>,
) {
    let Some(piece) = piece else {
        for (_, _, _, mut vis) in &mut query {
            *vis = Visibility::Hidden;
        }
        return;
    };
    let cells = piece.board_cells();
    let color = piece.kind.color();

    for (idx, mut transform, mut sprite, mut visibility) in &mut query {
        let (row, col) = cells[idx.0];
        sprite.color = color;

        if row >= 0
            && (row as usize) < GRID_VISIBLE_ROWS
            && col >= 0
            && (col as usize) < GRID_COLS
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

fn sync_ghost_piece_cells(
    piece: Option<Res<ActivePiece>>,
    board: Res<Board>,
    mut query: Query<
        (&GhostPieceCell, &mut Transform, &mut Sprite, &mut Visibility),
        Without<ActivePieceCell>,
    >,
) {
    let Some(piece) = piece else {
        for (_, _, _, mut vis) in &mut query {
            *vis = Visibility::Hidden;
        }
        return;
    };

    // Find hard-drop row.
    let mut ghost_row = piece.row;
    loop {
        let test_cells = piece
            .kind
            .cells(piece.rotation)
            .map(|(r, c)| (ghost_row - 1 + r, piece.col + c));
        if !board.is_valid_cells(&test_cells) {
            break;
        }
        ghost_row -= 1;
    }

    let ghost_cells = piece
        .kind
        .cells(piece.rotation)
        .map(|(r, c)| (ghost_row + r, piece.col + c));
    let color = piece.kind.color().with_alpha(GHOST_ALPHA);

    for (idx, mut transform, mut sprite, mut visibility) in &mut query {
        let (row, col) = ghost_cells[idx.0];
        sprite.color = color;

        if row >= 0
            && (row as usize) < GRID_VISIBLE_ROWS
            && col >= 0
            && (col as usize) < GRID_COLS
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
