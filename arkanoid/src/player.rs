use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::Paddle;
use crate::constants::*;
use crate::input::InputActions;
use crate::states::AppState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_paddle)
            .add_systems(
                FixedUpdate,
                paddle_control.run_if(in_state(AppState::Playing)),
            );
    }
}

fn spawn_paddle(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Paddle {
            half_width: PADDLE_WIDTH / 2.0,
        },
        Sprite::from_image(assets.sprites.vaus.clone()),
        Transform::from_xyz(0.0, PADDLE_Y, Z_PADDLE),
        DespawnOnExit(AppState::Playing),
    ));
}

/// Moves the Vaus from keyboard/gamepad (velocity-based), clamped to the playfield. The
/// clamp uses the paddle's current `half_width` so an expanded Vaus still stays in bounds.
pub fn paddle_control(
    input: Res<InputActions>,
    time: Res<Time>,
    mut paddle: Query<(&mut Transform, &Paddle)>,
) {
    let Ok((mut transform, paddle)) = paddle.single_mut() else {
        return;
    };

    let mut dx = 0.0;
    if input.move_left {
        dx -= PADDLE_SPEED * time.delta_secs();
    }
    if input.move_right {
        dx += PADDLE_SPEED * time.delta_secs();
    }

    let half = paddle.half_width;
    transform.translation.x = (transform.translation.x + dx)
        .clamp(PLAYFIELD_LEFT + half, PLAYFIELD_RIGHT - half);
}
