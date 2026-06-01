use bevy::prelude::*;
use bevy::window::WindowResolution;

mod assets;
mod audio;
mod ball;
mod bricks;
mod collision;
mod components;
mod constants;
mod enemy;
mod input;
mod player;
mod resources;
mod states;
mod ui;

use assets::GameAssets;
use constants::*;
use resources::{Round, Score};
use states::AppState;

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
        .init_resource::<GameAssets>()
        .init_resource::<Score>()
        .init_resource::<Round>()
        // Start directly in Playing for now; Phase 3 adds the title screen and the
        // StartScreen → Playing transition.
        .insert_state(AppState::Playing)
        .add_plugins((
            input::InputPlugin,
            player::PlayerPlugin,
            ball::BallPlugin,
            bricks::BrickPlugin,
            enemy::EnemyPlugin,
            collision::CollisionPlugin,
            ui::UiPlugin,
            audio::AudioPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn(Camera2d);
    // Playfield border frame (full-screen sprite, transparent interior).
    commands.spawn((
        Sprite::from_image(assets.sprites.border_frame.clone()),
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND),
    ));
}
