use bevy::prelude::*;
use bevy::asset::io::Reader as AssetReader;
use bevy::asset::{AssetLoader, LoadContext};
use serde::Deserialize;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::ui;

// ── Level Data Asset ──

/// Level data loaded from a RON file.
///
/// RON format:
/// ```ron
/// (
///     rows: [ "..row0..", "..row1..", ... ],
///     // All fields below are optional (defaults used if omitted):
///     time: 400,
///     world_name: "1-1",
///     background_color: (0.36, 0.53, 0.95),
///     gravity_multiplier: 1.0,
/// )
/// ```
#[derive(Asset, Reflect, Deserialize)]
pub struct LevelData {
    pub rows: Vec<String>,

    /// Starting timer value. Defaults to TIMER_START (400).
    #[serde(default = "default_time")]
    pub time: f32,

    /// Display name for HUD (e.g. "1-1", "1-2"). Defaults to "1-1".
    #[serde(default = "default_world_name")]
    pub world_name: String,

    /// Background color as (r, g, b) in sRGB 0.0–1.0. Defaults to sky blue.
    #[serde(default = "default_background_color")]
    pub background_color: (f32, f32, f32),

    /// Multiplier for gravity constants. 1.0 = normal, <1.0 = floaty (underwater).
    #[serde(default = "default_gravity_multiplier")]
    pub gravity_multiplier: f32,

    /// Level theme. Controls which decorations are spawned.
    /// "overworld" (default) = clouds, bushes, hills.
    /// "underground" / "underwater" = no outdoor decorations.
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_time() -> f32 {
    TIMER_START
}
fn default_world_name() -> String {
    "1-1".to_string()
}
fn default_background_color() -> (f32, f32, f32) {
    (0.36, 0.53, 0.95)
}
fn default_gravity_multiplier() -> f32 {
    1.0
}
fn default_theme() -> String {
    "overworld".to_string()
}

impl LevelData {
    /// Convert the loaded row strings into the fixed-size char grid used at runtime.
    pub fn to_grid(&self) -> [[char; LEVEL_WIDTH]; LEVEL_HEIGHT] {
        let mut grid = [['.'; LEVEL_WIDTH]; LEVEL_HEIGHT];
        for (r, row_str) in self.rows.iter().enumerate() {
            if r >= LEVEL_HEIGHT {
                break;
            }
            for (c, ch) in row_str.chars().enumerate() {
                if c >= LEVEL_WIDTH {
                    break;
                }
                grid[r][c] = ch;
            }
        }
        grid
    }
}

/// Custom asset loader for `.level.ron` files.
#[derive(Default, Reflect)]
pub struct LevelAssetLoader;

impl AssetLoader for LevelAssetLoader {
    type Asset = LevelData;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn AssetReader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        bevy::asset::io::Reader::read_to_end(reader, &mut bytes).await?;
        let data: LevelData = ron::de::from_bytes(&bytes)?;
        Ok(data)
    }

    fn extensions(&self) -> &[&str] {
        &["level.ron"]
    }
}

/// Stores the handle to the currently active level asset.
#[derive(Resource)]
pub struct LevelHandle(pub Handle<LevelData>);

// ── Spawner Registry ──

/// Closure-based spawner. Each closure captures its own asset handles at registration time.
pub type TileSpawner = Box<dyn Fn(&mut Commands, f32, f32, usize, usize) + Send + Sync>;

/// Data-driven registry mapping level characters to spawn functions.
/// Modules register their spawners at startup; `spawn_level` looks them up.
#[derive(Resource, Default)]
pub struct SpawnerRegistry {
    spawners: std::collections::HashMap<char, TileSpawner>,
}

impl SpawnerRegistry {
    pub fn register(&mut self, ch: char, spawner: TileSpawner) {
        self.spawners.insert(ch, spawner);
    }

    pub fn get(&self, ch: char) -> Option<&TileSpawner> {
        self.spawners.get(&ch)
    }
}

/// Register the built-in tile and entity spawners.
/// Each closure captures the asset handles it needs so spawners are self-contained.
pub fn init_spawner_registry(
    mut reg: ResMut<SpawnerRegistry>,
    assets: Res<crate::assets::GameAssets>,
) {
    // Tiles — capture tile handles once
    let t = assets.tile.clone();
    reg.register('#', { let t = t.clone(); Box::new(move |c, wx, wy, _, _| { t.spawn(c, TileType::Ground, wx, wy, None); }) });
    reg.register('B', { let t = t.clone(); Box::new(move |c, wx, wy, col, row| { t.spawn(c, TileType::Brick, wx, wy, Some((col as i32, row as i32))); }) });
    reg.register('?', { let t = t.clone(); Box::new(move |c, wx, wy, col, row| { t.spawn(c, TileType::QuestionBlock, wx, wy, Some((col as i32, row as i32))); }) });
    reg.register('M', { let t = t.clone(); Box::new(move |c, wx, wy, col, row| { t.spawn(c, TileType::QuestionBlock, wx, wy, Some((col as i32, row as i32))); }) });
    reg.register('T', { let t = t.clone(); Box::new(move |c, wx, wy, col, row| { t.spawn(c, TileType::QuestionBlock, wx, wy, Some((col as i32, row as i32))); }) });
    reg.register('L', { let t = t.clone(); Box::new(move |c, wx, wy, col, row| { t.spawn(c, TileType::QuestionBlock, wx, wy, Some((col as i32, row as i32))); }) });
    reg.register('X', { let t = t.clone(); Box::new(move |c, wx, wy, _, _| { t.spawn(c, TileType::Solid, wx, wy, None); }) });
    reg.register('[', { let t = t.clone(); Box::new(move |c, wx, wy, _, _| { t.spawn(c, TileType::PipeTopLeft, wx, wy, None); }) });
    reg.register(']', { let t = t.clone(); Box::new(move |c, wx, wy, _, _| { t.spawn(c, TileType::PipeTopRight, wx, wy, None); }) });
    reg.register('{', { let t = t.clone(); Box::new(move |c, wx, wy, _, _| { t.spawn(c, TileType::PipeBodyLeft, wx, wy, None); }) });
    reg.register('}', { let t = t.clone(); Box::new(move |c, wx, wy, _, _| { t.spawn(c, TileType::PipeBodyRight, wx, wy, None); }) });

    // Non-enemy entities
    let fc = assets.floating_coin.clone();
    reg.register('C', Box::new(move |c, wx, wy, _, _| { fc.spawn(c, wx, wy); }));

    let fp = assets.flagpole.clone();
    reg.register('F', Box::new(move |c, wx, wy, _, row| {
        fp.spawn_pole(c, wx, wy);
        if row == FLAGPOLE_TOP_ROW {
            fp.spawn_top(c, wx, wy);
        }
    }));
}

/// Load the current level asset based on LevelList.
pub fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_list: Res<LevelList>,
) {
    let handle: Handle<LevelData> = asset_server.load(level_list.current_path());
    commands.insert_resource(LevelHandle(handle));
}

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
            '#' | 'B' | '?' | 'M' | 'X' | '[' | ']' | '{' | '}' | 'E' | 'T' | 'L'
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
        matches!(self.get_char(col, row), '?' | 'M' | 'B' | 'T' | 'L')
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
/// `T` question (starman)   `L` question (1-up mushroom)
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

    // Overhead blocks: ? (coin), M (mushroom) x2, T (star), L (1-up), B (brick)
    g[9][5] = 'M';
    g[9][6] = 'M';
    g[9][8] = '?';
    g[9][9] = 'T';
    g[9][10] = 'M';
    g[9][11] = 'B';
    g[9][12] = 'L';

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
    assets: Res<crate::assets::GameAssets>,
    mut game_data: ResMut<GameData>,
    mut game_timer: ResMut<GameTimer>,
    mut spawn_point: ResMut<SpawnPoint>,
    level_handle: Res<LevelHandle>,
    level_assets: Res<Assets<LevelData>>,
    registry: Res<SpawnerRegistry>,
    deco_assets: Res<crate::decoration::DecorationAssets>,
) {
    let level_data = level_assets.get(&level_handle.0);

    // Apply level metadata (or defaults)
    let (time, world_name, bg_color) = match level_data {
        Some(data) => (
            data.time,
            data.world_name.clone(),
            Color::srgb(data.background_color.0, data.background_color.1, data.background_color.2),
        ),
        None => (
            TIMER_START,
            "1-1".to_string(),
            Color::srgb(0.36, 0.53, 0.95),
        ),
    };

    *game_data = GameData {
        world_name,
        ..GameData::default()
    };
    *game_timer = GameTimer { time };
    commands.insert_resource(ClearColor(bg_color));

    // Load grid from the RON asset, fall back to hardcoded test level
    let grid = level_data
        .map(|data| data.to_grid())
        .unwrap_or_else(level_test);
    commands.insert_resource(LevelGrid { grid });

    let mut sp = (0.0_f32, 0.0_f32);

    for row in 0..LEVEL_HEIGHT {
        for col in 0..LEVEL_WIDTH {
            let ch = grid[row][col];
            if ch == '.' {
                continue;
            }

            let (wx, wy) = tile_to_world(col, row);

            if ch == 'S' {
                sp = (wx, wy);
            } else if let Some(spawner) = registry.get(ch) {
                spawner(&mut commands, wx, wy, col, row);
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

        assets.castle.spawn(&mut commands, castle_x, ground_top_y);
    }

    *spawn_point = SpawnPoint { x: sp.0, y: sp.1 };

    // Mario
    assets.player.spawn(&mut commands, sp.0, sp.1);

    // Decorations (only for overworld theme)
    let theme = level_data
        .map(|d| d.theme.as_str())
        .unwrap_or("overworld");
    if theme == "overworld" {
        let level_grid = LevelGrid { grid };
        crate::decoration::spawn_decorations(&mut commands, &level_grid, &deco_assets);
    }

    // HUD
    ui::spawn_hud(&mut commands, &game_data, &game_timer);
}

// ── RON file generation (dev helper) ──

fn grid_to_ron(grid: &[[char; LEVEL_WIDTH]; LEVEL_HEIGHT]) -> String {
    let rows: Vec<String> = grid
        .iter()
        .map(|row| {
            let s: String = row.iter().collect();
            format!("        \"{}\"", s)
        })
        .collect();
    format!("(\n    rows: [\n{}\n    ]\n)\n", rows.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_level_ron_files() {
        let dir = std::path::Path::new("assets/levels");
        std::fs::create_dir_all(dir).unwrap();

        let test_grid = level_test();
        std::fs::write(dir.join("test.level.ron"), grid_to_ron(&test_grid)).unwrap();

        let grid_1_1 = level_1_1();
        std::fs::write(dir.join("1-1.level.ron"), grid_to_ron(&grid_1_1)).unwrap();
    }
}
