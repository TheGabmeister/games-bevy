use bevy::prelude::*;

use crate::components::{Player, Velocity};
use crate::constants::{PLAYER_SIZE, PLAYER_SPEED, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::states::AppState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(OnExit(AppState::Playing), cleanup_player)
            .add_systems(
                Update,
                (player_input, player_movement.after(player_input))
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        Velocity::default(),
        Sprite::from_image(asset_server.load("player_ship.png")),
        Transform::from_translation(Vec3::new(0.0, -WINDOW_HEIGHT / 2.0 + 60.0, 0.0)),
    ));
}

fn player_input(
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

fn player_movement(
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

fn cleanup_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
