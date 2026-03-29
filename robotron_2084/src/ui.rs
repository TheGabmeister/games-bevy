#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Start screen
            .add_systems(OnEnter(AppState::StartScreen), spawn_start_screen)
            .add_systems(
                Update,
                start_screen_input.run_if(in_state(AppState::StartScreen)),
            )
            // HUD
            .add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(
                Update,
                update_hud.run_if(in_state(AppState::Playing)),
            )
            // Wave overlay
            .add_systems(OnEnter(PlayState::WaveIntro), spawn_wave_overlay)
            // Pause
            .add_systems(Update, pause_input.in_set(GameSet::Input))
            .add_systems(OnEnter(PlayState::Paused), spawn_pause_overlay)
            .add_systems(OnExit(PlayState::Paused), despawn_pause_overlay)
            .add_systems(
                Update,
                unpause_input.run_if(in_state(PlayState::Paused)),
            )
            // Game over
            .add_systems(
                OnEnter(AppState::GameOver),
                (persist_high_score, spawn_game_over, start_game_over_timer),
            )
            .add_systems(
                Update,
                game_over_input.run_if(in_state(AppState::GameOver)),
            );
    }
}

// --- Start Screen ---

fn spawn_start_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            DespawnOnExit(AppState::StartScreen),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("ROBOTRON 2084"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(5.0, 1.0, 0.2)),
            ));
            parent.spawn((
                Text::new("WASD to move  |  Arrow keys to aim & fire"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("ESC to pause"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
            parent.spawn((
                Text::new("Press SPACE to start"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn start_screen_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

// --- HUD ---

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Text::new("SCORE: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
        DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        Text::new("WAVE: 1"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(5.0, 5.0, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Percent(50.0),
            ..default()
        },
        WaveText,
        DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        Text::new(format!("LIVES: {}", STARTING_LIVES)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        LivesText,
        DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        Text::new("HI: 0"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        HighScoreText,
        DespawnOnExit(AppState::Playing),
    ));
}

// FIX: High score tracked in memory but no longer saved to disk every frame
fn update_hud(
    mut game: ResMut<GameState>,
    mut score_q: Query<
        &mut Text,
        (
            With<ScoreText>,
            Without<LivesText>,
            Without<WaveText>,
            Without<HighScoreText>,
        ),
    >,
    mut lives_q: Query<
        &mut Text,
        (
            With<LivesText>,
            Without<ScoreText>,
            Without<WaveText>,
            Without<HighScoreText>,
        ),
    >,
    mut wave_q: Query<
        &mut Text,
        (
            With<WaveText>,
            Without<ScoreText>,
            Without<LivesText>,
            Without<HighScoreText>,
        ),
    >,
    mut hi_q: Query<
        &mut Text,
        (
            With<HighScoreText>,
            Without<ScoreText>,
            Without<LivesText>,
            Without<WaveText>,
        ),
    >,
) {
    if game.score > game.high_score {
        game.high_score = game.score;
    }
    if let Ok(mut text) = score_q.single_mut() {
        *text = Text::new(format!("SCORE: {}", game.score));
    }
    if let Ok(mut text) = lives_q.single_mut() {
        *text = Text::new(format!("LIVES: {}", game.lives));
    }
    if let Ok(mut text) = wave_q.single_mut() {
        *text = Text::new(format!("WAVE: {}", game.current_wave));
    }
    if let Ok(mut text) = hi_q.single_mut() {
        *text = Text::new(format!("HI: {}", game.high_score));
    }
}

// --- Wave Overlay ---

fn spawn_wave_overlay(mut commands: Commands, game: Res<GameState>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            DespawnOnExit(PlayState::WaveIntro),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("WAVE {}", game.current_wave)),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(5.0, 5.0, 0.5)),
            ));
        });
}

// --- Pause ---

fn pause_input(input: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<PlayState>>) {
    if input.just_pressed(KeyCode::Escape) {
        next_state.set(PlayState::Paused);
    }
}

fn unpause_input(input: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<PlayState>>) {
    if input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::Space) {
        next_state.set(PlayState::WaveActive);
    }
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            PauseOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn despawn_pause_overlay(mut commands: Commands, query: Query<Entity, With<PauseOverlay>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// --- Game Over ---

// FIX: High score saved to disk once on game over instead of every frame
fn persist_high_score(game: Res<GameState>) {
    save_high_score(game.high_score);
}

fn start_game_over_timer(mut timer: ResMut<GameOverTimer>) {
    timer.0.reset();
}

fn spawn_game_over(mut commands: Commands, game: Res<GameState>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            DespawnOnExit(AppState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(5.0, 0.2, 0.2)),
            ));
            parent.spawn((
                Text::new(format!("Score: {}", game.score)),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(format!("High Score: {}", game.high_score)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(5.0, 5.0, 0.5)),
            ));
            parent.spawn((
                Text::new(format!("Wave: {}", game.current_wave)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("Press SPACE to restart"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn game_over_input(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut timer: ResMut<GameOverTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}
