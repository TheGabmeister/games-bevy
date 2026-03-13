use bevy::prelude::*;
use crate::components::*;
use crate::world::*;

/// Despawn old room walls and spawn new ones when CurrentRoom changes.
pub fn spawn_room_walls(
    mut commands: Commands,
    current_room: Res<CurrentRoom>,
    world: Res<WorldMap>,
    mut room_walls: ResMut<RoomWalls>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    old_walls: Query<Entity, With<RoomWallMarker>>,
) {
    if !current_room.is_changed() {
        return;
    }

    // Despawn old wall meshes
    for e in old_walls.iter() {
        commands.entity(e).despawn();
    }

    let room = world.room(current_room.0);
    let walls = build_room_walls(room);

    let wall_color = Color::srgb(0.5, 0.5, 0.5);

    for w in &walls {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(w.hw * 2.0, w.hh * 2.0))),
            MeshMaterial2d(materials.add(wall_color)),
            Transform::from_xyz(w.x, w.y, 1.0),
            RoomWallMarker,
        ));
    }

    room_walls.0 = walls;
}

/// Update background ClearColor when room changes.
pub fn update_background_color(
    current_room: Res<CurrentRoom>,
    world: Res<WorldMap>,
    mut clear_color: ResMut<ClearColor>,
) {
    if !current_room.is_changed() {
        return;
    }
    clear_color.0 = world.room(current_room.0).color;
}

/// Show entities in current room, hide others.
/// Skips Carried items (they follow the player regardless of room).
pub fn update_visibility(
    current_room: Res<CurrentRoom>,
    mut q: Query<(&InRoom, &mut Visibility), Without<Carried>>,
) {
    if !current_room.is_changed() {
        return;
    }
    for (in_room, mut vis) in q.iter_mut() {
        *vis = if in_room.0 == current_room.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Detect player reaching a passage edge and transition to adjacent room.
pub fn room_transition(
    mut player_q: Query<(&mut Transform, &mut InRoom), With<Player>>,
    world: Res<WorldMap>,
    mut current_room: ResMut<CurrentRoom>,
    gates: Query<(&GateData, &InRoom), (With<Gate>, Without<Player>)>,
) {
    let Ok((mut transform, mut player_room)) = player_q.single_mut() else { return; };

    let pos = transform.translation.truncate();
    let room_def = world.room(current_room.0);

    let half_w = ROOM_W / 2.0;
    let half_h = ROOM_H / 2.0;
    let threshold_h = half_h - PASSAGE_THRESHOLD;
    let threshold_w = half_w - PASSAGE_THRESHOLD;
    let half_p = PASSAGE_W / 2.0 - PLAYER_HW;

    // Check each direction
    let directions = [
        // (condition, exit_index, new_x, new_y)
        (pos.y > threshold_h && pos.x.abs() < half_p,  0usize, pos.x, -(threshold_h - 4.0)),
        (pos.y < -threshold_h && pos.x.abs() < half_p, 1usize, pos.x,  threshold_h - 4.0),
        (pos.x > threshold_w && pos.y.abs() < half_p,  2usize, -(threshold_w - 4.0), pos.y),
        (pos.x < -threshold_w && pos.y.abs() < half_p, 3usize,  threshold_w - 4.0, pos.y),
    ];

    for (triggered, exit_idx, new_x, new_y) in directions {
        if !triggered {
            continue;
        }
        let Some(dest_room) = room_def.exits[exit_idx] else { continue; };

        // Check if a gate blocks this exit
        let gate_blocked = gates.iter().any(|(gate_data, gate_in_room)| {
            gate_in_room.0 == current_room.0
                && gate_data.exit_dir as usize == exit_idx
                && !gate_data.open
        });
        if gate_blocked {
            continue;
        }

        // Transition
        current_room.0 = dest_room;
        player_room.0 = dest_room;
        transform.translation.x = new_x;
        transform.translation.y = new_y;
        break;
    }
}
