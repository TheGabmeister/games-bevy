use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::audio::BounceSound;
use crate::bricks::BrickDestroyed;
use crate::components::{Ball, Paddle, Velocity};
use crate::constants::*;
use crate::input::InputActions;
use crate::resources::BallSpeed;
use crate::states::{AppState, PlayState};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_ball)
            // Each serve starts the ball at the base speed; it ramps up while running.
            .add_systems(OnEnter(PlayState::Serving), reset_ball_speed)
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
                    accelerate_ball
                        .after(ball_movement)
                        .run_if(in_state(PlayState::Running)),
                ),
            );
    }
}

/// Resets the ball speed to its base for a fresh serve (clears the accel timer and the
/// per-serve brick milestone counter).
fn reset_ball_speed(mut speed: ResMut<BallSpeed>) {
    *speed = BallSpeed::default();
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
    speed: Res<BallSpeed>,
) {
    if !input.launch {
        return;
    }
    for (mut ball, mut velocity) in &mut balls {
        if ball.stuck {
            ball.stuck = false;
            velocity.0 = Vec2::new(0.3, 1.0).normalize() * speed.current;
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

/// Ramps the ball's speed up within a round — on a fixed time cadence and at brick-count
/// milestones — up to [`BALL_SPEED_MAX`]. Each bump rescales the live balls' velocity
/// (preserving direction) and plays a speed-up cue.
pub fn accelerate_ball(
    time: Res<Time>,
    mut speed: ResMut<BallSpeed>,
    mut destroyed: MessageReader<BrickDestroyed>,
    mut balls: Query<(&Ball, &mut Velocity)>,
    mut cue: MessageWriter<BounceSound>,
) {
    // Already at the cap — just drain the milestone reader so it can't backlog.
    if speed.current >= BALL_SPEED_MAX {
        destroyed.clear();
        return;
    }

    let mut bumps = 0u32;

    // Time-based: one bump per acceleration interval elapsed this tick.
    speed.timer.tick(time.delta());
    bumps += speed.timer.times_finished_this_tick();

    // Milestone-based: one bump each time the round's destroyed count crosses a multiple
    // of BALL_SPEEDUP_BRICKS.
    let before = speed.bricks_destroyed;
    let n = destroyed.read().count() as u32;
    if n > 0 {
        speed.bricks_destroyed += n;
        bumps += speed.bricks_destroyed / BALL_SPEEDUP_BRICKS - before / BALL_SPEEDUP_BRICKS;
    }

    if bumps == 0 {
        return;
    }

    let new_speed = (speed.current + BALL_SPEEDUP_STEP * bumps as f32).min(BALL_SPEED_MAX);
    if new_speed > speed.current {
        speed.current = new_speed;
        for (ball, mut velocity) in &mut balls {
            if !ball.stuck && velocity.0.length() > f32::EPSILON {
                velocity.0 = velocity.0.normalize() * speed.current;
            }
        }
        cue.write(BounceSound::SpeedUp);
    }
}
