use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_shared_assets)
            .add_systems(
                Update,
                (wing_animation_system, facing_update_system, egg_animation_system)
                    .run_if(in_state(crate::states::AppState::Playing)),
            );
    }
}

fn setup_shared_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let platform_meshes = PLATFORMS.map(|p| meshes.add(Rectangle::new(p.width, PLATFORM_THICKNESS)));
    let lava_height = (LAVA_Y - ARENA_BOTTOM).abs() + 80.0;

    commands.insert_resource(SharedMeshes {
        rect_body: meshes.add(Rectangle::new(30.0, 18.0)),
        circle_head: meshes.add(Circle::new(6.0)),
        rect_lance: meshes.add(Rectangle::new(22.0, 3.0)),
        rect_wing: meshes.add(Rectangle::new(16.0, 6.0)),
        circle_egg: meshes.add(Circle::new(EGG_RADIUS)),
        circle_particle: meshes.add(Circle::new(3.0)),
        rect_platform: platform_meshes,
        rect_lava: meshes.add(Rectangle::new(ARENA_WIDTH + WRAP_MARGIN * 2.0, lava_height)),
    });

    commands.insert_resource(SharedMaterials {
        player1_body: materials.add(Color::srgb(0.2, 0.5, 1.0)),
        player2_body: materials.add(Color::srgb(1.0, 0.3, 0.3)),
        bounder_body: materials.add(Color::srgb(0.8, 0.2, 0.15)),
        hunter_body: materials.add(Color::srgb(0.55, 0.55, 0.65)),
        shadow_lord_body: materials.add(Color::srgb(0.45, 0.1, 0.55)),
        lance: materials.add(Color::srgb(0.9, 0.85, 0.2)),
        head: materials.add(Color::srgb(0.85, 0.75, 0.55)),
        egg: materials.add(Color::srgb(0.95, 0.93, 0.85)),
        egg_hatch: materials.add(Color::srgb(1.0, 0.5, 0.3)),
        platform: materials.add(Color::srgb(0.35, 0.3, 0.25)),
        lava_front: materials.add(Color::srgb(1.0, 0.35, 0.0)),
        lava_back: materials.add(Color::srgb(0.7, 0.15, 0.0)),
    });
}

/// Spawns rider visual hierarchy (body, head, lance, wing). Returns the parent entity.
/// Caller is responsible for inserting gameplay components on the returned entity.
pub fn spawn_rider_visual(
    commands: &mut Commands,
    meshes: &SharedMeshes,
    materials: &SharedMaterials,
    position: Vec2,
    z_layer: f32,
    body_mat: Handle<ColorMaterial>,
) -> Entity {
    commands
        .spawn((
            Transform::from_xyz(position.x, position.y, z_layer),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Body
            parent.spawn((
                Mesh2d(meshes.rect_body.clone()),
                MeshMaterial2d(body_mat),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            // Head
            parent.spawn((
                Mesh2d(meshes.circle_head.clone()),
                MeshMaterial2d(materials.head.clone()),
                Transform::from_xyz(10.0, 12.0, 0.1),
            ));
            // Lance
            parent.spawn((
                Mesh2d(meshes.rect_lance.clone()),
                MeshMaterial2d(materials.lance.clone()),
                Transform::from_xyz(22.0, 6.0, 0.1),
            ));
            // Wing
            parent.spawn((
                Mesh2d(meshes.rect_wing.clone()),
                MeshMaterial2d(materials.lance.clone()),
                Transform::from_xyz(-2.0, 10.0, 0.2),
                Wing { base_y: 10.0 },
            ));
        })
        .id()
}

pub fn spawn_egg_entity(
    commands: &mut Commands,
    meshes: &SharedMeshes,
    materials: &SharedMaterials,
    position: Vec2,
    tier: EnemyTier,
    hatch_time: f32,
) -> Entity {
    commands
        .spawn((
            Mesh2d(meshes.circle_egg.clone()),
            MeshMaterial2d(materials.egg.clone()),
            Transform::from_xyz(position.x, position.y, Z_EGGS),
            Egg {
                tier,
                hatch_timer: Timer::from_seconds(hatch_time, TimerMode::Once),
            },
            Velocity::default(),
            PreviousPosition(position),
            DespawnOnExit(crate::states::AppState::Playing),
        ))
        .id()
}

pub fn spawn_arena(
    commands: &mut Commands,
    meshes: &SharedMeshes,
    materials: &SharedMaterials,
) {
    let lava_center_y = (LAVA_Y + ARENA_BOTTOM) * 0.5;

    // Platforms
    for (i, plat) in PLATFORMS.iter().enumerate() {
        commands.spawn((
            Mesh2d(meshes.rect_platform[i].clone()),
            MeshMaterial2d(materials.platform.clone()),
            Transform::from_xyz(plat.center_x, plat.y, Z_PLATFORMS),
            DespawnOnExit(crate::states::AppState::Playing),
        ));
    }

    // Lava back layer
    commands.spawn((
        Mesh2d(meshes.rect_lava.clone()),
        MeshMaterial2d(materials.lava_back.clone()),
        Transform::from_xyz(0.0, lava_center_y - 10.0, Z_LAVA_BACK),
        DespawnOnExit(crate::states::AppState::Playing),
    ));

    // Lava front layer
    commands.spawn((
        Mesh2d(meshes.rect_lava.clone()),
        MeshMaterial2d(materials.lava_front.clone()),
        Transform::from_xyz(0.0, lava_center_y, Z_LAVA_FRONT),
        DespawnOnExit(crate::states::AppState::Playing),
    ));
}

fn wing_animation_system(
    time: Res<Time>,
    parents: Query<&Velocity>,
    mut wings: Query<(&ChildOf, &Wing, &mut Transform)>,
) {
    for (parent, wing, mut transform) in &mut wings {
        let Ok(velocity) = parents.get(parent.get()) else {
            continue;
        };
        let t = time.elapsed_secs() * 12.0;
        if velocity.0.y > 50.0 {
            transform.translation.y = wing.base_y - 4.0;
            transform.rotation = Quat::from_rotation_z(-0.4);
        } else if velocity.0.y < -50.0 {
            transform.translation.y = wing.base_y + 4.0;
            transform.rotation = Quat::from_rotation_z(0.3);
        } else {
            transform.translation.y = wing.base_y + t.sin() * 2.0;
            transform.rotation = Quat::from_rotation_z(t.sin() * 0.15);
        }
    }
}

fn facing_update_system(
    mut query: Query<(&FacingDirection, &mut Transform), (Changed<FacingDirection>, Without<Wing>)>,
) {
    for (facing, mut transform) in &mut query {
        transform.scale.x = match facing {
            FacingDirection::Right => 1.0,
            FacingDirection::Left => -1.0,
        };
    }
}

fn egg_animation_system(
    materials: Res<SharedMaterials>,
    mut eggs: Query<(&Egg, &mut Transform, &mut MeshMaterial2d<ColorMaterial>)>,
) {
    for (egg, mut transform, mut material) in &mut eggs {
        let progress = egg.hatch_timer.fraction();

        transform.scale = Vec3::splat(1.0 + progress * 0.12);
        material.0 = if progress > 0.7 && ((progress * 20.0) as i32 % 2 == 0) {
            materials.egg_hatch.clone()
        } else {
            materials.egg.clone()
        };
    }
}
