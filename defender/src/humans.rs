use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::scheduling::GameplaySet;
use crate::terrain::get_terrain_y_at;

pub struct HumanPlugin;

impl Plugin for HumanPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, human_walk.in_set(GameplaySet::Input))
            .add_systems(
                Update,
                (human_falling, human_grabbed_follow, human_caught_follow)
                    .in_set(GameplaySet::Movement),
            );
    }
}

pub fn human_walk(
    time: Res<Time>,
    terrain: Res<TerrainData>,
    mut query: Query<
        (&mut WorldPosition, &mut Transform, &mut Velocity),
        (
            With<Human>,
            Without<GrabbedBy>,
            Without<HumanFalling>,
            Without<CaughtByPlayer>,
        ),
    >,
) {
    for (mut wp, mut tf, mut vel) in &mut query {
        wp.0 += vel.0.x * time.delta_secs();

        // Random direction change
        let t = time.elapsed_secs();
        if ((t * 100.0 + wp.0) as u32).is_multiple_of(200) {
            vel.0.x = -vel.0.x;
        }

        // Keep on terrain
        let terrain_y = get_terrain_y_at(&terrain, wp.0);
        tf.translation.y = terrain_y + 5.0;
    }
}

pub fn human_grabbed_follow(
    lander_q: Query<(&WorldPosition, &Transform), (With<Lander>, Without<Human>)>,
    mut humans: Query<
        (&GrabbedBy, &mut WorldPosition, &mut Transform),
        (With<Human>, Without<Lander>),
    >,
) {
    for (grabbed, mut wp, mut tf) in &mut humans {
        if let Ok((l_wp, l_tf)) = lander_q.get(grabbed.0) {
            wp.0 = l_wp.0;
            tf.translation.y = l_tf.translation.y - 12.0;
        }
    }
}

pub fn human_falling(
    time: Res<Time>,
    mut commands: Commands,
    terrain: Res<TerrainData>,
    mut query: Query<
        (Entity, &WorldPosition, &mut Transform),
        (With<Human>, With<HumanFalling>, Without<CaughtByPlayer>),
    >,
) {
    for (entity, wp, mut tf) in &mut query {
        tf.translation.y -= HUMAN_FALL_GRAVITY * time.delta_secs();

        let terrain_y = get_terrain_y_at(&terrain, wp.0);

        if tf.translation.y <= terrain_y + 5.0 {
            // Landed safely
            tf.translation.y = terrain_y + 5.0;
            commands.entity(entity).remove::<HumanFalling>();
        }

        // Fell off screen bottom
        if tf.translation.y < TERRAIN_BOTTOM_Y - 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn human_caught_follow(
    player_q: Query<(&WorldPosition, &Transform), With<Player>>,
    mut humans: Query<
        (Entity, &mut WorldPosition, &mut Transform),
        (With<Human>, With<CaughtByPlayer>, Without<Player>),
    >,
    terrain: Res<TerrainData>,
    mut commands: Commands,
) {
    let Ok((p_wp, p_tf)) = player_q.single() else {
        return;
    };

    for (entity, mut wp, mut tf) in &mut humans {
        wp.0 = p_wp.0;
        tf.translation.y = p_tf.translation.y - 15.0;

        // If player is near ground, deposit human
        let terrain_y = get_terrain_y_at(&terrain, wp.0);
        if p_tf.translation.y < terrain_y + 40.0 {
            tf.translation.y = terrain_y + 5.0;
            commands.entity(entity).remove::<CaughtByPlayer>();
            commands.entity(entity).remove::<HumanFalling>();
        }
    }
}
