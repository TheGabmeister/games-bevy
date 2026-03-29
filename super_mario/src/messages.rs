#![allow(dead_code)]

use bevy::prelude::*;

// Cross-system gameplay messages.

#[derive(Message)]
pub struct AddScore {
    pub points: u32,
    pub position: Vec2,
}

#[derive(Message)]
pub struct PlayerDamaged;

#[derive(Message)]
pub struct PlayerDied;

#[derive(Message)]
pub struct BlockHit {
    pub entity: Entity,
}

#[derive(Message)]
pub struct EnemyStomped {
    pub position: Vec2,
}

#[derive(Message)]
pub struct LevelCompleted;

#[derive(Message)]
pub struct SpawnParticles {
    pub position: Vec2,
    pub color: Color,
    pub count: u32,
}

#[derive(Message)]
pub struct CameraShakeRequested {
    pub intensity: f32,
}
