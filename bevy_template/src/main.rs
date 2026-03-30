use bevy::prelude::*;

mod audio;
mod collision;
mod enemy;
mod input;
mod player;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            input::InputPlugin,
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            collision::CollisionPlugin,
            ui::UiPlugin,
            audio::AudioPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text::new("Hello, World!"),
        Node {
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        },
    ));
}
