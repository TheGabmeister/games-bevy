use bevy::prelude::*;

use crate::audio::BounceSound;
use crate::bricks::BrickDestroyed;
use crate::components::{Ball, Brick, Paddle, Velocity};
use crate::constants::*;
use crate::states::AppState;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                ball_collision
                    .after(crate::ball::ball_movement)
                    .after(crate::player::paddle_control),
                ball_brick_collision.after(ball_collision),
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Reflects launched balls off the side/top walls and the paddle, and re-serves a
/// ball that falls past the open bottom. Emits [`BounceSound`] on each contact.
fn ball_collision(
    paddle: Query<&Transform, (With<Paddle>, Without<Ball>)>,
    mut balls: Query<(&mut Ball, &mut Transform, &mut Velocity), Without<Paddle>>,
    mut bounce: MessageWriter<BounceSound>,
) {
    let Ok(paddle_t) = paddle.single() else {
        return;
    };
    let paddle_half_w = PADDLE_WIDTH / 2.0;
    let paddle_top = paddle_t.translation.y + PADDLE_HEIGHT / 2.0;

    for (mut ball, mut transform, mut velocity) in &mut balls {
        if ball.stuck {
            continue;
        }

        let pos = &mut transform.translation;
        let mut hit_wall = false;

        if pos.x - BALL_RADIUS < PLAYFIELD_LEFT {
            pos.x = PLAYFIELD_LEFT + BALL_RADIUS;
            velocity.0.x = velocity.0.x.abs();
            hit_wall = true;
        }
        if pos.x + BALL_RADIUS > PLAYFIELD_RIGHT {
            pos.x = PLAYFIELD_RIGHT - BALL_RADIUS;
            velocity.0.x = -velocity.0.x.abs();
            hit_wall = true;
        }
        if pos.y + BALL_RADIUS > PLAYFIELD_TOP {
            pos.y = PLAYFIELD_TOP - BALL_RADIUS;
            velocity.0.y = -velocity.0.y.abs();
            hit_wall = true;
        }
        if hit_wall {
            bounce.write(BounceSound::Wall);
        }

        // Paddle: only when descending and overlapping the paddle's top face.
        let over_paddle = pos.x >= paddle_t.translation.x - paddle_half_w
            && pos.x <= paddle_t.translation.x + paddle_half_w;
        if velocity.0.y < 0.0
            && over_paddle
            && pos.y - BALL_RADIUS <= paddle_top
            && pos.y > paddle_t.translation.y
        {
            // Reflection angle depends on where the ball struck the paddle:
            // center → straight up, edges → BALL_MAX_BOUNCE_ANGLE off vertical.
            let offset = ((pos.x - paddle_t.translation.x) / paddle_half_w).clamp(-1.0, 1.0);
            let angle = offset * BALL_MAX_BOUNCE_ANGLE;
            velocity.0 = Vec2::new(angle.sin(), angle.cos()) * BALL_SPEED;
            pos.y = paddle_top + BALL_RADIUS;
            bounce.write(BounceSound::Paddle);
        }

        // Fell past the open bottom — re-serve on the paddle.
        if pos.y + BALL_RADIUS < PLAYFIELD_BOTTOM {
            ball.stuck = true;
            velocity.0 = Vec2::ZERO;
        }
    }
}

/// Destroys bricks the ball overlaps, deflecting it off the struck face and emitting
/// a [`BrickDestroyed`] message (scored elsewhere) plus a brick-break sound. Only the
/// first brick hit per ball per tick is resolved, to keep the deflection clean.
fn ball_brick_collision(
    mut commands: Commands,
    mut balls: Query<(&Ball, &mut Transform, &mut Velocity), Without<Brick>>,
    bricks: Query<(Entity, &Transform, &Brick), Without<Ball>>,
    mut destroyed: MessageWriter<BrickDestroyed>,
    mut bounce: MessageWriter<BounceSound>,
) {
    let half = Vec2::new(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0);
    for (ball, mut transform, mut velocity) in &mut balls {
        if ball.stuck {
            continue;
        }
        for (entity, brick_t, brick) in &bricks {
            let delta = transform.translation.truncate() - brick_t.translation.truncate();
            // Overlap of the ball (treated as an AABB of side 2*BALL_RADIUS) and brick.
            let overlap_x = (half.x + BALL_RADIUS) - delta.x.abs();
            let overlap_y = (half.y + BALL_RADIUS) - delta.y.abs();
            if overlap_x <= 0.0 || overlap_y <= 0.0 {
                continue;
            }

            // Reflect off whichever face is least penetrated, and nudge the ball out.
            if overlap_x < overlap_y {
                velocity.0.x = velocity.0.x.abs().copysign(delta.x);
                transform.translation.x += overlap_x.copysign(delta.x);
            } else {
                velocity.0.y = velocity.0.y.abs().copysign(delta.y);
                transform.translation.y += overlap_y.copysign(delta.y);
            }

            commands.entity(entity).despawn();
            destroyed.write(BrickDestroyed {
                points: brick.points,
            });
            bounce.write(BounceSound::Brick);
            break;
        }
    }
}
