mod combat;
mod components;
mod constants;
mod effects;
mod enemy;
mod physics;
mod player;
mod rendering;
mod resources;
mod states;
mod ui;
mod waves;

use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
};

use constants::*;
use resources::*;
use states::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Joust".to_string(),
                resolution: bevy::window::WindowResolution::new(
                    WINDOW_WIDTH as u32,
                    WINDOW_HEIGHT as u32,
                ),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_sub_state::<PlayState>()
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Ai,
                GameSet::Physics,
                GameSet::Combat,
                GameSet::Progression,
            )
                .chain(),
        )
        .init_resource::<GameState>()
        .init_resource::<RespawnTimers>()
        .init_resource::<PlayerCount>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(AppState::Playing), spawn_arena)
        .add_plugins((
            rendering::RenderingPlugin,
            physics::PhysicsPlugin,
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            combat::CombatPlugin,
            waves::WavesPlugin,
            ui::UiPlugin,
            effects::EffectsPlugin,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.04, 0.04, 0.07)),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        DebandDither::Enabled,
    ));
}

fn spawn_arena(
    mut commands: Commands,
    meshes: Res<SharedMeshes>,
    materials: Res<SharedMaterials>,
) {
    rendering::spawn_arena(&mut commands, &meshes, &materials);
}
