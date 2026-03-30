use bevy::prelude::*;

use crate::{
    constants,
    input::InputActions,
    rendering::{circle_mesh, color_material, rectangle_mesh, WorldColor},
    states::AppState,
};

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_systems(OnEnter(AppState::Boot), advance_to_title)
            .add_systems(Update, start_from_title.run_if(in_state(AppState::Title)))
            .add_systems(Update, handle_playing_input.run_if(in_state(AppState::Playing)))
            .add_systems(
                Update,
                handle_paused_input.run_if(in_state(AppState::PausedInventory)),
            )
            .add_systems(OnEnter(AppState::Playing), spawn_playing_shell);
    }
}

fn advance_to_title(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Title);
}

fn start_from_title(actions: Res<InputActions>, mut next_state: ResMut<NextState<AppState>>) {
    if actions.confirm || actions.attack {
        next_state.set(AppState::Playing);
    }
}

fn handle_playing_input(
    actions: Res<InputActions>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if actions.pause {
        next_state.set(AppState::PausedInventory);
    } else if actions.cancel {
        next_state.set(AppState::Title);
    }
}

fn handle_paused_input(
    actions: Res<InputActions>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if actions.pause || actions.confirm || actions.cancel {
        next_state.set(AppState::Playing);
    }
}

fn spawn_playing_shell(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Name::new("HudPanel"),
        DespawnOnExit(AppState::Playing),
        rectangle_mesh(
            meshes.as_mut(),
            Vec2::new(constants::LOGICAL_SCREEN_WIDTH, constants::HUD_HEIGHT),
        ),
        color_material(materials.as_mut(), WorldColor::HudPanel),
        Transform::from_xyz(
            constants::HUD_CENTER.x,
            constants::HUD_CENTER.y,
            constants::render_layers::UI_BACKGROUND,
        ),
    ));

    commands.spawn((
        Name::new("RoomFloor"),
        DespawnOnExit(AppState::Playing),
        rectangle_mesh(
            meshes.as_mut(),
            Vec2::new(constants::ROOM_WIDTH, constants::ROOM_HEIGHT),
        ),
        color_material(materials.as_mut(), WorldColor::RoomFloor),
        Transform::from_xyz(
            constants::ROOM_ORIGIN.x,
            constants::ROOM_ORIGIN.y,
            constants::render_layers::FLOOR,
        ),
    ));

    for (name, anchor) in [
        ("NorthDoorAnchor", constants::NORTH_DOOR_ANCHOR),
        ("SouthDoorAnchor", constants::SOUTH_DOOR_ANCHOR),
        ("EastDoorAnchor", constants::EAST_DOOR_ANCHOR),
        ("WestDoorAnchor", constants::WEST_DOOR_ANCHOR),
    ] {
        commands.spawn((
            Name::new(name),
            DespawnOnExit(AppState::Playing),
            circle_mesh(meshes.as_mut(), 6.0),
            color_material(materials.as_mut(), WorldColor::Doorway),
            Transform::from_xyz(anchor.x, anchor.y, constants::render_layers::ENTITIES),
        ));
    }

    commands.spawn((
        Name::new("RoomOriginMarker"),
        DespawnOnExit(AppState::Playing),
        circle_mesh(meshes.as_mut(), 8.0),
        color_material(materials.as_mut(), WorldColor::Player),
        Transform::from_xyz(
            constants::ROOM_ORIGIN.x,
            constants::ROOM_ORIGIN.y,
            constants::render_layers::ENTITIES,
        ),
    ));
}
