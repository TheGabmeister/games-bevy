mod bullet;
mod centipede;
mod collision;
mod components;
mod constants;
mod enemies;
mod mushroom;
mod player;
mod resources;
mod states;
mod ui;

use bevy::prelude::*;

use bullet::BulletPlugin;
use centipede::CentipedePlugin;
use collision::CollisionPlugin;
use components::GameplayEntity;
use constants::{CENTIPEDE_INTERVAL_BASE, WINDOW_HEIGHT, WINDOW_WIDTH};
use enemies::EnemiesPlugin;
use mushroom::MushroomPlugin;
use player::PlayerPlugin;
use resources::*;
use states::AppState;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Centipede".to_string(),
                    resolution: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
        )
        .init_state::<AppState>()
        // Resources
        .insert_resource(Score {
            value: 0,
            extra_life_threshold: crate::constants::EXTRA_LIFE_SCORE,
        })
        .init_resource::<Lives>()
        .init_resource::<Wave>()
        .init_resource::<NextChainId>()
        .init_resource::<CentipedeChains>()
        .init_resource::<MushroomGrid>()
        .insert_resource(CentipedeTimer::new(CENTIPEDE_INTERVAL_BASE))
        .init_resource::<FleaSpawnTimer>()
        .init_resource::<SpiderSpawnTimer>()
        .init_resource::<ScorpionSpawnTimer>()
        .init_resource::<RespawnTimer>()
        .init_resource::<ExtraLifeGranted>()
        // Plugins
        .add_plugins((
            UiPlugin,
            MushroomPlugin,
            PlayerPlugin,
            BulletPlugin,
            CentipedePlugin,
            EnemiesPlugin,
            CollisionPlugin,
        ))
        // Camera
        .add_systems(Startup, setup_camera)
        // State lifecycle
        .add_systems(OnEnter(AppState::Playing), reset_game_state)
        .add_systems(OnExit(AppState::Playing), despawn_gameplay_entities)
        // Extra life check
        .add_systems(Update, check_extra_life.run_if(in_state(AppState::Playing)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn reset_game_state(
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut wave: ResMut<Wave>,
    mut next_id: ResMut<NextChainId>,
    mut chains: ResMut<CentipedeChains>,
    mut mushroom_grid: ResMut<MushroomGrid>,
    mut timer: ResMut<CentipedeTimer>,
    mut flea_timer: ResMut<FleaSpawnTimer>,
    mut spider_timer: ResMut<SpiderSpawnTimer>,
    mut scorpion_timer: ResMut<ScorpionSpawnTimer>,
    mut respawn: ResMut<RespawnTimer>,
    mut extra_life: ResMut<ExtraLifeGranted>,
) {
    score.value = 0;
    score.extra_life_threshold = crate::constants::EXTRA_LIFE_SCORE;
    *lives = Lives(crate::constants::INITIAL_LIVES);
    wave.0 = 0;
    next_id.0 = 0;
    chains.0.clear();
    mushroom_grid.0.clear();
    *timer = CentipedeTimer::new(CENTIPEDE_INTERVAL_BASE);
    *flea_timer = FleaSpawnTimer::default();
    *spider_timer = SpiderSpawnTimer::default();
    *scorpion_timer = ScorpionSpawnTimer::default();
    respawn.0 = None;
    extra_life.0 = false;
}

fn despawn_gameplay_entities(
    mut commands: Commands,
    query: Query<Entity, With<GameplayEntity>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn check_extra_life(
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut granted: ResMut<ExtraLifeGranted>,
) {
    if !granted.0 && score.value >= score.extra_life_threshold {
        lives.0 += 1;
        granted.0 = true;
        score.extra_life_threshold += crate::constants::EXTRA_LIFE_SCORE;
    }
}
