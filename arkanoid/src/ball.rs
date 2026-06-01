use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{Ball, Paddle, Velocity};
use crate::constants::*;
use crate::input::InputActions;
use crate::states::{AppState, PlayState};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_ball)
            .add_systems(Update, ball_launch.run_if(in_state(PlayState::Serving)))
            .add_systems(
                FixedUpdate,
                (
                    // A stuck ball tracks the paddle through Ready/Serving.
                    ball_follow_paddle
                        .after(crate::player::paddle_control)
                        .run_if(in_state(AppState::Playing)),
                    // The launched ball only integrates while actually running.
                    ball_movement.run_if(in_state(PlayState::Running)),
                ),
            );
    }
}

fn spawn_ball(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Ball { stuck: true },
        Velocity(Vec2::ZERO),
        Sprite::from_image(assets.sprites.ball.clone()),
        Transform::from_xyz(0.0, ball_rest_y(PADDLE_Y), Z_BALL),
        DespawnOnExit(AppState::Playing),
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

/// Releases a stuck ball upward (slightly angled) when the launch action fires, and
/// moves play into the `Running` state.
pub fn ball_launch(
    input: Res<InputActions>,
    mut balls: Query<(&mut Ball, &mut Velocity)>,
    mut next: ResMut<NextState<PlayState>>,
) {
    if !input.launch {
        return;
    }
    for (mut ball, mut velocity) in &mut balls {
        if ball.stuck {
            ball.stuck = false;
            velocity.0 = Vec2::new(0.3, 1.0).normalize() * BALL_SPEED;
        }
    }
    next.set(PlayState::Running);
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
