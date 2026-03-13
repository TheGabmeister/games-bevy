mod game;

use bevy::{audio::AudioPlugin, prelude::*, window::WindowResolution};
use game::{SpaceInvadersPlugin, WINDOW_HEIGHT, WINDOW_WIDTH, background_color};

fn main() {
    App::new()
        .insert_resource(ClearColor(background_color()))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Space Invaders".into(),
                        resolution: WindowResolution::new(
                            WINDOW_WIDTH as u32,
                            WINDOW_HEIGHT as u32,
                        ),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<AudioPlugin>(),
        )
        .add_systems(Startup, setup_camera)
        .add_plugins(SpaceInvadersPlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
