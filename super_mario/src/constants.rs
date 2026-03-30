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

// Physics
pub const GRAVITY_ASCENDING: f32 = 600.0;
pub const GRAVITY_DESCENDING: f32 = 980.0;
pub const TERMINAL_VELOCITY: f32 = 500.0;
