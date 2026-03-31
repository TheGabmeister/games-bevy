use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::input::ActionInput;
use crate::resources::*;
use crate::states::*;

/// Spawn a floating "+{score}" text popup at the given world position.
pub fn spawn_score_popup(commands: &mut Commands, score: u32, x: f32, y: f32) {
    spawn_score_popup_colored(commands, score, x, y, Color::WHITE);
}

/// Spawn a floating "+{score}" text popup with a custom color.
pub fn spawn_score_popup_colored(commands: &mut Commands, score: u32, x: f32, y: f32, color: Color) {
    commands.spawn((
        ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
        Text2d::new(format!("+{score}")),
        TextFont { font_size: 8.0, ..default() },
        TextColor(color),
        Transform::from_xyz(x, y, Z_PLAYER + 1.0),
        DespawnOnExit(AppState::Playing),
    ));
}

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
            // Game events → GameData
            .add_systems(
                Update,
                apply_game_events.run_if(in_state(AppState::Playing)),
            )
            // HUD (only when data actually changes)
            .add_systems(
                Update,
                update_score_hud
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_changed::<GameData>),
            )
            .add_systems(
                Update,
                update_timer_hud
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_changed::<GameTimer>),
            )
            .add_systems(
                Update,
                countdown_timer.in_set(GameplaySet::Late),
            )
            // Pause
            .add_systems(
                Update,
                pause_input.run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnEnter(PlayState::Paused), spawn_pause_overlay)
            // Game over
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over_screen)
            .add_systems(
                Update,
                game_over_input.run_if(in_state(AppState::GameOver)),
            );
    }
}

// ── Start Screen ──

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
                Text::new("SUPER MARIO BROS"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press Enter to Start"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

fn start_screen_input(
    action: Res<ActionInput>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if action.confirm_just_pressed {
        next_state.set(AppState::Playing);
    }
}

// ── HUD ──

pub fn spawn_hud(commands: &mut Commands, game_data: &GameData, game_timer: &GameTimer) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("MARIO\n{:06}", game_data.score)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ScoreText,
            ));
            parent.spawn((
                Text::new(format!("COINS\n\u{00d7}{:02}", game_data.coins)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                CoinText,
            ));
            parent.spawn((
                Text::new(format!("WORLD\n{}", game_data.world_name)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(format!("TIME\n{}", game_timer.time as u32)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TimerText,
            ));
        });
}

// ── Game Event Processing ──

fn apply_game_events(
    mut game_data: ResMut<GameData>,
    mut score_events: MessageReader<ScoreEvent>,
    mut coin_events: MessageReader<CoinEvent>,
) {
    for event in score_events.read() {
        game_data.score += event.points;
    }
    for _ in coin_events.read() {
        game_data.add_coin();
    }
}

// ── HUD Updates (only when data changes) ──

fn update_score_hud(
    game_data: Res<GameData>,
    mut score_q: Query<&mut Text, With<ScoreText>>,
    mut coin_q: Query<&mut Text, (With<CoinText>, Without<ScoreText>)>,
) {
    if let Ok(mut text) = score_q.single_mut() {
        *text = Text::new(format!("MARIO\n{:06}", game_data.score));
    }
    if let Ok(mut text) = coin_q.single_mut() {
        *text = Text::new(format!("COINS\n\u{00d7}{:02}", game_data.coins));
    }
}

fn update_timer_hud(
    game_timer: Res<GameTimer>,
    mut timer_q: Query<&mut Text, With<TimerText>>,
) {
    if let Ok(mut text) = timer_q.single_mut() {
        *text = Text::new(format!("TIME\n{}", game_timer.time as u32));
    }
}

fn countdown_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    game_timer.time -= TIMER_TICK_RATE * time.delta_secs();
    if game_timer.time <= 0.0 {
        game_timer.time = 0.0;
        next_play_state.set(PlayState::Dying);
    }
}

// ── Pause ──

fn pause_input(
    action: Res<ActionInput>,
    current_state: Res<State<PlayState>>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    if action.pause_just_pressed {
        match current_state.get() {
            PlayState::Running => next_state.set(PlayState::Paused),
            PlayState::Paused => next_state.set(PlayState::Running),
            _ => {}
        }
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            DespawnOnExit(PlayState::Paused),
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

// ── Game Over ──

fn spawn_game_over_screen(mut commands: Commands) {
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
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press Enter to Continue"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

fn game_over_input(
    action: Res<ActionInput>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if action.confirm_just_pressed {
        next_state.set(AppState::StartScreen);
    }
}
