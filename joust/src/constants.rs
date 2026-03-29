// Window
pub const WINDOW_WIDTH: f32 = 1200.0;
pub const WINDOW_HEIGHT: f32 = 900.0;

// Arena
pub const ARENA_LEFT: f32 = -600.0;
pub const ARENA_RIGHT: f32 = 600.0;
pub const ARENA_TOP: f32 = 450.0;
pub const ARENA_BOTTOM: f32 = -450.0;
pub const ARENA_WIDTH: f32 = 1200.0;

// Physics
pub const GRAVITY: f32 = 980.0;
pub const FLAP_IMPULSE: f32 = 320.0;
pub const MAX_RISE_SPEED: f32 = 400.0;
pub const MAX_FALL_SPEED: f32 = 600.0;
pub const AIR_ACCEL: f32 = 800.0;
pub const GROUND_ACCEL: f32 = 400.0;
pub const AIR_DRAG: f32 = 350.0;
pub const GROUND_DRAG: f32 = 500.0;
pub const MAX_AIR_SPEED: f32 = 300.0;
pub const MAX_GROUND_SPEED: f32 = 150.0;

// Platforms
pub const PLATFORM_SNAP_DISTANCE: f32 = 3.0;
pub const PLATFORM_THICKNESS: f32 = 12.0;

// Combat
pub const JOUST_POINT_Y: f32 = 18.0;
pub const JOUST_DEAD_ZONE: f32 = 8.0;
pub const BOUNCE_HORIZONTAL: f32 = 250.0;
pub const BOUNCE_VERTICAL: f32 = 150.0;

// Radii
pub const RIDER_RADIUS: f32 = 18.0;
pub const EGG_RADIUS: f32 = 10.0;

// Timers
pub const EGG_HATCH_TIME_BASE: f32 = 15.0;
pub const LAVA_Y: f32 = -410.0;
pub const PLAYER_RESPAWN_DELAY: f32 = 1.5;
pub const BOUNCE_INVINCIBILITY: f32 = 0.3;
pub const RESPAWN_INVINCIBILITY: f32 = 2.0;
pub const FLAP_COOLDOWN: f32 = 0.15;

// Scoring
pub const MAX_LIVES: u32 = 5;
pub const EXTRA_LIFE_INTERVAL: u32 = 10_000;
pub const SCORE_KILL_BOUNDER: u32 = 500;
pub const SCORE_KILL_HUNTER: u32 = 750;
pub const SCORE_KILL_SHADOW_LORD: u32 = 1000;
pub const SCORE_COLLECT_EGG: u32 = 250;

// Screen wrap
pub const WRAP_MARGIN: f32 = 20.0;

// Platform layout
#[derive(Clone, Copy)]
pub struct PlatformDef {
    pub center_x: f32,
    pub y: f32,
    pub width: f32,
    pub wraps: bool,
}

pub const PLATFORMS: [PlatformDef; 6] = [
    PlatformDef { center_x: 0.0, y: -380.0, width: 1200.0, wraps: true },
    PlatformDef { center_x: -350.0, y: -180.0, width: 280.0, wraps: false },
    PlatformDef { center_x: 350.0, y: -180.0, width: 280.0, wraps: false },
    PlatformDef { center_x: 0.0, y: 50.0, width: 320.0, wraps: false },
    PlatformDef { center_x: -400.0, y: 230.0, width: 220.0, wraps: false },
    PlatformDef { center_x: 400.0, y: 230.0, width: 220.0, wraps: false },
];

// Wave timing
pub const WAVE_INTRO_DURATION: f32 = 2.0;
pub const WAVE_CLEAR_DURATION: f32 = 1.5;

// Z layers
pub const Z_LAVA_BACK: f32 = 1.0;
pub const Z_PLATFORMS: f32 = 2.0;
pub const Z_EGGS: f32 = 3.0;
pub const Z_ENEMIES: f32 = 4.0;
pub const Z_PLAYERS: f32 = 5.0;
pub const Z_PARTICLES: f32 = 6.0;
pub const Z_LAVA_FRONT: f32 = 7.0;
pub const Z_WAVE_TEXT: f32 = 8.0;

// Spawn positions
pub const PLAYER1_SPAWN: (f32, f32) = (-200.0, -350.0);
pub const PLAYER2_SPAWN: (f32, f32) = (200.0, -350.0);

pub const ENEMY_SPAWN_POSITIONS: [(f32, f32); 5] = [
    (-350.0, -168.0),
    (350.0, -168.0),
    (0.0, 62.0),
    (-400.0, 242.0),
    (400.0, 242.0),
];

// AI
pub const AI_FLAP_INTERVAL_BASE: f32 = 0.4;
pub const AI_FLAP_RANDOMNESS: f32 = 0.3;
pub const AI_PURSUE_RANGE: f32 = 400.0;
pub const AI_DIRECTION_CHANGE_INTERVAL: f32 = 2.0;
