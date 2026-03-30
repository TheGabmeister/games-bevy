use bevy::prelude::*;
use bevy::window::WindowResolution;

mod audio;
mod camera;
mod collision;
mod components;
mod constants;
mod enemy;
mod game_state;
mod input;
mod player;
mod rendering;
mod resources;
mod states;
mod ui;

fn main() {
    App::new()
        .insert_resource(ClearColor(rendering::WorldColor::Backdrop.color()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Zelda".into(),
                resolution: WindowResolution::new(constants::WINDOW_WIDTH, constants::WINDOW_HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            game_state::GameStatePlugin,
            camera::CameraPlugin,
            rendering::PrimitiveRenderingPlugin,
            input::InputPlugin,
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            collision::CollisionPlugin,
            ui::UiPlugin,
            audio::AudioPlugin,
        ))
        .run();
}
