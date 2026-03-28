use bevy::prelude::*;

use crate::{
    components::{GameOverScreen, LivesText, MenuScreen, ScoreText, WaveText},
    resources::{Lives, Score, Wave},
    scheduling::GameplaySet,
    states::AppState,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(OnEnter(AppState::MainMenu), show_menu)
            .add_systems(OnExit(AppState::MainMenu), hide_menu)
            .add_systems(OnEnter(AppState::GameOver), show_game_over)
            .add_systems(OnExit(AppState::GameOver), hide_game_over)
            .add_systems(
                Update,
                (update_hud, menu_input, game_over_input).run_if(not(in_state(AppState::Playing))),
            )
            .add_systems(
                Update,
                update_hud
                    .in_set(GameplaySet::Cleanup)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn setup_ui(mut commands: Commands) {
    // HUD — score
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
    ));

    // HUD — lives
    commands.spawn((
        Text::new("Lives: 3"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.2, 0.8, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(10.0),
            ..default()
        },
        LivesText,
    ));

    // HUD — wave
    commands.spawn((
        Text::new("Wave: 1"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(280.0),
            ..default()
        },
        WaveText,
    ));

    // Menu screen
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            Visibility::Visible,
            MenuScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CENTIPEDE"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.8, 0.2)),
            ));
            parent.spawn((
                Text::new("Press ENTER to Play"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Arrow Keys / WASD to move   Space to shoot"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });

    // Game Over screen
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            Visibility::Hidden,
            GameOverScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
            ));
            parent.spawn((
                Text::new("Score: 0"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ScoreText,
            ));
            parent.spawn((
                Text::new("Press ENTER to Play Again"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

fn update_hud(
    score: Res<Score>,
    lives: Res<Lives>,
    wave: Res<Wave>,
    mut score_q: Query<&mut Text, With<ScoreText>>,
    mut lives_q: Query<&mut Text, With<LivesText>>,
    mut wave_q: Query<&mut Text, With<WaveText>>,
) {
    if !(score.is_changed() || lives.is_changed() || wave.is_changed()) {
        return;
    }

    for mut text in &mut score_q {
        *text = Text::new(format!("Score: {}", score.value));
    }
    for mut text in &mut lives_q {
        *text = Text::new(format!("Lives: {}", lives.0));
    }
    for mut text in &mut wave_q {
        *text = Text::new(format!("Wave: {}", wave.0 + 1));
    }
}

fn show_menu(mut menu_q: Query<&mut Visibility, With<MenuScreen>>) {
    for mut vis in &mut menu_q {
        *vis = Visibility::Visible;
    }
}

fn hide_menu(mut menu_q: Query<&mut Visibility, With<MenuScreen>>) {
    for mut vis in &mut menu_q {
        *vis = Visibility::Hidden;
    }
}

fn show_game_over(mut go_q: Query<&mut Visibility, With<GameOverScreen>>) {
    for mut vis in &mut go_q {
        *vis = Visibility::Visible;
    }
}

fn hide_game_over(mut go_q: Query<&mut Visibility, With<GameOverScreen>>) {
    for mut vis in &mut go_q {
        *vis = Visibility::Hidden;
    }
}

fn menu_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        next.set(AppState::Playing);
    }
}

fn game_over_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    if *state.get() != AppState::GameOver {
        return;
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        next.set(AppState::Playing);
    }
}
