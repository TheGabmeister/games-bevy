use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Start Screen
            .add_systems(OnEnter(AppState::StartScreen), spawn_start_screen)
            .add_systems(OnExit(AppState::StartScreen), cleanup::<StartScreenUI>)
            .add_systems(
                Update,
                start_game_input.run_if(in_state(AppState::StartScreen)),
            )
            // Playing HUD
            .add_systems(OnEnter(AppState::Playing), spawn_game_hud)
            .add_systems(OnExit(AppState::Playing), cleanup::<GameHudUI>)
            .add_systems(
                Update,
                (
                    update_score_text,
                    update_high_score_text,
                    update_level_text,
                    update_lives_text,
                    update_status_text,
                    update_timer_bar,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            // Game Over
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over)
            .add_systems(OnExit(AppState::GameOver), cleanup::<GameOverUI>)
            .add_systems(Update, restart_input.run_if(in_state(AppState::GameOver)));
    }
}

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// --- Start Screen ---

fn spawn_start_screen(mut commands: Commands) {
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
                Text::new("FROGGER"),
                TextFont {
                    font_size: FONT_SIZE_TITLE,
                    ..default()
                },
                TextColor(COLOR_TITLE),
            ));
            parent.spawn((
                Text::new("Press Space or Enter to Play"),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new("Move with WASD or Arrow Keys"),
                TextFont {
                    font_size: FONT_SIZE_HUD,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new("Fill all 5 home bays before the timer runs out"),
                TextFont {
                    font_size: FONT_SIZE_HUD,
                    ..default()
                },
                TextColor(COLOR_HIGHLIGHT_TEXT),
            ));
        });
}

fn start_game_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_data: ResMut<GameData>,
    mut timer: ResMut<FrogTimer>,
    mut level_state: ResMut<LevelState>,
    mut frog_event: ResMut<FrogEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        game_data.reset_for_new_run();
        timer.reset();
        level_state.reset_for_new_run();
        *frog_event = FrogEvent::None;
        next_state.set(AppState::Playing);
    }
}

// --- Game HUD ---

fn spawn_game_hud(mut commands: Commands) {
    commands
        .spawn((
            GameHudUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|root| {
            // Top bar: Score | Level | Lives
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(HUD_PADDING)),
                justify_content: JustifyContent::SpaceBetween,
                flex_wrap: FlexWrap::Wrap,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|top| {
                top.spawn((
                    ScoreText,
                    Text::new("Score: 0"),
                    TextFont {
                        font_size: FONT_SIZE_HUD,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                ));
                top.spawn((
                    HighScoreText,
                    Text::new("High: 0"),
                    TextFont {
                        font_size: FONT_SIZE_HUD,
                        ..default()
                    },
                    TextColor(COLOR_HIGHLIGHT_TEXT),
                ));
                top.spawn((
                    LevelText,
                    Text::new("Level: 1"),
                    TextFont {
                        font_size: FONT_SIZE_HUD,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                ));
                top.spawn((
                    LivesText,
                    Text::new("Lives: 3"),
                    TextFont {
                        font_size: FONT_SIZE_HUD,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                ));
            });

            // Bottom: Timer bar
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::new(
                    Val::Px(HUD_SIDE_MARGIN),
                    Val::Px(HUD_SIDE_MARGIN),
                    Val::Px(0.0),
                    Val::Px(HUD_PADDING),
                ),
                ..default()
            })
            .with_children(|bottom| {
                bottom
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(TIMER_BAR_HEIGHT),
                            ..default()
                        },
                        BackgroundColor(COLOR_TIMER_BACKGROUND),
                    ))
                    .with_children(|bar_bg| {
                        bar_bg.spawn((
                            TimerBar,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(COLOR_TIMER_SAFE),
                        ));
                    });
            });

            root.spawn((
                StatusText,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(56.0),
                    left: Val::Percent(50.0),
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font_size: FONT_SIZE_STATUS,
                    ..default()
                },
                TextColor(COLOR_HIGHLIGHT_TEXT),
                Transform::from_translation(Vec3::new(-120.0, 0.0, 0.0)),
            ));
        });
}

fn update_score_text(game_data: Res<GameData>, mut query: Query<&mut Text, With<ScoreText>>) {
    if !game_data.is_changed() {
        return;
    }
    if let Ok(mut text) = query.single_mut() {
        **text = format!("Score: {}", game_data.score);
    }
}

fn update_high_score_text(
    game_data: Res<GameData>,
    mut query: Query<&mut Text, With<HighScoreText>>,
) {
    if !game_data.is_changed() {
        return;
    }
    if let Ok(mut text) = query.single_mut() {
        **text = format!("High: {}", game_data.high_score);
    }
}

fn update_level_text(game_data: Res<GameData>, mut query: Query<&mut Text, With<LevelText>>) {
    if !game_data.is_changed() {
        return;
    }
    if let Ok(mut text) = query.single_mut() {
        **text = format!("Level: {}", game_data.level);
    }
}

fn update_lives_text(game_data: Res<GameData>, mut query: Query<&mut Text, With<LivesText>>) {
    if !game_data.is_changed() {
        return;
    }
    if let Ok(mut text) = query.single_mut() {
        **text = format!("Lives: {}", game_data.lives);
    }
}

fn update_status_text(level_state: Res<LevelState>, mut query: Query<&mut Text, With<StatusText>>) {
    if !level_state.is_changed() {
        return;
    }
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    text.clear();
    if level_state.celebrating {
        text.push_str("Level Clear!");
    }
}

fn update_timer_bar(
    timer: Res<FrogTimer>,
    mut query: Query<(&mut Node, &mut BackgroundColor), With<TimerBar>>,
) {
    if !timer.is_changed() {
        return;
    }
    let Ok((mut node, mut color)) = query.single_mut() else {
        return;
    };
    let pct = (timer.remaining_secs / LIFE_TIMER_SECS * 100.0).clamp(0.0, 100.0);
    node.width = Val::Percent(pct);
    color.0 = if pct <= 25.0 {
        COLOR_TIMER_DANGER
    } else if pct <= 50.0 {
        COLOR_TIMER_WARNING
    } else {
        COLOR_TIMER_SAFE
    };
}

// --- Game Over ---

fn spawn_game_over(mut commands: Commands, game_data: Res<GameData>) {
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
                Text::new("GAME OVER"),
                TextFont {
                    font_size: FONT_SIZE_TITLE,
                    ..default()
                },
                TextColor(COLOR_GAME_OVER),
            ));
            parent.spawn((
                Text::new(format!(
                    "Score: {}  |  Level: {}",
                    game_data.score, game_data.level
                )),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new(format!("High Score: {}", game_data.high_score)),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(COLOR_HIGHLIGHT_TEXT),
            ));
            parent.spawn((
                Text::new("Press Space or Enter to Restart"),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

fn restart_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::StartScreen);
    }
}
