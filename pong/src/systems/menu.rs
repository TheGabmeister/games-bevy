use bevy::prelude::*;

use crate::{
    components::{MenuEntity, Score, Winner},
    state::Phase,
};

pub fn spawn_menu(mut commands: Commands) {
    commands.spawn((
        MenuEntity,
        Text2d::new("PONG"),
        TextFont {
            font_size: 88.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, 120.0, 20.0),
    ));

    commands.spawn((
        MenuEntity,
        Text2d::new("Press Enter to Start\n\nLeft Paddle: W / S\nRight Paddle: Up / Down"),
        TextFont {
            font_size: 34.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, -40.0, 20.0),
    ));
}

pub fn menu_input(
    input: Res<ButtonInput<KeyCode>>,
    mut score: ResMut<Score>,
    mut winner: ResMut<Winner>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    if input.just_pressed(KeyCode::Enter) {
        score.reset();
        winner.side = None;
        next_phase.set(Phase::Playing);
    }
}
