use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    window::WindowResolution,
};

mod board;
mod constants;
mod effects;
mod gameplay;
mod hud;
mod input;
mod resources;
mod sidebar;
mod states;
mod tetromino;

use board::BoardPlugin;
use constants::*;
use effects::EffectsPlugin;
use gameplay::GameplayPlugin;
use hud::HudPlugin;
use input::InputPlugin;
use resources::StatsPlugin;
use sidebar::SidebarPlugin;
use states::StatePlugin;
use tetromino::TetrominoPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tetris".to_string(),
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            StatePlugin,
            BoardPlugin,
            TetrominoPlugin,
            InputPlugin,
            GameplayPlugin,
            StatsPlugin,
            HudPlugin,
            SidebarPlugin,
            EffectsPlugin,
        ))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(CLEAR_COLOR),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        DebandDither::Enabled,
    ));
}
