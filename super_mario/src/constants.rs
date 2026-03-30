// Window
pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

// Camera — visible area in world units (~NES resolution)
pub const CAMERA_VISIBLE_WIDTH: f32 = 267.0;
pub const CAMERA_VISIBLE_HEIGHT: f32 = 200.0;
pub const CAMERA_SCALE: f32 = CAMERA_VISIBLE_HEIGHT / WINDOW_HEIGHT;

// Grid
pub const TILE_SIZE: f32 = 16.0;

// Player dimensions
pub const PLAYER_WIDTH: f32 = 14.0;
pub const PLAYER_SMALL_HEIGHT: f32 = 16.0;
pub const PLAYER_BIG_HEIGHT: f32 = 32.0;

// Player movement
pub const PLAYER_WALK_SPEED: f32 = 130.0;
pub const PLAYER_RUN_SPEED: f32 = 200.0;
pub const PLAYER_ACCELERATION: f32 = 600.0;
pub const PLAYER_DECELERATION: f32 = 500.0;
pub const PLAYER_AIR_ACCELERATION: f32 = 400.0;

// Jump
pub const PLAYER_JUMP_IMPULSE: f32 = 330.0;
pub const JUMP_CUT_MULTIPLIER: f32 = 0.4;

// Ground (temporary — replaced by tile map in Phase 2)
pub const GROUND_Y: f32 = -80.0;
pub const GROUND_WIDTH: f32 = 800.0;
pub const GROUND_HEIGHT: f32 = 16.0;

// Level origin — world position of tile (col=0, row=14) bottom-left corner.
// Row 14 is the bottom row; row 0 is the top. Y increases upward in world space.
pub const LEVEL_ORIGIN_X: f32 = 0.0;
pub const LEVEL_ORIGIN_Y: f32 = -120.0;

// Z-layers (render ordering, back to front)
pub const Z_DECORATION: f32 = 1.0;
pub const Z_PIPE: f32 = 2.0;
pub const Z_TILE: f32 = 3.0;
pub const Z_ITEM: f32 = 4.0;
pub const Z_ENEMY: f32 = 5.0;
pub const Z_PLAYER: f32 = 6.0;

// Physics
pub const GRAVITY_ASCENDING: f32 = 600.0;
pub const GRAVITY_DESCENDING: f32 = 980.0;
pub const TERMINAL_VELOCITY: f32 = 500.0;
