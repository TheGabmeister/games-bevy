use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    StartScreen,
    WaveIntro,
    Playing,
    PlayerDeath,
    GameOver,
}
