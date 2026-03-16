use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn camera_follow(
    player_q: Query<&WorldPosition, With<Player>>,
    mut cam_pos: ResMut<CameraWorldPos>,
) {
    if let Ok(wp) = player_q.single() {
        // Smooth follow
        let target = wp.0;
        let mut dx = target - cam_pos.0;
        if dx > WORLD_WIDTH / 2.0 {
            dx -= WORLD_WIDTH;
        }
        if dx < -WORLD_WIDTH / 2.0 {
            dx += WORLD_WIDTH;
        }
        cam_pos.0 = wrap_x(cam_pos.0 + dx * 0.1);
    }
}

pub fn world_wrap_positions(mut query: Query<&mut WorldPosition>) {
    for mut wp in &mut query {
        wp.0 = wrap_x(wp.0);
    }
}

pub fn sync_transforms(
    cam_pos: Res<CameraWorldPos>,
    mut query: Query<(&WorldPosition, &mut Transform), Without<Camera2d>>,
) {
    let cam_x = cam_pos.0;
    for (wp, mut tf) in &mut query {
        let mut dx = wp.0 - cam_x;
        if dx > WORLD_WIDTH / 2.0 {
            dx -= WORLD_WIDTH;
        }
        if dx < -WORLD_WIDTH / 2.0 {
            dx += WORLD_WIDTH;
        }
        tf.translation.x = dx;
    }
}

pub fn wrap_x(x: f32) -> f32 {
    ((x % WORLD_WIDTH) + WORLD_WIDTH) % WORLD_WIDTH
}

pub fn world_dx(a: f32, b: f32) -> f32 {
    let mut dx = a - b;
    if dx > WORLD_WIDTH / 2.0 {
        dx -= WORLD_WIDTH;
    }
    if dx < -WORLD_WIDTH / 2.0 {
        dx += WORLD_WIDTH;
    }
    dx
}

pub fn world_distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = world_dx(ax, bx);
    let dy = ay - by;
    (dx * dx + dy * dy).sqrt()
}
