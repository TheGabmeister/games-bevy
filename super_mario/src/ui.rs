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
            .add_systems(
                Update,
                update_hud.run_if(in_state(AppState::Playing)),
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

// ── HUD ──

pub fn spawn_hud(commands: &mut Commands, game_data: &GameData) {
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
                Text::new(format!("TIME\n{}", game_data.timer as u32)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TimerText,
            ));
        });
}

fn update_hud(
    game_data: Res<GameData>,
    mut score_q: Query<&mut Text, With<ScoreText>>,
    mut coin_q: Query<&mut Text, (With<CoinText>, Without<ScoreText>)>,
    mut timer_q: Query<
        &mut Text,
        (
            With<TimerText>,
            Without<ScoreText>,
            Without<CoinText>,
        ),
    >,
) {
    if let Ok(mut text) = score_q.single_mut() {
        *text = Text::new(format!("MARIO\n{:06}", game_data.score));
    }
    if let Ok(mut text) = coin_q.single_mut() {
        *text = Text::new(format!("COINS\n\u{00d7}{:02}", game_data.coins));
    }
    if let Ok(mut text) = timer_q.single_mut() {
        *text = Text::new(format!("TIME\n{}", game_data.timer as u32));
    }
}

fn countdown_timer(
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    game_data.timer -= TIMER_TICK_RATE * time.delta_secs();
    if game_data.timer <= 0.0 {
        game_data.timer = 0.0;
        next_play_state.set(PlayState::Dying);
    }
}

// ── Pause ──

fn pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<PlayState>>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::StartScreen);
    }
}
