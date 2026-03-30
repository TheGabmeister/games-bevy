use bevy::{camera::ScalingMode, prelude::*};

use crate::constants;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("MainCamera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: constants::LOGICAL_SCREEN_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
