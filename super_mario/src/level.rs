use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::AppState;
use crate::ui;

/// The level grid stored as a resource for collision lookups.
#[derive(Resource)]
pub struct LevelGrid {
    pub grid: [[char; LEVEL_WIDTH]; LEVEL_HEIGHT],
}

impl LevelGrid {
    /// Returns true if the tile at (col, row) is solid. Out-of-bounds = not solid.
    pub fn is_solid(&self, col: i32, row: i32) -> bool {
        if col < 0 || row < 0 || col >= LEVEL_WIDTH as i32 || row >= LEVEL_HEIGHT as i32 {
            return false;
        }
        matches!(
            self.grid[row as usize][col as usize],
            '#' | 'B' | '?' | 'M' | 'X' | '[' | ']' | '{' | '}'
        )
    }
}

/// Convert grid (col, row) to world-space center of that tile.
/// Row 0 = top, Row 14 = bottom. Y increases upward.
pub fn tile_to_world(col: usize, row: usize) -> (f32, f32) {
    let x = LEVEL_ORIGIN_X + col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let y = LEVEL_ORIGIN_Y + (LEVEL_HEIGHT - 1 - row) as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    (x, y)
}

/// Convert world X to grid column.
pub fn world_to_col(wx: f32) -> i32 {
    ((wx - LEVEL_ORIGIN_X) / TILE_SIZE).floor() as i32
}

/// Convert world Y to grid row.
pub fn world_to_row(wy: f32) -> i32 {
    (LEVEL_HEIGHT as i32 - 1) - ((wy - LEVEL_ORIGIN_Y) / TILE_SIZE).floor() as i32
}

/// Level 1-1 tile grid (211 columns × 15 rows).
///
/// Tile legend:
/// `.` empty   `#` ground   `B` brick   `?` question (coin)   `M` question (mushroom)
/// `[` pipe-top-left   `]` pipe-top-right   `{` pipe-body-left   `}` pipe-body-right
/// `X` solid (staircase)   `S` spawn   `G` Goomba   `K` Koopa   `F` flagpole
///
/// Row 0 = top of screen, Row 14 = bottom. Ground is 2 tiles thick (rows 13–14).

pub const LEVEL_WIDTH: usize = 211;
pub const LEVEL_HEIGHT: usize = 15;

pub fn level_1_1() -> [[char; LEVEL_WIDTH]; LEVEL_HEIGHT] {
    let mut g = [['.'; LEVEL_WIDTH]; LEVEL_HEIGHT];

    // ── Ground (rows 13–14) with pits ──
    for col in 0..LEVEL_WIDTH {
        let is_pit = matches!(col, 69..=70 | 86..=88 | 153..=155);
        if !is_pit {
            g[13][col] = '#';
            g[14][col] = '#';
        }
    }

    // ── Mario spawn ──
    g[12][3] = 'S';

    // ── Flagpole (col 198, rows 3–12) ──
    for row in 3..=12 {
        g[row][198] = 'F';
    }

    // ── Question blocks — coins ──
    set_tiles(&mut g, '?', &[
        (16, 9), (21, 5), (22, 9), (23, 9),
        (94, 9), (106, 5), (109, 5), (109, 9), (112, 5),
        (130, 9), (170, 9),
    ]);

    // ── Question blocks — mushroom / power-up ──
    set_tiles(&mut g, 'M', &[(20, 9), (78, 9), (129, 9), (171, 9)]);

    // ── Brick blocks ──
    set_tiles(&mut g, 'B', &[
        (20, 5), (22, 5), (24, 9),
        (77, 9), (79, 9), (80, 5), (81, 5),
        (91, 9), (92, 9), (93, 9), (94, 5), (100, 5), (101, 9),
        (106, 9), (118, 9),
        (121, 5), (122, 5), (123, 5),
        (128, 9), (129, 5), (130, 5), (131, 9),
        (168, 5), (169, 5), (168, 9), (169, 9),
    ]);

    // ── Pipes — (left_col, top_row) ──
    // Body fills from top_row+1 down to row 13 (ground surface).
    pipe(&mut g, 28, 12);   // height 2
    pipe(&mut g, 38, 11);   // height 3
    pipe(&mut g, 46, 10);   // height 4
    pipe(&mut g, 57, 10);   // height 4
    pipe(&mut g, 163, 12);  // height 2
    pipe(&mut g, 179, 12);  // height 2

    // ── Staircases ──
    staircase(&mut g, 134, 4, true);   // ascending right
    staircase(&mut g, 140, 4, false);  // descending right
    staircase(&mut g, 148, 4, true);   // ascending right + flat top:
    for row in 9..=12 {
        g[row][152] = 'X';
    }
    staircase(&mut g, 190, 8, true); // final grand staircase

    // ── Goombas ──
    set_tiles(&mut g, 'G', &[
        (22, 12), (40, 12), (51, 12), (52, 12),
        (82, 12), (83, 12), (97, 12), (98, 12),
        (114, 12), (115, 12), (124, 12), (125, 12),
        (128, 12), (129, 12), (174, 12), (175, 12),
    ]);

    // ── Koopa Troopa ──
    g[12][107] = 'K';

    g
}

fn set_tiles(g: &mut [[char; LEVEL_WIDTH]; LEVEL_HEIGHT], ch: char, positions: &[(usize, usize)]) {
    for &(col, row) in positions {
        g[row][col] = ch;
    }
}

fn pipe(g: &mut [[char; LEVEL_WIDTH]; LEVEL_HEIGHT], col: usize, top_row: usize) {
    g[top_row][col] = '[';
    g[top_row][col + 1] = ']';
    for row in (top_row + 1)..=13 {
        g[row][col] = '{';
        g[row][col + 1] = '}';
    }
}

fn staircase(
    g: &mut [[char; LEVEL_WIDTH]; LEVEL_HEIGHT],
    start_col: usize,
    height: usize,
    ascending: bool,
) {
    for i in 0..height {
        let col = start_col + i;
        let blocks = if ascending { i + 1 } else { height - i };
        for b in 0..blocks {
            g[12 - b][col] = 'X';
        }
    }
}

// ── Level spawning system (OnEnter AppState::Playing) ──

pub fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_data: ResMut<GameData>,
    mut spawn_point: ResMut<SpawnPoint>,
) {
    *game_data = GameData::default();

    let tile_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE));
    let pipe_top_mesh = meshes.add(Rectangle::new(TILE_SIZE + PIPE_LIP_OVERHANG, TILE_SIZE));
    let ground_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.55, 0.27, 0.07)));
    let brick_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.72, 0.40, 0.10)));
    let question_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.90, 0.75, 0.10)));
    let solid_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.45, 0.30, 0.15)));
    let pipe_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.0, 0.65, 0.15)));
    let player_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.8, 0.1, 0.1)));

    let grid = level_1_1();
    commands.insert_resource(LevelGrid { grid });

    let mut sp = (0.0_f32, 0.0_f32);

    for row in 0..LEVEL_HEIGHT {
        for col in 0..LEVEL_WIDTH {
            let ch = grid[row][col];
            let (wx, wy) = tile_to_world(col, row);

            match ch {
                '#' => {
                    commands.spawn((
                        Tile, TileType::Ground,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(ground_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_TILE),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                'B' => {
                    commands.spawn((
                        Tile, TileType::Brick,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(brick_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_TILE),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                '?' | 'M' => {
                    commands.spawn((
                        Tile, TileType::QuestionBlock,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(question_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_TILE),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                'X' => {
                    commands.spawn((
                        Tile, TileType::Solid,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(solid_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_TILE),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                '[' => {
                    commands.spawn((
                        Tile, TileType::PipeTopLeft,
                        Mesh2d(pipe_top_mesh.clone()),
                        MeshMaterial2d(pipe_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_PIPE),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                ']' => {
                    commands.spawn((
                        Tile, TileType::PipeTopRight,
                        Mesh2d(pipe_top_mesh.clone()),
                        MeshMaterial2d(pipe_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_PIPE),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                '{' => {
                    commands.spawn((
                        Tile, TileType::PipeBodyLeft,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(pipe_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_PIPE),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                '}' => {
                    commands.spawn((
                        Tile, TileType::PipeBodyRight,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(pipe_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_PIPE),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                'S' => {
                    sp = (wx, wy);
                }
                _ => {}
            }
        }
    }

    *spawn_point = SpawnPoint { x: sp.0, y: sp.1 };

    // Mario
    commands.spawn((
        Player,
        Velocity::default(),
        FacingDirection::default(),
        Grounded::default(),
        Mesh2d(meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_SMALL_HEIGHT))),
        MeshMaterial2d(player_mat),
        Transform::from_xyz(sp.0, sp.1, Z_PLAYER),
        DespawnOnExit(AppState::Playing),
    ));

    // HUD
    ui::spawn_hud(&mut commands, &game_data);
}
