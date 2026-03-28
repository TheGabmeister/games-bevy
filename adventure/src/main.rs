use bevy::prelude::*;

mod components;
mod world;
mod setup;
mod rooms;
mod player;
mod enemies;
mod ui;

use world::{WorldMap, CurrentRoom, PlayerInventory, RoomWalls, DeadDragonMaterial};
use components::WallBypass;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    Playing,
    Swallowed,
    GameOver,
    Win,
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
        .add_systems(Startup, spawn_camera)
        .insert_resource(WorldMap::new())
        .insert_resource(CurrentRoom(1))
        .insert_resource(PlayerInventory { item: None })
        .insert_resource(RoomWalls::default())
        .insert_resource(WallBypass::default())
        .insert_resource(ClearColor(Color::srgb(0.35, 0.35, 0.35)))
        // Title state
        .add_systems(OnEnter(AppState::Title), (despawn_game_world, ui::spawn_title).chain())
        .add_systems(OnExit(AppState::Title), ui::despawn_title)
        .add_systems(Update, title_to_playing.run_if(in_state(AppState::Title)))
        // Playing state
        .add_systems(OnEnter(AppState::Playing), (
            init_game_resources,
            setup::spawn_world,
            rooms::spawn_room_walls,
            ui::spawn_ui,
        ).chain())
        .add_systems(OnExit(AppState::Playing), ui::despawn_ui)
        .add_systems(Update, (
            player::compute_wall_bypass.before(player::player_movement),
            player::player_movement,
            player::item_pickup,
            player::item_drop,
            player::carry_item_follow,
            player::gate_interaction,
            player::magnet_pull,
            rooms::room_transition,
            rooms::spawn_room_walls,
            rooms::update_background_color,
            rooms::update_visibility,
            enemies::dragon_ai,
            enemies::update_dragon_heads,
            enemies::dragon_collision,
            enemies::sword_combat,
            enemies::bat_ai,
            enemies::bat_revive_dragons,
            player::check_win,
            ui::update_ui,
        ).run_if(in_state(AppState::Playing)))
        // Swallowed state (dragon eating animation)
        .add_systems(Update, enemies::swallow_animation.run_if(in_state(AppState::Swallowed)))
        // GameOver state
        .add_systems(OnEnter(AppState::GameOver), ui::spawn_game_over)
        .add_systems(OnExit(AppState::GameOver), ui::despawn_game_over)
        .add_systems(Update, ui::restart_game.run_if(in_state(AppState::GameOver)))
        // Win state
        .add_systems(OnEnter(AppState::Win), ui::spawn_win_screen)
        .add_systems(OnExit(AppState::Win), ui::despawn_win_screen)
        .add_systems(Update, ui::restart_game.run_if(in_state(AppState::Win)))
        .run();
}

fn title_to_playing(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

/// Reset game resources before spawning a new game.
fn init_game_resources(
    mut current_room: ResMut<CurrentRoom>,
    mut inventory: ResMut<PlayerInventory>,
    mut room_walls: ResMut<RoomWalls>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    current_room.0 = 1;
    inventory.item = None;
    room_walls.0.clear();

    // Create dead dragon material
    let dead_mat = materials.add(Color::srgb(0.4, 0.4, 0.4));
    commands.insert_resource(DeadDragonMaterial(dead_mat));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Despawn all game world entities when returning to title.
fn despawn_game_world(
    mut commands: Commands,
    entities: Query<Entity, Or<(
        With<components::Player>,
        With<components::Item>,
        With<components::Dragon>,
        With<components::Bat>,
        With<components::Gate>,
        With<components::RoomWallMarker>,
    )>>,
) {
    for e in entities.iter() {
        commands.entity(e).despawn();
    }
}
