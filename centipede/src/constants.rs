pub const GRID_COLS: i32 = 25;
pub const GRID_ROWS: i32 = 30;
pub const CELL_SIZE: f32 = 28.0;
pub const WINDOW_WIDTH: f32 = 700.0;
pub const WINDOW_HEIGHT: f32 = 840.0;

pub const PLAYER_ZONE_ROW_START: i32 = 25;
pub const PLAYER_SPEED: f32 = 200.0;
pub const BULLET_SPEED: f32 = 500.0;

pub const CENTIPEDE_INTERVAL_BASE: f32 = 0.15;
pub const CENTIPEDE_LENGTH: usize = 12;

pub const FLEA_SPEED: f32 = 250.0;
pub const SPIDER_SPEED: f32 = 130.0;
pub const SCORPION_SPEED: f32 = 180.0;

pub const INITIAL_MUSHROOM_COUNT: usize = 40;
pub const INITIAL_LIVES: u32 = 3;
pub const EXTRA_LIFE_SCORE: u32 = 12000;
pub const MUSHROOM_MAX_HITS: u8 = 4;
pub const _FLEA_MAX_HITS: u8 = 2;
pub const MIN_MUSHROOMS_IN_PLAYER_ZONE: usize = 5;

pub const FLEA_SPAWN_INTERVAL: f32 = 3.0;
pub const SPIDER_SPAWN_INTERVAL: f32 = 15.0;
pub const SCORPION_SPAWN_INTERVAL: f32 = 20.0;
pub const RESPAWN_DELAY: f32 = 2.0;

/// Convert grid column to world X
pub fn grid_to_world_x(col: i32) -> f32 {
    (col as f32 - 12.0) * CELL_SIZE + CELL_SIZE / 2.0
}

/// Convert grid row to world Y
pub fn grid_to_world_y(row: i32) -> f32 {
    (14.5 - row as f32) * CELL_SIZE
}

/// Convert world X to nearest grid column
pub fn world_to_grid_col(x: f32) -> i32 {
    ((x / CELL_SIZE) + 12.0).floor() as i32
}

/// Convert world Y to nearest grid row
pub fn world_to_grid_row(y: f32) -> i32 {
    (14.5 - y / CELL_SIZE).floor() as i32
}

pub fn player_zone_min_y() -> f32 {
    grid_to_world_y(GRID_ROWS - 1) - CELL_SIZE / 2.0
}

pub fn player_zone_max_y() -> f32 {
    grid_to_world_y(PLAYER_ZONE_ROW_START) + CELL_SIZE / 2.0
}
