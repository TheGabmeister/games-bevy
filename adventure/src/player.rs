use bevy::prelude::*;
use crate::components::*;
use crate::world::*;

pub fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    room_walls: Res<RoomWalls>,
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
    if !collides_walls(new_x, pos.y, PLAYER_HW, PLAYER_HH, &room_walls.0) {
        transform.translation.x = new_x;
    }

    // Try Y axis
    let cur_x = transform.translation.x;
    let new_y = pos.y + delta.y;
    if !collides_walls(cur_x, new_y, PLAYER_HW, PLAYER_HH, &room_walls.0) {
        transform.translation.y = new_y;
    }
}

fn collides_walls(px: f32, py: f32, phw: f32, phh: f32, walls: &[WallRect]) -> bool {
    walls.iter().any(|w| w.overlaps(px, py, phw, phh))
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
