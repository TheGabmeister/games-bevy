use bevy::prelude::*;

use crate::constants::*;
use crate::input::InputActions;
use crate::resources::GameStats;

// ---------------------------------------------------------------------------
// State enums
// ---------------------------------------------------------------------------

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum AppState {
    #[default]
    StartScreen,
    Playing,
    GameOver,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
pub enum PlayState {
    #[default]
    Running,
    Paused,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<PlayState>()
            .add_systems(OnEnter(AppState::StartScreen), spawn_start_screen)
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over_screen)
            .add_systems(OnEnter(PlayState::Paused), spawn_pause_overlay)
            .add_systems(
                Update,
                handle_start_input.run_if(in_state(AppState::StartScreen)),
            )
            .add_systems(
                Update,
                handle_game_over_input.run_if(in_state(AppState::GameOver)),
            )
            .add_systems(
                Update,
                handle_pause_input.run_if(in_state(AppState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Start screen
// ---------------------------------------------------------------------------

fn spawn_start_screen(mut commands: Commands) {
    commands.spawn((
        Text2d::new("TETRIS"),
        TextFont {
            font_size: 64.0,
            ..default()
        },
        TextColor(Color::srgb(2.0, 2.0, 4.0)),
        Transform::from_xyz(0.0, 80.0, 10.0),
        DespawnOnExit(AppState::StartScreen),
    ));
    commands.spawn((
        Text2d::new("Press Enter to Start"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
        Transform::from_xyz(0.0, -20.0, 10.0),
        DespawnOnExit(AppState::StartScreen),
    ));
}

fn handle_start_input(
    actions: Res<InputActions>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if actions.start_restart {
        next_state.set(AppState::Playing);
    }
}

// ---------------------------------------------------------------------------
// Game over screen
// ---------------------------------------------------------------------------

fn spawn_game_over_screen(mut commands: Commands, stats: Res<GameStats>) {
    // Dark overlay
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.7),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 9.0),
        DespawnOnExit(AppState::GameOver),
    ));
    commands.spawn((
        Text2d::new("GAME OVER"),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::srgb(4.0, 0.5, 0.5)),
        Transform::from_xyz(0.0, 80.0, 10.0),
        DespawnOnExit(AppState::GameOver),
    ));
    commands.spawn((
        Text2d::new(format!("Score: {}", stats.score)),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(2.0, 2.0, 2.0)),
        Transform::from_xyz(0.0, 20.0, 10.0),
        DespawnOnExit(AppState::GameOver),
    ));
    commands.spawn((
        Text2d::new("Press Enter to Restart"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
        Transform::from_xyz(0.0, -30.0, 10.0),
        DespawnOnExit(AppState::GameOver),
    ));
}

fn handle_game_over_input(
    actions: Res<InputActions>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if actions.start_restart {
        next_state.set(AppState::Playing);
    }
}

// ---------------------------------------------------------------------------
// Pause overlay
// ---------------------------------------------------------------------------

fn spawn_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.5),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 9.0),
        DespawnOnExit(PlayState::Paused),
    ));
    commands.spawn((
        Text2d::new("PAUSED"),
        TextFont {
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        Transform::from_xyz(0.0, 0.0, 10.0),
        DespawnOnExit(PlayState::Paused),
    ));
}

fn handle_pause_input(
    actions: Res<InputActions>,
    current: Res<State<PlayState>>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    if actions.pause {
        match current.get() {
            PlayState::Running => next_state.set(PlayState::Paused),
            PlayState::Paused => next_state.set(PlayState::Running),
        }
    }
}
