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

/// A single-hit colored brick. Destroyed in one ball contact, awarding `points`.
#[derive(Component)]
pub struct Brick {
    pub points: u32,
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
