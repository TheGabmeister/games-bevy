use bevy::prelude::*;
use crate::STARTING_LIVES;

/// All mutable game state in one place.
#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub lives: u32,
    pub wave: u32,
    pub shoot_timer: f32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            lives: STARTING_LIVES,
            wave: 1,
            shoot_timer: 0.0,
        }
    }
}

/// Pre-built mesh and material handles reused by dynamic spawning.
/// Stored as a resource so systems can clone handles cheaply without
/// borrowing Assets<Mesh> / Assets<ColorMaterial> every frame.
#[derive(Resource)]
pub struct GameAssets {
    pub ship_mesh: Handle<Mesh>,
    pub ship_material: Handle<ColorMaterial>,
    pub bullet_mesh: Handle<Mesh>,
    pub bullet_material: Handle<ColorMaterial>,
    pub asteroid_large_mesh: Handle<Mesh>,
    pub asteroid_medium_mesh: Handle<Mesh>,
    pub asteroid_small_mesh: Handle<Mesh>,
    pub asteroid_material: Handle<ColorMaterial>,
}
