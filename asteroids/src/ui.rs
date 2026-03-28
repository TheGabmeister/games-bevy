use bevy::prelude::*;

use crate::components::*;
use crate::resources::{GameAssets, GameData};
use crate::spawn::{spawn_ship, spawn_wave};
use crate::state::AppState;
use crate::{GameSet, STARTING_LIVES};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ui_setup)
            .add_systems(OnEnter(AppState::GameOver), game_over_setup_system)
            .add_systems(
                Update,
                hud_update_system
                    .in_set(GameSet::Cleanup)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                game_over_input_system.run_if(in_state(AppState::GameOver)),
            );
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Spawns the score, lives, and game-over overlay UI entities.
fn ui_setup(mut commands: Commands) {
    // Score — top-left
    commands.spawn((
        ScoreText,
        Text::new("Score: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Lives — top-right
    commands.spawn((
        LivesText,
        Text::new(format!("Lives: {}", STARTING_LIVES)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));

    // Game-over overlay — hidden until the GameOver state is entered
    commands.spawn((
        GameOverText,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(25.0),
            ..default()
        },
        TextFont {
            font_size: 32.0,
            ..default()
        },
        Visibility::Hidden,
    ));
}

/// Refreshes the score and lives HUD text whenever GameData changes.
fn hud_update_system(
    game_data: Res<GameData>,
    // The Without<LivesText> / Without<ScoreText> guards prove to Bevy that
    // these two queries with &mut Text are mutually exclusive (no overlap).
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<LivesText>)>,
    mut lives_q: Query<&mut Text, (With<LivesText>, Without<ScoreText>)>,
) {
    if !game_data.is_changed() {
        return;
    }
    if let Ok(mut text) = score_q.single_mut() {
        *text = Text::new(format!("Score: {}", game_data.score));
    }
    if let Ok(mut text) = lives_q.single_mut() {
        *text = Text::new(format!("Lives: {}", game_data.lives));
    }
}

/// Called once when entering the GameOver state.
/// Cleans up any remaining bullets/asteroids and shows the overlay.
fn game_over_setup_system(
    game_data: Res<GameData>,
    mut overlay_q: Query<(&mut Visibility, &mut Text), With<GameOverText>>,
    to_despawn: Query<Entity, Or<(With<Bullet>, With<Asteroid>)>>,
    mut commands: Commands,
) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
    if let Ok((mut vis, mut text)) = overlay_q.single_mut() {
        *vis = Visibility::Visible;
        *text = Text::new(format!(
            "GAME OVER\n\nFinal Score: {}\n\nPress R to restart",
            game_data.score
        ));
    }
}

/// Waits for the player to press R, then resets everything and returns to Playing.
fn game_over_input_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<AppState>>,
    mut overlay_q: Query<&mut Visibility, With<GameOverText>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        *game_data = GameData::default();

        if let Ok(mut vis) = overlay_q.single_mut() {
            *vis = Visibility::Hidden;
        }

        spawn_ship(&mut commands, &assets);
        spawn_wave(&mut commands, &assets, 1);

        next_state.set(AppState::Playing);
    }
}
