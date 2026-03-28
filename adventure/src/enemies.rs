use bevy::prelude::*;
use crate::components::*;
use crate::world::*;

const DRAGON_HW: f32 = 12.0;
const DRAGON_HH: f32 = 10.0;
const ROOM_BOUND_X: f32 = ROOM_W / 2.0 - WALL_T - DRAGON_HW;
const ROOM_BOUND_Y: f32 = ROOM_H / 2.0 - WALL_T - DRAGON_HH;

pub fn dragon_ai(
    time: Res<Time>,
    current_room: Res<CurrentRoom>,
    player_q: Query<&Transform, With<Player>>,
    inventory: Res<PlayerInventory>,
    item_q: Query<&ItemKind, With<Item>>,
    mut dragon_q: Query<(&mut Transform, &DragonData, &InRoom), (With<Dragon>, With<DragonAlive>, Without<Player>)>,
) {
    let Ok(p_transform) = player_q.single() else { return; };
    let p_pos = p_transform.translation.truncate();

    let player_has_sword = inventory.item
        .and_then(|e| item_q.get(e).ok())
        .map(|k| *k == ItemKind::Sword)
        .unwrap_or(false);

    for (mut transform, data, in_room) in dragon_q.iter_mut() {
        if in_room.0 != current_room.0 { continue; }

        let d_pos = transform.translation.truncate();
        let to_player = p_pos - d_pos;
        let dir = if to_player.length() > 0.1 { to_player.normalize() } else { Vec2::ZERO };
        let speed = data.kind.speed();

        let velocity = if player_has_sword { -dir * speed } else { dir * speed };
        let mut new_pos = d_pos + velocity * time.delta_secs();

        // Clamp to room bounds
        new_pos.x = new_pos.x.clamp(-ROOM_BOUND_X, ROOM_BOUND_X);
        new_pos.y = new_pos.y.clamp(-ROOM_BOUND_Y, ROOM_BOUND_Y);

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;

        // Rotate head to face movement direction
        // (head child entity update is done in update_dragon_heads)
    }
}

/// Update dragon head rotation to face movement direction.
pub fn update_dragon_heads(
    dragon_q: Query<(&Transform, &Children), (With<Dragon>, With<DragonAlive>)>,
    player_q: Query<&Transform, With<Player>>,
    _current_room: Res<CurrentRoom>,
    _in_room_q: Query<&InRoom, With<Dragon>>,
    inventory: Res<PlayerInventory>,
    item_q: Query<&ItemKind, With<Item>>,
    mut head_q: Query<&mut Transform, (With<DragonHead>, Without<Dragon>, Without<Player>)>,
) {
    let Ok(p_transform) = player_q.single() else { return; };
    let p_pos = p_transform.translation.truncate();

    let player_has_sword = inventory.item
        .and_then(|e| item_q.get(e).ok())
        .map(|k| *k == ItemKind::Sword)
        .unwrap_or(false);

    for (d_transform, children) in dragon_q.iter() {
        // Find head child
        let head_entity = children.iter().find(|&c| head_q.contains(c));
        let Some(head_entity) = head_entity else { continue; };

        let d_pos = d_transform.translation.truncate();
        let to_player = p_pos - d_pos;
        if to_player.length() < 0.1 { continue; }
        let dir = if player_has_sword { -to_player.normalize() } else { to_player.normalize() };
        let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;

        if let Ok(mut head_transform) = head_q.get_mut(head_entity) {
            let offset = dir * 14.0;
            head_transform.translation = Vec3::new(offset.x, offset.y, 0.1);
            head_transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

/// Player touching a living dragon without sword → swallow animation then GameOver.
pub fn dragon_collision(
    mut commands: Commands,
    current_room: Res<CurrentRoom>,
    player_q: Query<&Transform, With<Player>>,
    inventory: Res<PlayerInventory>,
    item_q: Query<&ItemKind, With<Item>>,
    dragon_q: Query<(Entity, &Transform, &InRoom), (With<Dragon>, With<DragonAlive>)>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    let player_has_sword = inventory.item
        .and_then(|e| item_q.get(e).ok())
        .map(|k| *k == ItemKind::Sword)
        .unwrap_or(false);

    if player_has_sword { return; }

    let Ok(p_transform) = player_q.single() else { return; };
    let p_pos = p_transform.translation.truncate();

    for (dragon_entity, d_transform, in_room) in dragon_q.iter() {
        if in_room.0 != current_room.0 { continue; }
        let d_pos = d_transform.translation.truncate();
        if p_pos.distance(d_pos) < 20.0 {
            commands.insert_resource(SwallowInfo {
                dragon: dragon_entity,
                timer: Timer::from_seconds(0.8, TimerMode::Once),
            });
            next_state.set(crate::AppState::Swallowed);
            return;
        }
    }
}

/// Animate the dragon swallowing the player, then transition to GameOver.
pub fn swallow_animation(
    time: Res<Time>,
    mut swallow: ResMut<SwallowInfo>,
    mut dragon_q: Query<&mut Transform, With<Dragon>>,
    mut player_q: Query<&mut Visibility, With<Player>>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    swallow.timer.tick(time.delta());

    // Hide player (inside dragon's mouth)
    if let Ok(mut vis) = player_q.single_mut() {
        *vis = Visibility::Hidden;
    }

    // Scale up dragon to show swallowing
    if let Ok(mut transform) = dragon_q.get_mut(swallow.dragon) {
        let progress = swallow.timer.fraction();
        let scale = 1.0 + progress * 0.5;
        transform.scale = Vec3::splat(scale);
    }

    if swallow.timer.just_finished() {
        next_state.set(crate::AppState::GameOver);
    }
}

/// Bat can revive dead dragons by touching them.
pub fn bat_revive_dragons(
    mut commands: Commands,
    bat_q: Query<(&Transform, &InRoom), With<Bat>>,
    dead_dragons: Query<(Entity, &Transform, &InRoom, &DragonData, &Children), (With<Dragon>, Without<DragonAlive>)>,
    mut body_q: Query<&mut MeshMaterial2d<ColorMaterial>, With<DragonBody>>,
    mut head_q: Query<&mut MeshMaterial2d<ColorMaterial>, (With<DragonHead>, Without<DragonBody>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok((bat_transform, bat_room)) = bat_q.single() else { return; };
    let bat_pos = bat_transform.translation.truncate();

    for (entity, d_transform, d_room, data, children) in dead_dragons.iter() {
        if d_room.0 != bat_room.0 { continue; }
        let d_pos = d_transform.translation.truncate();
        if bat_pos.distance(d_pos) < 30.0 {
            // Revive dragon
            commands.entity(entity).insert(DragonAlive);

            // Restore original color
            let mat = materials.add(data.kind.color());
            for &child in children {
                if let Ok(mut body_mat) = body_q.get_mut(child) {
                    *body_mat = MeshMaterial2d(mat.clone());
                }
                if let Ok(mut head_mat) = head_q.get_mut(child) {
                    *head_mat = MeshMaterial2d(mat.clone());
                }
            }
        }
    }
}

/// Player with sword touching a living dragon → dragon dies.
pub fn sword_combat(
    mut commands: Commands,
    current_room: Res<CurrentRoom>,
    player_q: Query<&Transform, With<Player>>,
    inventory: Res<PlayerInventory>,
    item_q: Query<&ItemKind, With<Item>>,
    dragon_q: Query<(Entity, &Transform, &InRoom, &Children), (With<Dragon>, With<DragonAlive>)>,
    dead_mat: Res<DeadDragonMaterial>,
    mut body_q: Query<&mut MeshMaterial2d<ColorMaterial>, With<DragonBody>>,
    mut head_q: Query<&mut MeshMaterial2d<ColorMaterial>, (With<DragonHead>, Without<DragonBody>)>,
) {
    let player_has_sword = inventory.item
        .and_then(|e| item_q.get(e).ok())
        .map(|k| *k == ItemKind::Sword)
        .unwrap_or(false);

    if !player_has_sword { return; }

    let Ok(p_transform) = player_q.single() else { return; };
    let p_pos = p_transform.translation.truncate();

    for (dragon_entity, d_transform, in_room, children) in dragon_q.iter() {
        if in_room.0 != current_room.0 { continue; }
        let d_pos = d_transform.translation.truncate();
        if p_pos.distance(d_pos) < 24.0 {
            // Kill dragon
            commands.entity(dragon_entity)
                .remove::<DragonAlive>();

            // Turn body and head gray
            for &child in children {
                if let Ok(mut mat) = body_q.get_mut(child) {
                    *mat = MeshMaterial2d(dead_mat.0.clone());
                }
                if let Ok(mut mat) = head_q.get_mut(child) {
                    *mat = MeshMaterial2d(dead_mat.0.clone());
                }
            }
        }
    }
}

pub fn bat_ai(
    time: Res<Time>,
    mut bat_q: Query<(&mut Transform, &mut InRoom, &mut BatData), With<Bat>>,
    mut item_q: Query<(Entity, &mut Transform, &mut InRoom, &mut Visibility), (With<Item>, Without<Bat>, Without<Carried>)>,
    inventory: Res<PlayerInventory>,
) {
    let Ok((mut bat_transform, mut bat_room, mut bat_data)) = bat_q.single_mut() else { return; };

    bat_data.wander_timer.tick(time.delta());
    bat_data.grab_timer.tick(time.delta());

    // Wander: random movement each tick
    if bat_data.wander_timer.just_finished() {
        // Pseudo-random using time
        let t = time.elapsed_secs();
        let dx = (t * 3.7).sin() * 60.0;
        let dy = (t * 2.3).cos() * 60.0;
        let new_x = (bat_transform.translation.x + dx).clamp(-350.0, 350.0);
        let new_y = (bat_transform.translation.y + dy).clamp(-250.0, 250.0);
        bat_transform.translation.x = new_x;
        bat_transform.translation.y = new_y;
    }

    // If bat holds item, keep it at bat position
    if let Some(held) = bat_data.held_item {
        // Check the item still exists and isn't carried by player
        if inventory.item == Some(held) {
            // Player picked it up from bat — release it
            bat_data.held_item = None;
        } else if let Ok((_, mut it, _, _)) = item_q.get_mut(held) {
            it.translation = bat_transform.translation.with_z(2.0);
        }
    }

    if bat_data.grab_timer.just_finished() {
        if bat_data.held_item.is_some() {
            // Drop item in random room
            let t = time.elapsed_secs();
            let target_room = ((t * 17.3).abs() as u8) % 13;

            if let Some(held) = bat_data.held_item {
                if let Ok((_, mut it, mut it_room, mut vis)) = item_q.get_mut(held) {
                    let tx = (t * 5.1).sin() * 150.0;
                    let ty = (t * 4.3).cos() * 100.0;
                    it.translation = Vec3::new(tx, ty, 2.0);
                    it_room.0 = target_room;
                    *vis = Visibility::Hidden; // will be updated by visibility system
                    bat_data.held_item = None;
                }
            }

            // Move bat to new room too
            bat_room.0 = target_room;
        } else {
            // Grab a random item from any room (not carried by player)
            let t = time.elapsed_secs();
            let items: Vec<Entity> = item_q
                .iter()
                .filter(|(e, _, _, _)| inventory.item != Some(*e))
                .map(|(e, _, _, _)| e)
                .collect();

            if !items.is_empty() {
                let idx = ((t * 11.7).abs() as usize) % items.len();
                let target = items[idx];
                bat_data.held_item = Some(target);

                // Move bat to item's room
                if let Ok((_, it_transform, it_room, _)) = item_q.get(target) {
                    bat_room.0 = it_room.0;
                    bat_transform.translation = it_transform.translation.with_z(4.0);
                }
            }
        }
    }
}
