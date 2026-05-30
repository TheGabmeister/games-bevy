use bevy::prelude::*;
use bevy::window::WindowResolution;

mod audio;
mod ball;
mod collision;
mod components;
mod constants;
mod enemy;
mod input;
mod player;
mod resources;
mod states;
mod ui;

use constants::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                title: "Arkanoid".into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            input::InputPlugin,
            player::PlayerPlugin,
            ball::BallPlugin,
            enemy::EnemyPlugin,
            collision::CollisionPlugin,
            ui::UiPlugin,
            audio::AudioPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    // Playfield border frame (full-screen sprite, transparent interior).
    commands.spawn((
        Sprite::from_image(asset_server.load("sprites/playfield/border-frame.png")),
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND),
    ));
}
