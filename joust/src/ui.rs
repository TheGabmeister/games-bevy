use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::states::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartScreen), setup_start_screen)
            .add_systems(
                Update,
                start_screen_input.run_if(in_state(AppState::StartScreen)),
            )
            .add_systems(OnEnter(AppState::Playing), setup_hud)
            .add_systems(Update, update_hud.run_if(in_state(AppState::Playing)))
            .add_systems(OnEnter(AppState::GameOver), setup_game_over)
            .add_systems(
                Update,
                game_over_input.run_if(in_state(AppState::GameOver)),
            );
    }
}

// --- Start Screen ---

fn setup_start_screen(mut commands: Commands) {
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
            StateScoped(AppState::StartScreen),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("JOUST"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
            ));
            parent.spawn((
                Text::new("Press SPACE to start"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("Press 2 for two players"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

fn start_screen_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut player_count: ResMut<PlayerCount>,
) {
    if keys.just_pressed(KeyCode::Digit2) {
        player_count.0 = if player_count.0 == 2 { 1 } else { 2 };
    }
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);
    }
}

// --- HUD ---

fn setup_hud(mut commands: Commands, player_count: Res<PlayerCount>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                padding: UiRect::horizontal(Val::Px(20.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                ..default()
            },
            StateScoped(AppState::Playing),
        ))
        .with_children(|parent| {
            // P1 score
            parent.spawn((
                Text::new("P1: 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.6, 1.0)),
                ScoreText(0),
            ));
            // P1 lives
            parent.spawn((
                Text::new("***"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.6, 1.0)),
                LivesText(0),
            ));
            // Wave
            parent.spawn((
                Text::new("Wave 1"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.3)),
                WaveText,
            ));
            if player_count.0 > 1 {
                // P2 lives
                parent.spawn((
                    Text::new("***"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                    LivesText(1),
                ));
                // P2 score
                parent.spawn((
                    Text::new("P2: 0"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                    ScoreText(1),
                ));
            }
        });
}

fn update_hud(
    game_state: Res<GameState>,
    mut score_texts: Query<(&ScoreText, &mut Text)>,
    mut lives_texts: Query<(&LivesText, &mut Text), Without<ScoreText>>,
    mut wave_texts: Query<&mut Text, (With<WaveText>, Without<ScoreText>, Without<LivesText>)>,
) {
    for (st, mut text) in &mut score_texts {
        let prefix = if st.0 == 0 { "P1" } else { "P2" };
        **text = format!("{}: {}", prefix, game_state.scores[st.0 as usize]);
    }
    for (lt, mut text) in &mut lives_texts {
        let lives = game_state.lives[lt.0 as usize];
        **text = "*".repeat(lives as usize);
    }
    for mut text in &mut wave_texts {
        **text = format!("Wave {}", game_state.wave);
    }
}

// --- Game Over ---

fn setup_game_over(mut commands: Commands, game_state: Res<GameState>) {
    let final_score = game_state.scores[0].max(game_state.scores[1]);

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            StateScoped(AppState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));
            parent.spawn((
                Text::new(format!("Score: {}", final_score)),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            if game_state.high_score > 0 {
                parent.spawn((
                    Text::new(format!("High Score: {}", game_state.high_score)),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                ));
            }
            parent.spawn((
                Text::new("Press SPACE to restart"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn game_over_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(AppState::StartScreen);
    }
}
