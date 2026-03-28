use bevy::prelude::*;

use crate::components::*;
use crate::world::*;
use crate::{AppState, PlayingEnterSet, PlayingSet};

pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Playing),
            (spawn_room_walls, update_background_color, update_visibility)
                .chain()
                .in_set(PlayingEnterSet::RoomState),
        )
        .add_systems(Update, room_transition.in_set(PlayingSet::Room))
        .add_systems(
            Update,
            (spawn_room_walls, update_background_color, update_visibility)
                .chain()
                .in_set(PlayingSet::Presentation)
                .run_if(current_room_changed),
        );
    }
}

type GatePresenceQuery<'w, 's> =
    Query<'w, 's, (&'static GateData, &'static InRoom), (With<Gate>, Without<Player>)>;

pub fn current_room_changed(current_room: Res<CurrentRoom>) -> bool {
    current_room.is_changed()
}

pub fn spawn_room_walls(
    mut commands: Commands,
    current_room: Res<CurrentRoom>,
    world: Res<WorldMap>,
    mut room_walls: ResMut<RoomWalls>,
    old_walls: Query<Entity, With<RoomWallMarker>>,
) {
    for entity in old_walls.iter() {
        commands.entity(entity).despawn();
    }

    let walls = build_room_walls(world.room(current_room.0));

    for wall in &walls {
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.5, 0.5, 0.5),
                Vec2::new(wall.hw * 2.0, wall.hh * 2.0),
            ),
            Transform::from_xyz(wall.x, wall.y, 1.0),
            GameEntity,
            RoomWallMarker,
        ));
    }

    room_walls.0 = walls;
}

pub fn update_background_color(
    current_room: Res<CurrentRoom>,
    world: Res<WorldMap>,
    mut clear_color: ResMut<ClearColor>,
) {
    clear_color.0 = world.room(current_room.0).color;
}

pub fn update_visibility(
    current_room: Res<CurrentRoom>,
    mut q: Query<(&InRoom, &mut Visibility), Without<Carried>>,
) {
    for (in_room, mut visibility) in q.iter_mut() {
        *visibility = if in_room.0 == current_room.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn room_transition(
    mut player_q: Query<(&mut Transform, &mut InRoom), With<Player>>,
    world: Res<WorldMap>,
    mut current_room: ResMut<CurrentRoom>,
    gates: GatePresenceQuery<'_, '_>,
    inventory_items: InventoryItems<'_, '_>,
) {
    let Ok((mut transform, mut player_room)) = player_q.single_mut() else {
        return;
    };

    let pos = transform.translation.truncate();
    let room_def = world.room(current_room.0);

    let half_w = ROOM_W / 2.0;
    let half_h = ROOM_H / 2.0;
    let threshold_h = half_h - PASSAGE_THRESHOLD;
    let threshold_w = half_w - PASSAGE_THRESHOLD;
    let half_p = PASSAGE_W / 2.0 - PLAYER_HW;

    let directions = [
        (
            pos.y > threshold_h && pos.x.abs() < half_p,
            0usize,
            pos.x,
            -(threshold_h - 4.0),
        ),
        (
            pos.y < -threshold_h && pos.x.abs() < half_p,
            1usize,
            pos.x,
            threshold_h - 4.0,
        ),
        (
            pos.x > threshold_w && pos.y.abs() < half_p,
            2usize,
            -(threshold_w - 4.0),
            pos.y,
        ),
        (
            pos.x < -threshold_w && pos.y.abs() < half_p,
            3usize,
            threshold_w - 4.0,
            pos.y,
        ),
    ];

    for (triggered, exit_idx, new_x, new_y) in directions {
        if !triggered {
            continue;
        }

        let Some(dest_room) = room_def.exits[exit_idx] else {
            continue;
        };

        let gate_blocked = gates.iter().any(|(gate_data, gate_room)| {
            gate_room.0 == current_room.0
                && gate_data.exit_dir.index() == exit_idx
                && !gate_data.open
        });
        if gate_blocked {
            continue;
        }

        current_room.0 = dest_room;
        player_room.0 = dest_room;
        transform.translation.x = new_x;
        transform.translation.y = new_y;
        return;
    }

    if current_room.0 == 6
        && pos.y > threshold_h
        && pos.x.abs() < half_p
        && inventory_items.is_carrying(ItemKind::Dot)
    {
        current_room.0 = 13;
        player_room.0 = 13;
        transform.translation.y = -(threshold_h - 4.0);
    }
}
