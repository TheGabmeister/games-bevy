use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    collision::CollisionSet,
    components::{
        Damage, Door, Enemy, Facing, Health, Hitbox, Hurtbox, Label, Npc, PickupKind, Player,
        PushBlock, RoomEntity, ShopItem, SolidBody, StaticBlocker, Velocity, Wall,
    },
    constants,
    input::InputActions,
    items::{self, ItemTable},
    resources::{
        CurrentRoom, DialogueState, DoorKind, DungeonId, DungeonState, EquippedItem,
        ExitDirection, Inventory, PersistentRoomKey, PlayerVitals, RoomId, RoomPersistence,
        RoomPersistenceCategory, RoomTransitionState, RoomType,
    },
    rendering::{circle_mesh, color_material, rectangle_mesh, WorldColor},
    states::AppState,
};

const WALL_THICKNESS: f32 = 16.0;
const DOOR_OPENING: f32 = 32.0;
const PICKUP_RADIUS: f32 = 7.0;
const EDGE_EXIT_PADDING: f32 = 6.0;
const PLAYER_INTERACT_RADIUS: f32 = 14.0;
const STAIRCASE_RADIUS: f32 = 10.0;

// ── Secret trigger kinds ──────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub enum SecretTriggerKind {
    #[default]
    HiddenStaircase,
    BurnableBush,
    BombableWall,
    PushBlock,
}

// ── RON deserialization types ─────────────────────────────────────────

#[derive(Deserialize)]
struct RoomExitRon {
    direction: ExitDirection,
    target: RoomId,
    #[serde(default)]
    door_kind: DoorKind,
}

#[derive(Deserialize)]
struct RoomObstacleRon {
    label: String,
    position: (f32, f32),
    size: (f32, f32),
}

#[derive(Deserialize)]
struct RoomEnemyRon {
    position: (f32, f32),
    health: u8,
    damage: u8,
    radius: f32,
}

#[derive(Deserialize)]
struct RoomPickupRon {
    key: String,
    kind: PickupKind,
    position: (f32, f32),
}

#[derive(Deserialize)]
struct RoomStaircaseRon {
    position: (f32, f32),
    target: RoomId,
    target_spawn: (f32, f32),
    label: String,
}

#[derive(Deserialize)]
struct RoomNpcRon {
    position: (f32, f32),
    dialogue: Vec<String>,
}

#[derive(Deserialize)]
struct RoomShopOfferRon {
    key: String,
    position: (f32, f32),
    kind: PickupKind,
    price: u16,
}

#[derive(Deserialize)]
struct RoomSecretRon {
    key: String,
    #[serde(default)]
    trigger_kind: SecretTriggerKind,
    trigger_label: String,
    trigger_position: (f32, f32),
    trigger_radius: f32,
    reveal_label: String,
    reveal_position: (f32, f32),
    reveal_size: (f32, f32),
    #[serde(default)]
    reveal_target: Option<RoomId>,
    #[serde(default)]
    reveal_spawn: Option<(f32, f32)>,
}

#[derive(Deserialize)]
struct RoomEntryRon {
    id: RoomId,
    #[serde(default)]
    room_type: RoomType,
    floor_color: (f32, f32, f32),
    exits: Vec<RoomExitRon>,
    #[serde(default)]
    staircases: Vec<RoomStaircaseRon>,
    obstacles: Vec<RoomObstacleRon>,
    enemies: Vec<RoomEnemyRon>,
    unique_pickups: Vec<RoomPickupRon>,
    temporary_pickups: Vec<RoomPickupRon>,
    secrets: Vec<RoomSecretRon>,
    #[serde(default)]
    npcs: Vec<RoomNpcRon>,
    #[serde(default)]
    shop_offers: Vec<RoomShopOfferRon>,
}

// ── Runtime data types ────────────────────────────────────────────────

pub struct RoomExit {
    pub direction: ExitDirection,
    pub target: RoomId,
    pub door_kind: DoorKind,
}

pub struct RoomObstacle {
    pub label: &'static str,
    pub position: Vec2,
    pub size: Vec2,
}

pub struct RoomEnemy {
    pub position: Vec2,
    pub health: u8,
    pub damage: u8,
    pub radius: f32,
}

pub struct RoomPickup {
    pub key: &'static str,
    pub kind: PickupKind,
    pub position: Vec2,
}

pub struct RoomStaircase {
    pub position: Vec2,
    pub target: RoomId,
    pub target_spawn: Vec2,
    pub label: &'static str,
}

pub struct RoomNpc {
    pub position: Vec2,
    pub dialogue: Vec<String>,
}

pub struct RoomShopOffer {
    pub key: &'static str,
    pub position: Vec2,
    pub kind: PickupKind,
    pub price: u16,
}

pub struct RoomSecret {
    pub key: &'static str,
    pub trigger_kind: SecretTriggerKind,
    pub trigger_label: &'static str,
    pub trigger_position: Vec2,
    pub trigger_radius: f32,
    pub reveal_label: &'static str,
    pub reveal_position: Vec2,
    pub reveal_size: Vec2,
    pub reveal_target: Option<RoomId>,
    pub reveal_spawn: Option<Vec2>,
}

pub struct RoomData {
    pub id: RoomId,
    pub room_type: RoomType,
    pub floor_color: Color,
    pub exits: Vec<RoomExit>,
    pub staircases: Vec<RoomStaircase>,
    pub obstacles: Vec<RoomObstacle>,
    pub enemies: Vec<RoomEnemy>,
    pub unique_pickups: Vec<RoomPickup>,
    pub temporary_pickups: Vec<RoomPickup>,
    pub secrets: Vec<RoomSecret>,
    pub npcs: Vec<RoomNpc>,
    pub shop_offers: Vec<RoomShopOffer>,
}

// ── Room table resource ───────────────────────────────────────────────

#[derive(Resource)]
pub struct RoomTable {
    rooms: Vec<RoomData>,
}

impl RoomTable {
    pub fn lookup(&self, id: RoomId) -> &RoomData {
        self.rooms
            .iter()
            .find(|r| r.id == id)
            .expect("missing room data entry")
    }

    pub fn adjacent_room(&self, room: RoomId, direction: ExitDirection) -> Option<RoomId> {
        self.lookup(room)
            .exits
            .iter()
            .find(|e| e.direction == direction)
            .map(|e| e.target)
    }
}

fn load_room_table() -> RoomTable {
    let ron_str = std::fs::read_to_string("assets/data/rooms.ron")
        .expect("failed to read assets/data/rooms.ron");
    let entries: Vec<RoomEntryRon> = ron::from_str(&ron_str).expect("failed to parse rooms.ron");
    let rooms = entries
        .into_iter()
        .map(|e| RoomData {
            id: e.id,
            room_type: e.room_type,
            floor_color: Color::srgb(e.floor_color.0, e.floor_color.1, e.floor_color.2),
            exits: e
                .exits
                .into_iter()
                .map(|ex| RoomExit {
                    direction: ex.direction,
                    target: ex.target,
                    door_kind: ex.door_kind,
                })
                .collect(),
            staircases: e
                .staircases
                .into_iter()
                .map(|s| RoomStaircase {
                    position: Vec2::new(s.position.0, s.position.1),
                    target: s.target,
                    target_spawn: Vec2::new(s.target_spawn.0, s.target_spawn.1)
                        + constants::ROOM_ORIGIN,
                    label: Box::leak(s.label.into_boxed_str()),
                })
                .collect(),
            obstacles: e
                .obstacles
                .into_iter()
                .map(|o| RoomObstacle {
                    label: Box::leak(o.label.into_boxed_str()),
                    position: Vec2::new(o.position.0, o.position.1),
                    size: Vec2::new(o.size.0, o.size.1),
                })
                .collect(),
            enemies: e
                .enemies
                .into_iter()
                .map(|en| RoomEnemy {
                    position: Vec2::new(en.position.0, en.position.1),
                    health: en.health,
                    damage: en.damage,
                    radius: en.radius,
                })
                .collect(),
            unique_pickups: e
                .unique_pickups
                .into_iter()
                .map(|p| RoomPickup {
                    key: Box::leak(p.key.into_boxed_str()),
                    kind: p.kind,
                    position: Vec2::new(p.position.0, p.position.1),
                })
                .collect(),
            temporary_pickups: e
                .temporary_pickups
                .into_iter()
                .map(|p| RoomPickup {
                    key: Box::leak(p.key.into_boxed_str()),
                    kind: p.kind,
                    position: Vec2::new(p.position.0, p.position.1),
                })
                .collect(),
            secrets: e
                .secrets
                .into_iter()
                .map(|s| RoomSecret {
                    key: Box::leak(s.key.into_boxed_str()),
                    trigger_kind: s.trigger_kind,
                    trigger_label: Box::leak(s.trigger_label.into_boxed_str()),
                    trigger_position: Vec2::new(s.trigger_position.0, s.trigger_position.1),
                    trigger_radius: s.trigger_radius,
                    reveal_label: Box::leak(s.reveal_label.into_boxed_str()),
                    reveal_position: Vec2::new(s.reveal_position.0, s.reveal_position.1),
                    reveal_size: Vec2::new(s.reveal_size.0, s.reveal_size.1),
                    reveal_target: s.reveal_target,
                    reveal_spawn: s.reveal_spawn.map(|(x, y)| {
                        Vec2::new(x, y) + constants::ROOM_ORIGIN
                    }),
                })
                .collect(),
            npcs: e
                .npcs
                .into_iter()
                .map(|n| RoomNpc {
                    position: Vec2::new(n.position.0, n.position.1),
                    dialogue: n.dialogue,
                })
                .collect(),
            shop_offers: e
                .shop_offers
                .into_iter()
                .map(|o| RoomShopOffer {
                    key: Box::leak(o.key.into_boxed_str()),
                    position: Vec2::new(o.position.0, o.position.1),
                    kind: o.kind,
                    price: o.price,
                })
                .collect(),
        })
        .collect();
    RoomTable { rooms }
}

// ── Plugin ────────────────────────────────────────────────────────────

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
    pub trigger_kind: SecretTriggerKind,
    pub reveal_at: Vec2,
    pub radius: f32,
    pub reveal_label: &'static str,
    pub reveal_size: Vec2,
    pub reveal_target: Option<RoomId>,
    pub reveal_spawn: Option<Vec2>,
}

#[derive(Component, Clone, Copy)]
pub struct PersistentRoomEntity {
    pub key: PersistentRoomKey,
    pub category: RoomPersistenceCategory,
}

#[derive(Component)]
pub struct StaircaseEntrance {
    pub target_room: RoomId,
    pub target_spawn: Vec2,
}

#[derive(Component)]
pub struct NpcDialogue(pub Vec<String>);

#[derive(Component)]
pub struct DungeonDoorBlocker {
    pub direction: ExitDirection,
    pub kind: DoorKind,
    pub persist_key: PersistentRoomKey,
}

#[derive(SystemParam)]
struct RoomLoadContext<'w, 's> {
    current_room: ResMut<'w, CurrentRoom>,
    persistence: Res<'w, RoomPersistence>,
    item_table: Res<'w, ItemTable>,
    room_table: Res<'w, RoomTable>,
    dungeon_state: ResMut<'w, DungeonState>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    room_entities: Query<'w, 's, Entity, With<RoomEntity>>,
}

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load_room_table())
            .init_resource::<CurrentRoom>()
            .init_resource::<RoomPersistence>()
            .init_resource::<RoomTransitionState>()
            .init_resource::<DungeonState>()
            .init_resource::<DialogueState>()
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
            .add_systems(
                Update,
                tick_room_transition_state.in_set(RoomSet::TransitionTick),
            )
            .add_systems(
                Update,
                (
                    request_room_reload,
                    request_screen_edge_transition,
                    check_staircase_transitions,
                )
                    .in_set(RoomSet::RequestLoad)
                    .after(CollisionSet::Resolve),
            )
            .add_systems(Update, process_room_loads.in_set(RoomSet::Load))
            .add_systems(
                Update,
                (
                    (interact_with_npcs, purchase_shop_items),
                    (
                        unlock_locked_doors,
                        use_bomb_on_targets,
                        collect_unique_pickups,
                        collect_temporary_pickups,
                        reveal_secret_triggers,
                        handle_push_blocks,
                        check_shutter_doors,
                    ),
                )
                    .chain()
                    .in_set(RoomSet::Interact),
            );
    }
}

// ── Systems ───────────────────────────────────────────────────────────

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
    room_table: Res<RoomTable>,
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
    let Some(target_room) = room_table.adjacent_room(current_room.id, direction) else {
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

fn check_staircase_transitions(
    transition: Res<RoomTransitionState>,
    player: Query<(&Transform, &Velocity), With<Player>>,
    staircases: Query<(&Transform, &StaircaseEntrance)>,
    mut loads: MessageWriter<LoadRoomMessage>,
) {
    if transition.locked {
        return;
    }

    let Ok((player_transform, player_velocity)) = player.single() else {
        return;
    };

    if player_velocity.0.length_squared() < 0.01 {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    for (staircase_transform, entrance) in &staircases {
        let staircase_pos = staircase_transform.translation.truncate();
        if player_pos.distance(staircase_pos) <= STAIRCASE_RADIUS {
            loads.write(LoadRoomMessage {
                room: entrance.target_room,
                player_spawn: entrance.target_spawn,
            });
            return;
        }
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

        let prev_dungeon = context.dungeon_state.current_dungeon;
        let room_data = context.room_table.lookup(message.room);
        let new_dungeon = dungeon_for_room(message.room);

        if room_data.room_type == RoomType::Dungeon {
            context.dungeon_state.current_dungeon = new_dungeon;
        } else if prev_dungeon.is_some() {
            context.dungeon_state.rooms_cleared.clear();
            context.dungeon_state.current_dungeon = None;
        }

        context.current_room.id = message.room;
        spawn_room(
            &mut commands,
            context.meshes.as_mut(),
            context.materials.as_mut(),
            &context.persistence,
            &context.item_table,
            &context.room_table,
            &context.dungeon_state,
            message.room,
        );
        room_loaded.write(RoomLoadedMessage {
            room: message.room,
            player_spawn: message.player_spawn,
        });
    }
}

fn interact_with_npcs(
    actions: Res<InputActions>,
    mut dialogue: ResMut<DialogueState>,
    player: Query<&Transform, With<Player>>,
    npcs: Query<(&Transform, &NpcDialogue), With<Npc>>,
) {
    if !actions.confirm {
        return;
    }

    if dialogue.is_active() {
        dialogue.advance();
        return;
    }

    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (npc_transform, npc_dialogue) in &npcs {
        let npc_pos = npc_transform.translation.truncate();
        if player_pos.distance(npc_pos) <= 24.0 {
            dialogue.start(npc_dialogue.0.clone());
            return;
        }
    }
}

fn purchase_shop_items(
    mut commands: Commands,
    actions: Res<InputActions>,
    mut inventory: ResMut<Inventory>,
    mut player_vitals: ResMut<PlayerVitals>,
    mut dungeon_state: ResMut<DungeonState>,
    mut dialogue: ResMut<DialogueState>,
    item_table: Res<ItemTable>,
    mut player: Query<(&Transform, &mut Health), With<Player>>,
    mut persistence: ResMut<RoomPersistence>,
    shop_items: Query<
        (Entity, &Transform, &ShopItem, &PersistentRoomEntity),
        With<UniquePickup>,
    >,
) {
    if dialogue.is_active() || !actions.confirm {
        return;
    }

    let Ok((player_transform, mut player_health)) = player.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, shop_item, persistent) in &shop_items {
        if player_pos.distance(transform.translation.truncate()) <= 20.0 {
            if inventory.rupees >= shop_item.price {
                inventory.rupees -= shop_item.price;
                let data = item_table.lookup(shop_item.kind);
                items::apply_pickup_effect(
                    &data.effect,
                    &mut inventory,
                    &mut player_health,
                    &mut player_vitals,
                    &mut dungeon_state,
                );
                persistence.record(persistent.category, persistent.key);
                commands.entity(entity).despawn();
                dialogue.start(vec!["THANKS!".to_string()]);
            } else {
                dialogue.start(vec!["NOT ENOUGH RUPEES".to_string()]);
            }
            return;
        }
    }
}

fn unlock_locked_doors(
    mut commands: Commands,
    actions: Res<InputActions>,
    dialogue: Res<DialogueState>,
    player: Query<&Transform, With<Player>>,
    doors: Query<(Entity, &Transform, &DungeonDoorBlocker), With<StaticBlocker>>,
    mut dungeon_state: ResMut<DungeonState>,
    mut persistence: ResMut<RoomPersistence>,
) {
    if dialogue.is_active() || !actions.confirm {
        return;
    }

    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, blocker) in &doors {
        if blocker.kind != DoorKind::Locked {
            continue;
        }
        if player_pos.distance(transform.translation.truncate()) <= 24.0 {
            if dungeon_state.spend_key() {
                persistence.record(RoomPersistenceCategory::DungeonDoor, blocker.persist_key);
                commands.entity(entity).despawn();
            }
            return;
        }
    }
}

fn use_bomb_on_targets(
    mut commands: Commands,
    actions: Res<InputActions>,
    mut inventory: ResMut<Inventory>,
    dialogue: Res<DialogueState>,
    player: Query<(&Transform, &Facing), With<Player>>,
    doors: Query<(Entity, &Transform, &DungeonDoorBlocker), With<StaticBlocker>>,
    secrets: Query<(Entity, &Transform, &SecretTrigger)>,
    mut persistence: ResMut<RoomPersistence>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if dialogue.is_active() || !actions.item_use {
        return;
    }

    if inventory.equipped != Some(EquippedItem::Bomb) || inventory.bombs == 0 {
        return;
    }

    let Ok((player_transform, _facing)) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    // Check bombable doors
    for (entity, transform, blocker) in &doors {
        if blocker.kind != DoorKind::Bombable {
            continue;
        }
        if player_pos.distance(transform.translation.truncate()) <= 24.0 {
            inventory.bombs -= 1;
            persistence.record(RoomPersistenceCategory::DungeonDoor, blocker.persist_key);
            commands.entity(entity).despawn();
            return;
        }
    }

    // Check bombable wall secrets
    for (entity, transform, trigger) in &secrets {
        if trigger.trigger_kind != SecretTriggerKind::BombableWall {
            continue;
        }
        if persistence.contains(RoomPersistenceCategory::Secret, trigger.key) {
            continue;
        }
        if player_pos.distance(transform.translation.truncate()) <= trigger.radius + PLAYER_INTERACT_RADIUS {
            inventory.bombs -= 1;
            persistence.record(RoomPersistenceCategory::Secret, trigger.key);
            spawn_revealed_secret(
                &mut commands,
                meshes.as_mut(),
                materials.as_mut(),
                trigger.key,
                trigger.reveal_at,
                trigger.reveal_size,
                trigger.reveal_label,
                trigger.reveal_target,
                trigger.reveal_spawn,
            );
            commands.entity(entity).despawn();
            return;
        }
    }
}

fn collect_unique_pickups(
    mut commands: Commands,
    mut persistence: ResMut<RoomPersistence>,
    mut inventory: ResMut<Inventory>,
    mut player_vitals: ResMut<PlayerVitals>,
    mut dungeon_state: ResMut<DungeonState>,
    item_table: Res<ItemTable>,
    mut player: Query<(&Transform, &mut Health), With<Player>>,
    pickups: Query<
        (Entity, &Transform, &PersistentRoomEntity, &PickupKind),
        (With<UniquePickup>, Without<ShopItem>),
    >,
) {
    let Ok((player_transform, mut player_health)) = player.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, persistent, &kind) in &pickups {
        if player_pos.distance(transform.translation.truncate()) <= PICKUP_RADIUS + 8.0 {
            let data = item_table.lookup(kind);
            items::apply_pickup_effect(
                &data.effect,
                &mut inventory,
                &mut player_health,
                &mut player_vitals,
                &mut dungeon_state,
            );
            persistence.record(persistent.category, persistent.key);
            commands.entity(entity).despawn();
        }
    }
}

fn collect_temporary_pickups(
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    mut player_vitals: ResMut<PlayerVitals>,
    mut dungeon_state: ResMut<DungeonState>,
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
            items::apply_pickup_effect(
                &data.effect,
                &mut inventory,
                &mut player_health,
                &mut player_vitals,
                &mut dungeon_state,
            );
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

    for (entity, transform, trigger) in &triggers {
        // Only HiddenStaircase and BurnableBush are activated by attack
        if trigger.trigger_kind != SecretTriggerKind::HiddenStaircase
            && trigger.trigger_kind != SecretTriggerKind::BurnableBush
        {
            continue;
        }

        if persistence.contains(RoomPersistenceCategory::Secret, trigger.key) {
            continue;
        }

        if player_pos.distance(transform.translation.truncate())
            <= trigger.radius + PLAYER_INTERACT_RADIUS
        {
            persistence.record(RoomPersistenceCategory::Secret, trigger.key);
            spawn_revealed_secret(
                &mut commands,
                meshes.as_mut(),
                materials.as_mut(),
                trigger.key,
                trigger.reveal_at,
                trigger.reveal_size,
                trigger.reveal_label,
                trigger.reveal_target,
                trigger.reveal_spawn,
            );

            // BurnableBush: despawn the trigger entity after revealing
            if trigger.trigger_kind == SecretTriggerKind::BurnableBush {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn handle_push_blocks(
    mut commands: Commands,
    player: Query<(&Transform, &Facing, &Velocity), With<Player>>,
    mut blocks: Query<(Entity, &Transform, &mut PushBlock, Option<&SecretTrigger>)>,
    mut persistence: ResMut<RoomPersistence>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok((player_transform, facing, velocity)) = player.single() else {
        return;
    };

    // If player not moving, reset all push counters
    if velocity.0.length_squared() < 0.01 {
        for (_, _, mut block, _) in &mut blocks {
            block.pushes = 0;
        }
        return;
    }

    let player_pos = player_transform.translation.truncate();
    let facing_dir = match facing {
        Facing::Up => Vec2::Y,
        Facing::Down => Vec2::NEG_Y,
        Facing::Left => Vec2::NEG_X,
        Facing::Right => Vec2::X,
    };

    for (entity, transform, mut block, secret) in &mut blocks {
        let block_pos = transform.translation.truncate();
        if player_pos.distance(block_pos) > 20.0 {
            continue;
        }

        let to_block = (block_pos - player_pos).normalize_or_zero();
        if facing_dir.dot(to_block) > 0.5 {
            block.pushes += 1;
            if block.pushes >= block.pushes_needed {
                if let Some(trigger) = secret {
                    if !persistence.contains(RoomPersistenceCategory::Secret, trigger.key) {
                        persistence.record(RoomPersistenceCategory::Secret, trigger.key);
                        spawn_revealed_secret(
                            &mut commands,
                            meshes.as_mut(),
                            materials.as_mut(),
                            trigger.key,
                            trigger.reveal_at,
                            trigger.reveal_size,
                            trigger.reveal_label,
                            trigger.reveal_target,
                            trigger.reveal_spawn,
                        );
                    }
                }
                commands.entity(entity).despawn();
            }
        }
    }
}

fn check_shutter_doors(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    doors: Query<(Entity, &DungeonDoorBlocker)>,
    current_room: Res<CurrentRoom>,
    mut dungeon_state: ResMut<DungeonState>,
) {
    if !enemies.is_empty() {
        return;
    }

    for (entity, blocker) in &doors {
        if blocker.kind == DoorKind::Shutter {
            commands.entity(entity).despawn();
        }
    }

    dungeon_state.rooms_cleared.insert(current_room.id);
}

// ── Room spawning ─────────────────────────────────────────────────────

fn spawn_room(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    persistence: &RoomPersistence,
    item_table: &ItemTable,
    room_table: &RoomTable,
    dungeon_state: &DungeonState,
    room_id: RoomId,
) {
    let room = room_table.lookup(room_id);

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
        rectangle_mesh(
            meshes,
            Vec2::new(constants::ROOM_WIDTH, constants::ROOM_HEIGHT),
        ),
        MeshMaterial2d(materials.add(room.floor_color)),
        Transform::from_xyz(
            constants::ROOM_ORIGIN.x,
            constants::ROOM_ORIGIN.y,
            constants::render_layers::FLOOR,
        ),
    ));

    spawn_perimeter_walls(commands, meshes, materials, room);
    spawn_door_markers(commands, meshes, materials, room);
    spawn_door_blockers(commands, meshes, materials, persistence, dungeon_state, room_id, room);
    spawn_staircases(commands, meshes, materials, room);
    spawn_obstacles(commands, meshes, materials, room);
    spawn_enemies(commands, meshes, materials, room);
    spawn_pickups(commands, meshes, materials, persistence, item_table, room_id, room);
    spawn_secrets(commands, meshes, materials, persistence, room_id, room);
    spawn_npcs(commands, meshes, materials, room);
    spawn_shop_items(commands, meshes, materials, persistence, item_table, room_id, room);
}

fn spawn_perimeter_walls(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    room: &RoomData,
) {
    for direction in [
        ExitDirection::North,
        ExitDirection::South,
        ExitDirection::East,
        ExitDirection::West,
    ] {
        if room.exits.iter().any(|e| e.direction == direction) {
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
    room: &RoomData,
) {
    for exit in &room.exits {
        let (name, anchor) = match exit.direction {
            ExitDirection::North => ("NorthDoor", constants::NORTH_DOOR_ANCHOR),
            ExitDirection::South => ("SouthDoor", constants::SOUTH_DOOR_ANCHOR),
            ExitDirection::East => ("EastDoor", constants::EAST_DOOR_ANCHOR),
            ExitDirection::West => ("WestDoor", constants::WEST_DOOR_ANCHOR),
        };

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

fn spawn_door_blockers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    persistence: &RoomPersistence,
    dungeon_state: &DungeonState,
    room_id: RoomId,
    room: &RoomData,
) {
    for exit in &room.exits {
        if exit.door_kind == DoorKind::Open {
            continue;
        }

        let persist_key = PersistentRoomKey {
            room: room_id,
            key: door_persist_key(exit.direction),
        };

        // Already opened permanently
        if persistence.contains(RoomPersistenceCategory::DungeonDoor, persist_key) {
            continue;
        }

        // Shutter doors open when room is cleared
        if exit.door_kind == DoorKind::Shutter
            && dungeon_state.rooms_cleared.contains(&room_id)
        {
            continue;
        }

        let anchor = match exit.direction {
            ExitDirection::North => constants::NORTH_DOOR_ANCHOR,
            ExitDirection::South => constants::SOUTH_DOOR_ANCHOR,
            ExitDirection::East => constants::EAST_DOOR_ANCHOR,
            ExitDirection::West => constants::WEST_DOOR_ANCHOR,
        };

        let (size, color, label_text) = match exit.door_kind {
            DoorKind::Locked => {
                let s = match exit.direction {
                    ExitDirection::North | ExitDirection::South => Vec2::new(32.0, 8.0),
                    ExitDirection::East | ExitDirection::West => Vec2::new(8.0, 32.0),
                };
                (s, WorldColor::Accent, "locked")
            }
            DoorKind::Shutter => {
                let s = match exit.direction {
                    ExitDirection::North | ExitDirection::South => Vec2::new(32.0, 8.0),
                    ExitDirection::East | ExitDirection::West => Vec2::new(8.0, 32.0),
                };
                (s, WorldColor::Doorway, "shutter")
            }
            DoorKind::Bombable => {
                let s = match exit.direction {
                    ExitDirection::North | ExitDirection::South => Vec2::new(32.0, 8.0),
                    ExitDirection::East | ExitDirection::West => Vec2::new(8.0, 32.0),
                };
                (s, WorldColor::Hazard, "crack")
            }
            DoorKind::Open => unreachable!(),
        };

        commands.spawn((
            Name::new("DungeonDoorBlocker"),
            RoomEntity,
            StaticBlocker,
            SolidBody {
                half_size: size * 0.5,
            },
            Label(label_text.to_string()),
            DungeonDoorBlocker {
                direction: exit.direction,
                kind: exit.door_kind,
                persist_key,
            },
            rectangle_mesh(meshes, size),
            color_material(materials, color),
            Transform::from_xyz(anchor.x, anchor.y, constants::render_layers::WALLS),
        ));
    }
}

fn spawn_staircases(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    room: &RoomData,
) {
    for staircase in &room.staircases {
        commands.spawn((
            Name::new("Staircase"),
            RoomEntity,
            StaircaseEntrance {
                target_room: staircase.target,
                target_spawn: staircase.target_spawn,
            },
            Label(staircase.label.to_string()),
            rectangle_mesh(meshes, Vec2::new(16.0, 12.0)),
            color_material(materials, WorldColor::Accent),
            Transform::from_xyz(
                staircase.position.x + constants::ROOM_ORIGIN.x,
                staircase.position.y + constants::ROOM_ORIGIN.y,
                constants::render_layers::FLOOR + 1.0,
            ),
        ));
    }
}

fn spawn_obstacles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    room: &RoomData,
) {
    for obstacle in &room.obstacles {
        commands.spawn((
            Name::new(obstacle.label.to_string()),
            RoomEntity,
            Wall,
            StaticBlocker,
            Label(obstacle.label.to_string()),
            SolidBody {
                half_size: obstacle.size * 0.5,
            },
            rectangle_mesh(meshes, obstacle.size),
            color_material(materials, WorldColor::Hazard),
            Transform::from_xyz(
                obstacle.position.x + constants::ROOM_ORIGIN.x,
                obstacle.position.y + constants::ROOM_ORIGIN.y,
                constants::render_layers::WALLS,
            ),
        ));
    }
}

fn spawn_enemies(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    room: &RoomData,
) {
    for enemy in &room.enemies {
        commands.spawn((
            Name::new("Enemy"),
            RoomEntity,
            Enemy,
            Label("enemy".into()),
            Health::new(enemy.health),
            Damage(enemy.damage),
            Hitbox {
                half_size: Vec2::splat(enemy.radius),
            },
            Hurtbox {
                half_size: Vec2::splat(enemy.radius),
            },
            circle_mesh(meshes, enemy.radius),
            color_material(materials, WorldColor::Enemy),
            Transform::from_xyz(
                enemy.position.x + constants::ROOM_ORIGIN.x,
                enemy.position.y + constants::ROOM_ORIGIN.y,
                constants::render_layers::ENTITIES,
            ),
        ));
    }
}

fn spawn_pickups(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    persistence: &RoomPersistence,
    item_table: &ItemTable,
    room_id: RoomId,
    room: &RoomData,
) {
    for pickup in &room.unique_pickups {
        let key = PersistentRoomKey {
            room: room_id,
            key: pickup.key,
        };
        if persistence.contains(RoomPersistenceCategory::UniquePickup, key) {
            continue;
        }
        let data = item_table.lookup(pickup.kind);
        commands.spawn((
            Name::new("UniquePickup"),
            RoomEntity,
            UniquePickup,
            pickup.kind,
            Label(data.label.clone()),
            PersistentRoomEntity {
                key,
                category: RoomPersistenceCategory::UniquePickup,
            },
            circle_mesh(meshes, data.radius),
            MeshMaterial2d(materials.add(data.color)),
            Transform::from_xyz(
                pickup.position.x + constants::ROOM_ORIGIN.x,
                pickup.position.y + constants::ROOM_ORIGIN.y,
                constants::render_layers::PICKUPS,
            ),
        ));
    }

    for pickup in &room.temporary_pickups {
        let data = item_table.lookup(pickup.kind);
        commands.spawn((
            Name::new("TemporaryPickup"),
            RoomEntity,
            TemporaryPickup,
            pickup.kind,
            Label(data.label.clone()),
            PersistentRoomEntity {
                key: PersistentRoomKey {
                    room: room_id,
                    key: pickup.key,
                },
                category: RoomPersistenceCategory::ResetOnRoomLoad,
            },
            circle_mesh(meshes, data.radius),
            MeshMaterial2d(materials.add(data.color)),
            Transform::from_xyz(
                pickup.position.x + constants::ROOM_ORIGIN.x,
                pickup.position.y + constants::ROOM_ORIGIN.y,
                constants::render_layers::PICKUPS,
            ),
        ));
    }
}

fn spawn_secrets(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    persistence: &RoomPersistence,
    room_id: RoomId,
    room: &RoomData,
) {
    for secret in &room.secrets {
        let key = PersistentRoomKey {
            room: room_id,
            key: secret.key,
        };

        if secret.trigger_kind == SecretTriggerKind::PushBlock {
            // Spawn as a push block entity
            commands.spawn((
                Name::new("PushBlock"),
                RoomEntity,
                Wall,
                StaticBlocker,
                SolidBody {
                    half_size: Vec2::new(10.0, 10.0),
                },
                PushBlock {
                    pushes: 0,
                    pushes_needed: 30,
                },
                SecretTrigger {
                    key,
                    trigger_kind: secret.trigger_kind,
                    reveal_at: secret.reveal_position + constants::ROOM_ORIGIN,
                    radius: secret.trigger_radius,
                    reveal_label: secret.reveal_label,
                    reveal_size: secret.reveal_size,
                    reveal_target: secret.reveal_target,
                    reveal_spawn: secret.reveal_spawn,
                },
                Label(secret.trigger_label.to_string()),
                rectangle_mesh(meshes, Vec2::new(20.0, 20.0)),
                color_material(materials, WorldColor::Doorway),
                Transform::from_xyz(
                    secret.trigger_position.x + constants::ROOM_ORIGIN.x,
                    secret.trigger_position.y + constants::ROOM_ORIGIN.y,
                    constants::render_layers::ENTITIES,
                ),
            ));
        } else {
            // Spawn normal secret trigger entity
            commands.spawn((
                Name::new("SecretTrigger"),
                RoomEntity,
                Label(secret.trigger_label.to_string()),
                SecretTrigger {
                    key,
                    trigger_kind: secret.trigger_kind,
                    reveal_at: secret.reveal_position + constants::ROOM_ORIGIN,
                    radius: secret.trigger_radius,
                    reveal_label: secret.reveal_label,
                    reveal_size: secret.reveal_size,
                    reveal_target: secret.reveal_target,
                    reveal_spawn: secret.reveal_spawn,
                },
                circle_mesh(meshes, secret.trigger_radius),
                color_material(materials, WorldColor::Hazard),
                Transform::from_xyz(
                    secret.trigger_position.x + constants::ROOM_ORIGIN.x,
                    secret.trigger_position.y + constants::ROOM_ORIGIN.y,
                    constants::render_layers::ENTITIES,
                ),
            ));
        }

        if persistence.contains(RoomPersistenceCategory::Secret, key) {
            spawn_revealed_secret(
                commands,
                meshes,
                materials,
                key,
                secret.reveal_position + constants::ROOM_ORIGIN,
                secret.reveal_size,
                secret.reveal_label,
                secret.reveal_target,
                secret.reveal_spawn,
            );
        }
    }
}

fn spawn_npcs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    room: &RoomData,
) {
    for npc in &room.npcs {
        commands.spawn((
            Name::new("NPC"),
            RoomEntity,
            Npc,
            NpcDialogue(npc.dialogue.clone()),
            StaticBlocker,
            SolidBody {
                half_size: Vec2::new(8.0, 8.0),
            },
            Label("old man".to_string()),
            rectangle_mesh(meshes, Vec2::new(16.0, 16.0)),
            color_material(materials, WorldColor::UiText),
            Transform::from_xyz(
                npc.position.x + constants::ROOM_ORIGIN.x,
                npc.position.y + constants::ROOM_ORIGIN.y,
                constants::render_layers::ENTITIES,
            ),
        ));
    }
}

fn spawn_shop_items(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    persistence: &RoomPersistence,
    item_table: &ItemTable,
    room_id: RoomId,
    room: &RoomData,
) {
    for offer in &room.shop_offers {
        let key = PersistentRoomKey {
            room: room_id,
            key: offer.key,
        };
        if persistence.contains(RoomPersistenceCategory::UniquePickup, key) {
            continue;
        }
        let data = item_table.lookup(offer.kind);
        commands.spawn((
            Name::new("ShopItem"),
            RoomEntity,
            UniquePickup,
            ShopItem {
                kind: offer.kind,
                price: offer.price,
            },
            offer.kind,
            Label(format!("{} {}r", data.label, offer.price)),
            PersistentRoomEntity {
                key,
                category: RoomPersistenceCategory::UniquePickup,
            },
            circle_mesh(meshes, data.radius),
            MeshMaterial2d(materials.add(data.color)),
            Transform::from_xyz(
                offer.position.x + constants::ROOM_ORIGIN.x,
                offer.position.y + constants::ROOM_ORIGIN.y,
                constants::render_layers::PICKUPS,
            ),
        ));
    }
}

fn spawn_revealed_secret(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    key: PersistentRoomKey,
    position: Vec2,
    size: Vec2,
    label: &str,
    target: Option<RoomId>,
    target_spawn: Option<Vec2>,
) {
    let mut entity_commands = commands.spawn((
        Name::new("RevealedSecret"),
        RoomEntity,
        Label(label.to_string()),
        PersistentRoomEntity {
            key,
            category: RoomPersistenceCategory::Secret,
        },
        rectangle_mesh(meshes, size),
        color_material(materials, WorldColor::Accent),
        Transform::from_xyz(position.x, position.y, constants::render_layers::PICKUPS),
    ));

    if let (Some(room), Some(spawn)) = (target, target_spawn) {
        entity_commands.insert(StaircaseEntrance {
            target_room: room,
            target_spawn: spawn,
        });
    }
}

// ── Helpers ───────────────────────────────────────────────────────────

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

fn dungeon_for_room(room: RoomId) -> Option<DungeonId> {
    match room {
        RoomId::Dungeon1Entry
        | RoomId::Dungeon1North
        | RoomId::Dungeon1East
        | RoomId::Dungeon1Boss
        | RoomId::Dungeon1Triforce => Some(DungeonId::Dungeon1),
        _ => None,
    }
}

fn door_persist_key(direction: ExitDirection) -> &'static str {
    match direction {
        ExitDirection::North => "door_north",
        ExitDirection::South => "door_south",
        ExitDirection::East => "door_east",
        ExitDirection::West => "door_west",
    }
}
