use bevy::prelude::*;

#[derive(States, Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
pub enum AppState {
    #[default]
    StartScreen,
    Playing,
    GameOver,
}

#[derive(SubStates, Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
pub enum PlayState {
    #[default]
    WaveIntro,
    WaveActive,
    WaveClear,
    PlayerDeath,
    Paused,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Movement,
    Confinement,
    Combat,
    Resolution,
}
