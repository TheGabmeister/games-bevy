use bevy::prelude::*;

use crate::PlayingSet;
use crate::components::*;
use crate::world::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, compute_wall_bypass.in_set(PlayingSet::Prepare))
            .add_systems(Update, player_movement.in_set(PlayingSet::Movement))
            .add_systems(
                Update,
                (
                    item_drop,
                    item_pickup,
                    carry_item_follow,
                    gate_interaction,
                    magnet_pull,
                )
                    .chain()
                    .in_set(PlayingSet::Interaction),
            )
            .add_systems(Update, check_win.in_set(PlayingSet::WinCheck));
    }
}

type GroundItemsQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static InRoom, &'static ItemKind),
    (With<Item>, Without<Carried>, Without<Player>),
>;
type GroundItemsMutQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static InRoom,
        &'static ItemKind,
    ),
    (With<Item>, Without<Carried>, Without<Player>),
>;
type PickupItemsQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform, &'static InRoom),
    (With<Item>, Without<Carried>, Without<Player>),
>;
type CarriedItemRoomQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut InRoom),
    (With<Item>, With<Carried>, Without<Player>),
>;
type CarriedItemTransformQuery<'w, 's> =
    Query<'w, 's, &'static mut Transform, (With<Item>, With<Carried>, Without<Player>)>;

pub fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    room_walls: Res<RoomWalls>,
    bypass: Res<WallBypass>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = player_q.single_mut() else {
        return;
    };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }

    if dir == Vec2::ZERO {
        return;
    }

    let delta = dir.normalize() * PLAYER_SPEED * time.delta_secs();
    let pos = transform.translation.truncate();

    let new_x = pos.x + delta.x;
    if !collides_walls(new_x, pos.y, PLAYER_HW, PLAYER_HH, &room_walls.0, &bypass) {
        transform.translation.x = new_x;
    }

    let cur_x = transform.translation.x;
    let new_y = pos.y + delta.y;
    if !collides_walls(cur_x, new_y, PLAYER_HW, PLAYER_HH, &room_walls.0, &bypass) {
        transform.translation.y = new_y;
    }
}

fn collides_walls(
    px: f32,
    py: f32,
    phw: f32,
    phh: f32,
    walls: &[WallRect],
    bypass: &WallBypass,
) -> bool {
    walls.iter().any(|wall| {
        if !wall.overlaps(px, py, phw, phh) {
            return false;
        }

        if let Some(ref bridge) = bypass.bridge
            && wall.overlaps(bridge.x, bridge.y, bridge.hw, bridge.hh)
            && bridge.overlaps(px, py, phw, phh)
        {
            return false;
        }

        if bypass.easter_egg_north {
            let north_wall_y = ROOM_H / 2.0 - WALL_T / 2.0;
            if (wall.y - north_wall_y).abs() < 1.0 && px.abs() < PASSAGE_W / 2.0 {
                return false;
            }
        }

        true
    })
}

pub fn compute_wall_bypass(
    current_room: Res<CurrentRoom>,
    inventory_items: InventoryItems<'_, '_>,
    ground_items: GroundItemsQuery<'_, '_>,
    mut bypass: ResMut<WallBypass>,
) {
    bypass.bridge = ground_items
        .iter()
        .find(|(_, room, kind)| room.0 == current_room.0 && **kind == ItemKind::Bridge)
        .map(|(transform, _, _)| {
            WallRect::new(transform.translation.x, transform.translation.y, 48.0, 32.0)
        });

    bypass.easter_egg_north = current_room.0 == 6 && inventory_items.is_carrying(ItemKind::Dot);
}

pub fn magnet_pull(
    time: Res<Time>,
    current_room: Res<CurrentRoom>,
    player_q: Query<&Transform, With<Player>>,
    inventory_items: InventoryItems<'_, '_>,
    mut items_q: GroundItemsMutQuery<'_, '_>,
) {
    let carrying_magnet = inventory_items.is_carrying(ItemKind::Magnet);

    let (mag_pos, mag_room) = if carrying_magnet {
        let Ok(player_transform) = player_q.single() else {
            return;
        };
        (player_transform.translation.truncate(), current_room.0)
    } else {
        let Some((_, transform, room, _)) = items_q
            .iter_mut()
            .find(|(_, _, _, kind)| **kind == ItemKind::Magnet)
        else {
            return;
        };
        (transform.translation.truncate(), room.0)
    };

    for (entity, mut transform, in_room, kind) in items_q.iter_mut() {
        if *kind == ItemKind::Magnet {
            continue;
        }
        if in_room.0 != mag_room {
            continue;
        }
        if inventory_items.carried_entity() == Some(entity) {
            continue;
        }

        let item_pos = transform.translation.truncate();
        let to_magnet = mag_pos - item_pos;
        let dist = to_magnet.length();
        if dist > 3.0 && dist < 200.0 {
            let pull_strength = 40.0 * (1.0 - dist / 200.0);
            let pull = to_magnet.normalize() * pull_strength * time.delta_secs();
            transform.translation.x += pull.x;
            transform.translation.y += pull.y;
        }
    }
}

pub fn item_pickup(
    mut commands: Commands,
    player_q: Query<&Transform, With<Player>>,
    mut inventory: ResMut<PlayerInventory>,
    current_room: Res<CurrentRoom>,
    items: PickupItemsQuery<'_, '_>,
) {
    if inventory.item.is_some() {
        return;
    }

    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, item_transform, in_room) in items.iter() {
        if in_room.0 != current_room.0 {
            continue;
        }
        if player_pos.distance(item_transform.translation.truncate()) < 20.0 {
            inventory.item = Some(entity);
            commands.entity(entity).insert(Carried);
            break;
        }
    }
}

pub fn item_drop(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    player_q: Query<&Transform, With<Player>>,
    mut inventory: ResMut<PlayerInventory>,
    current_room: Res<CurrentRoom>,
    mut item_q: CarriedItemRoomQuery<'_, '_>,
) {
    if !keys.just_pressed(KeyCode::Space) && !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Some(item_entity) = inventory.item else {
        return;
    };
    let Ok(player_transform) = player_q.single() else {
        return;
    };

    if let Ok((mut item_transform, mut item_in_room)) = item_q.get_mut(item_entity) {
        item_transform.translation = player_transform.translation.with_z(2.0);
        item_in_room.0 = current_room.0;
        commands.entity(item_entity).remove::<Carried>();
    }
    inventory.item = None;
}

pub fn carry_item_follow(
    player_q: Query<&Transform, With<Player>>,
    inventory: Res<PlayerInventory>,
    mut item_q: CarriedItemTransformQuery<'_, '_>,
) {
    let Some(item_entity) = inventory.item else {
        return;
    };
    let Ok(player_transform) = player_q.single() else {
        return;
    };

    if let Ok(mut transform) = item_q.get_mut(item_entity) {
        transform.translation =
            player_transform.translation.with_z(2.0) + Vec3::new(12.0, 12.0, 0.0);
    }
}

pub fn gate_interaction(
    mut commands: Commands,
    player_q: Query<&Transform, With<Player>>,
    inventory_items: InventoryItems<'_, '_>,
    current_room: Res<CurrentRoom>,
    mut gates: Query<(Entity, &Transform, &mut GateData, &InRoom), With<Gate>>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let Some(key_color) = inventory_items.carried_key_color() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (gate_entity, gate_transform, mut gate_data, gate_room) in gates.iter_mut() {
        if gate_room.0 != current_room.0 || gate_data.open || gate_data.key_color != key_color {
            continue;
        }

        if player_pos.distance(gate_transform.translation.truncate()) < 50.0 {
            gate_data.open = true;
            commands.entity(gate_entity).insert(Visibility::Hidden);
        }
    }
}

pub fn check_win(
    player_q: Query<&InRoom, With<Player>>,
    inventory_items: InventoryItems<'_, '_>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    let Ok(in_room) = player_q.single() else {
        return;
    };

    if in_room.0 == 0 && inventory_items.is_carrying(ItemKind::Chalice) {
        next_state.set(crate::AppState::Win);
    }
}
