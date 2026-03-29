use bevy::prelude::*;

// Cross-system event queues using shared Resources with Vecs.
// (Bevy 0.18 EventWriter/EventReader are not available per project rules.)

#[derive(Resource, Default)]
pub struct AddScoreEvents(pub Vec<AddScore>);

pub struct AddScore {
    pub points: u32,
    pub position: Vec2,
}

#[derive(Resource, Default)]
pub struct PlayerDamagedEvents(pub Vec<PlayerDamaged>);

pub struct PlayerDamaged;

#[derive(Resource, Default)]
pub struct PlayerDiedEvents(pub Vec<PlayerDied>);

pub struct PlayerDied;

#[derive(Resource, Default)]
pub struct BlockHitEvents(pub Vec<BlockHit>);

pub struct BlockHit {
    pub entity: Entity,
}

#[derive(Resource, Default)]
pub struct EnemyStompedEvents(pub Vec<EnemyStomped>);

pub struct EnemyStomped {
    pub position: Vec2,
}

#[derive(Resource, Default)]
pub struct LevelCompletedEvents(pub Vec<LevelCompleted>);

pub struct LevelCompleted;

#[derive(Resource, Default)]
pub struct SpawnParticlesEvents(pub Vec<SpawnParticles>);

pub struct SpawnParticles {
    pub position: Vec2,
    pub color: Color,
    pub count: u32,
}

#[derive(Resource, Default)]
pub struct CameraShakeEvents(pub Vec<CameraShakeRequested>);

pub struct CameraShakeRequested {
    pub intensity: f32,
}
