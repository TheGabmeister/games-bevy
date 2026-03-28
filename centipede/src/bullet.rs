use bevy::prelude::*;

use crate::{
    components::Bullet,
    constants::{BULLET_SPEED, WINDOW_HEIGHT},
    scheduling::GameplaySet,
};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                bullet_movement.in_set(GameplaySet::Movement),
                despawn_offscreen_bullets.in_set(GameplaySet::Cleanup),
            ),
        );
    }
}

fn bullet_movement(mut query: Query<&mut Transform, With<Bullet>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation.y += BULLET_SPEED * time.delta_secs();
    }
}

fn despawn_offscreen_bullets(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Bullet>>,
) {
    let top = WINDOW_HEIGHT / 2.0 + 10.0;
    for (entity, transform) in &query {
        if transform.translation.y > top {
            commands.entity(entity).despawn();
        }
    }
}
