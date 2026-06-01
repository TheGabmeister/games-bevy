use bevy::prelude::*;

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum AppState {
    #[default]
    StartScreen,
    Playing,
    GameOver,
}

/// The serve/play flow within `AppState::Playing`.
/// - `Ready`: the "ROUND n READY" intro is showing; the ball waits on the paddle.
/// - `Serving`: ball rests on the paddle, waiting for the launch input.
/// - `Running`: ball is live and gameplay physics runs.
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
pub enum PlayState {
    #[default]
    Ready,
    Serving,
    Running,
}
