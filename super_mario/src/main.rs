use bevy::prelude::*;

mod components;
mod constants;

use constants::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(
                    WINDOW_WIDTH as u32,
                    WINDOW_HEIGHT as u32,
                ),
                title: "Super Mario Bros".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.36, 0.53, 0.95)))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = CAMERA_SCALE;

    commands.spawn((
        Camera2d,
        Projection::from(projection),
    ));
}
