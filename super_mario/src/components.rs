#![allow(dead_code)]

use bevy::prelude::*;

// Player
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Default for Velocity {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Component, Default)]
pub struct Grounded(pub bool);

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
pub enum FacingDirection {
    Left,
    #[default]
    Right,
}

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
pub enum PowerState {
    #[default]
    Small,
    Big,
}

// Enemies
#[derive(Component)]
pub struct Goomba;

#[derive(Component)]
pub struct KoopaTroopa;

#[derive(Component)]
pub struct Shell;

// Tiles
#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct BrickBlock;

#[derive(Component)]
pub struct QuestionBlock {
    pub contents: BlockContents,
    pub spent: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlockContents {
    Coin,
    Mushroom,
}

#[derive(Component)]
pub struct HardBlock;

#[derive(Component)]
pub struct Pipe;

#[derive(Component)]
pub struct Flagpole;

#[derive(Component)]
pub struct Castle;

#[derive(Component)]
pub struct QuestionMarkVisual;

// Items
#[derive(Component)]
pub struct Coin;

#[derive(Component)]
pub struct Mushroom;

// Generic
#[derive(Component)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
}
