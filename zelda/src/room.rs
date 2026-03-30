use bevy::prelude::*;
use bevy::ecs::system::SystemParam;

use crate::{
    components::{Door, RoomEntity, SolidBody, StaticBlocker, Wall},
    constants,
    input::InputActions,
    resources::{CurrentRoom, PersistentRoomKey, RoomId, RoomPersistence, RoomPersistenceCategory},
    rendering::{circle_mesh, color_material, rectangle_mesh, WorldColor},
    states::AppState,
};

const WALL_THICKNESS: f32 = 16.0;
const DOOR_OPENING: f32 = 32.0;
const OBSTACLE_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const PICKUP_RADIUS: f32 = 7.0;
const SECRET_TRIGGER_RADIUS: f32 = 14.0;

pub struct RoomPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RoomSet {
    RequestLoad,
    Load,
    Interact,
}

#[derive(Clone, Copy, Debug, Message)]
pub struct LoadRoomMessage {
    pub room: RoomId,
    pub player_spawn: Vec2,
}

#[derive(Clone, Copy, Debug, Message)]
pub struct RoomLoadedMessage {
    pub room: RoomId,
    pub player_spawn: Vec2,
}

#[derive(Component)]
pub struct UniquePickup;

#[derive(Component)]
pub struct TemporaryPickup;

#[derive(Component)]
pub struct SecretTrigger {
    pub key: PersistentRoomKey,
    pub reveal_at: Vec2,
}

#[derive(Component, Clone, Copy)]
pub struct PersistentRoomEntity {
    pub key: PersistentRoomKey,
    pub category: RoomPersistenceCategory,
}

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentRoom>()
            .init_resource::<RoomPersistence>()
            .add_message::<LoadRoomMessage>()
            .add_message::<RoomLoadedMessage>()
            .configure_sets(
                Update,
                (RoomSet::RequestLoad, RoomSet::Load, RoomSet::Interact)
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnEnter(AppState::Playing), ensure_room_loaded)
            .add_systems(OnEnter(AppState::Title), cleanup_room_entities)
            .add_systems(
                Update,
                request_room_reload.in_set(RoomSet::RequestLoad),
            )
            .add_systems(Update, process_room_loads.in_set(RoomSet::Load))
            .add_systems(
                Update,
                (collect_unique_pickups, collect_temporary_pickups, reveal_secret_triggers)
                    .in_set(RoomSet::Interact),
            );
    }
}

#[derive(SystemParam)]
struct RoomLoadContext<'w, 's> {
    current_room: ResMut<'w, CurrentRoom>,
    persistence: Res<'w, RoomPersistence>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    room_entities: Query<'w, 's, Entity, With<RoomEntity>>,
}

fn ensure_room_loaded(
    current_room: Res<CurrentRoom>,
    room_entities: Query<Entity, With<RoomEntity>>,
    mut loads: MessageWriter<LoadRoomMessage>,
) {
    if room_entities.is_empty() {
        loads.write(LoadRoomMessage {
            room: current_room.id,
            player_spawn: constants::WEST_ENTRY_OFFSET,
        });
    }
}

fn cleanup_room_entities(mut commands: Commands, room_entities: Query<Entity, With<RoomEntity>>) {
    for entity in &room_entities {
        commands.entity(entity).despawn();
    }
}

fn request_room_reload(
    actions: Res<InputActions>,
    current_room: Res<CurrentRoom>,
    mut loads: MessageWriter<LoadRoomMessage>,
) {
    if actions.confirm {
        loads.write(LoadRoomMessage {
            room: current_room.id,
            player_spawn: constants::WEST_ENTRY_OFFSET,
        });
    }
}

fn process_room_loads(
    mut commands: Commands,
    mut loads: MessageReader<LoadRoomMessage>,
    mut room_loaded: MessageWriter<RoomLoadedMessage>,
    mut context: RoomLoadContext,
) {
    for message in loads.read() {
        for entity in &context.room_entities {
            commands.entity(entity).despawn();
        }

        context.current_room.id = message.room;
        spawn_room(
            &mut commands,
            context.meshes.as_mut(),
            context.materials.as_mut(),
            &context.persistence,
            message.room,
        );
        room_loaded.write(RoomLoadedMessage {
            room: message.room,
            player_spawn: message.player_spawn,
        });
    }
}

fn spawn_room(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    persistence: &RoomPersistence,
    room: RoomId,
) {
    commands.spawn((
        Name::new("HudPanel"),
        RoomEntity,
        rectangle_mesh(
            meshes,
            Vec2::new(constants::LOGICAL_SCREEN_WIDTH, constants::HUD_HEIGHT),
        ),
        color_material(materials, WorldColor::HudPanel),
        Transform::from_xyz(
            constants::HUD_CENTER.x,
            constants::HUD_CENTER.y,
            constants::render_layers::UI_BACKGROUND,
        ),
    ));

    commands.spawn((
        Name::new("RoomFloor"),
        RoomEntity,
        rectangle_mesh(meshes, Vec2::new(constants::ROOM_WIDTH, constants::ROOM_HEIGHT)),
        color_material(materials, WorldColor::RoomFloor),
        Transform::from_xyz(
            constants::ROOM_ORIGIN.x,
            constants::ROOM_ORIGIN.y,
            constants::render_layers::FLOOR,
        ),
    ));

    spawn_perimeter_walls(commands, meshes, materials);
    spawn_door_markers(commands, meshes, materials);
    spawn_test_obstacles(commands, meshes, materials);
    spawn_test_pickups(commands, meshes, materials, persistence, room);
    spawn_secret_entities(commands, meshes, materials, persistence, room);
}

fn spawn_perimeter_walls(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let horizontal_segment_width = (constants::ROOM_WIDTH - DOOR_OPENING) * 0.5;
    let vertical_segment_height = (constants::ROOM_HEIGHT - DOOR_OPENING) * 0.5;
    let top_y = constants::ROOM_ORIGIN.y + constants::ROOM_HALF_HEIGHT - WALL_THICKNESS * 0.5;
    let bottom_y = constants::ROOM_ORIGIN.y - constants::ROOM_HALF_HEIGHT + WALL_THICKNESS * 0.5;
    let left_x = constants::ROOM_ORIGIN.x - constants::ROOM_HALF_WIDTH + WALL_THICKNESS * 0.5;
    let right_x = constants::ROOM_ORIGIN.x + constants::ROOM_HALF_WIDTH - WALL_THICKNESS * 0.5;
    let horizontal_offset = DOOR_OPENING * 0.5 + horizontal_segment_width * 0.5;
    let vertical_offset = DOOR_OPENING * 0.5 + vertical_segment_height * 0.5;

    for (name, size, center) in [
        (
            "NorthWallWest",
            Vec2::new(horizontal_segment_width, WALL_THICKNESS),
            Vec2::new(-horizontal_offset, top_y),
        ),
        (
            "NorthWallEast",
            Vec2::new(horizontal_segment_width, WALL_THICKNESS),
            Vec2::new(horizontal_offset, top_y),
        ),
        (
            "SouthWallWest",
            Vec2::new(horizontal_segment_width, WALL_THICKNESS),
            Vec2::new(-horizontal_offset, bottom_y),
        ),
        (
            "SouthWallEast",
            Vec2::new(horizontal_segment_width, WALL_THICKNESS),
            Vec2::new(horizontal_offset, bottom_y),
        ),
        (
            "WestWallTop",
            Vec2::new(WALL_THICKNESS, vertical_segment_height),
            Vec2::new(left_x, constants::ROOM_ORIGIN.y + vertical_offset),
        ),
        (
            "WestWallBottom",
            Vec2::new(WALL_THICKNESS, vertical_segment_height),
            Vec2::new(left_x, constants::ROOM_ORIGIN.y - vertical_offset),
        ),
        (
            "EastWallTop",
            Vec2::new(WALL_THICKNESS, vertical_segment_height),
            Vec2::new(right_x, constants::ROOM_ORIGIN.y + vertical_offset),
        ),
        (
            "EastWallBottom",
            Vec2::new(WALL_THICKNESS, vertical_segment_height),
            Vec2::new(right_x, constants::ROOM_ORIGIN.y - vertical_offset),
        ),
    ] {
        commands.spawn((
            Name::new(name),
            RoomEntity,
            Wall,
            StaticBlocker,
            SolidBody {
                half_size: size * 0.5,
            },
            rectangle_mesh(meshes, size),
            color_material(materials, WorldColor::Doorway),
            Transform::from_xyz(center.x, center.y, constants::render_layers::WALLS),
        ));
    }
}

fn spawn_door_markers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    for (name, anchor) in [
        ("NorthDoor", constants::NORTH_DOOR_ANCHOR),
        ("SouthDoor", constants::SOUTH_DOOR_ANCHOR),
        ("EastDoor", constants::EAST_DOOR_ANCHOR),
        ("WestDoor", constants::WEST_DOOR_ANCHOR),
    ] {
        commands.spawn((
            Name::new(name),
            RoomEntity,
            Door,
            circle_mesh(meshes, 6.0),
            color_material(materials, WorldColor::Accent),
            Transform::from_xyz(anchor.x, anchor.y, constants::render_layers::ENTITIES),
        ));
    }
}

fn spawn_test_obstacles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    for (name, position) in [
        ("RockA", Vec2::new(-48.0, constants::ROOM_ORIGIN.y + 10.0)),
        ("RockB", Vec2::new(36.0, constants::ROOM_ORIGIN.y - 20.0)),
    ] {
        commands.spawn((
            Name::new(name),
            RoomEntity,
            Wall,
            StaticBlocker,
            SolidBody {
                half_size: OBSTACLE_SIZE * 0.5,
            },
            rectangle_mesh(meshes, OBSTACLE_SIZE),
            color_material(materials, WorldColor::Hazard),
            Transform::from_xyz(position.x, position.y, constants::render_layers::WALLS),
        ));
    }
}

fn spawn_test_pickups(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    persistence: &RoomPersistence,
    room: RoomId,
) {
    let unique_key = PersistentRoomKey {
        room,
        key: "starter_rupee",
    };
    if !persistence.contains(RoomPersistenceCategory::UniquePickup, unique_key) {
        commands.spawn((
            Name::new("UniquePickup"),
            RoomEntity,
            UniquePickup,
            PersistentRoomEntity {
                key: unique_key,
                category: RoomPersistenceCategory::UniquePickup,
            },
            circle_mesh(meshes, PICKUP_RADIUS),
            color_material(materials, WorldColor::Pickup),
            Transform::from_xyz(-80.0, constants::ROOM_ORIGIN.y - 16.0, constants::render_layers::PICKUPS),
        ));
    }

    commands.spawn((
        Name::new("TemporaryPickup"),
        RoomEntity,
        TemporaryPickup,
        PersistentRoomEntity {
            key: PersistentRoomKey {
                room,
                key: "temporary_heart",
            },
            category: RoomPersistenceCategory::ResetOnRoomLoad,
        },
        circle_mesh(meshes, PICKUP_RADIUS),
        color_material(materials, WorldColor::Hazard),
        Transform::from_xyz(72.0, constants::ROOM_ORIGIN.y - 10.0, constants::render_layers::PICKUPS),
    ));
}

fn spawn_secret_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    persistence: &RoomPersistence,
    room: RoomId,
) {
    let secret_key = PersistentRoomKey {
        room,
        key: "hidden_stair",
    };

    commands.spawn((
        Name::new("SecretBush"),
        RoomEntity,
        SecretTrigger {
            key: secret_key,
            reveal_at: Vec2::new(88.0, constants::ROOM_ORIGIN.y + 42.0),
        },
        circle_mesh(meshes, SECRET_TRIGGER_RADIUS),
        color_material(materials, WorldColor::Hazard),
        Transform::from_xyz(
            -88.0,
            constants::ROOM_ORIGIN.y + 44.0,
            constants::render_layers::ENTITIES,
        ),
    ));

    if persistence.contains(RoomPersistenceCategory::Secret, secret_key) {
        spawn_revealed_secret(commands, meshes, materials, secret_key, Vec2::new(88.0, constants::ROOM_ORIGIN.y + 42.0));
    }
}

fn collect_unique_pickups(
    mut commands: Commands,
    mut persistence: ResMut<RoomPersistence>,
    player: Query<&Transform, With<crate::components::Player>>,
    pickups: Query<(Entity, &Transform, &PersistentRoomEntity), With<UniquePickup>>,
) {
    let Some(player_transform) = player.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, persistent) in &pickups {
        if player_pos.distance(transform.translation.truncate()) <= PICKUP_RADIUS + 8.0 {
            persistence.record(persistent.category, persistent.key);
            commands.entity(entity).despawn();
        }
    }
}

fn collect_temporary_pickups(
    mut commands: Commands,
    player: Query<&Transform, With<crate::components::Player>>,
    pickups: Query<(Entity, &Transform), (With<TemporaryPickup>, With<RoomEntity>)>,
) {
    let Some(player_transform) = player.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform) in &pickups {
        if player_pos.distance(transform.translation.truncate()) <= PICKUP_RADIUS + 8.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn reveal_secret_triggers(
    mut commands: Commands,
    actions: Res<InputActions>,
    mut persistence: ResMut<RoomPersistence>,
    player: Query<&Transform, With<crate::components::Player>>,
    triggers: Query<(Entity, &Transform, &SecretTrigger)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !actions.attack {
        return;
    }

    let Some(player_transform) = player.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (_entity, transform, trigger) in &triggers {
        if persistence.contains(RoomPersistenceCategory::Secret, trigger.key) {
            continue;
        }

        if player_pos.distance(transform.translation.truncate()) <= SECRET_TRIGGER_RADIUS + 14.0 {
            persistence.record(RoomPersistenceCategory::Secret, trigger.key);
            spawn_revealed_secret(
                &mut commands,
                meshes.as_mut(),
                materials.as_mut(),
                trigger.key,
                trigger.reveal_at,
            );
        }
    }
}

fn spawn_revealed_secret(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    key: PersistentRoomKey,
    position: Vec2,
) {
    commands.spawn((
        Name::new("RevealedSecret"),
        RoomEntity,
        PersistentRoomEntity {
            key,
            category: RoomPersistenceCategory::Secret,
        },
        rectangle_mesh(meshes, Vec2::new(20.0, 14.0)),
        color_material(materials, WorldColor::Accent),
        Transform::from_xyz(position.x, position.y, constants::render_layers::PICKUPS),
    ));
}
