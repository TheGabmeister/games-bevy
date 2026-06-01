use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{Ball, Velocity};
use crate::constants::*;
use crate::input::InputActions;
use crate::resources::{Lives, Round, Score};
use crate::states::{AppState, PlayState};

/// Fired when the ball falls past the open bottom. An observer spends a life and routes
/// to a re-serve (lives remain) or to Game Over (lives exhausted).
#[derive(Event)]
pub struct BallLost;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReadyTimer>()
            .add_observer(on_ball_lost)
            .add_systems(OnEnter(AppState::Playing), start_run)
            .add_systems(OnEnter(AppState::GameOver), play_game_over_music)
            .add_systems(OnEnter(PlayState::Ready), (reset_ready_timer, stick_ball))
            .add_systems(OnEnter(PlayState::Serving), stick_ball)
            .add_systems(
                Update,
                (
                    start_game.run_if(in_state(AppState::StartScreen)),
                    restart_game.run_if(in_state(AppState::GameOver)),
                    advance_ready.run_if(in_state(PlayState::Ready)),
                ),
            );
    }
}

/// Counts down the "ROUND n READY" intro before the ball can be served.
#[derive(Resource)]
struct ReadyTimer(Timer);

impl Default for ReadyTimer {
    fn default() -> Self {
        ReadyTimer(Timer::from_seconds(READY_DURATION, TimerMode::Once))
    }
}

/// Fresh run: reset lives, score, and round before the first round spawns.
fn start_run(mut lives: ResMut<Lives>, mut score: ResMut<Score>, mut round: ResMut<Round>) {
    lives.0 = LIVES_START;
    score.current = 0;
    round.0 = 1;
}

fn reset_ready_timer(mut timer: ResMut<ReadyTimer>) {
    timer.0.reset();
}

/// Parks every ball back on the paddle, stationary. `ball_follow_paddle` then keeps it
/// glued to the Vaus until launch.
fn stick_ball(mut balls: Query<(&mut Ball, &mut Velocity)>) {
    for (mut ball, mut velocity) in &mut balls {
        ball.stuck = true;
        velocity.0 = Vec2::ZERO;
    }
}

fn advance_ready(
    time: Res<Time>,
    mut timer: ResMut<ReadyTimer>,
    mut next: ResMut<NextState<PlayState>>,
) {
    if timer.0.tick(time.delta()).is_finished() {
        next.set(PlayState::Serving);
    }
}

fn start_game(input: Res<InputActions>, mut next: ResMut<NextState<AppState>>) {
    if input.launch {
        next.set(AppState::Playing);
    }
}

fn restart_game(input: Res<InputActions>, mut next: ResMut<NextState<AppState>>) {
    if input.launch {
        next.set(AppState::StartScreen);
    }
}

fn play_game_over_music(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        AudioPlayer(assets.music.game_over.clone()),
        PlaybackSettings::DESPAWN,
    ));
}

fn on_ball_lost(
    _trigger: On<BallLost>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut lives: ResMut<Lives>,
    mut next_app: ResMut<NextState<AppState>>,
    mut next_play: ResMut<NextState<PlayState>>,
) {
    commands.spawn((
        AudioPlayer(assets.sfx.ball_lost.clone()),
        PlaybackSettings::DESPAWN,
    ));
    lives.0 = lives.0.saturating_sub(1);
    if lives.0 == 0 {
        next_app.set(AppState::GameOver);
    } else {
        next_play.set(PlayState::Serving);
    }
}
