mod game;

use bevy::{
    prelude::*,
    window::{WindowPlugin, WindowResolution},
};

use game::PacmanPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pac-Man".into(),
                resolution: WindowResolution::new(900, 860),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PacmanPlugin)
        .run();
}
