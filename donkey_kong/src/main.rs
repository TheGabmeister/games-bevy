mod audio;
mod combat;
mod components;
mod constants;
mod enemy;
mod player;
mod resources;
mod states;
mod ui;

use bevy::prelude::*;

use constants::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use resources::GameData;
use states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                resolution: bevy::window::WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            player::PlayerPlugin,
            ui::UiPlugin,
            audio::GameAudioPlugin,
            enemy::EnemyPlugin,
            combat::CombatPlugin,
        ))
        .init_state::<AppState>()
        .init_resource::<GameData>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
