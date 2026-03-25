use bevy::prelude::*;

use crate::components::{Player, Velocity};
use crate::constants::{PLAYER_COLOR, PLAYER_SIZE, PLAYER_SPEED, WINDOW_HEIGHT, WINDOW_WIDTH};

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Player,
        Velocity::default(),
        Mesh2d(meshes.add(Rectangle::new(PLAYER_SIZE, PLAYER_SIZE))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(PLAYER_COLOR))),
        Transform::from_translation(Vec3::ZERO),
    ));
}

pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let Ok(mut velocity) = query.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    velocity.0 = direction.normalize_or_zero() * PLAYER_SPEED;
}

pub fn player_movement(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<Player>>,
) {
    let Ok((velocity, mut transform)) = query.single_mut() else {
        return;
    };

    transform.translation.x += velocity.0.x * time.delta_secs();
    transform.translation.y += velocity.0.y * time.delta_secs();

    // Clamp to window bounds
    let half_w = WINDOW_WIDTH / 2.0 - PLAYER_SIZE / 2.0;
    let half_h = WINDOW_HEIGHT / 2.0 - PLAYER_SIZE / 2.0;
    transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
    transform.translation.y = transform.translation.y.clamp(-half_h, half_h);
}

pub fn reset_player(mut query: Query<(&mut Transform, &mut Velocity), With<Player>>) {
    let Ok((mut transform, mut velocity)) = query.single_mut() else {
        return;
    };

    transform.translation = Vec3::ZERO;
    velocity.0 = Vec2::ZERO;
}
