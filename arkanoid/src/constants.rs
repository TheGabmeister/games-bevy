use std::f32::consts::FRAC_PI_3;

// --- Window ---
// Portrait orientation (taller than wide), matching the classic vertical arcade cabinet.
pub const WINDOW_WIDTH: f32 = 600.0;
pub const WINDOW_HEIGHT: f32 = 800.0;

// --- Playfield ---
// The border frame is 20px thick on the top/left/right edges; the bottom is open.
// World space has the camera centered at the origin, so x ∈ [-300, 300], y ∈ [-400, 400].
pub const WALL_THICKNESS: f32 = 20.0;
pub const PLAYFIELD_LEFT: f32 = -WINDOW_WIDTH / 2.0 + WALL_THICKNESS;
pub const PLAYFIELD_RIGHT: f32 = WINDOW_WIDTH / 2.0 - WALL_THICKNESS;
pub const PLAYFIELD_TOP: f32 = WINDOW_HEIGHT / 2.0 - WALL_THICKNESS;
pub const PLAYFIELD_BOTTOM: f32 = -WINDOW_HEIGHT / 2.0;

// --- Vaus paddle ---
pub const PADDLE_WIDTH: f32 = 96.0;
/// Width of the Vaus while the Expand power-up is active (matches `vaus-expanded.png`).
pub const PADDLE_EXPANDED_WIDTH: f32 = 160.0;
pub const PADDLE_HEIGHT: f32 = 24.0;
pub const PADDLE_Y: f32 = PLAYFIELD_BOTTOM + 60.0;
/// Keyboard / gamepad travel speed in pixels per second.
pub const PADDLE_SPEED: f32 = 600.0;

// --- Ball ---
pub const BALL_SIZE: f32 = 16.0;
pub const BALL_RADIUS: f32 = BALL_SIZE / 2.0;
pub const BALL_SPEED: f32 = 340.0;
/// Maximum rebound angle off the paddle, measured from straight up (a hit at the
/// paddle's edge deflects this far; a centered hit goes straight up).
pub const BALL_MAX_BOUNCE_ANGLE: f32 = FRAC_PI_3; // 60°

// --- Ball speed progression ---
// The ball starts each serve at `BALL_SPEED` and accelerates within the round —
// on a fixed time cadence and at brick-count milestones — capped at `BALL_SPEED_MAX`.
/// Upper bound the ball speed ramps toward; it never exceeds this within a round.
pub const BALL_SPEED_MAX: f32 = 560.0;
/// Pixels/second added to the ball speed at each acceleration step.
pub const BALL_SPEEDUP_STEP: f32 = 28.0;
/// Seconds of live play between time-based speed bumps.
pub const BALL_SPEEDUP_INTERVAL: f32 = 8.0;
/// Bricks destroyed between milestone speed bumps (every Nth brick nudges the speed).
pub const BALL_SPEEDUP_BRICKS: u32 = 8;
/// Floor the ball speed can be slowed to by the Slow power-up.
pub const BALL_SPEED_MIN: f32 = 220.0;
/// Multiplier the Slow power-up applies to the current ball speed.
pub const SLOW_FACTOR: f32 = 0.6;

// --- Lives & round flow ---
pub const LIVES_START: u32 = 3;
/// Seconds the "ROUND n READY" intro is shown before the ball can be served.
pub const READY_DURATION: f32 = 1.6;

// --- Bricks ---
pub const BRICK_WIDTH: f32 = 56.0;
pub const BRICK_HEIGHT: f32 = 28.0;
/// Columns across the playfield. 9 × 56 = 504px, leaving ~28px of clearance to each
/// side wall inside the 560px-wide interior.
pub const BRICK_COLS: usize = 9;
/// World-space Y of the center of the top brick row; rows descend from here.
pub const BRICK_FIELD_TOP: f32 = 300.0;

// --- Silver & gold bricks ---
/// Hits a silver brick takes in round 1; rises by one every `SILVER_HITS_ROUND_STEP` rounds.
pub const SILVER_BASE_HITS: u32 = 2;
/// Rounds between each +1 to a silver brick's required hit count.
pub const SILVER_HITS_ROUND_STEP: u32 = 8;
/// A destroyed silver brick scores this times the current round number.
pub const SILVER_POINTS_PER_ROUND: u32 = 50;

// --- Power-up capsules ---
pub const CAPSULE_WIDTH: f32 = 32.0;
pub const CAPSULE_HEIGHT: f32 = 16.0;
/// Fall speed of a released capsule, in pixels/second.
pub const CAPSULE_FALL_SPEED: f32 = 150.0;
/// Bricks destroyed between capsule drops. Only one capsule falls at a time.
pub const CAPSULE_DROP_INTERVAL: u32 = 5;

// --- Laser (Laser power-up) ---
pub const LASER_WIDTH: f32 = 8.0;
pub const LASER_HEIGHT: f32 = 24.0;
pub const LASER_SPEED: f32 = 620.0;
/// Max laser bolts in flight at once. Bolts fire in pairs, so this caps the volleys.
pub const LASER_MAX_BOLTS: usize = 4;
/// Horizontal offset of each laser muzzle from the paddle center.
pub const LASER_MUZZLE_OFFSET: f32 = 30.0;

// --- Warp-exit gate (Break power-up) ---
pub const WARP_GATE_WIDTH: f32 = 32.0;

// --- Enemy aliens ---
pub const ENEMY_SIZE: f32 = 32.0;
/// Base downward drift speed, in pixels/second (scaled per enemy type).
pub const ENEMY_SPEED: f32 = 70.0;
/// Peak horizontal wander speed, in pixels/second.
pub const ENEMY_DRIFT_SPEED: f32 = 60.0;
pub const ENEMY_ANIM_FRAME_TIME: f32 = 0.16;
/// Seconds between enemy spawns while a round is running.
pub const ENEMY_SPAWN_INTERVAL: f32 = 5.0;
/// Max enemies alive at once.
pub const ENEMY_MAX_ACTIVE: usize = 3;
/// Pyramid weave frequency (radians/second of its sine sway).
pub const ENEMY_PYRAMID_WEAVE: f32 = 2.4;
/// Cube weave frequency (radians/second); cubes sway gently and sink slowly.
pub const ENEMY_CUBE_WEAVE: f32 = 1.2;
/// Seconds between a molecule's sharp zig-zag direction flips.
pub const ENEMY_MOLECULE_FLIP: f32 = 0.7;

// --- Enemy spawn gates ---
pub const GATE_HEIGHT: f32 = 24.0;
/// World-space Y of the spawn gates, tucked just under the top wall.
pub const GATE_Y: f32 = PLAYFIELD_TOP - GATE_HEIGHT / 2.0;
/// Gate X positions are ±this offset from center (two gates).
pub const GATE_X_OFFSET: f32 = 120.0;
/// Seconds a gate stays open after launching an enemy.
pub const GATE_OPEN_DURATION: f32 = 0.6;

// --- VFX ---
/// Seconds each frame of an animated VFX flipbook is shown.
pub const VFX_FRAME_TIME: f32 = 0.05;

// --- Z layers ---
pub const Z_BACKGROUND: f32 = 0.0;
pub const Z_GATE: f32 = 0.5;
pub const Z_BRICK: f32 = 1.0;
pub const Z_WARP_GATE: f32 = 1.5;
pub const Z_PADDLE: f32 = 2.0;
pub const Z_ENEMY: f32 = 2.5;
pub const Z_BALL: f32 = 3.0;
pub const Z_CAPSULE: f32 = 4.0;
pub const Z_LASER: f32 = 4.0;
pub const Z_VFX: f32 = 5.0;
/// Full-screen overlay art (e.g. the title screen) drawn above the playfield.
pub const Z_OVERLAY: f32 = 100.0;
