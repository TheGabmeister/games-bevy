use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    window::WindowResolution,
};

mod components;
mod constants;
mod messages;
mod resources;
mod states;

use constants::*;
use messages::*;
use resources::GameData;
use states::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Super Mario Bros".to_string(),
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        // States
        .init_state::<AppState>()
        .add_sub_state::<PlayState>()
        // Resources
        .init_resource::<GameData>()
        // Messages
        .add_message::<AddScore>()
        .add_message::<PlayerDamaged>()
        .add_message::<PlayerDied>()
        .add_message::<BlockHit>()
        .add_message::<EnemyStomped>()
        .add_message::<LevelCompleted>()
        .add_message::<SpawnParticles>()
        .add_message::<CameraShakeRequested>()
        // Startup
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(COLOR_SKY),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        DebandDither::Enabled,
    ));
}
