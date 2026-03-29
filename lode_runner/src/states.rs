use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum AppState {
    #[default]
    StartScreen,
    Playing,
    LevelComplete,
    GameOver,
}
