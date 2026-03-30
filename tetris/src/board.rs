use bevy::prelude::*;

use crate::constants::*;

/// The playfield grid. Each cell is `None` (empty) or `Some(Color)` (filled).
#[derive(Resource)]
pub struct Board {
    pub cells: [[Option<Color>; GRID_COLS]; GRID_TOTAL_ROWS],
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cells: [[None; GRID_COLS]; GRID_TOTAL_ROWS],
        }
    }
}

/// Marker for a rendered cell entity, storing its grid position.
#[derive(Component)]
pub struct BoardCell {
    pub row: usize,
    pub col: usize,
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Board>()
            .add_systems(Startup, (spawn_playfield_border, spawn_cell_entities))
            .add_systems(Update, sync_board_cells);
    }
}

/// Convert grid (row, col) to world-space center position.
pub fn grid_to_world(row: usize, col: usize) -> Vec2 {
    Vec2::new(
        PLAYFIELD_LEFT + col as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        PLAYFIELD_BOTTOM + row as f32 * CELL_SIZE + CELL_SIZE / 2.0,
    )
}

fn spawn_playfield_border(mut commands: Commands) {
    let half_w = PLAYFIELD_WIDTH / 2.0;
    let half_h = PLAYFIELD_HEIGHT / 2.0;
    let bt = BORDER_THICKNESS;

    // Bottom
    commands.spawn((
        Sprite {
            color: BORDER_COLOR,
            custom_size: Some(Vec2::new(PLAYFIELD_WIDTH + bt * 2.0, bt)),
            ..default()
        },
        Transform::from_xyz(0.0, -half_h - bt / 2.0, 0.0),
    ));
    // Top
    commands.spawn((
        Sprite {
            color: BORDER_COLOR,
            custom_size: Some(Vec2::new(PLAYFIELD_WIDTH + bt * 2.0, bt)),
            ..default()
        },
        Transform::from_xyz(0.0, half_h + bt / 2.0, 0.0),
    ));
    // Left
    commands.spawn((
        Sprite {
            color: BORDER_COLOR,
            custom_size: Some(Vec2::new(bt, PLAYFIELD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(-half_w - bt / 2.0, 0.0, 0.0),
    ));
    // Right
    commands.spawn((
        Sprite {
            color: BORDER_COLOR,
            custom_size: Some(Vec2::new(bt, PLAYFIELD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(half_w + bt / 2.0, 0.0, 0.0),
    ));
}

fn spawn_cell_entities(mut commands: Commands) {
    for row in 0..GRID_VISIBLE_ROWS {
        for col in 0..GRID_COLS {
            let pos = grid_to_world(row, col);
            let (color, visibility) = empty_cell_style();
            commands.spawn((
                BoardCell { row, col },
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(CELL_INNER_SIZE, CELL_INNER_SIZE)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 1.0),
                visibility,
            ));
        }
    }
}

/// Returns the color and visibility for an empty cell.
/// In debug builds, cells are faintly visible to show the grid.
fn empty_cell_style() -> (Color, Visibility) {
    if cfg!(debug_assertions) {
        (Color::srgba(1.0, 1.0, 1.0, 0.03), Visibility::Visible)
    } else {
        (Color::srgba(0.0, 0.0, 0.0, 0.0), Visibility::Hidden)
    }
}

fn sync_board_cells(
    board: Res<Board>,
    mut query: Query<(&BoardCell, &mut Sprite, &mut Visibility)>,
) {
    if !board.is_changed() {
        return;
    }
    let (empty_color, empty_vis) = empty_cell_style();
    for (cell, mut sprite, mut visibility) in &mut query {
        match board.cells[cell.row][cell.col] {
            Some(color) => {
                sprite.color = color;
                *visibility = Visibility::Visible;
            }
            None => {
                sprite.color = empty_color;
                *visibility = empty_vis;
            }
        }
    }
}
