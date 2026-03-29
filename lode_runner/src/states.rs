use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum AppState {
    #[default]
    StartScreen,
    Playing,
    /// Transient state: exits Playing (cleanup), then immediately re-enters.
    Restarting,
    LevelComplete,
    GameOver,
}

#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[source(AppState = AppState::Playing)]
pub enum PlayState {
    #[default]
    Running,
    Paused,
    Dying,
}
