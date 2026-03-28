use bevy::prelude::*;

mod asteroids;
mod collision;
mod components;
mod resources;
mod ship;
mod spawn;
mod state;
mod ui;

use components::*;
use resources::{GameAssets, GameData, ShootCooldown};
use spawn::{spawn_ship, spawn_wave};
use state::AppState;

// ── Window ────────────────────────────────────────────────────────────────────
pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;
pub const HALF_W: f32 = WINDOW_WIDTH / 2.0;
pub const HALF_H: f32 = WINDOW_HEIGHT / 2.0;

// ── Ship ──────────────────────────────────────────────────────────────────────
pub const SHIP_ROTATION_SPEED: f32 = 3.0; // radians/sec
pub const SHIP_THRUST: f32 = 250.0; // pixels/sec²
pub const SHIP_DRAG: f32 = 0.97; // velocity multiplier per frame
pub const SHIP_MAX_SPEED: f32 = 400.0; // pixels/sec
pub const SHIP_RADIUS: f32 = 12.0; // collision radius

// ── Bullet ────────────────────────────────────────────────────────────────────
pub const BULLET_SPEED: f32 = 500.0;
pub const BULLET_LIFETIME: f32 = 1.2; // seconds
pub const BULLET_RADIUS: f32 = 3.0;
pub const SHOOT_COOLDOWN: f32 = 0.25; // seconds between shots

// ── Asteroids ─────────────────────────────────────────────────────────────────
pub const ASTEROID_LARGE_RADIUS: f32 = 40.0;
pub const ASTEROID_MEDIUM_RADIUS: f32 = 22.0;
pub const ASTEROID_SMALL_RADIUS: f32 = 12.0;
pub const ASTEROID_LARGE_SPEED: f32 = 60.0;
pub const ASTEROID_MEDIUM_SPEED: f32 = 100.0;
pub const ASTEROID_SMALL_SPEED: f32 = 150.0;

// ── Gameplay ──────────────────────────────────────────────────────────────────
pub const STARTING_LIVES: u32 = 3;
pub const INITIAL_ASTEROIDS: u32 = 4; // large asteroids on wave 1 (increases by 1 per wave)
pub const SCORE_LARGE: u32 = 20;
pub const SCORE_MEDIUM: u32 = 50;
pub const SCORE_SMALL: u32 = 100;
pub const INVINCIBILITY_DURATION: f32 = 2.0; // seconds after respawn

// ── System ordering ───────────────────────────────────────────────────────────

/// Labels that enforce execution order across plugins:
///   Input → Movement → Collision → Cleanup
///
/// Each plugin tags its systems with the appropriate set.
/// main() calls configure_sets() to establish the chain.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Movement,
    Collision,
    Cleanup,
}

// ── Startup ───────────────────────────────────────────────────────────────────

/// One-time setup: camera, mesh/material assets, initial ship and asteroid wave.
/// HUD entities are spawned by UiPlugin's own startup system.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // 2D camera — no configuration needed
    commands.spawn(Camera2d);

    // Build all mesh and material handles once and store them as a resource.
    // Other systems clone these cheap handles rather than re-creating meshes.
    let assets = GameAssets {
        // Triangle pointing up: nose at +Y aligns with transform.up()
        ship_mesh: meshes.add(Triangle2d::new(
            Vec2::new(0.0, 18.0),    // nose
            Vec2::new(-12.0, -12.0), // bottom-left
            Vec2::new(12.0, -12.0),  // bottom-right
        )),
        ship_material: materials.add(Color::WHITE),

        bullet_mesh: meshes.add(Circle::new(4.0)),
        bullet_material: materials.add(Color::srgb(1.0, 1.0, 0.0)), // yellow

        // Octagons (8 sides); circumradius matches the collision radius constant
        asteroid_large_mesh: meshes.add(RegularPolygon::new(ASTEROID_LARGE_RADIUS, 8)),
        asteroid_medium_mesh: meshes.add(RegularPolygon::new(ASTEROID_MEDIUM_RADIUS, 8)),
        asteroid_small_mesh: meshes.add(RegularPolygon::new(ASTEROID_SMALL_RADIUS, 8)),
        asteroid_material: materials.add(Color::srgb(0.6, 0.6, 0.6)), // gray
    };

    spawn_ship(&mut commands, &assets);
    spawn_wave(&mut commands, &assets, 1);

    commands.insert_resource(assets);
}

// ── Generic movement systems ──────────────────────────────────────────────────
// These don't belong to any single domain (ship / asteroid / bullet all use
// them), so they live here and are registered directly in main().

/// Moves every entity that has a Velocity component.
fn movement_system(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let dt = time.delta_secs();
    for (vel, mut transform) in &mut query {
        transform.translation.x += vel.0.x * dt;
        transform.translation.y += vel.0.y * dt;
    }
}

/// Wraps any moving entity that drifts off-screen to the opposite edge.
fn screen_wrap_system(mut query: Query<&mut Transform, With<Velocity>>) {
    for mut transform in &mut query {
        let pos = &mut transform.translation;
        if pos.x > HALF_W {
            pos.x -= WINDOW_WIDTH;
        }
        if pos.x < -HALF_W {
            pos.x += WINDOW_WIDTH;
        }
        if pos.y > HALF_H {
            pos.y -= WINDOW_HEIGHT;
        }
        if pos.y < -HALF_H {
            pos.y += WINDOW_HEIGHT;
        }
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Asteroids".into(),
                resolution: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .insert_resource(GameData::default())
        .init_resource::<ShootCooldown>()
        // Establish execution order across all plugins:
        //   Input → Movement → Collision → Cleanup
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Movement,
                GameSet::Collision,
                GameSet::Cleanup,
            )
                .chain(),
        )
        .add_systems(Startup, setup)
        // Generic movement is shared across all entity types, so it's
        // registered here rather than inside any specific plugin.
        .add_systems(
            Update,
            (movement_system, screen_wrap_system)
                .chain()
                .in_set(GameSet::Movement)
                .run_if(in_state(AppState::Playing)),
        )
        .add_plugins((
            ship::ShipPlugin,
            asteroids::AsteroidPlugin,
            collision::CollisionPlugin,
            ui::UiPlugin,
        ))
        .run();
}
