use bevy::prelude::*;
use rand::Rng;

use crate::board::grid_to_world;
use crate::constants::*;
use crate::resources::{LineClearMsg, PieceLockedMsg};
use crate::states::AppState;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime: f32,
    max_lifetime: f32,
}

#[derive(Component)]
struct LockFlash(f32);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_line_clear_particles.run_if(in_state(AppState::Playing)),
                spawn_lock_flash.run_if(in_state(AppState::Playing)),
                animate_particles,
                animate_lock_flash,
            ),
        );
    }
}

// ---------------------------------------------------------------------------
// Line clear particles
// ---------------------------------------------------------------------------

fn spawn_line_clear_particles(
    mut commands: Commands,
    mut line_clears: MessageReader<LineClearMsg>,
) {
    let mut rng = rand::rng();
    for msg in line_clears.read() {
        for &row in &msg.rows {
            let y = PLAYFIELD_BOTTOM + row as f32 * CELL_SIZE + CELL_SIZE / 2.0;
            for _ in 0..PARTICLE_COUNT_PER_ROW {
                let x = PLAYFIELD_LEFT + rng.random_range(0.0..PLAYFIELD_WIDTH);
                let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
                let speed = rng.random_range(PARTICLE_SPEED * 0.5..PARTICLE_SPEED);
                let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                let lifetime = rng.random_range(PARTICLE_LIFETIME * 0.5..PARTICLE_LIFETIME);

                let color = Color::srgb(
                    rng.random_range(1.0..4.0),
                    rng.random_range(1.0..4.0),
                    rng.random_range(1.0..4.0),
                );

                commands.spawn((
                    Particle {
                        velocity,
                        lifetime,
                        max_lifetime: lifetime,
                    },
                    Sprite {
                        color,
                        custom_size: Some(Vec2::splat(PARTICLE_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(x, y, Z_PARTICLE),
                ));
            }
        }
    }
}

fn animate_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        particle.velocity.y -= 400.0 * dt;
        let alpha = particle.lifetime / particle.max_lifetime;
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

// ---------------------------------------------------------------------------
// Lock flash
// ---------------------------------------------------------------------------

fn spawn_lock_flash(mut commands: Commands, mut piece_locked: MessageReader<PieceLockedMsg>) {
    for msg in piece_locked.read() {
        for &(row, col) in &msg.cells {
            if row >= 0
                && (row as usize) < GRID_VISIBLE_ROWS
                && col >= 0
                && (col as usize) < GRID_COLS
            {
                let pos = grid_to_world(row as usize, col as usize);
                commands.spawn((
                    LockFlash(LOCK_FLASH_DURATION),
                    Sprite {
                        color: Color::srgba(3.0, 3.0, 3.0, 0.8),
                        custom_size: Some(Vec2::new(CELL_INNER_SIZE, CELL_INNER_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, Z_FLASH),
                ));
            }
        }
    }
}

fn animate_lock_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LockFlash, &mut Sprite)>,
) {
    for (entity, mut flash, mut sprite) in &mut query {
        flash.0 -= time.delta_secs();
        if flash.0 <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            let alpha = (flash.0 / LOCK_FLASH_DURATION) * 0.8;
            sprite.color = Color::srgba(3.0, 3.0, 3.0, alpha);
        }
    }
}
