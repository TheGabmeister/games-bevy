use bevy::prelude::*;

use crate::components::{ExplosionParticle, Star};
use crate::constants::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_starfield)
            .add_systems(Update, (scroll_stars, update_explosion_particles));
    }
}

/// Deterministic pseudo-random in [0, 1) from an index.
fn pseudo_random(seed: usize) -> f32 {
    let x = (seed as u32).wrapping_mul(2654435761);
    (x & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
}

fn spawn_starfield(mut commands: Commands) {
    let half_w = WINDOW_WIDTH / 2.0;
    let half_h = WINDOW_HEIGHT / 2.0;

    for i in 0..STAR_COUNT {
        let t = i as f32 / STAR_COUNT as f32;
        let speed = STAR_SPEED_MIN + t * (STAR_SPEED_MAX - STAR_SPEED_MIN);
        let size = STAR_SIZE_MIN + t * (STAR_SIZE_MAX - STAR_SIZE_MIN);
        let brightness = STAR_BRIGHTNESS_MIN + t * (1.0 - STAR_BRIGHTNESS_MIN);

        let x = pseudo_random(i * 3) * WINDOW_WIDTH - half_w;
        let y = pseudo_random(i * 3 + 1) * WINDOW_HEIGHT - half_h;

        commands.spawn((
            Star { speed },
            Sprite {
                color: Color::srgba(brightness, brightness, brightness * 1.1, brightness),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, -10.0)),
        ));
    }
}

fn scroll_stars(time: Res<Time>, mut query: Query<(&Star, &mut Transform)>) {
    let half_h = WINDOW_HEIGHT / 2.0;
    let dt = time.delta_secs();

    for (star, mut transform) in &mut query {
        transform.translation.y -= star.speed * dt;

        if transform.translation.y < -half_h - 5.0 {
            transform.translation.y = half_h + 5.0;
        }
    }
}

pub fn spawn_explosion(commands: &mut Commands, position: Vec3, color: Color) {
    let count = EXPLOSION_PARTICLE_COUNT;
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let speed_variation = 0.5 + pseudo_random(i * 7 + 13) * 0.8;
        let speed = EXPLOSION_PARTICLE_SPEED * speed_variation;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        commands.spawn((
            ExplosionParticle {
                velocity,
                lifetime: Timer::from_seconds(EXPLOSION_PARTICLE_LIFETIME, TimerMode::Once),
            },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(EXPLOSION_PARTICLE_SIZE)),
                ..default()
            },
            Transform::from_translation(Vec3::new(position.x, position.y, 5.0)),
        ));
    }
}

fn update_explosion_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionParticle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.lifetime.tick(time.delta());

        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Fade out
        let remaining = 1.0 - particle.lifetime.elapsed_secs() / EXPLOSION_PARTICLE_LIFETIME;
        sprite.color = sprite.color.with_alpha(remaining.max(0.0));

        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
