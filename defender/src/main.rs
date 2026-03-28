mod camera;
mod collision;
mod components;
mod constants;
mod enemies;
mod humans;
mod player;
mod projectiles;
mod resources;
mod scanner;
mod scheduling;
mod spawning;
mod states;
mod terrain;
mod ui;
mod waves;

use bevy::prelude::*;

use camera::CameraPlugin;
use collision::CollisionPlugin;
use constants::*;
use enemies::EnemyPlugin;
use humans::HumanPlugin;
use player::PlayerPlugin;
use projectiles::ProjectilePlugin;
use resources::*;
use scanner::ScannerPlugin;
use scheduling::GameplaySet;
use states::AppState;
use terrain::TerrainPlugin;
use ui::UiPlugin;
use waves::WavePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Defender".to_string(),
                resolution: UVec2::new(SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .init_resource::<GameState>()
        .init_resource::<WaveState>()
        .init_resource::<CameraWorldPos>()
        .init_resource::<GameRng>()
        .init_resource::<GameplayAssets>()
        .init_resource::<player::SmartBombTriggered>()
        .init_resource::<player::HyperspaceTriggered>()
        .configure_sets(
            Update,
            (
                GameplaySet::Input,
                GameplaySet::Movement,
                GameplaySet::Collision,
                GameplaySet::Camera,
                GameplaySet::Sync,
                GameplaySet::Post,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .add_plugins((
            TerrainPlugin,
            CameraPlugin,
            UiPlugin,
            ScannerPlugin,
            PlayerPlugin,
            EnemyPlugin,
            HumanPlugin,
            ProjectilePlugin,
            WavePlugin,
            CollisionPlugin,
        ))
        .run();
}
