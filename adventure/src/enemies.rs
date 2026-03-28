use bevy::{ecs::system::SystemParam, prelude::*};

use crate::components::*;
use crate::world::*;
use crate::{AppState, PlayingSet};

const DRAGON_HW: f32 = 12.0;
const DRAGON_HH: f32 = 10.0;
const ROOM_BOUND_X: f32 = ROOM_W / 2.0 - WALL_T - DRAGON_HW;
const ROOM_BOUND_Y: f32 = ROOM_H / 2.0 - WALL_T - DRAGON_HH;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                dragon_ai,
                update_dragon_heads,
                sword_combat,
                dragon_collision,
                bat_ai,
                bat_revive_dragons,
            )
                .chain()
                .in_set(PlayingSet::Enemies),
        )
        .add_systems(
            Update,
            swallow_animation.run_if(in_state(AppState::Swallowed)),
        );
    }
}

type LivingDragonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static DragonData, &'static InRoom),
    (With<Dragon>, With<DragonAlive>, Without<Player>),
>;
type LivingDragonHeadsQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static Children, &'static InRoom),
    (With<Dragon>, With<DragonAlive>),
>;
type DragonHeadTransformQuery<'w, 's> =
    Query<'w, 's, &'static mut Transform, (With<DragonHead>, Without<Dragon>, Without<Player>)>;
type LivingDragonCollisionQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform, &'static InRoom), (With<Dragon>, With<DragonAlive>)>;
type DeadDragonQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static InRoom,
        &'static DragonData,
        &'static Children,
    ),
    (With<Dragon>, Without<DragonAlive>),
>;
type LivingDragonChildrenQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static InRoom,
        &'static Children,
    ),
    (With<Dragon>, With<DragonAlive>),
>;
type FreeItemQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static mut InRoom,
        &'static mut Visibility,
    ),
    (With<Item>, Without<Bat>, Without<Carried>),
>;

#[derive(SystemParam)]
pub struct DragonVisuals<'w, 's> {
    dead_material: Res<'w, DeadDragonMaterial>,
    body_q: Query<'w, 's, &'static mut MeshMaterial2d<ColorMaterial>, With<DragonBody>>,
    head_q: Query<
        'w,
        's,
        &'static mut MeshMaterial2d<ColorMaterial>,
        (With<DragonHead>, Without<DragonBody>),
    >,
}

impl<'w, 's> DragonVisuals<'w, 's> {
    fn dead_material(&self) -> &Handle<ColorMaterial> {
        &self.dead_material.0
    }

    fn set_children_material(&mut self, children: &Children, material: &Handle<ColorMaterial>) {
        for &child in children {
            if let Ok(mut body_mat) = self.body_q.get_mut(child) {
                *body_mat = MeshMaterial2d(material.clone());
            }
            if let Ok(mut head_mat) = self.head_q.get_mut(child) {
                *head_mat = MeshMaterial2d(material.clone());
            }
        }
    }
}

pub fn dragon_ai(
    time: Res<Time>,
    current_room: Res<CurrentRoom>,
    player_q: Query<&Transform, With<Player>>,
    inventory_items: InventoryItems<'_, '_>,
    mut dragon_q: LivingDragonQuery<'_, '_>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let player_has_sword = inventory_items.is_carrying(ItemKind::Sword);

    for (mut transform, data, in_room) in dragon_q.iter_mut() {
        if in_room.0 != current_room.0 {
            continue;
        }

        let dragon_pos = transform.translation.truncate();
        let to_player = player_pos - dragon_pos;
        let dir = if to_player.length() > 0.1 {
            to_player.normalize()
        } else {
            Vec2::ZERO
        };
        let velocity = if player_has_sword {
            -dir * data.kind.speed()
        } else {
            dir * data.kind.speed()
        };

        let mut new_pos = dragon_pos + velocity * time.delta_secs();
        new_pos.x = new_pos.x.clamp(-ROOM_BOUND_X, ROOM_BOUND_X);
        new_pos.y = new_pos.y.clamp(-ROOM_BOUND_Y, ROOM_BOUND_Y);
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

pub fn update_dragon_heads(
    current_room: Res<CurrentRoom>,
    dragon_q: LivingDragonHeadsQuery<'_, '_>,
    player_q: Query<&Transform, With<Player>>,
    inventory_items: InventoryItems<'_, '_>,
    mut head_q: DragonHeadTransformQuery<'_, '_>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let player_has_sword = inventory_items.is_carrying(ItemKind::Sword);

    for (dragon_transform, children, in_room) in dragon_q.iter() {
        if in_room.0 != current_room.0 {
            continue;
        }

        let head_entity = children.iter().find(|&child| head_q.contains(child));
        let Some(head_entity) = head_entity else {
            continue;
        };

        let dragon_pos = dragon_transform.translation.truncate();
        let to_player = player_pos - dragon_pos;
        if to_player.length() < 0.1 {
            continue;
        }

        let dir = if player_has_sword {
            -to_player.normalize()
        } else {
            to_player.normalize()
        };
        let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;

        if let Ok(mut head_transform) = head_q.get_mut(head_entity) {
            let offset = dir * 14.0;
            head_transform.translation = Vec3::new(offset.x, offset.y, 0.1);
            head_transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

pub fn dragon_collision(
    mut commands: Commands,
    current_room: Res<CurrentRoom>,
    player_q: Query<&Transform, With<Player>>,
    inventory_items: InventoryItems<'_, '_>,
    dragon_q: LivingDragonCollisionQuery<'_, '_>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    if inventory_items.is_carrying(ItemKind::Sword) {
        return;
    }

    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (dragon_entity, dragon_transform, in_room) in dragon_q.iter() {
        if in_room.0 != current_room.0 {
            continue;
        }

        if player_pos.distance(dragon_transform.translation.truncate()) < 20.0 {
            commands.insert_resource(SwallowInfo {
                dragon: dragon_entity,
                timer: Timer::from_seconds(0.8, TimerMode::Once),
            });
            next_state.set(crate::AppState::Swallowed);
            return;
        }
    }
}

pub fn swallow_animation(
    time: Res<Time>,
    mut swallow: ResMut<SwallowInfo>,
    mut dragon_q: Query<&mut Transform, With<Dragon>>,
    mut player_q: Query<&mut Visibility, With<Player>>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    swallow.timer.tick(time.delta());

    if let Ok(mut visibility) = player_q.single_mut() {
        *visibility = Visibility::Hidden;
    }

    if let Ok(mut transform) = dragon_q.get_mut(swallow.dragon) {
        let progress = swallow.timer.fraction();
        transform.scale = Vec3::splat(1.0 + progress * 0.5);
    }

    if swallow.timer.just_finished() {
        next_state.set(crate::AppState::GameOver);
    }
}

pub fn bat_revive_dragons(
    mut commands: Commands,
    bat_q: Query<(&Transform, &InRoom), With<Bat>>,
    dead_dragons: DeadDragonQuery<'_, '_>,
    mut dragon_visuals: DragonVisuals<'_, '_>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok((bat_transform, bat_room)) = bat_q.single() else {
        return;
    };
    let bat_pos = bat_transform.translation.truncate();

    for (entity, dragon_transform, dragon_room, data, children) in dead_dragons.iter() {
        if dragon_room.0 != bat_room.0 {
            continue;
        }

        if bat_pos.distance(dragon_transform.translation.truncate()) < 30.0 {
            commands.entity(entity).insert(DragonAlive);
            let material = materials.add(data.kind.color());
            dragon_visuals.set_children_material(children, &material);
        }
    }
}

pub fn sword_combat(
    mut commands: Commands,
    current_room: Res<CurrentRoom>,
    player_q: Query<&Transform, With<Player>>,
    inventory_items: InventoryItems<'_, '_>,
    dragon_q: LivingDragonChildrenQuery<'_, '_>,
    mut dragon_visuals: DragonVisuals<'_, '_>,
) {
    if !inventory_items.is_carrying(ItemKind::Sword) {
        return;
    }

    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (dragon_entity, dragon_transform, in_room, children) in dragon_q.iter() {
        if in_room.0 != current_room.0 {
            continue;
        }

        if player_pos.distance(dragon_transform.translation.truncate()) < 24.0 {
            commands.entity(dragon_entity).remove::<DragonAlive>();
            let dead_material = dragon_visuals.dead_material().clone();
            dragon_visuals.set_children_material(children, &dead_material);
        }
    }
}

pub fn bat_ai(
    time: Res<Time>,
    mut bat_q: Query<(&mut Transform, &mut InRoom, &mut BatData), With<Bat>>,
    mut item_q: FreeItemQuery<'_, '_>,
    inventory_items: InventoryItems<'_, '_>,
    world: Res<WorldMap>,
) {
    let Ok((mut bat_transform, mut bat_room, mut bat_data)) = bat_q.single_mut() else {
        return;
    };

    bat_data.wander_timer.tick(time.delta());
    bat_data.grab_timer.tick(time.delta());

    if bat_data.wander_timer.just_finished() {
        let t = time.elapsed_secs();
        let dx = (t * 3.7).sin() * 60.0;
        let dy = (t * 2.3).cos() * 60.0;
        bat_transform.translation.x = (bat_transform.translation.x + dx).clamp(-350.0, 350.0);
        bat_transform.translation.y = (bat_transform.translation.y + dy).clamp(-250.0, 250.0);
    }

    if let Some(held) = bat_data.held_item {
        if inventory_items.carried_entity() == Some(held) {
            bat_data.held_item = None;
        } else if let Ok((_, mut item_transform, _, _)) = item_q.get_mut(held) {
            item_transform.translation = bat_transform.translation.with_z(2.0);
        }
    }

    if bat_data.grab_timer.just_finished() {
        if bat_data.held_item.is_some() {
            let t = time.elapsed_secs();
            let target_room = ((t * 17.3).abs() as u8) % world.room_count();

            if let Some(held) = bat_data.held_item
                && let Ok((_, mut item_transform, mut item_room, mut visibility)) =
                    item_q.get_mut(held)
            {
                item_transform.translation =
                    Vec3::new((t * 5.1).sin() * 150.0, (t * 4.3).cos() * 100.0, 2.0);
                item_room.0 = target_room;
                *visibility = Visibility::Hidden;
                bat_data.held_item = None;
            }

            bat_room.0 = target_room;
        } else {
            let t = time.elapsed_secs();
            let items: Vec<Entity> = item_q
                .iter()
                .filter(|(entity, _, _, _)| inventory_items.carried_entity() != Some(*entity))
                .map(|(entity, _, _, _)| entity)
                .collect();

            if !items.is_empty() {
                let target = items[((t * 11.7).abs() as usize) % items.len()];
                bat_data.held_item = Some(target);

                if let Ok((_, item_transform, item_room, _)) = item_q.get(target) {
                    bat_room.0 = item_room.0;
                    bat_transform.translation = item_transform.translation.with_z(4.0);
                }
            }
        }
    }
}
