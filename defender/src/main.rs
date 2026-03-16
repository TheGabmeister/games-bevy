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
mod spawning;
mod states;
mod terrain;
mod ui;
mod waves;

use bevy::prelude::*;

use constants::*;
use resources::*;
use states::AppState;

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
        .insert_resource(GameState::default())
        .insert_resource(WaveState::default())
        .insert_resource(CameraWorldPos::default())
        .insert_resource(player::SmartBombTriggered::default())
        .insert_resource(player::HyperspaceTriggered::default())
        // Startup systems
        .add_systems(Startup, (
            setup_camera,
            terrain::setup_terrain,
            ui::setup_ui,
            scanner::setup_scanner,
        ))
        // Start screen
        .add_systems(OnEnter(AppState::StartScreen), (
            ui::reset_game,
            ui::start_screen_setup,
        ).chain())
        .add_systems(OnExit(AppState::StartScreen), ui::start_screen_cleanup)
        .add_systems(Update, ui::start_screen_input.run_if(in_state(AppState::StartScreen)))
        // Wave intro
        .add_systems(OnEnter(AppState::WaveIntro), (
            ui::respawn_player,
            waves::wave_intro_setup,
            waves::spawn_initial_humans_if_needed,
        ))
        .add_systems(OnExit(AppState::WaveIntro), waves::wave_intro_cleanup)
        .add_systems(Update, waves::wave_intro_timer.run_if(in_state(AppState::WaveIntro)))
        // Playing
        .add_systems(Update, (
            // Input & AI phase
            player::player_input,
            enemies::lander_ai,
            enemies::mutant_ai,
            enemies::bomber_ai,
            enemies::swarmer_ai,
            enemies::baiter_ai,
            humans::human_walk,
            waves::spawn_wave_enemies,
        ).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (
            // Movement phase
            player::player_movement,
            enemies::enemy_movement,
            projectiles::projectile_movement,
            humans::human_falling,
            humans::human_grabbed_follow,
            humans::human_caught_follow,
        )
            .after(player::player_input)
            .run_if(in_state(AppState::Playing)))
        .add_systems(Update,
            // Collision phase
            collision::collision_detection
                .after(player::player_movement)
                .after(enemies::enemy_movement)
                .after(projectiles::projectile_movement)
                .run_if(in_state(AppState::Playing)))
        .add_systems(Update, (
            // Camera & wrapping phase
            camera::camera_follow,
            camera::world_wrap_positions,
        ).chain()
            .after(collision::collision_detection)
            .run_if(in_state(AppState::Playing)))
        .add_systems(Update,
            // Sync transforms
            camera::sync_transforms
                .after(camera::world_wrap_positions)
                .run_if(in_state(AppState::Playing)))
        .add_systems(Update, (
            // UI & wave management
            scanner::scanner_update,
            ui::ui_update,
            waves::wave_check,
            waves::check_all_humans_dead,
            player::explosion_system,
            player::smart_bomb_system,
            player::hyperspace_system,
        )
            .after(camera::sync_transforms)
            .run_if(in_state(AppState::Playing)))
        // Player death
        .add_systems(OnEnter(AppState::PlayerDeath), ui::player_death_setup)
        .add_systems(Update, (
            ui::player_death_timer,
            player::explosion_system,
        ).run_if(in_state(AppState::PlayerDeath)))
        // Game over
        .add_systems(OnEnter(AppState::GameOver), ui::game_over_setup)
        .add_systems(OnExit(AppState::GameOver), ui::game_over_cleanup)
        .add_systems(Update, ui::game_over_input.run_if(in_state(AppState::GameOver)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
