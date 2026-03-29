use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::GameAssets;
use crate::states::*;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_arena)
            .add_systems(
                Update,
                (
                    apply_velocity.in_set(GameSet::Movement),
                    confine_entities.in_set(GameSet::Confinement),
                    despawn_oob_bullets.in_set(GameSet::Combat),
                ),
            )
            .add_systems(Update, tick_lifetimes.run_if(in_state(AppState::Playing)));
    }
}

fn spawn_arena(mut commands: Commands, assets: Res<GameAssets>) {
    // Top border
    commands.spawn((
        Mesh2d(assets.border_mesh_h.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(0.0, ARENA_HALF_HEIGHT, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        DespawnOnExit(AppState::Playing),
    ));
    // Bottom border
    commands.spawn((
        Mesh2d(assets.border_mesh_h.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(0.0, -ARENA_HALF_HEIGHT, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        DespawnOnExit(AppState::Playing),
    ));
    // Left border
    commands.spawn((
        Mesh2d(assets.border_mesh_v.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(-ARENA_HALF_WIDTH, 0.0, 0.0),
        DespawnOnExit(AppState::Playing),
    ));
    // Right border
    commands.spawn((
        Mesh2d(assets.border_mesh_v.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(ARENA_HALF_WIDTH, 0.0, 0.0),
        DespawnOnExit(AppState::Playing),
    ));
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let dt = time.delta_secs();
    for (vel, mut tf) in &mut query {
        tf.translation.x += vel.0.x * dt;
        tf.translation.y += vel.0.y * dt;
    }
}

fn confine_entities(mut query: Query<(&mut Transform, &CollisionRadius), With<Confined>>) {
    for (mut tf, radius) in &mut query {
        tf.translation.x = tf
            .translation
            .x
            .clamp(-ARENA_HALF_WIDTH + radius.0, ARENA_HALF_WIDTH - radius.0);
        tf.translation.y = tf
            .translation
            .y
            .clamp(-ARENA_HALF_HEIGHT + radius.0, ARENA_HALF_HEIGHT - radius.0);
    }
}

fn despawn_oob_bullets(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<PlayerBullet>>,
) {
    for (entity, tf) in &query {
        let pos = tf.translation;
        if pos.x < -ARENA_HALF_WIDTH - 50.0
            || pos.x > ARENA_HALF_WIDTH + 50.0
            || pos.y < -ARENA_HALF_HEIGHT - 50.0
            || pos.y > ARENA_HALF_HEIGHT + 50.0
        {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lt) in &mut query {
        lt.0.tick(time.delta());
        if lt.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
