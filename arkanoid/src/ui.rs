use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::bricks::ScoreChanged;
use crate::constants::*;
use crate::resources::{Lives, Round, Score};
use crate::states::{AppState, PlayState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartScreen), spawn_title)
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over)
            .add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(OnEnter(PlayState::Ready), spawn_ready_banner)
            .add_systems(
                Update,
                (update_score_hud, update_state_hud).run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct HighScoreText;

#[derive(Component)]
struct RoundText;

/// One reserve-life icon, identified by its slot index (0-based).
#[derive(Component)]
struct LifeIcon(u32);

const HUD_FONT: f32 = 20.0;

fn label(top: f32, anchor: Node, text: String, marker: impl Bundle) -> impl Bundle {
    (
        marker,
        Text::new(text),
        TextFont {
            font_size: HUD_FONT,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(top),
            ..anchor
        },
        DespawnOnExit(AppState::Playing),
    )
}

fn spawn_hud(mut commands: Commands, assets: Res<GameAssets>, score: Res<Score>, round: Res<Round>) {
    commands.spawn(label(
        28.0,
        Node {
            left: Val::Px(30.0),
            ..default()
        },
        format!("1UP\n{}", score.current),
        ScoreText,
    ));
    commands.spawn(label(
        28.0,
        Node {
            right: Val::Px(30.0),
            ..default()
        },
        format!("HIGH SCORE\n{}", score.high),
        HighScoreText,
    ));
    commands.spawn(label(
        70.0,
        Node {
            left: Val::Px(30.0),
            ..default()
        },
        format!("ROUND {}", round.0),
        RoundText,
    ));

    // Reserve-life icons along the bottom-left.
    for i in 0..LIVES_START {
        commands.spawn((
            LifeIcon(i),
            ImageNode::new(assets.sprites.vaus_life_icon.clone()),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(30.0 + i as f32 * 54.0),
                width: Val::Px(48.0),
                height: Val::Px(12.0),
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ));
    }
}

fn update_score_hud(
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

fn update_state_hud(
    lives: Res<Lives>,
    round: Res<Round>,
    mut round_text: Query<&mut Text, With<RoundText>>,
    mut icons: Query<(&LifeIcon, &mut Visibility)>,
) {
    if !lives.is_changed() && !round.is_changed() {
        return;
    }
    if let Ok(mut text) = round_text.single_mut() {
        text.0 = format!("ROUND {}", round.0);
    }
    for (icon, mut visibility) in &mut icons {
        *visibility = if icon.0 < lives.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Centered overlay column used by the ready/title/game-over screens.
fn centered_overlay(despawn: impl Bundle) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(16.0),
            ..default()
        },
        despawn,
    )
}

fn big_text(text: String, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

fn spawn_ready_banner(mut commands: Commands, round: Res<Round>) {
    commands
        .spawn(centered_overlay(DespawnOnExit(PlayState::Ready)))
        .with_children(|parent| {
            parent.spawn(big_text(
                format!("ROUND {} READY", round.0),
                44.0,
                Color::WHITE,
            ));
        });
}

fn spawn_title(mut commands: Commands, assets: Res<GameAssets>) {
    // Full-screen title art (600×800 == window), drawn above the playfield border.
    commands.spawn((
        Sprite::from_image(assets.sprites.title_screen.clone()),
        Transform::from_xyz(0.0, 0.0, Z_OVERLAY),
        DespawnOnExit(AppState::StartScreen),
    ));
}

fn spawn_game_over(mut commands: Commands, score: Res<Score>) {
    commands
        .spawn(centered_overlay(DespawnOnExit(AppState::GameOver)))
        .with_children(|parent| {
            parent.spawn(big_text("GAME OVER".into(), 56.0, Color::srgb(1.0, 0.3, 0.3)));
            parent.spawn(big_text(
                format!("SCORE  {}", score.current),
                28.0,
                Color::WHITE,
            ));
            parent.spawn(big_text(
                "PRESS START".into(),
                24.0,
                Color::srgb(0.7, 0.85, 1.0),
            ));
        });
}
