#![allow(clippy::type_complexity)]

use bevy::prelude::*;

mod components;
mod enemies;
mod player;
mod rooms;
mod setup;
mod ui;
mod world;

use components::WallBypass;
use world::{CurrentRoom, PlayerInventory, RoomWalls, WorldMap};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    Playing,
    Swallowed,
    GameOver,
    Win,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayingEnterSet {
    Reset,
    SpawnWorld,
    RoomState,
    Ui,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayingSet {
    Prepare,
    Movement,
    Interaction,
    Room,
    Enemies,
    WinCheck,
    Presentation,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Adventure".to_string(),
                resolution: UVec2::new(800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .init_resource::<WorldMap>()
        .init_resource::<CurrentRoom>()
        .init_resource::<PlayerInventory>()
        .init_resource::<RoomWalls>()
        .init_resource::<WallBypass>()
        .insert_resource(ClearColor(Color::srgb(0.35, 0.35, 0.35)))
        .configure_sets(
            OnEnter(AppState::Playing),
            (
                PlayingEnterSet::Reset,
                PlayingEnterSet::SpawnWorld,
                PlayingEnterSet::RoomState,
                PlayingEnterSet::Ui,
            )
                .chain(),
        )
        .configure_sets(
            Update,
            (
                PlayingSet::Prepare,
                PlayingSet::Movement,
                PlayingSet::Interaction,
                PlayingSet::Room,
                PlayingSet::Enemies,
                PlayingSet::WinCheck,
                PlayingSet::Presentation,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .add_plugins((
            setup::SetupPlugin,
            rooms::RoomsPlugin,
            player::PlayerPlugin,
            enemies::EnemiesPlugin,
            ui::UiPlugin,
        ))
        .run();
}
