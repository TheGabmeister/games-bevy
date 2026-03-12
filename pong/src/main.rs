mod audio;
mod components;
mod state;
mod systems;

use bevy::prelude::*;
use components::{MatchConfig, Score, Winner};
use state::Phase;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pong (1972)".into(),
                resolution: (960, 540).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(MatchConfig::default())
        .insert_resource(Score::default())
        .insert_resource(Winner::default())
        .init_state::<Phase>()
        .add_plugins((audio::PongAudioPlugin, systems::PongSystemsPlugin))
        .run();
}
