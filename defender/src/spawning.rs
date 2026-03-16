use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;

pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
) -> Entity {
    let mesh = meshes.add(Triangle2d::new(
        Vec2::new(20.0, 0.0),
        Vec2::new(-10.0, -8.0),
        Vec2::new(-10.0, 8.0),
    ));
    commands
        .spawn((
            Player,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_PLAYER))),
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
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
    ground_y: f32,
) -> Entity {
    let mesh = meshes.add(Rectangle::new(4.0, 10.0));
    commands
        .spawn((
            Human,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_HUMAN))),
            Transform::from_xyz(0.0, ground_y + 5.0, 2.0),
            WorldPosition(world_x),
            Velocity(Vec2::new(HUMAN_WALK_SPEED, 0.0)),
            CollisionRadius(HUMAN_RADIUS),
        ))
        .id()
}

pub fn spawn_lander(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
) -> Entity {
    let mesh = meshes.add(Rectangle::new(14.0, 14.0));
    commands
        .spawn((
            Lander,
            Enemy,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_LANDER))),
            Transform::from_xyz(0.0, CEILING_Y, 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::new(
                LANDER_HORIZONTAL_SPEED * if rand_sign() { 1.0 } else { -1.0 },
                -LANDER_DESCENT_SPEED,
            )),
            CollisionRadius(LANDER_RADIUS),
            LanderState::Descending,
            LanderTarget(None),
            EnemyShootTimer(Timer::from_seconds(ENEMY_SHOOT_INTERVAL, TimerMode::Repeating)),
        ))
        .id()
}

pub fn spawn_mutant(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
    y: f32,
) -> Entity {
    let mesh = meshes.add(RegularPolygon::new(12.0, 5));
    commands
        .spawn((
            Mutant,
            Enemy,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_MUTANT))),
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
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
) -> Entity {
    let mesh = meshes.add(Rectangle::new(18.0, 10.0));
    commands
        .spawn((
            Bomber,
            Enemy,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_BOMBER))),
            Transform::from_xyz(0.0, rand_y_range(), 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::new(
                BOMBER_SPEED * if rand_sign() { 1.0 } else { -1.0 },
                0.0,
            )),
            CollisionRadius(BOMBER_RADIUS),
            BomberDropTimer(Timer::from_seconds(BOMBER_DROP_INTERVAL, TimerMode::Repeating)),
        ))
        .id()
}

pub fn spawn_pod(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
) -> Entity {
    let mesh = meshes.add(Circle::new(10.0));
    commands
        .spawn((
            Pod,
            Enemy,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_POD))),
            Transform::from_xyz(0.0, rand_y_range(), 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::new(
                50.0 * if rand_sign() { 1.0 } else { -1.0 },
                20.0 * if rand_sign() { 1.0 } else { -1.0 },
            )),
            CollisionRadius(POD_RADIUS),
        ))
        .id()
}

pub fn spawn_swarmer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
    y: f32,
) -> Entity {
    let mesh = meshes.add(Triangle2d::new(
        Vec2::new(8.0, 0.0),
        Vec2::new(-5.0, -4.0),
        Vec2::new(-5.0, 4.0),
    ));
    commands
        .spawn((
            Swarmer,
            Enemy,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_SWARMER))),
            Transform::from_xyz(0.0, y, 3.0),
            WorldPosition(world_x),
            Velocity(Vec2::ZERO),
            CollisionRadius(SWARMER_RADIUS),
        ))
        .id()
}

pub fn spawn_baiter(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
) -> Entity {
    let mesh = meshes.add(RegularPolygon::new(14.0, 4));
    commands
        .spawn((
            Baiter,
            Enemy,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_BAITER))),
            Transform::from_xyz(0.0, rand_y_range(), 3.0),
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
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
    y: f32,
    direction: f32,
) -> Entity {
    let mesh = meshes.add(Rectangle::new(14.0, 2.0));
    commands
        .spawn((
            PlayerProjectile,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_PLAYER_PROJECTILE))),
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
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
    y: f32,
    velocity: Vec2,
) -> Entity {
    let mesh = meshes.add(Rectangle::new(8.0, 3.0));
    commands
        .spawn((
            EnemyProjectile,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_ENEMY_PROJECTILE))),
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
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
    y: f32,
) -> Entity {
    let mesh = meshes.add(Circle::new(4.0));
    commands
        .spawn((
            Mine,
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_MINE))),
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
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    world_x: f32,
    y: f32,
    color: Color,
) -> Entity {
    let mesh = meshes.add(Circle::new(5.0));
    commands
        .spawn((
            Explosion(Timer::from_seconds(0.3, TimerMode::Once)),
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(0.0, y, 10.0),
            WorldPosition(world_x),
        ))
        .id()
}

// Simple pseudo-random helpers using system time
fn simple_rand() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    // Mix bits for better distribution
    let mixed = nanos.wrapping_mul(2654435761);
    (mixed as f32) / (u32::MAX as f32)
}

fn rand_sign() -> bool {
    simple_rand() > 0.5
}

fn rand_y_range() -> f32 {
    GROUND_Y + 50.0 + simple_rand() * (CEILING_Y - GROUND_Y - 100.0)
}

pub fn rand_world_x() -> f32 {
    simple_rand() * WORLD_WIDTH
}

pub fn rand_world_x_far_from(player_x: f32) -> f32 {
    let min_dist = WORLD_WIDTH * 0.2;
    loop {
        let x = rand_world_x();
        let dx = (x - player_x).abs().min(WORLD_WIDTH - (x - player_x).abs());
        if dx > min_dist {
            return x;
        }
    }
}
