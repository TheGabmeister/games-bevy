use bevy::prelude::*;

use crate::components::{Ball, Paddle, Velocity};
use crate::constants::*;
use crate::input::InputActions;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ball)
            .add_systems(Update, ball_launch)
            .add_systems(
                FixedUpdate,
                (
                    // Keep a stuck ball glued to the paddle after it has moved.
                    ball_follow_paddle.after(crate::player::paddle_control),
                    ball_movement,
                ),
            );
    }
}

fn spawn_ball(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Ball { stuck: true },
        Velocity(Vec2::ZERO),
        Sprite::from_image(asset_server.load("sprites/ball/ball.png")),
        Transform::from_xyz(0.0, ball_rest_y(PADDLE_Y), Z_BALL),
    ));
}

/// World-space Y where the ball rests on top of a paddle centered at `paddle_y`.
fn ball_rest_y(paddle_y: f32) -> f32 {
    paddle_y + PADDLE_HEIGHT / 2.0 + BALL_RADIUS
}

/// While stuck, the ball tracks the paddle's position so it launches from wherever
/// the Vaus currently sits.
pub fn ball_follow_paddle(
    paddle: Query<&Transform, (With<Paddle>, Without<Ball>)>,
    mut balls: Query<(&Ball, &mut Transform), Without<Paddle>>,
) {
    let Ok(paddle_t) = paddle.single() else {
        return;
    };
    for (ball, mut transform) in &mut balls {
        if ball.stuck {
            transform.translation.x = paddle_t.translation.x;
            transform.translation.y = ball_rest_y(paddle_t.translation.y);
        }
    }
}

/// Releases a stuck ball upward (slightly angled) when the launch action fires.
pub fn ball_launch(input: Res<InputActions>, mut balls: Query<(&mut Ball, &mut Velocity)>) {
    if !input.launch {
        return;
    }
    for (mut ball, mut velocity) in &mut balls {
        if ball.stuck {
            ball.stuck = false;
            velocity.0 = Vec2::new(0.3, 1.0).normalize() * BALL_SPEED;
        }
    }
}

/// Integrates the ball's position from its velocity (launched balls only).
pub fn ball_movement(time: Res<Time>, mut balls: Query<(&Ball, &mut Transform, &Velocity)>) {
    let dt = time.delta_secs();
    for (ball, mut transform, velocity) in &mut balls {
        if !ball.stuck {
            transform.translation.x += velocity.0.x * dt;
            transform.translation.y += velocity.0.y * dt;
        }
    }
}
