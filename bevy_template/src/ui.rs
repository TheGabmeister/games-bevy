use bevy::prelude::*;

use crate::components::{GameHudUI, GameOverUI, ScoreText, StartScreenUI};
use crate::constants::*;
use crate::resources::GameData;
use crate::states::AppState;

// --- Start Screen ---

pub fn spawn_start_screen(mut commands: Commands) {
    commands
        .spawn((
            StartScreenUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(WINDOW_TITLE),
                TextFont {
                    font_size: FONT_SIZE_TITLE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new("Press Space to Play"),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

pub fn cleanup_start_screen(
    mut commands: Commands,
    query: Query<Entity, With<StartScreenUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn start_game_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);
    }
}

// --- Game HUD ---

pub fn spawn_game_hud(mut commands: Commands) {
    commands
        .spawn((
            GameHudUI,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ScoreText,
                Text::new("Score: 0"),
                TextFont {
                    font_size: FONT_SIZE_HUD,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

pub fn cleanup_game_hud(mut commands: Commands, query: Query<Entity, With<GameHudUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn increment_score(time: Res<Time>, mut game_data: ResMut<GameData>) {
    // +1 score per second (accumulates via delta)
    let prev = game_data.score;
    let new = prev as f32 + time.delta_secs();
    game_data.score = new as u32;
}

pub fn update_score_display(
    game_data: Res<GameData>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    **text = format!("Score: {}", game_data.score);
}

pub fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::GameOver);
    }
}

// --- Game Over Screen ---

pub fn spawn_game_over(mut commands: Commands, game_data: Res<GameData>) {
    commands
        .spawn((
            GameOverUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Over"),
                TextFont {
                    font_size: FONT_SIZE_TITLE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new(format!("Final Score: {}", game_data.score)),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new("Press Space to Restart"),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

pub fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn restart_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_data: ResMut<GameData>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        game_data.score = 0;
        next_state.set(AppState::StartScreen);
    }
}
