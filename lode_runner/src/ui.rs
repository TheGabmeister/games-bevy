use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::levels::LEVELS;
use crate::resources::GameState;
use crate::states::{AppState, PlayState};

pub fn spawn_start_screen(mut commands: Commands) {
    commands.insert_resource(GameState::default());
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            DespawnOnExit(AppState::StartScreen),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("LODE RUNNER"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(COLOR_GOLD),
            ));
            parent.spawn((
                Text::new("Press SPACE to play"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

pub fn start_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);
    }
}

pub fn spawn_hud(mut commands: Commands, game_state: Res<GameState>) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(HUD_HEIGHT),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                padding: UiRect::horizontal(Val::Px(12.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|parent| {
            let font = TextFont {
                font_size: 18.0,
                ..default()
            };
            parent.spawn((
                HudScore,
                Text::new(format!("Score: {}", game_state.score)),
                font.clone(),
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                HudGold,
                Text::new(format!("Gold: {}", game_state.total_gold)),
                font.clone(),
                TextColor(COLOR_GOLD),
            ));
            parent.spawn((
                HudLevel,
                Text::new(format!("Level: {}", game_state.current_level + 1)),
                font.clone(),
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                HudLives,
                Text::new(format!("Lives: {}", game_state.lives)),
                font,
                TextColor(Color::srgb(1.0, 0.4, 0.4)),
            ));
        });
}

#[allow(clippy::type_complexity)]
pub fn update_hud(
    game_state: Res<GameState>,
    mut score_q: Query<
        &mut Text,
        (
            With<HudScore>,
            Without<HudGold>,
            Without<HudLevel>,
            Without<HudLives>,
        ),
    >,
    mut gold_q: Query<
        &mut Text,
        (
            With<HudGold>,
            Without<HudScore>,
            Without<HudLevel>,
            Without<HudLives>,
        ),
    >,
    mut level_q: Query<
        &mut Text,
        (
            With<HudLevel>,
            Without<HudScore>,
            Without<HudGold>,
            Without<HudLives>,
        ),
    >,
    mut lives_q: Query<
        &mut Text,
        (
            With<HudLives>,
            Without<HudScore>,
            Without<HudGold>,
            Without<HudLevel>,
        ),
    >,
) {
    if !game_state.is_changed() {
        return;
    }
    if let Ok(mut t) = score_q.single_mut() {
        **t = format!("Score: {}", game_state.score);
    }
    if let Ok(mut t) = gold_q.single_mut() {
        **t = format!("Gold: {}", game_state.total_gold);
    }
    if let Ok(mut t) = level_q.single_mut() {
        **t = format!("Level: {}", game_state.current_level + 1);
    }
    if let Ok(mut t) = lives_q.single_mut() {
        **t = format!("Lives: {}", game_state.lives);
    }
}

pub fn pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    play_state: Res<State<PlayState>>,
    mut next_play: ResMut<NextState<PlayState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match play_state.get() {
            PlayState::Running => next_play.set(PlayState::Paused),
            PlayState::Paused => next_play.set(PlayState::Running),
            _ => {}
        }
    }
}

pub fn spawn_pause_overlay(mut commands: Commands) {
    commands
        .spawn((
            PauseOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            DespawnOnExit(AppState::Playing),
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

pub fn despawn_pause_overlay(mut commands: Commands, query: Query<Entity, With<PauseOverlay>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_level_complete_screen(mut commands: Commands, mut game_state: ResMut<GameState>) {
    game_state.score += LEVEL_COMPLETE_SCORE;

    let has_next = game_state.current_level + 1 < LEVELS.len();
    let message = if has_next {
        "Press SPACE for next level"
    } else {
        "Press SPACE to continue"
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            DespawnOnExit(AppState::LevelComplete),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("LEVEL COMPLETE!"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(COLOR_GOLD),
            ));
            parent.spawn((
                Text::new(format!("Score: {}", game_state.score)),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(message),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

pub fn level_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        game_state.current_level += 1;
        if game_state.current_level < LEVELS.len() {
            next_state.set(AppState::Playing);
        } else {
            next_state.set(AppState::GameOver);
        }
    }
}

pub fn spawn_game_over_screen(mut commands: Commands, game_state: Res<GameState>) {
    let (title, color) = if game_state.lives == 0 {
        ("GAME OVER", Color::srgb(1.0, 0.3, 0.3))
    } else {
        ("YOU WIN!", COLOR_GOLD)
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            DespawnOnExit(AppState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(color),
            ));
            parent.spawn((
                Text::new(format!("Final Score: {}", game_state.score)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press SPACE to play again"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

pub fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::StartScreen);
    }
}
