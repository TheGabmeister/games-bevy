use bevy::prelude::*;

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum AppState {
    #[default]
    StartScreen,
    LevelTransition,
    Playing,
    GameOver,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
pub enum PlayState {
    #[default]
    Running,
    Dying,
    Paused,
    LevelComplete,
    Growing,
    Warping,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    Input,
    Physics,
    Camera,
    Late,
}
