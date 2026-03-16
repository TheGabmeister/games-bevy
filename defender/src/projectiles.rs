use bevy::prelude::*;

use crate::components::*;

pub fn projectile_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &Velocity,
        &mut WorldPosition,
        &mut Transform,
        &mut Lifetime,
    )>,
) {
    for (entity, vel, mut wp, mut tf, mut lifetime) in &mut query {
        wp.0 += vel.0.x * time.delta_secs();
        tf.translation.y += vel.0.y * time.delta_secs();
        lifetime.0.tick(time.delta());
        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
