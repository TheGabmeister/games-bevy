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

// Pipe visual
pub const PIPE_LIP_OVERHANG: f32 = 2.0;

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

// Camera scrolling
pub const CAMERA_FIXED_Y: f32 = -20.0;
pub const CAMERA_LERP_SPEED: f32 = 10.0;
pub const CAMERA_DEAD_ZONE_OFFSET: f32 = CAMERA_VISIBLE_WIDTH / 6.0;
pub const CAMERA_MIN_X: f32 = LEVEL_ORIGIN_X + CAMERA_VISIBLE_WIDTH / 2.0;
pub const CAMERA_MAX_X: f32 = LEVEL_ORIGIN_X + 211.0 * TILE_SIZE - CAMERA_VISIBLE_WIDTH / 2.0;

// Physics
pub const GRAVITY_ASCENDING: f32 = 600.0;
pub const GRAVITY_DESCENDING: f32 = 980.0;
pub const TERMINAL_VELOCITY: f32 = 500.0;

// Timer
pub const TIMER_START: f32 = 400.0;
pub const TIMER_TICK_RATE: f32 = 2.5; // ticks per second

// Death
pub const DEATH_Y: f32 = -160.0;
pub const DEATH_BOUNCE_IMPULSE: f32 = 280.0;
pub const DEATH_PAUSE_DURATION: f32 = 0.5;
pub const DEATH_FALL_DURATION: f32 = 3.0;

// Goomba
pub const GOOMBA_WIDTH: f32 = 14.0;
pub const GOOMBA_HEIGHT: f32 = 14.0;
pub const GOOMBA_SPEED: f32 = 30.0;
pub const STOMP_BOUNCE_IMPULSE: f32 = 200.0;
pub const SQUISH_DURATION: f32 = 0.4;
pub const STOMP_SCORE: u32 = 100;

// Score popup
pub const SCORE_POPUP_DURATION: f32 = 1.0;
pub const SCORE_POPUP_RISE_SPEED: f32 = 60.0;

// Block interactions
pub const BLOCK_BOUNCE_HEIGHT: f32 = 4.0;
pub const BLOCK_BOUNCE_DURATION: f32 = 0.15;
pub const COIN_POP_IMPULSE: f32 = 200.0;
pub const COIN_POP_DURATION: f32 = 0.4;
pub const COIN_SCORE: u32 = 200;
pub const FLOATING_COIN_SCORE: u32 = 200;
pub const FLOATING_COIN_SIZE: f32 = 8.0;
pub const COINS_PER_LIFE: u32 = 100;
pub const BRICK_PARTICLE_SIZE: f32 = 4.0;
pub const BRICK_PARTICLE_SPEED: f32 = 100.0;
pub const BRICK_PARTICLE_DURATION: f32 = 0.8;
pub const BRICK_BUMP_KILL_RANGE: f32 = 18.0;

// Mushroom
pub const MUSHROOM_SPEED: f32 = 50.0;
pub const MUSHROOM_EMERGE_SPEED: f32 = 30.0;
pub const MUSHROOM_WIDTH: f32 = 14.0;
pub const MUSHROOM_HEIGHT: f32 = 14.0;
pub const MUSHROOM_SCORE: u32 = 1000;

// Growth
pub const GROWTH_DURATION: f32 = 1.0;
pub const GROWTH_FLASH_INTERVAL: f32 = 0.125;

// Invincibility (damage)
pub const INVINCIBILITY_DURATION: f32 = 2.0;

// Koopa
pub const KOOPA_WIDTH: f32 = 14.0;
pub const KOOPA_HEIGHT: f32 = 22.0;
pub const KOOPA_SPEED: f32 = 30.0;
pub const SHELL_SPEED: f32 = 180.0;
pub const SHELL_WIDTH: f32 = 14.0;
pub const SHELL_HEIGHT: f32 = 14.0;
pub const SHELL_BASE_SCORE: u32 = 200;
