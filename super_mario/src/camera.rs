use bevy::prelude::*;

use crate::components::Player;
use crate::constants::*;
use crate::states::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(OnEnter(AppState::Playing), reset_camera)
            .add_systems(
                Update,
                camera_follow.in_set(GameplaySet::Camera),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = CAMERA_SCALE;
    commands.spawn((
        Camera2d,
        Projection::from(projection),
        Transform::from_xyz(CAMERA_MIN_X, CAMERA_FIXED_Y, 0.0),
    ));
}

fn reset_camera(mut camera_query: Query<&mut Transform, With<Camera2d>>) {
    if let Ok(mut camera_tf) = camera_query.single_mut() {
        camera_tf.translation.x = CAMERA_MIN_X;
        camera_tf.translation.y = CAMERA_FIXED_Y;
    }
}

fn camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    let Ok(mut camera_tf) = camera_query.single_mut() else { return };

    let desired_x = player_tf.translation.x - CAMERA_DEAD_ZONE_OFFSET;
    let target_x = desired_x.max(camera_tf.translation.x);
    let target_x = target_x.clamp(CAMERA_MIN_X, CAMERA_MAX_X);

    let t = 1.0 - (-CAMERA_LERP_SPEED * time.delta_secs()).exp();
    camera_tf.translation.x += (target_x - camera_tf.translation.x) * t;
    camera_tf.translation.y = CAMERA_FIXED_Y;
}
