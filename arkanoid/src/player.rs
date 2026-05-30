use bevy::prelude::*;

use crate::components::Paddle;
use crate::constants::*;
use crate::input::InputActions;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_paddle)
            .add_systems(FixedUpdate, paddle_control);
    }
}

fn spawn_paddle(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Paddle,
        Sprite::from_image(asset_server.load("sprites/vaus/vaus.png")),
        Transform::from_xyz(0.0, PADDLE_Y, Z_PADDLE),
    ));
}

/// Moves the Vaus from keyboard/gamepad (velocity-based), clamped to the playfield.
pub fn paddle_control(
    input: Res<InputActions>,
    time: Res<Time>,
    mut paddle: Query<&mut Transform, With<Paddle>>,
) {
    let Ok(mut transform) = paddle.single_mut() else {
        return;
    };

    let mut dx = 0.0;
    if input.move_left {
        dx -= PADDLE_SPEED * time.delta_secs();
    }
    if input.move_right {
        dx += PADDLE_SPEED * time.delta_secs();
    }

    let half = PADDLE_WIDTH / 2.0;
    transform.translation.x =
        (transform.translation.x + dx).clamp(PLAYFIELD_LEFT + half, PLAYFIELD_RIGHT - half);
}
