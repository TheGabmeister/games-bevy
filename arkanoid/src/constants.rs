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

// --- Bricks ---
pub const BRICK_WIDTH: f32 = 56.0;
pub const BRICK_HEIGHT: f32 = 28.0;
/// Columns across the playfield. 9 × 56 = 504px, leaving ~28px of clearance to each
/// side wall inside the 560px-wide interior.
pub const BRICK_COLS: usize = 9;
/// World-space Y of the center of the top brick row; rows descend from here.
pub const BRICK_FIELD_TOP: f32 = 300.0;

// --- Z layers ---
pub const Z_BACKGROUND: f32 = 0.0;
pub const Z_BRICK: f32 = 1.0;
pub const Z_PADDLE: f32 = 2.0;
pub const Z_BALL: f32 = 3.0;
