use bevy::prelude::*;

mod assets;
mod block;
mod camera;
mod collision;
mod components;
mod constants;
mod death;
mod decoration;
mod enemy;
mod input;
mod level;
mod level_complete;
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
        .init_resource::<GameTimer>()
        .init_resource::<SpawnPoint>()
        .init_resource::<LevelList>()
        .add_message::<ScoreEvent>()
        .add_message::<CoinEvent>()
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
            input::InputPlugin,
            player::PlayerPlugin,
            camera::CameraPlugin,
            ui::UiPlugin,
            enemy::EnemyPlugin,
            block::BlockPlugin,
            powerup::PowerUpPlugin,
            death::DeathPlugin,
            level_complete::LevelCompletePlugin,
            decoration::DecorationPlugin,
        ))
        .init_asset::<level::LevelData>()
        .register_asset_loader(level::LevelAssetLoader)
        .init_resource::<level::SpawnerRegistry>()
        .add_systems(Startup, (assets::init_game_assets, level::load_level, level::init_spawner_registry))
        .add_systems(OnEnter(AppState::Playing), level::spawn_level)
        .run();
}
