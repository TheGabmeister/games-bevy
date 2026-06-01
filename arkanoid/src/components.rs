use bevy::prelude::*;

/// Linear velocity in pixels per second.
#[derive(Component)]
pub struct Velocity(pub Vec2);

/// The player-controlled Vaus paddle.
#[derive(Component)]
pub struct Paddle;

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
