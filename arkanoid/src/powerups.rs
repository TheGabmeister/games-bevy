use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::audio::BounceSound;
use crate::bricks::BrickDestroyed;
use crate::components::{
    Ball, Brick, Capsule, Indestructible, Laser, Paddle, PowerupKind, Velocity, WarpGate,
};
use crate::constants::*;
use crate::flow::BallLost;
use crate::input::InputActions;
use crate::resources::{BallSpeed, CapsuleDirector, Lives, PaddleMode, Round};
use crate::states::{AppState, PlayState};
use crate::vfx::spawn_vfx;

/// The deterministic order capsules drop in, cycled by the [`CapsuleDirector`].
const SEQUENCE: [PowerupKind; 7] = [
    PowerupKind::Expand,
    PowerupKind::Catch,
    PowerupKind::Laser,
    PowerupKind::Disruption,
    PowerupKind::Slow,
    PowerupKind::Player,
    PowerupKind::Break,
];

/// Fired (and observed) when the Vaus catches a falling capsule. The observer applies the
/// power-up effect.
#[derive(Event)]
pub struct CapsuleCaught {
    pub kind: PowerupKind,
}

/// Everything power-up: capsule drops, the seven effects, lasers, and the Break warp gate.
pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaddleMode>()
            .init_resource::<CapsuleDirector>()
            .add_observer(on_capsule_caught)
            .add_observer(reset_powerups_on_life_loss)
            .add_systems(OnEnter(AppState::Playing), reset_powerups)
            .add_systems(
                Update,
                (
                    drop_capsules.run_if(in_state(PlayState::Running)),
                    fire_laser.run_if(in_state(PlayState::Running)),
                    warp_through_gate.run_if(in_state(PlayState::Running)),
                    apply_paddle_mode.run_if(in_state(AppState::Playing)),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    move_capsules,
                    catch_capsules.after(move_capsules),
                    move_lasers,
                    laser_brick_collision.after(move_lasers),
                )
                    .run_if(in_state(PlayState::Running)),
            );
    }
}

/// Fresh run: clear any active paddle power-up and reset the capsule schedule.
fn reset_powerups(mut mode: ResMut<PaddleMode>, mut director: ResMut<CapsuleDirector>) {
    *mode = PaddleMode::Normal;
    *director = CapsuleDirector::default();
}

/// Releases a capsule every `CAPSULE_DROP_INTERVAL` destroyed bricks — but only one capsule
/// falls at a time — cycling deterministically through [`SEQUENCE`] at the last brick's spot.
fn drop_capsules(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut director: ResMut<CapsuleDirector>,
    mut destroyed: MessageReader<BrickDestroyed>,
    falling: Query<(), With<Capsule>>,
) {
    let mut count = 0u32;
    let mut last_pos = None;
    for ev in destroyed.read() {
        count += 1;
        last_pos = Some(ev.position);
    }
    if count == 0 {
        return;
    }
    director.bricks_destroyed += count;

    // One capsule on screen at a time; wait until enough bricks have fallen.
    if !falling.is_empty() || director.bricks_destroyed < CAPSULE_DROP_INTERVAL {
        return;
    }
    let Some(pos) = last_pos else {
        return;
    };
    director.bricks_destroyed = 0;
    let kind = SEQUENCE[director.next % SEQUENCE.len()];
    director.next += 1;

    commands.spawn((
        Capsule { kind },
        Sprite::from_image(assets.sprites.capsules.handle(kind)),
        Transform::from_xyz(pos.x, pos.y, Z_CAPSULE),
        DespawnOnExit(AppState::Playing),
    ));
}

/// Falls capsules toward the paddle; despawns any that drop past the bottom uncaught.
fn move_capsules(
    mut commands: Commands,
    time: Res<Time>,
    mut capsules: Query<(Entity, &mut Transform), With<Capsule>>,
) {
    let dy = CAPSULE_FALL_SPEED * time.delta_secs();
    for (entity, mut transform) in &mut capsules {
        transform.translation.y -= dy;
        if transform.translation.y + CAPSULE_HEIGHT / 2.0 < PLAYFIELD_BOTTOM {
            commands.entity(entity).despawn();
        }
    }
}

/// Catches capsules the paddle touches: fires [`CapsuleCaught`], flashes VFX, plays the
/// catch SFX, and despawns the capsule.
fn catch_capsules(
    mut commands: Commands,
    assets: Res<GameAssets>,
    paddle: Query<(&Transform, &Paddle)>,
    capsules: Query<(Entity, &Transform, &Capsule)>,
) {
    let Ok((paddle_t, paddle)) = paddle.single() else {
        return;
    };
    for (entity, transform, capsule) in &capsules {
        let delta = transform.translation.truncate() - paddle_t.translation.truncate();
        if delta.x.abs() <= paddle.half_width + CAPSULE_WIDTH / 2.0
            && delta.y.abs() <= PADDLE_HEIGHT / 2.0 + CAPSULE_HEIGHT / 2.0
        {
            commands.trigger(CapsuleCaught { kind: capsule.kind });
            commands.entity(entity).despawn();
            spawn_vfx(
                &mut commands,
                &assets.vfx.capsule_catch,
                transform.translation.truncate(),
            );
            commands.spawn((
                AudioPlayer(assets.sfx.capsule_catch.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

/// Applies a caught power-up. The three paddle power-ups (Catch/Laser/Expand) are mutually
/// exclusive via [`PaddleMode`]; the rest apply instantly or coexist (multi-ball).
#[allow(clippy::too_many_arguments)]
fn on_capsule_caught(
    trigger: On<CapsuleCaught>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut mode: ResMut<PaddleMode>,
    mut lives: ResMut<Lives>,
    mut speed: ResMut<BallSpeed>,
    mut balls: Query<(&Ball, &Transform, &mut Velocity)>,
    gates: Query<(), With<WarpGate>>,
) {
    let sfx = &assets.sfx;
    match trigger.kind {
        PowerupKind::Catch => *mode = PaddleMode::Catch,
        PowerupKind::Laser => *mode = PaddleMode::Laser,
        PowerupKind::Expand => {
            *mode = PaddleMode::Expand;
            play(&mut commands, sfx.expand.clone());
        }
        PowerupKind::Player => {
            lives.0 += 1;
            play(&mut commands, sfx.extra_life.clone());
        }
        PowerupKind::Slow => {
            speed.current = (speed.current * SLOW_FACTOR).max(BALL_SPEED_MIN);
            for (ball, _, mut velocity) in &mut balls {
                if !ball.stuck && velocity.0.length() > f32::EPSILON {
                    velocity.0 = velocity.0.normalize() * speed.current;
                }
            }
            play(&mut commands, sfx.slow.clone());
        }
        PowerupKind::Disruption => {
            // Split each live ball into three: two extra balls fanned out by ±0.4 rad.
            let spawns: Vec<(Vec3, Vec2)> = balls
                .iter()
                .filter(|(ball, _, _)| !ball.stuck)
                .flat_map(|(_, transform, velocity)| {
                    [0.4_f32, -0.4].map(|angle| {
                        (transform.translation, Vec2::from_angle(angle).rotate(velocity.0))
                    })
                })
                .collect();
            for (position, velocity) in &spawns {
                commands.spawn((
                    Ball { stuck: false },
                    Velocity(*velocity),
                    Sprite::from_image(assets.sprites.ball.clone()),
                    Transform::from_xyz(position.x, position.y, Z_BALL),
                    DespawnOnExit(AppState::Playing),
                ));
            }
            if !spawns.is_empty() {
                play(&mut commands, sfx.multiball.clone());
            }
        }
        PowerupKind::Break => {
            // Open a warp exit on the right edge (only one at a time).
            if gates.is_empty() {
                commands.spawn((
                    WarpGate,
                    Sprite::from_image(assets.sprites.warp_gate.clone()),
                    Transform::from_xyz(
                        PLAYFIELD_RIGHT - WARP_GATE_WIDTH / 2.0,
                        PADDLE_Y,
                        Z_WARP_GATE,
                    ),
                    DespawnOnExit(AppState::Playing),
                ));
                play(&mut commands, sfx.warp_gate_open.clone());
            }
        }
    }
}

/// Updates the paddle sprite and `half_width` when the active paddle mode changes.
fn apply_paddle_mode(
    mode: Res<PaddleMode>,
    assets: Res<GameAssets>,
    mut paddle: Query<(&mut Sprite, &mut Paddle)>,
) {
    if !mode.is_changed() {
        return;
    }
    let Ok((mut sprite, mut paddle)) = paddle.single_mut() else {
        return;
    };
    if *mode == PaddleMode::Expand {
        sprite.image = assets.sprites.vaus_expanded.clone();
        paddle.half_width = PADDLE_EXPANDED_WIDTH / 2.0;
    } else {
        sprite.image = assets.sprites.vaus.clone();
        paddle.half_width = PADDLE_WIDTH / 2.0;
    }
}

/// Fires a pair of laser bolts from the Vaus muzzles when Laser is active and the player
/// presses launch, capped at [`LASER_MAX_BOLTS`] in flight.
fn fire_laser(
    mode: Res<PaddleMode>,
    input: Res<InputActions>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    paddle: Query<&Transform, With<Paddle>>,
    bolts: Query<(), With<Laser>>,
) {
    if *mode != PaddleMode::Laser || !input.launch {
        return;
    }
    if bolts.iter().count() + 2 > LASER_MAX_BOLTS {
        return;
    }
    let Ok(paddle_t) = paddle.single() else {
        return;
    };
    let muzzle_y = paddle_t.translation.y + PADDLE_HEIGHT / 2.0;
    for offset in [-LASER_MUZZLE_OFFSET, LASER_MUZZLE_OFFSET] {
        commands.spawn((
            Laser,
            Velocity(Vec2::new(0.0, LASER_SPEED)),
            Sprite::from_image(assets.sprites.laser_bolt.clone()),
            Transform::from_xyz(paddle_t.translation.x + offset, muzzle_y, Z_LASER),
            DespawnOnExit(AppState::Playing),
        ));
    }
    play(&mut commands, assets.sfx.laser_fire.clone());
}

/// Flies laser bolts upward; despawns those that clear the top of the playfield.
fn move_lasers(
    mut commands: Commands,
    time: Res<Time>,
    mut bolts: Query<(Entity, &mut Transform, &Velocity), With<Laser>>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, velocity) in &mut bolts {
        transform.translation.y += velocity.0.y * dt;
        if transform.translation.y - LASER_HEIGHT / 2.0 > PLAYFIELD_TOP {
            commands.entity(entity).despawn();
        }
    }
}

/// Laser bolts damage the first brick they overlap — one hit each, mirroring the ball's
/// brick rules (silver durability, gold only clinks) — then despawn with an impact flash.
fn laser_brick_collision(
    mut commands: Commands,
    assets: Res<GameAssets>,
    bolts: Query<(Entity, &Transform), With<Laser>>,
    mut bricks: Query<(Entity, &Transform, &mut Brick, Has<Indestructible>)>,
    mut destroyed: MessageWriter<BrickDestroyed>,
    mut bounce: MessageWriter<BounceSound>,
) {
    let reach = Vec2::new(
        (BRICK_WIDTH + LASER_WIDTH) / 2.0,
        (BRICK_HEIGHT + LASER_HEIGHT) / 2.0,
    );
    for (bolt, bolt_t) in &bolts {
        for (brick_entity, brick_t, mut brick, indestructible) in &mut bricks {
            let delta = bolt_t.translation.truncate() - brick_t.translation.truncate();
            if delta.x.abs() > reach.x || delta.y.abs() > reach.y {
                continue;
            }
            // A destructible brick already broken this tick (by another bolt/ball) is gone —
            // skip so we neither underflow its hit count nor score it twice.
            if !indestructible && brick.hits_remaining == 0 {
                continue;
            }

            spawn_vfx(&mut commands, &assets.vfx.laser_impact, bolt_t.translation.truncate());
            commands.entity(bolt).despawn();

            if indestructible {
                bounce.write(BounceSound::HardBrick);
            } else {
                brick.hits_remaining -= 1;
                if brick.hits_remaining == 0 {
                    commands.entity(brick_entity).despawn();
                    destroyed.write(BrickDestroyed {
                        points: brick.points,
                        position: brick_t.translation.truncate(),
                    });
                    bounce.write(BounceSound::Brick);
                } else {
                    bounce.write(BounceSound::HardBrick);
                }
            }
            break;
        }
    }
}

/// When a Break warp gate is open and the Vaus reaches the right edge, the player escapes
/// through it: advance to the next round (same effect as clearing it).
fn warp_through_gate(
    mut commands: Commands,
    assets: Res<GameAssets>,
    gates: Query<Entity, With<WarpGate>>,
    paddle: Query<(&Transform, &Paddle)>,
    mut round: ResMut<Round>,
    mut next: ResMut<NextState<PlayState>>,
) {
    if gates.is_empty() {
        return;
    }
    let Ok((paddle_t, paddle)) = paddle.single() else {
        return;
    };
    let gate_x = PLAYFIELD_RIGHT - WARP_GATE_WIDTH / 2.0;
    if paddle_t.translation.x + paddle.half_width < gate_x {
        return;
    }
    for entity in &gates {
        commands.entity(entity).despawn();
    }
    round.0 += 1;
    commands.spawn((
        AudioPlayer(assets.music.round_clear.clone()),
        PlaybackSettings::DESPAWN,
    ));
    next.set(PlayState::Ready);
}

/// Effects reset on losing a life: clear the active paddle mode and wipe any falling
/// capsules, in-flight lasers, and open warp gate.
#[allow(clippy::type_complexity)]
fn reset_powerups_on_life_loss(
    _trigger: On<BallLost>,
    mut commands: Commands,
    mut mode: ResMut<PaddleMode>,
    entities: Query<Entity, Or<(With<Capsule>, With<Laser>, With<WarpGate>)>>,
) {
    *mode = PaddleMode::Normal;
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

/// Spawns a one-shot, self-despawning audio player for a SFX source.
fn play(commands: &mut Commands, source: Handle<AudioSource>) {
    commands.spawn((AudioPlayer(source), PlaybackSettings::DESPAWN));
}
