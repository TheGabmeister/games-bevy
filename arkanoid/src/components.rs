use bevy::prelude::*;

/// Linear velocity in pixels per second.
#[derive(Component)]
pub struct Velocity(pub Vec2);

/// The player-controlled Vaus paddle. `half_width` is dynamic — the Expand power-up
/// widens it — and is read by movement clamping and ball/capsule collision.
#[derive(Component)]
pub struct Paddle {
    pub half_width: f32,
}

/// The energy ball. While `stuck`, it rides on top of the paddle and waits to be
/// launched; once launched it moves under its own `Velocity`.
#[derive(Component)]
pub struct Ball {
    pub stuck: bool,
}

/// A brick on the playfield. `hits_remaining` reaches 0 to destroy it (1 for colored
/// bricks, more for silver); `max_hits` is the starting durability, used to pick the
/// silver damage frame. Indestructible (gold) bricks carry the [`Indestructible`] marker
/// and never reach 0. Awards `points` when destroyed.
#[derive(Component)]
pub struct Brick {
    pub points: u32,
    pub hits_remaining: u32,
    pub max_hits: u32,
}

/// Marks a multi-hit silver brick, so the damage-feedback system can find it and swap in
/// the cracked sprite frame as its `hits_remaining` drops.
#[derive(Component)]
pub struct Silver;

/// Marks a gold brick: an indestructible obstacle that deflects the ball, scores nothing,
/// and is excluded from the round-clear check.
#[derive(Component)]
pub struct Indestructible;

/// What a brick is made of — governs durability, scoring, and which sprite it uses.
/// Parsed from a layout cell via [`BrickKind::from_code`].
#[derive(Clone, Copy)]
pub enum BrickKind {
    /// Single-hit colored brick.
    Colored(BrickColor),
    /// Multi-hit brick whose required hits and points scale with the round.
    Silver,
    /// Indestructible obstacle.
    Gold,
}

impl BrickKind {
    /// Maps a single-character layout code to a brick kind: a color code (see
    /// [`BrickColor::from_code`]), `'S'` for silver, or `'X'` for gold.
    pub fn from_code(code: char) -> Option<BrickKind> {
        if let Some(color) = BrickColor::from_code(code) {
            return Some(BrickKind::Colored(color));
        }
        match code {
            'S' => Some(BrickKind::Silver),
            'X' => Some(BrickKind::Gold),
            _ => None,
        }
    }
}

/// The seven Arkanoid power-ups, each released by a falling [`Capsule`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerupKind {
    /// Catch: the ball sticks to the paddle and re-launches on input.
    Catch,
    /// Laser: the Vaus fires laser bolts that destroy bricks.
    Laser,
    /// Expand: the paddle grows wider.
    Expand,
    /// Disruption: splits each live ball into three.
    Disruption,
    /// Slow: reduces the ball speed.
    Slow,
    /// Break: opens a warp exit on the right edge to skip to the next round.
    Break,
    /// Player: awards an extra life.
    Player,
}

/// A power-up capsule falling from a destroyed brick. Caught by paddle touch.
#[derive(Component)]
pub struct Capsule {
    pub kind: PowerupKind,
}

/// A laser bolt fired by the Vaus while the Laser power-up is active; travels straight up.
#[derive(Component)]
pub struct Laser;

/// The warp-exit gate opened by the Break power-up on the right edge; the Vaus escapes
/// through it to skip to the next round.
#[derive(Component)]
pub struct WarpGate;

/// A short-lived animated VFX sprite (frame flipbook) that despawns after its last frame.
#[derive(Component)]
pub struct VfxAnim {
    pub frames: Vec<Handle<Image>>,
    pub index: usize,
    pub timer: Timer,
}

/// The eight colored brick variants. Each maps to its sprite and point value.
#[derive(Clone, Copy)]
pub enum BrickColor {
    White,
    Orange,
    Cyan,
    Green,
    Red,
    Blue,
    Pink,
    Yellow,
}

impl BrickColor {
    /// Point value awarded when destroyed (ascending by color, classic-style).
    pub fn points(self) -> u32 {
        match self {
            BrickColor::White => 50,
            BrickColor::Orange => 60,
            BrickColor::Cyan => 70,
            BrickColor::Green => 80,
            BrickColor::Red => 90,
            BrickColor::Blue => 100,
            BrickColor::Pink => 110,
            BrickColor::Yellow => 120,
        }
    }

    /// Maps the single-character layout code used in hand-built round layouts.
    pub fn from_code(code: char) -> Option<BrickColor> {
        match code {
            'W' => Some(BrickColor::White),
            'O' => Some(BrickColor::Orange),
            'C' => Some(BrickColor::Cyan),
            'G' => Some(BrickColor::Green),
            'R' => Some(BrickColor::Red),
            'B' => Some(BrickColor::Blue),
            'P' => Some(BrickColor::Pink),
            'Y' => Some(BrickColor::Yellow),
            _ => None,
        }
    }
}
