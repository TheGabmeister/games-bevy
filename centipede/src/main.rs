mod bullet;
mod centipede;
mod collision;
mod components;
mod constants;
mod enemies;
mod mushroom;
mod player;
mod resources;
mod scheduling;
mod states;
mod ui;

use bevy::prelude::*;

use bullet::BulletPlugin;
use centipede::CentipedePlugin;
use collision::CollisionPlugin;
use constants::{CENTIPEDE_INTERVAL_BASE, WINDOW_HEIGHT, WINDOW_WIDTH};
use enemies::EnemiesPlugin;
use mushroom::MushroomPlugin;
use player::PlayerPlugin;
use resources::*;
use scheduling::GameplaySet;
use states::AppState;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Centipede".to_string(),
                resolution: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        // Resources
        .init_resource::<Score>()
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
        .init_resource::<NextExtraLifeScore>()
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
        .configure_sets(
            Update,
            (
                GameplaySet::Input,
                GameplaySet::Movement,
                GameplaySet::Collision,
                GameplaySet::Cleanup,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        // Camera
        .add_systems(Startup, setup_camera)
        // State lifecycle
        .add_systems(OnEnter(AppState::Playing), reset_game_state)
        // Extra life check
        .add_systems(Update, check_extra_life.in_set(GameplaySet::Cleanup))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[allow(clippy::too_many_arguments)]
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
    mut next_extra_life: ResMut<NextExtraLifeScore>,
) {
    score.value = 0;
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
    next_extra_life.0 = crate::constants::EXTRA_LIFE_SCORE;
}

fn check_extra_life(
    score: Res<Score>,
    mut lives: ResMut<Lives>,
    mut next_extra_life: ResMut<NextExtraLifeScore>,
) {
    if !score.is_changed() {
        return;
    }

    while score.value >= next_extra_life.0 {
        lives.0 += 1;
        next_extra_life.0 += crate::constants::EXTRA_LIFE_SCORE;
    }
}
