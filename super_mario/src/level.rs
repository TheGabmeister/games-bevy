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
            '#' | 'B' | '?' | 'M' | 'X' | '[' | ']' | '{' | '}' | 'E'
        )
    }

    /// Returns the char at (col, row). Out-of-bounds = '.'.
    pub fn get_char(&self, col: i32, row: i32) -> char {
        if col < 0 || row < 0 || col >= LEVEL_WIDTH as i32 || row >= LEVEL_HEIGHT as i32 {
            return '.';
        }
        self.grid[row as usize][col as usize]
    }

    /// Set the char at (col, row).
    pub fn set_char(&mut self, col: i32, row: i32, ch: char) {
        if col >= 0 && row >= 0 && col < LEVEL_WIDTH as i32 && row < LEVEL_HEIGHT as i32 {
            self.grid[row as usize][col as usize] = ch;
        }
    }

    /// Returns true if the tile can be hit from below (? or M or B, but not E/used).
    pub fn is_hittable(&self, col: i32, row: i32) -> bool {
        matches!(self.get_char(col, row), '?' | 'M' | 'B')
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
/// `C` floating coin
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
    pipe(&mut g, 28, 12);
    pipe(&mut g, 38, 11);
    pipe(&mut g, 46, 10);
    pipe(&mut g, 57, 10);
    pipe(&mut g, 163, 12);
    pipe(&mut g, 179, 12);

    // ── Staircases ──
    staircase(&mut g, 134, 4, true);
    staircase(&mut g, 140, 4, false);
    staircase(&mut g, 148, 4, true);
    for row in 9..=12 {
        g[row][152] = 'X';
    }
    staircase(&mut g, 190, 8, true);

    // ── Goombas ──
    set_tiles(&mut g, 'G', &[
        (22, 12), (40, 12), (51, 12), (52, 12),
        (82, 12), (83, 12), (97, 12), (98, 12),
        (114, 12), (115, 12), (124, 12), (125, 12),
        (128, 12), (129, 12), (174, 12), (175, 12),
    ]);

    // ── Koopa Troopa ──
    g[12][107] = 'K';

    // ── Floating coins ──
    set_tiles(&mut g, 'C', &[
        (41, 8), (42, 8), (43, 8),
        (64, 8), (65, 8), (66, 8),
        (155, 8), (156, 8),
    ]);

    g
}

/// Compact test level (~50 columns) with every mechanic near the spawn point.
pub fn level_test() -> [[char; LEVEL_WIDTH]; LEVEL_HEIGHT] {
    let mut g = [['.'; LEVEL_WIDTH]; LEVEL_HEIGHT];

    // Ground (rows 13–14) with one pit at cols 24–25
    for col in 0..58 {
        if !matches!(col, 24..=25) {
            g[13][col] = '#';
            g[14][col] = '#';
        }
    }

    // Spawn
    g[12][3] = 'S';

    // Overhead blocks: ? (coin), M (mushroom) x3, B (brick)
    g[9][5] = 'M';
    g[9][6] = 'M';
    g[9][8] = '?';
    g[9][9] = 'M';
    g[9][10] = 'B';

    // Goombas
    g[12][13] = 'G';
    g[12][14] = 'G';

    // Short pipe (2 tiles tall)
    pipe(&mut g, 17, 12);

    // Floating coins
    g[8][21] = 'C';
    g[8][22] = 'C';
    g[8][23] = 'C';

    // Bricks for breaking as Big Mario + ? block above
    g[9][28] = 'B';
    g[9][29] = 'B';
    g[9][30] = 'B';
    g[5][29] = '?';

    // Koopa
    g[12][33] = 'K';

    // Tall pipe (3 tiles tall)
    pipe(&mut g, 36, 11);

    // Staircase (ascending, 4 high)
    staircase(&mut g, 39, 4, true);

    // Goomba trio (shell-kick testing)
    g[12][44] = 'G';
    g[12][45] = 'G';
    g[12][46] = 'G';

    // Flagpole
    for row in 3..=12 {
        g[row][49] = 'F';
    }

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

    // Goomba meshes
    let goomba_body_mesh = meshes.add(Ellipse::new(6.0, 5.0));
    let goomba_feet_mesh = meshes.add(Rectangle::new(12.0, 4.0));
    let goomba_body_mat =
        materials.add(ColorMaterial::from_color(Color::srgb(0.55, 0.30, 0.10)));
    let goomba_feet_mat =
        materials.add(ColorMaterial::from_color(Color::srgb(0.35, 0.18, 0.05)));

    // Koopa meshes
    let koopa_body_mesh = meshes.add(Rectangle::new(KOOPA_WIDTH, 16.0));
    let koopa_body_mat =
        materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.7, 0.2)));
    let koopa_head_mesh = meshes.add(Circle::new(5.0));
    let koopa_head_mat =
        materials.add(ColorMaterial::from_color(Color::srgb(0.3, 0.8, 0.3)));

    // Floating coin mesh
    let floating_coin_mesh = meshes.add(Circle::new(FLOATING_COIN_SIZE / 2.0));
    let floating_coin_mat =
        materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.85, 0.0)));

    // Player meshes
    let player_small_mesh = meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_SMALL_HEIGHT));
    let player_big_mesh = meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_BIG_HEIGHT));
    let fire_player_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.95, 0.95, 0.95)));
    commands.insert_resource(PlayerMeshes {
        small: player_small_mesh.clone(),
        big: player_big_mesh,
        normal_mat: player_mat.clone(),
        fire_mat: fire_player_mat,
    });

    let grid = level_test();
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
                        TilePos { col: col as i32, row: row as i32 },
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(brick_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_TILE),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                '?' | 'M' => {
                    commands.spawn((
                        Tile, TileType::QuestionBlock,
                        TilePos { col: col as i32, row: row as i32 },
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
                'G' => {
                    commands.spawn((
                        Goomba,
                        EnemyWalker { speed: GOOMBA_SPEED, direction: -1.0 },
                        CollisionSize { width: GOOMBA_WIDTH, height: GOOMBA_HEIGHT },
                        Velocity::default(),
                        Grounded::default(),
                        Mesh2d(goomba_body_mesh.clone()),
                        MeshMaterial2d(goomba_body_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_ENEMY),
                        DespawnOnExit(AppState::Playing),
                    )).with_children(|parent| {
                        parent.spawn((
                            Mesh2d(goomba_feet_mesh.clone()),
                            MeshMaterial2d(goomba_feet_mat.clone()),
                            Transform::from_xyz(0.0, -5.0, 0.0),
                        ));
                    });
                }
                'K' => {
                    commands.spawn((
                        KoopaTroopa,
                        EnemyWalker { speed: KOOPA_SPEED, direction: -1.0 },
                        CollisionSize { width: KOOPA_WIDTH, height: KOOPA_HEIGHT },
                        Velocity::default(),
                        Grounded::default(),
                        Mesh2d(koopa_body_mesh.clone()),
                        MeshMaterial2d(koopa_body_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_ENEMY),
                        DespawnOnExit(AppState::Playing),
                    )).with_children(|parent| {
                        parent.spawn((
                            Mesh2d(koopa_head_mesh.clone()),
                            MeshMaterial2d(koopa_head_mat.clone()),
                            Transform::from_xyz(0.0, 11.0, 0.0),
                        ));
                    });
                }
                'C' => {
                    commands.spawn((
                        FloatingCoin,
                        Mesh2d(floating_coin_mesh.clone()),
                        MeshMaterial2d(floating_coin_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_ITEM),
                        DespawnOnExit(AppState::Playing),
                    ));
                }
                'F' => {
                    // Thin flagpole segment
                    let pole_mesh = meshes.add(Rectangle::new(FLAGPOLE_POLE_WIDTH, TILE_SIZE));
                    let pole_mat = materials.add(ColorMaterial::from_color(
                        Color::srgb(0.5, 0.5, 0.5),
                    ));
                    commands.spawn((
                        Tile,
                        Mesh2d(pole_mesh),
                        MeshMaterial2d(pole_mat),
                        Transform::from_xyz(wx, wy, Z_TILE),
                        DespawnOnExit(AppState::Playing),
                    ));

                    // Flag at topmost pole tile
                    if row == FLAGPOLE_TOP_ROW {
                        let flag_mesh =
                            meshes.add(Rectangle::new(FLAGPOLE_FLAG_SIZE, FLAGPOLE_FLAG_SIZE));
                        let flag_mat = materials.add(ColorMaterial::from_color(
                            Color::srgb(0.2, 0.8, 0.2),
                        ));
                        commands.spawn((
                            FlagpoleFlag,
                            Mesh2d(flag_mesh),
                            MeshMaterial2d(flag_mat),
                            Transform::from_xyz(
                                wx - FLAGPOLE_FLAG_SIZE / 2.0 - FLAGPOLE_POLE_WIDTH / 2.0,
                                wy,
                                Z_TILE + 0.1,
                            ),
                            DespawnOnExit(AppState::Playing),
                        ));

                        // Ball at top of pole
                        let ball_mesh = meshes.add(Circle::new(3.0));
                        let ball_mat = materials.add(ColorMaterial::from_color(
                            Color::srgb(0.9, 0.9, 0.0),
                        ));
                        commands.spawn((
                            Mesh2d(ball_mesh),
                            MeshMaterial2d(ball_mat),
                            Transform::from_xyz(wx, wy + TILE_SIZE / 2.0 + 3.0, Z_TILE + 0.1),
                            DespawnOnExit(AppState::Playing),
                        ));
                    }
                }
                'S' => {
                    sp = (wx, wy);
                }
                _ => {}
            }
        }
    }

    // Castle — placed a few tiles right of the flagpole
    let mut flagpole_col: Option<usize> = None;
    for col in 0..LEVEL_WIDTH {
        if grid[FLAGPOLE_BOTTOM_ROW][col] == 'F' {
            flagpole_col = Some(col);
            break;
        }
    }
    if let Some(pole_col) = flagpole_col {
        let castle_col = pole_col + CASTLE_OFFSET_TILES;
        let (castle_x, _) = tile_to_world(castle_col, 12);
        let (_, ground_center_y) = tile_to_world(castle_col, 13);
        let ground_top_y = ground_center_y + TILE_SIZE / 2.0;

        // Castle body
        let body_mesh = meshes.add(Rectangle::new(48.0, 48.0));
        let body_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.5, 0.35, 0.2)));
        commands.spawn((
            Castle,
            Mesh2d(body_mesh),
            MeshMaterial2d(body_mat),
            Transform::from_xyz(castle_x, ground_top_y + 24.0, Z_DECORATION),
            DespawnOnExit(AppState::Playing),
        ));

        // Castle roof (triangle)
        let roof_mesh = meshes.add(RegularPolygon::new(20.0, 3));
        let roof_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.6, 0.15, 0.15)));
        commands.spawn((
            Mesh2d(roof_mesh),
            MeshMaterial2d(roof_mat),
            Transform::from_xyz(castle_x, ground_top_y + 58.0, Z_DECORATION),
            DespawnOnExit(AppState::Playing),
        ));

        // Castle door
        let door_mesh = meshes.add(Rectangle::new(12.0, 16.0));
        let door_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.1, 0.1, 0.1)));
        commands.spawn((
            Mesh2d(door_mesh),
            MeshMaterial2d(door_mat),
            Transform::from_xyz(castle_x, ground_top_y + 8.0, Z_DECORATION + 0.1),
            DespawnOnExit(AppState::Playing),
        ));
    }

    *spawn_point = SpawnPoint { x: sp.0, y: sp.1 };

    // Mario
    commands.spawn((
        Player,
        PlayerSize::default(),
        CollisionSize { width: PLAYER_WIDTH, height: PLAYER_SMALL_HEIGHT },
        Velocity::default(),
        FacingDirection::default(),
        Grounded::default(),
        Mesh2d(player_small_mesh),
        MeshMaterial2d(player_mat),
        Transform::from_xyz(sp.0, sp.1, Z_PLAYER),
        DespawnOnExit(AppState::Playing),
    ));

    // HUD
    ui::spawn_hud(&mut commands, &game_data);
}
