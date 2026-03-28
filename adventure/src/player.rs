use bevy::prelude::*;
use crate::components::*;
use crate::world::*;

pub fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    room_walls: Res<RoomWalls>,
    bypass: Res<WallBypass>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = player_q.single_mut() else { return; };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowUp)    || keys.pressed(KeyCode::KeyW) { dir.y += 1.0; }
    if keys.pressed(KeyCode::ArrowDown)  || keys.pressed(KeyCode::KeyS) { dir.y -= 1.0; }
    if keys.pressed(KeyCode::ArrowLeft)  || keys.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) { dir.x += 1.0; }

    if dir == Vec2::ZERO { return; }

    let delta = dir.normalize() * PLAYER_SPEED * time.delta_secs();
    let pos = transform.translation.truncate();

    // Try X axis
    let new_x = pos.x + delta.x;
    if !collides_walls(new_x, pos.y, PLAYER_HW, PLAYER_HH, &room_walls.0, &bypass) {
        transform.translation.x = new_x;
    }

    // Try Y axis
    let cur_x = transform.translation.x;
    let new_y = pos.y + delta.y;
    if !collides_walls(cur_x, new_y, PLAYER_HW, PLAYER_HH, &room_walls.0, &bypass) {
        transform.translation.y = new_y;
    }
}

fn collides_walls(px: f32, py: f32, phw: f32, phh: f32, walls: &[WallRect], bypass: &WallBypass) -> bool {
    walls.iter().any(|w| {
        if !w.overlaps(px, py, phw, phh) {
            return false;
        }
        // Bridge bypass: if bridge overlaps this wall and player is within bridge area, allow passage
        if let Some(ref b) = bypass.bridge {
            if w.overlaps(b.x, b.y, b.hw, b.hh) && b.overlaps(px, py, phw, phh) {
                return false;
            }
        }
        // Easter egg: bypass north wall when carrying Dot in Room 6
        if bypass.easter_egg_north {
            let north_wall_y = ROOM_H / 2.0 - WALL_T / 2.0;
            if (w.y - north_wall_y).abs() < 1.0 && px.abs() < PASSAGE_W / 2.0 {
                return false;
            }
        }
        true
    })
}

/// Compute wall bypass info each frame (bridge position + easter egg).
pub fn compute_wall_bypass(
    current_room: Res<CurrentRoom>,
    inventory: Res<PlayerInventory>,
    ground_items: Query<(&Transform, &InRoom, &ItemKind), (With<Item>, Without<Carried>, Without<Player>)>,
    carried_q: Query<&ItemKind, With<Carried>>,
    mut bypass: ResMut<WallBypass>,
) {
    // Bridge: find bridge on ground in current room — use generous bypass area (48x32)
    bypass.bridge = ground_items.iter()
        .find(|(_, room, kind)| room.0 == current_room.0 && **kind == ItemKind::Bridge)
        .map(|(t, _, _)| WallRect::new(t.translation.x, t.translation.y, 48.0, 32.0));

    // Easter egg: carrying Dot in Room 6
    let carrying_dot = inventory.item
        .and_then(|e| carried_q.get(e).ok())
        .map(|k| *k == ItemKind::Dot)
        .unwrap_or(false);
    bypass.easter_egg_north = current_room.0 == 6 && carrying_dot;
}

/// Magnet attracts nearby items toward it (whether carried or on the ground).
pub fn magnet_pull(
    time: Res<Time>,
    inventory: Res<PlayerInventory>,
    current_room: Res<CurrentRoom>,
    player_q: Query<&Transform, With<Player>>,
    carried_q: Query<&ItemKind, With<Carried>>,
    mut items_q: Query<(Entity, &mut Transform, &InRoom, &ItemKind), (With<Item>, Without<Carried>, Without<Player>)>,
) {
    // Determine magnet position and room
    let carrying_magnet = inventory.item
        .and_then(|e| carried_q.get(e).ok())
        .map(|k| *k == ItemKind::Magnet)
        .unwrap_or(false);

    let (mag_pos, mag_room) = if carrying_magnet {
        let Ok(p_transform) = player_q.single() else { return; };
        (p_transform.translation.truncate(), current_room.0)
    } else {
        // Find magnet on ground
        let found = items_q.iter()
            .find(|(_, _, _, kind)| **kind == ItemKind::Magnet)
            .map(|(_, t, room, _)| (t.translation.truncate(), room.0));
        match found {
            Some(info) => info,
            None => return,
        }
    };

    // Pull items in same room toward magnet
    for (entity, mut transform, in_room, kind) in items_q.iter_mut() {
        if *kind == ItemKind::Magnet { continue; }
        if in_room.0 != mag_room { continue; }
        if inventory.item == Some(entity) { continue; }

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

/// Auto-pickup: player walks over item → pick it up if inventory empty.
pub fn item_pickup(
    mut commands: Commands,
    player_q: Query<&Transform, With<Player>>,
    mut inventory: ResMut<PlayerInventory>,
    current_room: Res<CurrentRoom>,
    items: Query<(Entity, &Transform, &InRoom), (With<Item>, Without<Carried>, Without<Player>)>,
) {
    if inventory.item.is_some() { return; }

    let Ok(p_transform) = player_q.single() else { return; };
    let p_pos = p_transform.translation.truncate();

    for (entity, i_transform, in_room) in items.iter() {
        if in_room.0 != current_room.0 { continue; }
        let i_pos = i_transform.translation.truncate();
        if p_pos.distance(i_pos) < 20.0 {
            inventory.item = Some(entity);
            commands.entity(entity).insert(Carried);
            break;
        }
    }
}

/// Drop carried item at player's feet on Space or E press.
pub fn item_drop(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    player_q: Query<&Transform, With<Player>>,
    mut inventory: ResMut<PlayerInventory>,
    current_room: Res<CurrentRoom>,
    mut item_q: Query<(&mut Transform, &mut InRoom), (With<Item>, With<Carried>, Without<Player>)>,
) {
    if !keys.just_pressed(KeyCode::Space) && !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Some(item_entity) = inventory.item else { return; };
    let Ok(p_transform) = player_q.single() else { return; };

    // Drop item at player position
    if let Ok((mut item_transform, mut item_in_room)) = item_q.get_mut(item_entity) {
        item_transform.translation = p_transform.translation.with_z(2.0);
        item_in_room.0 = current_room.0;
        commands.entity(item_entity).remove::<Carried>();
    }
    inventory.item = None;
}

/// Move carried item to follow player.
pub fn carry_item_follow(
    player_q: Query<&Transform, With<Player>>,
    inventory: Res<PlayerInventory>,
    mut item_q: Query<&mut Transform, (With<Item>, With<Carried>, Without<Player>)>,
) {
    let Some(item_entity) = inventory.item else { return; };
    let Ok(p_transform) = player_q.single() else { return; };
    if let Ok(mut it) = item_q.get_mut(item_entity) {
        it.translation = p_transform.translation.with_z(2.0) + Vec3::new(12.0, 12.0, 0.0);
    }
}

/// Open gates when player with matching key approaches.
pub fn gate_interaction(
    mut commands: Commands,
    player_q: Query<&Transform, With<Player>>,
    inventory: Res<PlayerInventory>,
    current_room: Res<CurrentRoom>,
    item_q: Query<&ItemKind, With<Item>>,
    mut gates: Query<(Entity, &Transform, &mut GateData, &InRoom), With<Gate>>,
) {
    let Ok(p_transform) = player_q.single() else { return; };
    let p_pos = p_transform.translation.truncate();

    // What key is the player carrying?
    let carried_key_color = inventory.item
        .and_then(|e| item_q.get(e).ok())
        .and_then(|kind| kind.key_color());

    let Some(key_color) = carried_key_color else { return; };

    for (gate_entity, g_transform, mut gate_data, gate_room) in gates.iter_mut() {
        if gate_room.0 != current_room.0 { continue; }
        if gate_data.open { continue; }
        if gate_data.key_color != key_color { continue; }

        let g_pos = g_transform.translation.truncate();
        if p_pos.distance(g_pos) < 50.0 {
            gate_data.open = true;
            commands.entity(gate_entity).insert(Visibility::Hidden);
        }
    }
}

/// Win condition: player in Room 0 carrying Chalice.
pub fn check_win(
    player_q: Query<&InRoom, With<Player>>,
    inventory: Res<PlayerInventory>,
    item_q: Query<&ItemKind, With<Item>>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    let Ok(in_room) = player_q.single() else { return; };
    if in_room.0 != 0 { return; }

    let has_chalice = inventory.item
        .and_then(|e| item_q.get(e).ok())
        .map(|k| *k == ItemKind::Chalice)
        .unwrap_or(false);

    if has_chalice {
        next_state.set(crate::AppState::Win);
    }
}
