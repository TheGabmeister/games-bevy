use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum FacingDirection {
    Left,
    #[default]
    Right,
}

#[derive(Component, Default)]
pub struct Grounded(pub bool);

#[derive(Component)]
pub struct Tile;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Ground,
    Brick,
    QuestionBlock,
    Empty,
    Solid,
    PipeTopLeft,
    PipeTopRight,
    PipeBodyLeft,
    PipeBodyRight,
}
