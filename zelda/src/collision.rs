use bevy::prelude::*;

use crate::{
    components::{Player, SolidBody, StaticBlocker, Velocity},
    player::PlayerSet,
    states::AppState,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            apply_player_velocity_with_static_collision
                .after(PlayerSet::Input)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

fn apply_player_velocity_with_static_collision(
    time: Res<Time>,
    mut movers: Query<(&mut Transform, &Velocity, &SolidBody), With<Player>>,
    blockers: Query<(&Transform, &SolidBody), (With<StaticBlocker>, Without<Player>)>,
) {
    let Ok((mut transform, velocity, body)) = movers.single_mut() else {
        return;
    };

    let mut position = transform.translation.truncate();
    let delta = velocity.0 * time.delta_secs();

    position.x += delta.x;
    for (blocker_transform, blocker_body) in &blockers {
        let blocker_position = blocker_transform.translation.truncate();
        if aabb_overlap(position, body.half_size, blocker_position, blocker_body.half_size) {
            if delta.x > 0.0 {
                position.x = blocker_position.x - blocker_body.half_size.x - body.half_size.x;
            } else if delta.x < 0.0 {
                position.x = blocker_position.x + blocker_body.half_size.x + body.half_size.x;
            }
        }
    }

    position.y += delta.y;
    for (blocker_transform, blocker_body) in &blockers {
        let blocker_position = blocker_transform.translation.truncate();
        if aabb_overlap(position, body.half_size, blocker_position, blocker_body.half_size) {
            if delta.y > 0.0 {
                position.y = blocker_position.y - blocker_body.half_size.y - body.half_size.y;
            } else if delta.y < 0.0 {
                position.y = blocker_position.y + blocker_body.half_size.y + body.half_size.y;
            }
        }
    }

    transform.translation.x = position.x;
    transform.translation.y = position.y;
}

fn aabb_overlap(a_pos: Vec2, a_half: Vec2, b_pos: Vec2, b_half: Vec2) -> bool {
    (a_pos.x - b_pos.x).abs() < (a_half.x + b_half.x)
        && (a_pos.y - b_pos.y).abs() < (a_half.y + b_half.y)
}
