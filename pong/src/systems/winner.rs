use bevy::prelude::*;

use crate::{
    components::{Score, Winner, WinnerEntity},
    state::Phase,
};

pub fn spawn_winner_screen(mut commands: Commands, winner: Res<Winner>, score: Res<Score>) {
    let title = match winner.side {
        Some(side) => format!("{} Wins!", side.name()),
        None => "Game Over".to_string(),
    };

    let score_line = format!("Final Score: {}", score.formatted());

    commands.spawn((
        WinnerEntity,
        Text2d::new(title),
        TextFont {
            font_size: 72.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, 100.0, 20.0),
    ));

    commands.spawn((
        WinnerEntity,
        Text2d::new(score_line),
        TextFont {
            font_size: 42.0,
            ..default()
        },
        TextColor(Color::srgb(0.88, 0.88, 0.88)),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, 15.0, 20.0),
    ));

    commands.spawn((
        WinnerEntity,
        Text2d::new("Press Enter to Return to Menu"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, -80.0, 20.0),
    ));
}

pub fn winner_input(input: Res<ButtonInput<KeyCode>>, mut next_phase: ResMut<NextState<Phase>>) {
    if input.just_pressed(KeyCode::Enter) {
        next_phase.set(Phase::Menu);
    }
}
