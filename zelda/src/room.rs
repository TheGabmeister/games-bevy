use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::{
    collision::CollisionSet,
    components::{
        Damage, Door, Enemy, Health, Hitbox, Hurtbox, Label, PickupKind, Player, RoomEntity,
        SolidBody, StaticBlocker, Wall,
    },
    constants,
    input::InputActions,
    items::{self, ItemTable},
    resources::{
        CurrentRoom, ExitDirection, Inventory, PersistentRoomKey, PlayerVitals, RoomId,
        RoomPersistence, RoomPersistenceCategory, RoomTransitionState,
    },
    rendering::{circle_mesh, color_material, rectangle_mesh, WorldColor},
    states::AppState,
};

const WALL_THICKNESS: f32 = 16.0;
const DOOR_OPENING: f32 = 32.0;
const OBSTACLE_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const PICKUP_RADIUS: f32 = 7.0;
const SECRET_TRIGGER_RADIUS: f32 = 14.0;
const EDGE_EXIT_PADDING: f32 = 6.0;

pub struct RoomPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RoomSet {
    TransitionTick,
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

#[derive(SystemParam)]
struct RoomLoadContext<'w, 's> {
    current_room: ResMut<'w, CurrentRoom>,
    persistence: Res<'w, RoomPersistence>,
    item_table: Res<'w, ItemTable>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    room_entities: Query<'w, 's, Entity, With<RoomEntity>>,
}

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentRoom>()
            .init_resource::<RoomPersistence>()
            .init_resource::<RoomTransitionState>()
            .add_message::<LoadRoomMessage>()
            .add_message::<RoomLoadedMessage>()
            .configure_sets(
                Update,
                (
                    RoomSet::TransitionTick,
                    RoomSet::RequestLoad,
                    RoomSet::Load,
                    RoomSet::Interact,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnEnter(AppState::Playing), ensure_room_loaded)
            .add_systems(OnEnter(AppState::Title), cleanup_room_entities)
            .add_systems(OnEnter(AppState::GameOver), cleanup_room_entities)
            .add_systems(Update, tick_room_transition_state.in_set(RoomSet::TransitionTick))
            .add_systems(
                Update,
                (request_room_reload, request_screen_edge_transition)
                    .in_set(RoomSet::RequestLoad)
                    .after(CollisionSet::Resolve),
            )
            .add_systems(Update, process_room_loads.in_set(RoomSet::Load))
            .add_systems(
                Update,
                (collect_unique_pickups, collect_temporary_pickups, reveal_secret_triggers)
                    .in_set(RoomSet::Interact),
            );
    }
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

fn cleanup_room_entities(
    mut commands: Commands,
    room_entities: Query<Entity, With<RoomEntity>>,
    mut transition: ResMut<RoomTransitionState>,
) {
    transition.locked = false;
    transition.direction = None;

    for entity in &room_entities {
        commands.entity(entity).despawn();
    }
}

fn tick_room_transition_state(time: Res<Time>, mut transition: ResMut<RoomTransitionState>) {
    if !transition.locked {
        return;
    }

    transition.timer.tick(time.delta());
    if transition.timer.is_finished() {
        transition.locked = false;
        transition.direction = None;
    }
}

fn request_room_reload(
    actions: Res<InputActions>,
    current_room: Res<CurrentRoom>,
    transition: Res<RoomTransitionState>,
    mut loads: MessageWriter<LoadRoomMessage>,
) {
    if transition.locked {
        return;
    }

    if actions.confirm {
        loads.write(LoadRoomMessage {
            room: current_room.id,
            player_spawn: constants::WEST_ENTRY_OFFSET,
        });
    }
}

fn request_screen_edge_transition(
    current_room: Res<CurrentRoom>,
    mut transition: ResMut<RoomTransitionState>,
    player: Query<&Transform, With<Player>>,
    mut loads: MessageWriter<LoadRoomMessage>,
) {
    if transition.locked {
        return;
    }

    let Ok(player_transform) = player.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let Some(direction) = detect_exit_direction(player_pos) else {
        return;
    };
    let Some(target_room) = adjacent_room(current_room.id, direction) else {
        return;
    };

    transition.locked = true;
    transition.direction = Some(direction);
    transition.timer.reset();

    loads.write(LoadRoomMessage {
        room: target_room,
        player_spawn: direction.target_spawn_offset(),
    });
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
            &context.item_table,
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
    item_table: &ItemTable,
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
        color_material(materials, room_floor_color(room)),
        Transform::from_xyz(
            constants::ROOM_ORIGIN.x,
            constants::ROOM_ORIGIN.y,
            constants::render_layers::FLOOR,
        ),
    ));

    spawn_perimeter_walls(commands, meshes, materials, room);
    spawn_door_markers(commands, meshes, materials, room);
    spawn_test_obstacles(commands, meshes, materials, room);
    spawn_test_enemies(commands, meshes, materials, room);
    spawn_test_pickups(commands, meshes, materials, persistence, item_table, room);
    spawn_secret_entities(commands, meshes, materials, persistence, room);
}

fn spawn_perimeter_walls(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    room: RoomId,
) {
    for direction in [
        ExitDirection::North,
        ExitDirection::South,
        ExitDirection::East,
        ExitDirection::West,
    ] {
        if adjacent_room(room, direction).is_some() {
            spawn_open_side(commands, meshes, materials, direction);
        } else {
            spawn_closed_side(commands, meshes, materials, direction);
        }
    }
}

fn spawn_open_side(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    direction: ExitDirection,
) {
    let horizontal_segment_width = (constants::ROOM_WIDTH - DOOR_OPENING) * 0.5;
    let vertical_segment_height = (constants::ROOM_HEIGHT - DOOR_OPENING) * 0.5;
    let top_y = constants::ROOM_ORIGIN.y + constants::ROOM_HALF_HEIGHT - WALL_THICKNESS * 0.5;
    let bottom_y = constants::ROOM_ORIGIN.y - constants::ROOM_HALF_HEIGHT + WALL_THICKNESS * 0.5;
    let left_x = constants::ROOM_ORIGIN.x - constants::ROOM_HALF_WIDTH + WALL_THICKNESS * 0.5;
    let right_x = constants::ROOM_ORIGIN.x + constants::ROOM_HALF_WIDTH - WALL_THICKNESS * 0.5;
    let horizontal_offset = DOOR_OPENING * 0.5 + horizontal_segment_width * 0.5;
    let vertical_offset = DOOR_OPENING * 0.5 + vertical_segment_height * 0.5;

    let segments: [(&str, Vec2, Vec2); 2] = match direction {
        ExitDirection::North => [
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
        ],
        ExitDirection::South => [
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
        ],
        ExitDirection::East => [
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
        ],
        ExitDirection::West => [
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
        ],
    };

    for (name, size, center) in segments {
        spawn_wall(commands, meshes, materials, name, size, center);
    }
}

fn spawn_closed_side(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    direction: ExitDirection,
) {
    let (name, size, center) = match direction {
        ExitDirection::North => (
            "NorthWallClosed",
            Vec2::new(constants::ROOM_WIDTH, WALL_THICKNESS),
            Vec2::new(
                constants::ROOM_ORIGIN.x,
                constants::ROOM_ORIGIN.y + constants::ROOM_HALF_HEIGHT - WALL_THICKNESS * 0.5,
            ),
        ),
        ExitDirection::South => (
            "SouthWallClosed",
            Vec2::new(constants::ROOM_WIDTH, WALL_THICKNESS),
            Vec2::new(
                constants::ROOM_ORIGIN.x,
                constants::ROOM_ORIGIN.y - constants::ROOM_HALF_HEIGHT + WALL_THICKNESS * 0.5,
            ),
        ),
        ExitDirection::East => (
            "EastWallClosed",
            Vec2::new(WALL_THICKNESS, constants::ROOM_HEIGHT),
            Vec2::new(
                constants::ROOM_ORIGIN.x + constants::ROOM_HALF_WIDTH - WALL_THICKNESS * 0.5,
                constants::ROOM_ORIGIN.y,
            ),
        ),
        ExitDirection::West => (
            "WestWallClosed",
            Vec2::new(WALL_THICKNESS, constants::ROOM_HEIGHT),
            Vec2::new(
                constants::ROOM_ORIGIN.x - constants::ROOM_HALF_WIDTH + WALL_THICKNESS * 0.5,
                constants::ROOM_ORIGIN.y,
            ),
        ),
    };

    spawn_wall(commands, meshes, materials, name, size, center);
}

fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    name: &str,
    size: Vec2,
    center: Vec2,
) {
    commands.spawn((
        Name::new(name.to_string()),
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

fn spawn_door_markers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    room: RoomId,
) {
    for (direction, name, anchor) in [
        (ExitDirection::North, "NorthDoor", constants::NORTH_DOOR_ANCHOR),
        (ExitDirection::South, "SouthDoor", constants::SOUTH_DOOR_ANCHOR),
        (ExitDirection::East, "EastDoor", constants::EAST_DOOR_ANCHOR),
        (ExitDirection::West, "WestDoor", constants::WEST_DOOR_ANCHOR),
    ] {
        if adjacent_room(room, direction).is_none() {
            continue;
        }

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
    room: RoomId,
) {
    let obstacles: &[(&str, Vec2)] = match room {
        RoomId::OverworldCenter => &[
            ("rock", Vec2::new(-48.0, constants::ROOM_ORIGIN.y + 10.0)),
            ("rock", Vec2::new(36.0, constants::ROOM_ORIGIN.y - 20.0)),
        ],
        RoomId::OverworldNorth => &[
            ("tree", Vec2::new(-30.0, constants::ROOM_ORIGIN.y + 4.0)),
            ("tree", Vec2::new(30.0, constants::ROOM_ORIGIN.y + 4.0)),
        ],
        RoomId::OverworldSouth => &[
            ("pond", Vec2::new(-24.0, constants::ROOM_ORIGIN.y - 6.0)),
            ("pond", Vec2::new(24.0, constants::ROOM_ORIGIN.y - 6.0)),
        ],
        RoomId::OverworldEast => &[
            ("ridge", Vec2::new(12.0, constants::ROOM_ORIGIN.y + 26.0)),
            ("ridge", Vec2::new(12.0, constants::ROOM_ORIGIN.y - 26.0)),
        ],
        RoomId::OverworldWest => &[
            ("statue", Vec2::new(-12.0, constants::ROOM_ORIGIN.y + 26.0)),
            ("statue", Vec2::new(-12.0, constants::ROOM_ORIGIN.y - 26.0)),
        ],
    };

    for &(name, position) in obstacles {
        commands.spawn((
            Name::new(name),
            RoomEntity,
            Wall,
            StaticBlocker,
            Label(name.into()),
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
    item_table: &ItemTable,
    room: RoomId,
) {
    let (unique_position, unique_kind) = match room {
        RoomId::OverworldCenter => (Vec2::new(-80.0, constants::ROOM_ORIGIN.y - 16.0), PickupKind::Rupee),
        RoomId::OverworldNorth => (Vec2::new(0.0, constants::ROOM_ORIGIN.y + 18.0), PickupKind::Bomb),
        RoomId::OverworldSouth => (Vec2::new(0.0, constants::ROOM_ORIGIN.y - 28.0), PickupKind::FiveRupees),
        RoomId::OverworldEast => (Vec2::new(60.0, constants::ROOM_ORIGIN.y), PickupKind::Key),
        RoomId::OverworldWest => (Vec2::new(-60.0, constants::ROOM_ORIGIN.y), PickupKind::HeartContainer),
    };
    let (temporary_position, temporary_kind) = match room {
        RoomId::OverworldCenter => (Vec2::new(72.0, constants::ROOM_ORIGIN.y - 10.0), PickupKind::Heart),
        RoomId::OverworldNorth => (Vec2::new(-58.0, constants::ROOM_ORIGIN.y - 4.0), PickupKind::Heart),
        RoomId::OverworldSouth => (Vec2::new(58.0, constants::ROOM_ORIGIN.y + 8.0), PickupKind::Heart),
        RoomId::OverworldEast => (Vec2::new(-50.0, constants::ROOM_ORIGIN.y + 18.0), PickupKind::Rupee),
        RoomId::OverworldWest => (Vec2::new(50.0, constants::ROOM_ORIGIN.y - 18.0), PickupKind::Heart),
    };

    let unique_key = PersistentRoomKey {
        room,
        key: "unique_pickup",
    };
    if !persistence.contains(RoomPersistenceCategory::UniquePickup, unique_key) {
        let data = item_table.lookup(unique_kind);
        commands.spawn((
            Name::new("UniquePickup"),
            RoomEntity,
            UniquePickup,
            unique_kind,
            Label(data.label.clone()),
            PersistentRoomEntity {
                key: unique_key,
                category: RoomPersistenceCategory::UniquePickup,
            },
            circle_mesh(meshes, data.radius),
            MeshMaterial2d(materials.add(data.color)),
            Transform::from_xyz(
                unique_position.x,
                unique_position.y,
                constants::render_layers::PICKUPS,
            ),
        ));
    }

    {
        let data = item_table.lookup(temporary_kind);
        commands.spawn((
            Name::new("TemporaryPickup"),
            RoomEntity,
            TemporaryPickup,
            temporary_kind,
            Label(data.label.clone()),
            PersistentRoomEntity {
                key: PersistentRoomKey {
                    room,
                    key: "temporary_pickup",
                },
                category: RoomPersistenceCategory::ResetOnRoomLoad,
            },
            circle_mesh(meshes, data.radius),
            MeshMaterial2d(materials.add(data.color)),
            Transform::from_xyz(
                temporary_position.x,
                temporary_position.y,
                constants::render_layers::PICKUPS,
            ),
        ));
    }
}

fn spawn_test_enemies(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    room: RoomId,
) {
    let enemy_positions: &[Vec2] = match room {
        RoomId::OverworldCenter => &[Vec2::new(0.0, constants::ROOM_ORIGIN.y + 24.0)],
        RoomId::OverworldNorth => &[Vec2::new(46.0, constants::ROOM_ORIGIN.y - 18.0)],
        RoomId::OverworldSouth => &[Vec2::new(-46.0, constants::ROOM_ORIGIN.y + 18.0)],
        RoomId::OverworldEast => &[Vec2::new(-12.0, constants::ROOM_ORIGIN.y)],
        RoomId::OverworldWest => &[Vec2::new(12.0, constants::ROOM_ORIGIN.y)],
    };

    for &position in enemy_positions {
        commands.spawn((
            Name::new("TestEnemy"),
            RoomEntity,
            Enemy,
            Label("enemy".into()),
            Health::new(1),
            Damage(1),
            Hitbox {
                half_size: Vec2::splat(8.0),
            },
            Hurtbox {
                half_size: Vec2::splat(8.0),
            },
            circle_mesh(meshes, 8.0),
            color_material(materials, WorldColor::Enemy),
            Transform::from_xyz(position.x, position.y, constants::render_layers::ENTITIES),
        ));
    }
}

fn spawn_secret_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    persistence: &RoomPersistence,
    room: RoomId,
) {
    if room != RoomId::OverworldCenter {
        return;
    }

    let secret_key = PersistentRoomKey {
        room,
        key: "hidden_stair",
    };

    commands.spawn((
        Name::new("SecretBush"),
        RoomEntity,
        Label("bush".into()),
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
        spawn_revealed_secret(
            commands,
            meshes,
            materials,
            secret_key,
            Vec2::new(88.0, constants::ROOM_ORIGIN.y + 42.0),
        );
    }
}

fn collect_unique_pickups(
    mut commands: Commands,
    mut persistence: ResMut<RoomPersistence>,
    mut inventory: ResMut<Inventory>,
    mut player_vitals: ResMut<PlayerVitals>,
    item_table: Res<ItemTable>,
    mut player: Query<(&Transform, &mut Health), With<Player>>,
    pickups: Query<(Entity, &Transform, &PersistentRoomEntity, &PickupKind), With<UniquePickup>>,
) {
    let Ok((player_transform, mut player_health)) = player.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, persistent, &kind) in &pickups {
        if player_pos.distance(transform.translation.truncate()) <= PICKUP_RADIUS + 8.0 {
            let data = item_table.lookup(kind);
            items::apply_pickup_effect(&data.effect, &mut inventory, &mut player_health, &mut player_vitals);
            persistence.record(persistent.category, persistent.key);
            commands.entity(entity).despawn();
        }
    }
}

fn collect_temporary_pickups(
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    mut player_vitals: ResMut<PlayerVitals>,
    item_table: Res<ItemTable>,
    mut player: Query<(&Transform, &mut Health), With<Player>>,
    pickups: Query<(Entity, &Transform, &PickupKind), (With<TemporaryPickup>, With<RoomEntity>)>,
) {
    let Ok((player_transform, mut player_health)) = player.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, &kind) in &pickups {
        if player_pos.distance(transform.translation.truncate()) <= PICKUP_RADIUS + 8.0 {
            let data = item_table.lookup(kind);
            items::apply_pickup_effect(&data.effect, &mut inventory, &mut player_health, &mut player_vitals);
            commands.entity(entity).despawn();
        }
    }
}

fn reveal_secret_triggers(
    mut commands: Commands,
    actions: Res<InputActions>,
    mut persistence: ResMut<RoomPersistence>,
    player: Query<&Transform, With<Player>>,
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
        Label("stair".into()),
        PersistentRoomEntity {
            key,
            category: RoomPersistenceCategory::Secret,
        },
        rectangle_mesh(meshes, Vec2::new(20.0, 14.0)),
        color_material(materials, WorldColor::Accent),
        Transform::from_xyz(position.x, position.y, constants::render_layers::PICKUPS),
    ));
}

fn detect_exit_direction(player_pos: Vec2) -> Option<ExitDirection> {
    let left_edge = constants::ROOM_ORIGIN.x - constants::ROOM_HALF_WIDTH - EDGE_EXIT_PADDING;
    let right_edge = constants::ROOM_ORIGIN.x + constants::ROOM_HALF_WIDTH + EDGE_EXIT_PADDING;
    let bottom_edge = constants::ROOM_ORIGIN.y - constants::ROOM_HALF_HEIGHT - EDGE_EXIT_PADDING;
    let top_edge = constants::ROOM_ORIGIN.y + constants::ROOM_HALF_HEIGHT + EDGE_EXIT_PADDING;

    if player_pos.x <= left_edge {
        Some(ExitDirection::West)
    } else if player_pos.x >= right_edge {
        Some(ExitDirection::East)
    } else if player_pos.y >= top_edge {
        Some(ExitDirection::North)
    } else if player_pos.y <= bottom_edge {
        Some(ExitDirection::South)
    } else {
        None
    }
}

fn adjacent_room(room: RoomId, direction: ExitDirection) -> Option<RoomId> {
    match (room, direction) {
        (RoomId::OverworldCenter, ExitDirection::North) => Some(RoomId::OverworldNorth),
        (RoomId::OverworldCenter, ExitDirection::South) => Some(RoomId::OverworldSouth),
        (RoomId::OverworldCenter, ExitDirection::East) => Some(RoomId::OverworldEast),
        (RoomId::OverworldCenter, ExitDirection::West) => Some(RoomId::OverworldWest),
        (RoomId::OverworldNorth, ExitDirection::South) => Some(RoomId::OverworldCenter),
        (RoomId::OverworldSouth, ExitDirection::North) => Some(RoomId::OverworldCenter),
        (RoomId::OverworldEast, ExitDirection::West) => Some(RoomId::OverworldCenter),
        (RoomId::OverworldWest, ExitDirection::East) => Some(RoomId::OverworldCenter),
        _ => None,
    }
}

fn room_floor_color(room: RoomId) -> WorldColor {
    match room {
        RoomId::OverworldCenter => WorldColor::RoomFloor,
        RoomId::OverworldNorth => WorldColor::HudPanel,
        RoomId::OverworldSouth => WorldColor::RoomFloor,
        RoomId::OverworldEast => WorldColor::Doorway,
        RoomId::OverworldWest => WorldColor::Backdrop,
    }
}

