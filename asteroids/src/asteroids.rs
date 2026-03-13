use bevy::prelude::*;

use crate::GameSet;
use crate::components::*;
use crate::resources::{GameAssets, GameData};
use crate::spawn::{asteroid_spin, spawn_wave};
use crate::state::AppState;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            asteroid_rotation_system
                .in_set(GameSet::Movement)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            wave_clear_system
                .in_set(GameSet::Cleanup)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Slowly spins asteroids for visual interest (cosmetic only).
fn asteroid_rotation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Asteroid)>,
) {
    let dt = time.delta_secs();
    for (mut transform, asteroid) in &mut query {
        transform.rotate_z(asteroid_spin(asteroid.size) * dt);
    }
}

/// When all asteroids are cleared, spawn the next wave with one extra asteroid.
fn wave_clear_system(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    assets: Res<GameAssets>,
    asteroids: Query<(), With<Asteroid>>,
) {
    if asteroids.iter().count() == 0 {
        game_data.wave += 1;
        spawn_wave(&mut commands, &assets, game_data.wave);
    }
}
