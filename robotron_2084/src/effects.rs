use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::{GameAssets, ScreenShake};
use crate::states::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (tick_particles, tick_score_popups, apply_screen_shake)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

pub fn spawn_particles(
    commands: &mut Commands,
    assets: &GameAssets,
    pos: Vec2,
    count: u32,
    material: &Handle<ColorMaterial>,
) {
    let mut rng = rand::rng();
    for _ in 0..count {
        let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(PARTICLE_SPEED * 0.3..PARTICLE_SPEED);
        commands.spawn((
            Particle,
            WaveEntity,
            Mesh2d(assets.particle_mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(pos.x, pos.y, 2.0),
            Velocity(Vec2::new(angle.cos(), angle.sin()) * speed),
            Lifetime(Timer::from_seconds(PARTICLE_LIFETIME, TimerMode::Once)),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

pub fn spawn_score_popup(commands: &mut Commands, pos: Vec2, points: u32) {
    commands.spawn((
        ScorePopup,
        Text2d::new(format!("+{}", points)),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(5.0, 5.0, 0.5)),
        Transform::from_xyz(pos.x, pos.y + 10.0, 5.0),
        Lifetime(Timer::from_seconds(SCORE_POPUP_LIFETIME, TimerMode::Once)),
        DespawnOnExit(AppState::Playing),
    ));
}

fn tick_particles(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform, &Lifetime), With<Particle>>,
) {
    let dt = time.delta_secs();
    for (mut vel, mut tf, lifetime) in &mut query {
        vel.0 *= (-PARTICLE_DRAG * dt).exp();
        let frac = lifetime.0.fraction_remaining();
        tf.scale = Vec3::splat(frac);
    }
}

fn tick_score_popups(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Lifetime), With<ScorePopup>>,
) {
    let dt = time.delta_secs();
    for (mut tf, lifetime) in &mut query {
        tf.translation.y += SCORE_POPUP_RISE_SPEED * dt;
        let alpha = lifetime.0.fraction_remaining();
        tf.scale = Vec3::splat(alpha);
    }
}

fn apply_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
) {
    let dt = time.delta_secs();
    shake.trauma = (shake.trauma - SCREEN_SHAKE_DECAY * dt).max(0.0);
    let Ok(mut cam_tf) = camera_q.single_mut() else {
        return;
    };
    if shake.trauma > 0.0 {
        let mut rng = rand::rng();
        let intensity = shake.trauma * shake.trauma;
        cam_tf.translation.x = rng.random_range(-1.0f32..1.0) * SCREEN_SHAKE_MAX_OFFSET * intensity;
        cam_tf.translation.y = rng.random_range(-1.0f32..1.0) * SCREEN_SHAKE_MAX_OFFSET * intensity;
    } else {
        cam_tf.translation.x = 0.0;
        cam_tf.translation.y = 0.0;
    }
}
