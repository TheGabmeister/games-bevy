use bevy::prelude::*;

use crate::constants::*;
use crate::resources::GameStats;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct LinesText;

const HUD_X: f32 = PLAYFIELD_WIDTH / 2.0 + SIDEBAR_MARGIN + 60.0;
const HUD_LABEL_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.4);
const HUD_VALUE_COLOR: Color = Color::srgb(2.0, 2.0, 2.0);
const HUD_LABEL_SIZE: f32 = 18.0;
const HUD_VALUE_SIZE: f32 = 28.0;
const HUD_SECTION_GAP: f32 = 70.0;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, update_hud);
    }
}

fn spawn_hud(mut commands: Commands) {
    let top_y = PLAYFIELD_HEIGHT / 2.0 - 20.0;

    // Score
    spawn_label(&mut commands, "SCORE", HUD_X, top_y);
    commands.spawn((
        ScoreText,
        Text2d::new("0"),
        TextFont {
            font_size: HUD_VALUE_SIZE,
            ..default()
        },
        TextColor(HUD_VALUE_COLOR),
        Transform::from_xyz(HUD_X, top_y - 28.0, 5.0),
    ));

    // Level
    let level_y = top_y - HUD_SECTION_GAP;
    spawn_label(&mut commands, "LEVEL", HUD_X, level_y);
    commands.spawn((
        LevelText,
        Text2d::new("1"),
        TextFont {
            font_size: HUD_VALUE_SIZE,
            ..default()
        },
        TextColor(HUD_VALUE_COLOR),
        Transform::from_xyz(HUD_X, level_y - 28.0, 5.0),
    ));

    // Lines
    let lines_y = top_y - HUD_SECTION_GAP * 2.0;
    spawn_label(&mut commands, "LINES", HUD_X, lines_y);
    commands.spawn((
        LinesText,
        Text2d::new("0"),
        TextFont {
            font_size: HUD_VALUE_SIZE,
            ..default()
        },
        TextColor(HUD_VALUE_COLOR),
        Transform::from_xyz(HUD_X, lines_y - 28.0, 5.0),
    ));
}

fn spawn_label(commands: &mut Commands, text: &str, x: f32, y: f32) {
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size: HUD_LABEL_SIZE,
            ..default()
        },
        TextColor(HUD_LABEL_COLOR),
        Transform::from_xyz(x, y, 5.0),
    ));
}

fn update_hud(
    stats: Res<GameStats>,
    mut score_q: Query<&mut Text2d, (With<ScoreText>, Without<LevelText>, Without<LinesText>)>,
    mut level_q: Query<&mut Text2d, (With<LevelText>, Without<ScoreText>, Without<LinesText>)>,
    mut lines_q: Query<&mut Text2d, (With<LinesText>, Without<ScoreText>, Without<LevelText>)>,
) {
    if !stats.is_changed() {
        return;
    }

    for mut text in &mut score_q {
        **text = stats.score.to_string();
    }
    for mut text in &mut level_q {
        **text = stats.level.to_string();
    }
    for mut text in &mut lines_q {
        **text = stats.lines_cleared.to_string();
    }
}
