use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    Input,
    Movement,
    Collision,
    Cleanup,
}
