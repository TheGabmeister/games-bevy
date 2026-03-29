mod collision;
mod components;
mod constants;
mod effects;
mod gameplay;
mod lanes;
mod player;
mod resources;
mod states;
mod ui;

use bevy::prelude::*;

use constants::*;
use gameplay::GameplayPlugin;
use resources::*;
use states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                resolution: bevy::window::WindowResolution::new(
                    WINDOW_WIDTH as u32,
                    WINDOW_HEIGHT as u32,
                ),
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .init_resource::<GameData>()
        .init_resource::<FrogTimer>()
        .init_resource::<LevelState>()
        .init_resource::<FrogEvent>()
        .init_resource::<PendingEffects>()
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .add_plugins((
            GameplayPlugin,
            player::PlayerPlugin,
            lanes::LanesPlugin,
            ui::UiPlugin,
            effects::EffectsPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
