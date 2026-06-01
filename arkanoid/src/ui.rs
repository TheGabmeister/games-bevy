use bevy::prelude::*;

use crate::bricks::ScoreChanged;
use crate::resources::Score;
use crate::states::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(Update, update_hud.run_if(in_state(AppState::Playing)));
    }
}

/// The "1UP" current-score readout.
#[derive(Component)]
struct ScoreText;

/// The "HIGH SCORE" readout.
#[derive(Component)]
struct HighScoreText;

fn spawn_hud(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        ScoreText,
        Text::new(format!("1UP\n{}", score.current)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(28.0),
            left: Val::Px(30.0),
            ..default()
        },
        DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        HighScoreText,
        Text::new(format!("HIGH SCORE\n{}", score.high)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(28.0),
            right: Val::Px(30.0),
            ..default()
        },
        DespawnOnExit(AppState::Playing),
    ));
}

fn update_hud(
    mut changed: MessageReader<ScoreChanged>,
    score: Res<Score>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<HighScoreText>)>,
    mut high_text: Query<&mut Text, (With<HighScoreText>, Without<ScoreText>)>,
) {
    if changed.read().count() == 0 {
        return;
    }
    if let Ok(mut text) = score_text.single_mut() {
        text.0 = format!("1UP\n{}", score.current);
    }
    if let Ok(mut text) = high_text.single_mut() {
        text.0 = format!("HIGH SCORE\n{}", score.high);
    }
}
