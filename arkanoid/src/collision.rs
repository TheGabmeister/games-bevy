use bevy::prelude::*;

use crate::audio::BounceSound;
use crate::bricks::BrickDestroyed;
use crate::components::{Ball, Brick, Indestructible, Paddle, Velocity};
use crate::constants::*;
use crate::flow::BallLost;
use crate::resources::{BallSpeed, PaddleMode};
use crate::schedule::Physics;
use crate::states::PlayState;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                ball_collision.in_set(Physics::Collision),
                ball_brick_collision
                    .in_set(Physics::Collision)
                    .after(ball_collision),
            )
                .run_if(in_state(PlayState::Running)),
        );
    }
}

/// Reflects launched balls off the side/top walls and the paddle, and re-serves a
/// ball that falls past the open bottom. Emits [`BounceSound`] on each contact.
fn ball_collision(
    mut commands: Commands,
    paddle: Query<(&Transform, &Paddle), Without<Ball>>,
    mut balls: Query<(Entity, &mut Ball, &mut Transform, &mut Velocity), Without<Paddle>>,
    mut bounce: MessageWriter<BounceSound>,
    ball_speed: Res<BallSpeed>,
    paddle_mode: Res<PaddleMode>,
) {
    let Ok((paddle_t, paddle)) = paddle.single() else {
        return;
    };
    let paddle_half_w = paddle.half_width;
    let paddle_top = paddle_t.translation.y + PADDLE_HEIGHT / 2.0;

    // Tracks how many balls are still in play this tick, so losing one of several balls
    // (multi-ball) just removes it, and only draining the last ball costs a life.
    let mut alive = balls.iter().count();

    for (entity, mut ball, mut transform, mut velocity) in &mut balls {
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
            pos.y = paddle_top + BALL_RADIUS;
            if *paddle_mode == PaddleMode::Catch {
                // Catch power-up: the ball sticks to the paddle until re-launched
                // (see `release_caught_balls`). `ball_follow_paddle` keeps it glued.
                ball.stuck = true;
                velocity.0 = Vec2::ZERO;
            } else {
                // Reflection angle depends on where the ball struck the paddle:
                // center → straight up, edges → BALL_MAX_BOUNCE_ANGLE off vertical.
                let offset = ((pos.x - paddle_t.translation.x) / paddle_half_w).clamp(-1.0, 1.0);
                let angle = offset * BALL_MAX_BOUNCE_ANGLE;
                velocity.0 = Vec2::new(angle.sin(), angle.cos()) * ball_speed.current;
            }
            bounce.write(BounceSound::Paddle);
        }

        // Fell past the open bottom — the ball is lost.
        if pos.y + BALL_RADIUS < PLAYFIELD_BOTTOM {
            if alive > 1 {
                // One of several balls (multi-ball): drop it without costing a life.
                alive -= 1;
                commands.entity(entity).despawn();
            } else {
                // The last ball drained. Park it (so this branch can't re-fire) and let
                // the observer spend a life and decide what happens next.
                ball.stuck = true;
                velocity.0 = Vec2::ZERO;
                commands.trigger(BallLost);
            }
        }
    }
}

/// Deflects the ball off any brick it overlaps. Colored bricks shatter in one hit; silver
/// bricks lose one `hits_remaining` per contact (clinking until the last hit destroys them
/// and scores); gold bricks (`Indestructible`) only clink and deflect, never breaking.
/// Destroying a brick emits a [`BrickDestroyed`] message (scored elsewhere). Only the first
/// brick hit per ball per tick is resolved, to keep the deflection clean.
fn ball_brick_collision(
    mut commands: Commands,
    mut balls: Query<(&Ball, &mut Transform, &mut Velocity), Without<Brick>>,
    mut bricks: Query<(Entity, &Transform, &mut Brick, Has<Indestructible>), Without<Ball>>,
    mut destroyed: MessageWriter<BrickDestroyed>,
    mut bounce: MessageWriter<BounceSound>,
) {
    let half = Vec2::new(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0);
    for (ball, mut transform, mut velocity) in &mut balls {
        if ball.stuck {
            continue;
        }
        for (entity, brick_t, mut brick, indestructible) in &mut bricks {
            // A destructible brick already broken this tick (by another ball) is gone —
            // skip it so we don't underflow its hit count or score it twice.
            if !indestructible && brick.hits_remaining == 0 {
                continue;
            }
            // Reflect off the brick (least-penetrated face); skip when there's no overlap.
            if !ball_box_bounce(
                &mut transform.translation,
                &mut velocity.0,
                brick_t.translation.truncate(),
                half,
            ) {
                continue;
            }
            let cue = damage_brick(
                &mut commands,
                entity,
                &mut brick,
                brick_t.translation.truncate(),
                indestructible,
                &mut destroyed,
            );
            bounce.write(cue);
            break;
        }
    }
}

/// AABB collision of a ball against a box centered at `box_pos` with half-extents `box_half`
/// (the ball treated as an AABB of side `2 * BALL_RADIUS`). On overlap, reflects `velocity`
/// off whichever face is least penetrated, nudges `ball_pos` clear of the box, and returns
/// `true`; with no overlap it leaves both untouched and returns `false`. Shared by ball↔brick
/// and ball↔enemy collision.
pub fn ball_box_bounce(
    ball_pos: &mut Vec3,
    velocity: &mut Vec2,
    box_pos: Vec2,
    box_half: Vec2,
) -> bool {
    let delta = ball_pos.truncate() - box_pos;
    let overlap_x = (box_half.x + BALL_RADIUS) - delta.x.abs();
    let overlap_y = (box_half.y + BALL_RADIUS) - delta.y.abs();
    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        return false;
    }
    if overlap_x < overlap_y {
        velocity.x = velocity.x.abs().copysign(delta.x);
        ball_pos.x += overlap_x.copysign(delta.x);
    } else {
        velocity.y = velocity.y.abs().copysign(delta.y);
        ball_pos.y += overlap_y.copysign(delta.y);
    }
    true
}

/// Applies one hit to a brick and returns the sound cue to play. Indestructible (gold) bricks
/// only clink; a destructible brick loses one `hits_remaining`, and on reaching zero it
/// despawns and reports a [`BrickDestroyed`] at `brick_pos` (scored elsewhere). Callers must
/// already have skipped bricks broken earlier this tick (`hits_remaining == 0`). Shared by the
/// ball and the laser. (A surviving silver brick's `hits_remaining` mutation lets
/// `update_silver_damage` swap in the cracked frame.)
pub fn damage_brick(
    commands: &mut Commands,
    entity: Entity,
    brick: &mut Brick,
    brick_pos: Vec2,
    indestructible: bool,
    destroyed: &mut MessageWriter<BrickDestroyed>,
) -> BounceSound {
    if indestructible {
        return BounceSound::HardBrick;
    }
    brick.hits_remaining -= 1;
    if brick.hits_remaining == 0 {
        commands.entity(entity).despawn();
        destroyed.write(BrickDestroyed {
            points: brick.points,
            position: brick_pos,
        });
        BounceSound::Brick
    } else {
        BounceSound::HardBrick
    }
}
