use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::{GameRng, GameplayAssets};

pub fn spawn_player(commands: &mut Commands, assets: &GameplayAssets, world_x: f32) -> Entity {
    commands
        .spawn((
            Player,
            Mesh2d(assets.player_mesh.clone()),
            MeshMaterial2d(assets.player_material.clone()),
            Transform::from_xyz(0.0, 0.0, 5.0),
            WorldPosition(world_x),
            Velocity(Vec2::ZERO),
            FacingDirection(1.0),
            CollisionRadius(PLAYER_RADIUS),
            FireCooldown(Timer::from_seconds(FIRE_COOLDOWN, TimerMode::Once)),
        ))
        .id()
}

pub fn spawn_human(
    commands: &mut Commands,
    assets: &GameplayAssets,
    world_x: f32,
    ground_y: f32,
) -> Entity {
    commands
        .spawn((
            Human,
            Mesh2d(assets.human_mesh.clone()),
            MeshMaterial2d(assets.human_material.clone()),
            Transform::from_xyz(0.0, ground_y + 5.0, 2.0),
            WorldPosition(world_x),
            Velocity(Vec2::new(HUMAN_WALK_SPEED, 0.0)),
            CollisionRadius(HUMAN_RADIUS),
        ))
        .id()
}

pub fn spawn_lander(
    commands: &mut Commands,
    assets: &GameplayAssets,
    rng: &mut GameRng,
    world_x: f32,
) -> Entity {
    commands
        .spawn((
            Lander,
            Enemy,
            Mesh2d(assets.lander_mesh.clone()),
            MeshMaterial2d(assets.lander_material.clone()),
            Transform::from_xyz(0.0, CEILING_Y, 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::new(
                LANDER_HORIZONTAL_SPEED * rng.sign(),
                -LANDER_DESCENT_SPEED,
            )),
            CollisionRadius(LANDER_RADIUS),
            LanderState::Descending,
            LanderTarget(None),
            EnemyShootTimer(Timer::from_seconds(
                ENEMY_SHOOT_INTERVAL,
                TimerMode::Repeating,
            )),
        ))
        .id()
}

pub fn spawn_mutant(
    commands: &mut Commands,
    assets: &GameplayAssets,
    world_x: f32,
    y: f32,
) -> Entity {
    commands
        .spawn((
            Mutant,
            Enemy,
            Mesh2d(assets.mutant_mesh.clone()),
            MeshMaterial2d(assets.mutant_material.clone()),
            Transform::from_xyz(0.0, y, 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::ZERO),
            CollisionRadius(MUTANT_RADIUS),
            EnemyShootTimer(Timer::from_seconds(
                ENEMY_SHOOT_INTERVAL * 0.5,
                TimerMode::Repeating,
            )),
        ))
        .id()
}

pub fn spawn_bomber(
    commands: &mut Commands,
    assets: &GameplayAssets,
    rng: &mut GameRng,
    world_x: f32,
) -> Entity {
    commands
        .spawn((
            Bomber,
            Enemy,
            Mesh2d(assets.bomber_mesh.clone()),
            MeshMaterial2d(assets.bomber_material.clone()),
            Transform::from_xyz(0.0, rng.y_range(), 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::new(BOMBER_SPEED * rng.sign(), 0.0)),
            CollisionRadius(BOMBER_RADIUS),
            BomberDropTimer(Timer::from_seconds(
                BOMBER_DROP_INTERVAL,
                TimerMode::Repeating,
            )),
        ))
        .id()
}

pub fn spawn_pod(
    commands: &mut Commands,
    assets: &GameplayAssets,
    rng: &mut GameRng,
    world_x: f32,
) -> Entity {
    commands
        .spawn((
            Pod,
            Enemy,
            Mesh2d(assets.pod_mesh.clone()),
            MeshMaterial2d(assets.pod_material.clone()),
            Transform::from_xyz(0.0, rng.y_range(), 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::new(50.0 * rng.sign(), 20.0 * rng.sign())),
            CollisionRadius(POD_RADIUS),
        ))
        .id()
}

pub fn spawn_swarmer(
    commands: &mut Commands,
    assets: &GameplayAssets,
    world_x: f32,
    y: f32,
) -> Entity {
    commands
        .spawn((
            Swarmer,
            Enemy,
            Mesh2d(assets.swarmer_mesh.clone()),
            MeshMaterial2d(assets.swarmer_material.clone()),
            Transform::from_xyz(0.0, y, 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::ZERO),
            CollisionRadius(SWARMER_RADIUS),
        ))
        .id()
}

pub fn spawn_baiter(
    commands: &mut Commands,
    assets: &GameplayAssets,
    rng: &mut GameRng,
    world_x: f32,
) -> Entity {
    commands
        .spawn((
            Baiter,
            Enemy,
            Mesh2d(assets.baiter_mesh.clone()),
            MeshMaterial2d(assets.baiter_material.clone()),
            Transform::from_xyz(0.0, rng.y_range(), 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::ZERO),
            CollisionRadius(BAITER_RADIUS),
            EnemyShootTimer(Timer::from_seconds(
                ENEMY_SHOOT_INTERVAL * 0.6,
                TimerMode::Repeating,
            )),
        ))
        .id()
}

pub fn spawn_player_projectile(
    commands: &mut Commands,
    assets: &GameplayAssets,
    world_x: f32,
    y: f32,
    direction: f32,
) -> Entity {
    commands
        .spawn((
            PlayerProjectile,
            Mesh2d(assets.player_projectile_mesh.clone()),
            MeshMaterial2d(assets.player_projectile_material.clone()),
            Transform::from_xyz(0.0, y, 4.0),
            WorldPosition(world_x),
            Velocity(Vec2::new(PROJECTILE_SPEED * direction, 0.0)),
            CollisionRadius(PROJECTILE_RADIUS),
            Lifetime(Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once)),
        ))
        .id()
}

pub fn spawn_enemy_projectile(
    commands: &mut Commands,
    assets: &GameplayAssets,
    world_x: f32,
    y: f32,
    velocity: Vec2,
) -> Entity {
    commands
        .spawn((
            EnemyProjectile,
            Mesh2d(assets.enemy_projectile_mesh.clone()),
            MeshMaterial2d(assets.enemy_projectile_material.clone()),
            Transform::from_xyz(0.0, y, 4.0),
            WorldPosition(world_x),
            Velocity(velocity),
            CollisionRadius(PROJECTILE_RADIUS),
            Lifetime(Timer::from_seconds(2.0, TimerMode::Once)),
        ))
        .id()
}

pub fn spawn_mine(
    commands: &mut Commands,
    assets: &GameplayAssets,
    world_x: f32,
    y: f32,
) -> Entity {
    commands
        .spawn((
            Mine,
            Mesh2d(assets.mine_mesh.clone()),
            MeshMaterial2d(assets.mine_material.clone()),
            Transform::from_xyz(0.0, y, 4.0),
            WorldPosition(world_x),
            Velocity(Vec2::ZERO),
            CollisionRadius(MINE_RADIUS),
            Lifetime(Timer::from_seconds(MINE_LIFETIME, TimerMode::Once)),
        ))
        .id()
}

pub fn spawn_explosion(
    commands: &mut Commands,
    assets: &GameplayAssets,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
    y: f32,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Explosion(Timer::from_seconds(0.3, TimerMode::Once)),
            Mesh2d(assets.explosion_mesh.clone()),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(0.0, y, 10.0),
            WorldPosition(world_x),
        ))
        .id()
}
