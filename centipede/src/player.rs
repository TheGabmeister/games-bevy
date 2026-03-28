use bevy::prelude::*;

use crate::{
    components::{Bullet, Player},
    constants::*,
    resources::RespawnTimer,
    scheduling::GameplaySet,
    states::AppState,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    player_movement.in_set(GameplaySet::Input),
                    player_shoot.in_set(GameplaySet::Input),
                    handle_respawn.in_set(GameplaySet::Cleanup),
                ),
            );
    }
}

fn spawn_player_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let triangle = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 12.0),
        Vec2::new(-10.0, -10.0),
        Vec2::new(10.0, -10.0),
    ));

    commands.spawn((
        Mesh2d(triangle),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.8, 1.0))),
        Transform::from_xyz(0.0, grid_to_world_y(GRID_ROWS - 2), 1.0),
        Player,
        DespawnOnExit(AppState::Playing),
    ));
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_player_entity(&mut commands, &mut meshes, &mut materials);
}

fn player_movement(
    mut query: Query<&mut Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    respawn: Res<RespawnTimer>,
) {
    if respawn.0.is_some() {
        return;
    }
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut dx = 0.0;
    let mut dy = 0.0;

    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        dx -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        dx += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        dy += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        dy -= 1.0;
    }

    let delta = Vec2::new(dx, dy).normalize_or_zero() * PLAYER_SPEED * time.delta_secs();
    transform.translation.x += delta.x;
    transform.translation.y += delta.y;

    // Clamp to player zone
    let half_w = WINDOW_WIDTH / 2.0 - CELL_SIZE / 2.0;
    transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
    transform.translation.y = transform
        .translation
        .y
        .clamp(player_zone_min_y() + 12.0, player_zone_max_y() - 12.0);
}

fn player_shoot(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_q: Query<&Transform, With<Player>>,
    bullet_q: Query<(), With<Bullet>>,
    keys: Res<ButtonInput<KeyCode>>,
    respawn: Res<RespawnTimer>,
) {
    if respawn.0.is_some() {
        return;
    }
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    if !bullet_q.is_empty() {
        return; // only one bullet at a time
    }
    let Ok(transform) = player_q.single() else {
        return;
    };

    let mesh = meshes.add(Rectangle::new(4.0, 14.0));
    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 0.3))),
        Transform::from_xyz(transform.translation.x, transform.translation.y + 20.0, 1.0),
        Bullet,
        DespawnOnExit(AppState::Playing),
    ));
}

fn handle_respawn(
    mut commands: Commands,
    mut respawn: ResMut<RespawnTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    let Some(timer) = &mut respawn.0 else {
        return;
    };
    timer.tick(time.delta());
    if timer.just_finished() {
        respawn.0 = None;
        spawn_player_entity(&mut commands, &mut meshes, &mut materials);
    }
}
