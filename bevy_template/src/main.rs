mod components;
mod constants;
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
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .init_resource::<GameData>()
        // Startup
        .add_systems(Startup, (setup, player::spawn_player))
        // Start Screen
        .add_systems(OnEnter(AppState::StartScreen), ui::spawn_start_screen)
        .add_systems(OnExit(AppState::StartScreen), ui::cleanup_start_screen)
        .add_systems(
            Update,
            ui::start_game_input.run_if(in_state(AppState::StartScreen)),
        )
        // Playing
        .add_systems(OnEnter(AppState::Playing), (ui::spawn_game_hud, player::reset_player))
        .add_systems(OnExit(AppState::Playing), ui::cleanup_game_hud)
        .add_systems(
            Update,
            (
                player::player_input,
                player::player_movement.after(player::player_input),
                ui::increment_score,
                ui::update_score_display.after(ui::increment_score),
                ui::game_over_input,
            )
                .run_if(in_state(AppState::Playing)),
        )
        // Game Over
        .add_systems(OnEnter(AppState::GameOver), ui::spawn_game_over)
        .add_systems(OnExit(AppState::GameOver), ui::cleanup_game_over)
        .add_systems(
            Update,
            ui::restart_input.run_if(in_state(AppState::GameOver)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
