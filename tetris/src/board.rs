use bevy::prelude::*;

use crate::constants::*;
use crate::resources::LineClearMsg;
use crate::states::AppState;
use crate::tetromino::TetrominoKind;

/// The playfield grid. Each cell is `None` (empty) or `Some(Color)` (filled).
#[derive(Resource)]
pub struct Board {
    pub cells: [[Option<TetrominoKind>; GRID_COLS]; GRID_TOTAL_ROWS],
}

impl Board {
    pub fn is_valid_cells(&self, cells: &[(i32, i32); 4]) -> bool {
        cells.iter().all(|&(r, c)| {
            r >= 0
                && (r as usize) < GRID_TOTAL_ROWS
                && c >= 0
                && (c as usize) < GRID_COLS
                && self.cells[r as usize][c as usize].is_none()
        })
    }

    pub fn lock_cells(&mut self, cells: &[(i32, i32); 4], kind: TetrominoKind) {
        for &(r, c) in cells {
            if r >= 0 && (r as usize) < GRID_TOTAL_ROWS && c >= 0 && (c as usize) < GRID_COLS {
                self.cells[r as usize][c as usize] = Some(kind);
            }
        }
    }

    pub fn clear_full_rows(&mut self) -> Vec<usize> {
        let full_rows: Vec<usize> = (0..GRID_TOTAL_ROWS)
            .filter(|&r| self.cells[r].iter().all(|c| c.is_some()))
            .collect();

        if full_rows.is_empty() {
            return full_rows;
        }

        let mut write = 0;
        for read in 0..GRID_TOTAL_ROWS {
            if !full_rows.contains(&read) {
                if write != read {
                    self.cells[write] = self.cells[read];
                }
                write += 1;
            }
        }
        for row in write..GRID_TOTAL_ROWS {
            self.cells[row] = [None; GRID_COLS];
        }

        full_rows
    }
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

/// Brief white flash overlay when a line is cleared.
#[derive(Component)]
struct LineClearFlash(f32);

fn spawn_line_flash(commands: &mut Commands, row: usize) {
    let y = PLAYFIELD_BOTTOM + row as f32 * CELL_SIZE + CELL_SIZE / 2.0;
    commands.spawn((
        LineClearFlash(LINE_FLASH_DURATION),
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.8),
            custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, CELL_INNER_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, y, Z_FLASH),
    ));
}

// ---------------------------------------------------------------------------
// Row collapse animation
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct RowCollapseAnim {
    timer: f32,
    row_offsets: [f32; GRID_VISIBLE_ROWS],
}

impl Default for RowCollapseAnim {
    fn default() -> Self {
        Self {
            timer: 0.0,
            row_offsets: [0.0; GRID_VISIBLE_ROWS],
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Board>()
            .init_resource::<RowCollapseAnim>()
            .add_systems(Startup, (spawn_playfield_border, spawn_cell_entities))
            .add_systems(OnEnter(AppState::Playing), reset_board)
            .add_systems(
                Update,
                (
                    sync_board_cells,
                    handle_line_clear_visuals.run_if(in_state(AppState::Playing)),
                    animate_board_collapse,
                    animate_line_clear_flash,
                ),
            );
    }
}

/// Convert grid (row, col) to world-space center position.
pub fn grid_to_world(row: usize, col: usize) -> Vec2 {
    Vec2::new(
        PLAYFIELD_LEFT + col as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        PLAYFIELD_BOTTOM + row as f32 * CELL_SIZE + CELL_SIZE / 2.0,
    )
}

fn reset_board(mut board: ResMut<Board>, mut anim: ResMut<RowCollapseAnim>) {
    *board = Board::default();
    *anim = RowCollapseAnim::default();
}

fn spawn_playfield_border(mut commands: Commands) {
    let half_w = PLAYFIELD_WIDTH / 2.0;
    let half_h = PLAYFIELD_HEIGHT / 2.0;
    let bt = BORDER_THICKNESS;

    commands.spawn((
        Sprite {
            color: BORDER_COLOR,
            custom_size: Some(Vec2::new(PLAYFIELD_WIDTH + bt * 2.0, bt)),
            ..default()
        },
        Transform::from_xyz(0.0, -half_h - bt / 2.0, Z_BORDER),
    ));
    commands.spawn((
        Sprite {
            color: BORDER_COLOR,
            custom_size: Some(Vec2::new(PLAYFIELD_WIDTH + bt * 2.0, bt)),
            ..default()
        },
        Transform::from_xyz(0.0, half_h + bt / 2.0, Z_BORDER),
    ));
    commands.spawn((
        Sprite {
            color: BORDER_COLOR,
            custom_size: Some(Vec2::new(bt, PLAYFIELD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(-half_w - bt / 2.0, 0.0, Z_BORDER),
    ));
    commands.spawn((
        Sprite {
            color: BORDER_COLOR,
            custom_size: Some(Vec2::new(bt, PLAYFIELD_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(half_w + bt / 2.0, 0.0, Z_BORDER),
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
                Transform::from_xyz(pos.x, pos.y, Z_BOARD_CELL),
                visibility,
            ));
        }
    }
}

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
            Some(kind) => {
                sprite.color = kind.color();
                *visibility = Visibility::Visible;
            }
            None => {
                sprite.color = empty_color;
                *visibility = empty_vis;
            }
        }
    }
}

fn handle_line_clear_visuals(
    mut commands: Commands,
    mut anim: ResMut<RowCollapseAnim>,
    mut line_clears: MessageReader<LineClearMsg>,
) {
    for msg in line_clears.read() {
        // Flash overlays
        for &row in &msg.rows {
            spawn_line_flash(&mut commands, row);
        }

        // Collapse animation offsets
        let non_cleared: Vec<usize> = (0..GRID_TOTAL_ROWS)
            .filter(|r| !msg.rows.contains(r))
            .collect();
        anim.timer = COLLAPSE_DURATION;
        anim.row_offsets = [0.0; GRID_VISIBLE_ROWS];
        for new_r in 0..GRID_VISIBLE_ROWS {
            if new_r < non_cleared.len() {
                anim.row_offsets[new_r] =
                    (non_cleared[new_r] as f32 - new_r as f32) * CELL_SIZE;
            }
        }
    }
}

fn animate_board_collapse(
    time: Res<Time>,
    mut anim: ResMut<RowCollapseAnim>,
    mut query: Query<(&BoardCell, &mut Transform)>,
) {
    if anim.timer <= 0.0 {
        return;
    }

    anim.timer -= time.delta_secs();
    let t = (anim.timer / COLLAPSE_DURATION).max(0.0);

    for (cell, mut transform) in &mut query {
        let base_pos = grid_to_world(cell.row, cell.col);
        let offset = anim.row_offsets[cell.row] * t;
        transform.translation.y = base_pos.y + offset;
    }
}

fn animate_line_clear_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LineClearFlash, &mut Sprite)>,
) {
    for (entity, mut flash, mut sprite) in &mut query {
        flash.0 -= time.delta_secs();
        if flash.0 <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            let alpha = (flash.0 / LINE_FLASH_DURATION) * 0.8;
            sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
        }
    }
}
