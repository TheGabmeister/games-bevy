use bevy::prelude::*;

mod block;
mod camera;
mod components;
mod constants;
mod enemy;
mod level;
mod player;
mod powerup;
mod resources;
mod states;
mod ui;

use constants::*;
use resources::*;
use states::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(
                    WINDOW_WIDTH as u32,
                    WINDOW_HEIGHT as u32,
                ),
                title: "Super Mario Bros".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.36, 0.53, 0.95)))
        .init_state::<AppState>()
        .add_sub_state::<PlayState>()
        .init_resource::<GameData>()
        .init_resource::<SpawnPoint>()
        .configure_sets(
            Update,
            (
                GameplaySet::Input,
                GameplaySet::Physics,
                GameplaySet::Camera,
                GameplaySet::Late,
            )
                .chain()
                .run_if(in_state(PlayState::Running)),
        )
        .add_plugins((
            player::PlayerPlugin,
            camera::CameraPlugin,
            ui::UiPlugin,
            enemy::EnemyPlugin,
            block::BlockPlugin,
            powerup::PowerUpPlugin,
        ))
        .add_systems(OnEnter(AppState::Playing), level::spawn_level)
        .run();
}
