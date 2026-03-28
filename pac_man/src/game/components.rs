use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component)]
pub struct RoundEntity;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PacmanMouth;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct MessageText;

#[derive(Component, Clone, Copy)]
pub struct Pellet {
    pub kind: PelletKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PelletKind {
    Dot,
    Power,
}

#[derive(Component)]
pub struct GridMover {
    pub current: Option<Direction>,
    pub desired: Option<Direction>,
    pub speed: f32,
    pub spawn_tile: IVec2,
    pub spawn_direction: Option<Direction>,
}

#[derive(Component)]
pub struct Ghost {
    pub personality: GhostPersonality,
    pub home_tile: IVec2,
    pub scatter_target: IVec2,
    pub returning_home: bool,
}

#[derive(Component, Clone)]
pub struct GhostAppearance {
    pub normal_material: Handle<ColorMaterial>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GhostPersonality {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

impl GhostPersonality {
    pub const ORDER: [GhostPersonality; 4] = [
        GhostPersonality::Blinky,
        GhostPersonality::Pinky,
        GhostPersonality::Inky,
        GhostPersonality::Clyde,
    ];

    pub fn index(self) -> usize {
        match self {
            GhostPersonality::Blinky => 0,
            GhostPersonality::Pinky => 1,
            GhostPersonality::Inky => 2,
            GhostPersonality::Clyde => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
    ];

    pub fn vec2(self) -> Vec2 {
        match self {
            Direction::Up => Vec2::Y,
            Direction::Left => -Vec2::X,
            Direction::Down => -Vec2::Y,
            Direction::Right => Vec2::X,
        }
    }

    pub fn ivec2(self) -> IVec2 {
        match self {
            Direction::Up => IVec2::new(0, -1),
            Direction::Left => IVec2::new(-1, 0),
            Direction::Down => IVec2::new(0, 1),
            Direction::Right => IVec2::new(1, 0),
        }
    }

    pub fn rotation(self) -> Quat {
        match self {
            Direction::Right => Quat::IDENTITY,
            Direction::Up => Quat::from_rotation_z(FRAC_PI_2),
            Direction::Left => Quat::from_rotation_z(PI),
            Direction::Down => Quat::from_rotation_z(-FRAC_PI_2),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
        }
    }

    pub fn is_horizontal(self) -> bool {
        matches!(self, Direction::Left | Direction::Right)
    }
}
