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
                (update_hud, update_timer_bar).run_if(in_state(AppState::Playing)),
            )
            // Game Over
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over)
            .add_systems(OnExit(AppState::GameOver), cleanup::<GameOverUI>)
            .add_systems(
                Update,
                restart_input.run_if(in_state(AppState::GameOver)),
            );
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
                TextColor(Color::srgb(0.1, 0.9, 0.1)),
            ));
            parent.spawn((
                Text::new("Press Space or Enter to Play"),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(TEXT_COLOR),
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
        game_data.score = 0;
        game_data.lives = STARTING_LIVES;
        game_data.level = 1;
        game_data.filled_bays = [false; 5];
        game_data.max_row_this_life = 0;
        timer.remaining_secs = LIFE_TIMER_SECS;
        level_state.speed_multiplier = 1.0;
        level_state.celebrating = false;
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
                padding: UiRect::all(Val::Px(12.0)),
                justify_content: JustifyContent::SpaceBetween,
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
                padding: UiRect::new(Val::Px(48.0), Val::Px(48.0), Val::Px(0.0), Val::Px(12.0)),
                ..default()
            })
            .with_children(|bottom| {
                bottom
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(12.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_children(|bar_bg| {
                        bar_bg.spawn((
                            TimerBar,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.1, 0.9, 0.1)),
                        ));
                    });
            });
        });
}

fn update_hud(
    game_data: Res<GameData>,
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<LivesText>, Without<LevelText>)>,
    mut level_q: Query<&mut Text, (With<LevelText>, Without<ScoreText>, Without<LivesText>)>,
    mut lives_q: Query<&mut Text, (With<LivesText>, Without<ScoreText>, Without<LevelText>)>,
) {
    if !game_data.is_changed() {
        return;
    }
    if let Ok(mut text) = score_q.single_mut() {
        **text = format!("Score: {}", game_data.score);
    }
    if let Ok(mut text) = level_q.single_mut() {
        **text = format!("Level: {}", game_data.level);
    }
    if let Ok(mut text) = lives_q.single_mut() {
        **text = format!("Lives: {}", game_data.lives);
    }
}

fn update_timer_bar(timer: Res<FrogTimer>, mut query: Query<&mut Node, With<TimerBar>>) {
    let Ok(mut node) = query.single_mut() else {
        return;
    };
    let pct = (timer.remaining_secs / LIFE_TIMER_SECS * 100.0).clamp(0.0, 100.0);
    node.width = Val::Percent(pct);
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
                TextColor(Color::srgb(0.9, 0.2, 0.2)),
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
                TextColor(Color::srgb(0.9, 0.9, 0.2)),
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

fn restart_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::StartScreen);
    }
}
