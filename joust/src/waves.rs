use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::*;

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), reset_game_state)
            .add_systems(OnEnter(PlayState::WaveIntro), setup_wave_intro)
            .add_systems(
                Update,
                wave_intro_tick.run_if(in_state(PlayState::WaveIntro)),
            )
            .add_systems(
                Update,
                check_wave_clear
                    .in_set(GameSet::Progression)
                    .run_if(in_state(PlayState::WaveActive)),
            )
            .add_systems(OnEnter(PlayState::WaveClear), setup_wave_clear)
            .add_systems(
                Update,
                wave_clear_tick.run_if(in_state(PlayState::WaveClear)),
            );
    }
}

fn reset_game_state(mut commands: Commands, mut game_state: ResMut<GameState>) {
    let high = game_state.high_score.max(game_state.scores[0]).max(game_state.scores[1]);
    *game_state = GameState {
        high_score: high,
        ..default()
    };
    commands.insert_resource(RespawnTimers::default());
}

fn setup_wave_intro(
    mut commands: Commands,
    game_state: Res<GameState>,
) {
    commands.insert_resource(WaveTimer(Timer::from_seconds(
        WAVE_INTRO_DURATION,
        TimerMode::Once,
    )));

    // Wave announcement text
    commands.spawn((
        Text2d::new(format!("WAVE {}", game_state.wave)),
        TextFont {
            font_size: 64.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.3)),
        Transform::from_xyz(0.0, 100.0, Z_WAVE_TEXT),
        DespawnOnExit(PlayState::WaveIntro),
    ));
}

fn wave_intro_tick(
    time: Res<Time>,
    mut timer: ResMut<WaveTimer>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next_state.set(PlayState::WaveActive);
    }
}

fn check_wave_clear(
    enemies: Query<(), With<Enemy>>,
    eggs: Query<(), With<Egg>>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    if enemies.is_empty() && eggs.is_empty() {
        next_state.set(PlayState::WaveClear);
    }
}

fn setup_wave_clear(mut commands: Commands) {
    commands.insert_resource(WaveTimer(Timer::from_seconds(
        WAVE_CLEAR_DURATION,
        TimerMode::Once,
    )));
}

fn wave_clear_tick(
    time: Res<Time>,
    mut timer: ResMut<WaveTimer>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        game_state.wave += 1;
        next_state.set(PlayState::WaveIntro);
    }
}
