use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::states::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                save_previous_positions,
                gravity_system,
                drag_system,
                apply_velocity_system,
                platform_collision_system,
                screen_wrap_system,
            )
                .chain()
                .in_set(GameSet::Physics)
                .run_if(in_state(PlayState::WaveActive)),
        );
    }
}

// --- Wrap helpers ---

pub fn wrap_x(x: f32) -> f32 {
    let margin = WRAP_MARGIN;
    let left = ARENA_LEFT - margin;
    let right = ARENA_RIGHT + margin;
    let width = right - left;
    if x < left {
        x + width
    } else if x > right {
        x - width
    } else {
        x
    }
}

pub fn wrapped_dx(from: f32, to: f32) -> f32 {
    let dx = to - from;
    if dx > ARENA_WIDTH / 2.0 {
        dx - ARENA_WIDTH
    } else if dx < -ARENA_WIDTH / 2.0 {
        dx + ARENA_WIDTH
    } else {
        dx
    }
}

pub fn wrapped_distance(a: f32, b: f32) -> f32 {
    wrapped_dx(a, b).abs()
}

// --- Systems ---

fn save_previous_positions(mut query: Query<(&Transform, &mut PreviousPosition)>) {
    for (transform, mut prev) in &mut query {
        prev.0 = transform.translation.truncate();
    }
}

fn gravity_system(time: Res<Time>, mut query: Query<&mut Velocity, Without<Grounded>>) {
    let dt = time.delta_secs();
    for mut vel in &mut query {
        vel.0.y -= GRAVITY * dt;
        vel.0.y = vel.0.y.max(-MAX_FALL_SPEED);
    }
}

fn drag_system(time: Res<Time>, mut query: Query<(&mut Velocity, Option<&Grounded>)>) {
    let dt = time.delta_secs();
    for (mut vel, grounded) in &mut query {
        let drag = if grounded.is_some() { GROUND_DRAG } else { AIR_DRAG };
        if vel.0.x.abs() > 0.01 {
            let sign = vel.0.x.signum();
            vel.0.x -= sign * drag * dt;
            if vel.0.x.signum() != sign {
                vel.0.x = 0.0;
            }
        }
    }
}

fn apply_velocity_system(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let dt = time.delta_secs();
    for (vel, mut transform) in &mut query {
        transform.translation.x += vel.0.x * dt;
        transform.translation.y += vel.0.y * dt;
    }
}

fn platform_collision_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut Velocity,
        &PreviousPosition,
        Option<&Grounded>,
    )>,
) {
    for (entity, mut transform, mut velocity, prev_pos, grounded) in &mut query {
        let prev_bottom = prev_pos.0.y - RIDER_RADIUS;
        let curr_bottom = transform.translation.y - RIDER_RADIUS;
        let curr_x = transform.translation.x;
        let mut landed_this_frame = false;

        // Check for landing on platforms
        if velocity.0.y <= 0.0 {
            for plat in &PLATFORMS {
                let plat_top = plat.y + PLATFORM_THICKNESS / 2.0;

                let on_x = if plat.wraps {
                    true
                } else {
                    let half = plat.width / 2.0;
                    curr_x >= plat.center_x - half && curr_x <= plat.center_x + half
                };

                if on_x
                    && prev_bottom >= plat_top - PLATFORM_SNAP_DISTANCE
                    && curr_bottom <= plat_top + PLATFORM_SNAP_DISTANCE
                {
                    transform.translation.y = plat_top + RIDER_RADIUS;
                    velocity.0.y = 0.0;
                    if grounded.is_none() {
                        commands.entity(entity).insert(Grounded);
                    }
                    landed_this_frame = true;
                    break;
                }
            }
        }

        // If grounded but walked off platform edge, remove Grounded
        if grounded.is_some() && !landed_this_frame {
            let still_on = PLATFORMS.iter().any(|plat| {
                let plat_top = plat.y + PLATFORM_THICKNESS / 2.0;
                let on_y = (transform.translation.y - RIDER_RADIUS - plat_top).abs()
                    < PLATFORM_SNAP_DISTANCE * 2.0;
                let on_x = if plat.wraps {
                    true
                } else {
                    let half = plat.width / 2.0;
                    curr_x >= plat.center_x - half && curr_x <= plat.center_x + half
                };
                on_x && on_y
            });
            if !still_on {
                commands.entity(entity).remove::<Grounded>();
            }
        }

        // Ceiling clamp
        if transform.translation.y + RIDER_RADIUS > ARENA_TOP {
            transform.translation.y = ARENA_TOP - RIDER_RADIUS;
            velocity.0.y = velocity.0.y.min(0.0);
        }
    }
}

fn screen_wrap_system(mut query: Query<&mut Transform, With<Velocity>>) {
    for mut transform in &mut query {
        transform.translation.x = wrap_x(transform.translation.x);
    }
}
