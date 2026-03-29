use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::resources::*;
use crate::states::AppState;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_death_particles.run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, update_particles.run_if(in_state(AppState::Playing)));
    }
}

fn spawn_death_particles(
    mut commands: Commands,
    mut kill_reader: MessageReader<JoustKillMessage>,
    meshes: Res<SharedMeshes>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    for msg in kill_reader.read() {
        let color = if msg.loser_tier.is_some() {
            Color::srgb(1.0, 0.5, 0.2)
        } else {
            Color::srgb(0.5, 0.7, 1.0)
        };
        let mat = materials.add(color);

        for _ in 0..8 {
            let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
            let speed: f32 = rng.random_range(80.0..200.0);
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;

            commands.spawn((
                Mesh2d(meshes.circle_particle.clone()),
                MeshMaterial2d(mat.clone()),
                Transform::from_xyz(
                    msg.loser_position.x,
                    msg.loser_position.y,
                    crate::constants::Z_PARTICLES,
                ),
                Velocity(Vec2::new(vx, vy)),
                Particle {
                    lifetime: Timer::from_seconds(0.6, TimerMode::Once),
                },
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Particle, &Velocity, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, vel, mut transform) in &mut query {
        particle.lifetime.tick(time.delta());
        transform.translation.x += vel.0.x * dt;
        transform.translation.y += vel.0.y * dt;

        let frac = particle.lifetime.fraction();
        transform.scale = Vec3::splat(1.0 - frac);

        if particle.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
