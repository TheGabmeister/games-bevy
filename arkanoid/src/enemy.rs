use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::bricks::ScoreChanged;
use crate::collision::ball_box_bounce;
use crate::components::{Ball, Enemy, EnemyKind, Laser, SpawnGate, Velocity};
use crate::constants::*;
use crate::resources::{EnemyDirector, EnemySpawnTimer, Score};
use crate::schedule::Physics;
use crate::states::{AppState, PlayState};
use crate::vfx::spawn_vfx;

/// Fired when an enemy launches from gate `gate`; the observer opens that gate and plays
/// the spawn cue.
#[derive(Event)]
pub struct EnemySpawned {
    pub gate: usize,
}

/// Fired when an enemy is killed (by ball or laser) — not when it merely exits the bottom.
/// The observer scores it, bursts VFX, and plays the death cue.
#[derive(Event)]
pub struct EnemyDestroyed {
    pub position: Vec2,
    pub points: u32,
}

/// Descending enemies: timed spawning from the top gates, per-type wandering descent, and
/// destruction by ball or laser. Reaching the bottom just exits.
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemySpawnTimer>()
            .init_resource::<EnemyDirector>()
            .add_observer(on_enemy_spawned)
            .add_observer(on_enemy_destroyed)
            .add_systems(OnEnter(AppState::Playing), spawn_gates)
            .add_systems(OnEnter(PlayState::Ready), reset_enemies)
            .add_systems(
                Update,
                (
                    spawn_enemies.run_if(in_state(PlayState::Running)),
                    animate_enemies.run_if(in_state(AppState::Playing)),
                    animate_gates.run_if(in_state(AppState::Playing)),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    move_enemies.in_set(Physics::Movement),
                    ball_enemy_collision.in_set(Physics::Collision),
                    laser_enemy_collision.in_set(Physics::Collision),
                )
                    .run_if(in_state(PlayState::Running)),
            );
    }
}

/// Spawns the two top gates (closed) for the run.
fn spawn_gates(mut commands: Commands, assets: Res<GameAssets>) {
    for (index, x) in [(0usize, -GATE_X_OFFSET), (1usize, GATE_X_OFFSET)] {
        // Start closed: a finished `Once` timer reads as "not open".
        let mut open_timer = Timer::from_seconds(GATE_OPEN_DURATION, TimerMode::Once);
        let full = open_timer.duration();
        open_timer.tick(full);
        commands.spawn((
            SpawnGate { index, open_timer },
            Sprite::from_image(assets.sprites.spawn_gate[0].clone()),
            Transform::from_xyz(x, GATE_Y, Z_GATE),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

/// Clears any leftover enemies and restarts the spawn cadence at each round's start.
fn reset_enemies(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut timer: ResMut<EnemySpawnTimer>,
    mut director: ResMut<EnemyDirector>,
) {
    for entity in &enemies {
        commands.entity(entity).despawn();
    }
    timer.0.reset();
    *director = EnemyDirector::default();
}

/// On each interval (capped at [`ENEMY_MAX_ACTIVE`]), launches the next enemy from the next
/// gate, deterministically rotating both the type and the gate.
fn spawn_enemies(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut timer: ResMut<EnemySpawnTimer>,
    mut director: ResMut<EnemyDirector>,
    enemies: Query<(), With<Enemy>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    if enemies.iter().count() >= ENEMY_MAX_ACTIVE {
        return;
    }
    let kind = match director.spawned % 3 {
        0 => EnemyKind::Pyramid,
        1 => EnemyKind::Molecule,
        _ => EnemyKind::Cube,
    };
    let gate = (director.spawned % 2) as usize;
    director.spawned += 1;

    let x = if gate == 0 { -GATE_X_OFFSET } else { GATE_X_OFFSET };
    let y = GATE_Y - GATE_HEIGHT / 2.0 - ENEMY_SIZE / 2.0;
    commands.spawn((
        Enemy {
            kind,
            age: 0.0,
            anim_index: 0,
            anim_timer: Timer::from_seconds(ENEMY_ANIM_FRAME_TIME, TimerMode::Repeating),
        },
        Sprite::from_image(assets.sprites.enemies.frames(kind)[0].clone()),
        Transform::from_xyz(x, y, Z_ENEMY),
        DespawnOnExit(AppState::Playing),
    ));
    commands.trigger(EnemySpawned { gate });
}

/// Horizontal wander direction for a molecule: flips sharply every [`ENEMY_MOLECULE_FLIP`].
fn molecule_dir(age: f32) -> f32 {
    if (age / ENEMY_MOLECULE_FLIP) as i64 % 2 == 0 {
        1.0
    } else {
        -1.0
    }
}

/// Drifts enemies down their per-type wander path, kept inside the side walls; an enemy that
/// sinks past the bottom simply exits (despawns).
fn move_enemies(
    time: Res<Time>,
    mut commands: Commands,
    mut enemies: Query<(Entity, &mut Enemy, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let half = ENEMY_SIZE / 2.0;
    for (entity, mut enemy, mut transform) in &mut enemies {
        enemy.age += dt;
        let (vx, speed_factor) = match enemy.kind {
            EnemyKind::Pyramid => (
                (enemy.age * ENEMY_PYRAMID_WEAVE).sin() * ENEMY_DRIFT_SPEED,
                ENEMY_PYRAMID_SPEED_FACTOR,
            ),
            EnemyKind::Molecule => (
                molecule_dir(enemy.age) * ENEMY_DRIFT_SPEED,
                ENEMY_MOLECULE_SPEED_FACTOR,
            ),
            EnemyKind::Cube => (
                (enemy.age * ENEMY_CUBE_WEAVE).sin() * ENEMY_DRIFT_SPEED * ENEMY_CUBE_DRIFT_FACTOR,
                ENEMY_CUBE_SPEED_FACTOR,
            ),
        };
        transform.translation.x = (transform.translation.x + vx * dt)
            .clamp(PLAYFIELD_LEFT + half, PLAYFIELD_RIGHT - half);
        transform.translation.y -= ENEMY_SPEED * speed_factor * dt;
        if transform.translation.y + half < PLAYFIELD_BOTTOM {
            commands.entity(entity).despawn();
        }
    }
}

/// Cycles each enemy's 4-frame looping animation.
fn animate_enemies(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut enemies: Query<(&mut Enemy, &mut Sprite)>,
) {
    for (mut enemy, mut sprite) in &mut enemies {
        if !enemy.anim_timer.tick(time.delta()).just_finished() {
            continue;
        }
        enemy.anim_index = (enemy.anim_index + 1) % 4;
        let (kind, index) = (enemy.kind, enemy.anim_index);
        sprite.image = assets.sprites.enemies.frames(kind)[index].clone();
    }
}

/// Shows a gate's open frame while its `open_timer` is running, else the closed frame.
fn animate_gates(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut gates: Query<(&mut SpawnGate, &mut Sprite)>,
) {
    for (mut gate, mut sprite) in &mut gates {
        gate.open_timer.tick(time.delta());
        let frame = if gate.open_timer.is_finished() { 0 } else { 1 };
        sprite.image = assets.sprites.spawn_gate[frame].clone();
    }
}

/// Bounces the ball off enemies it touches (least-penetrated face, as with bricks) and
/// destroys them. Tracks kills locally so two balls can't double-score one enemy this tick.
fn ball_enemy_collision(
    mut commands: Commands,
    mut balls: Query<(&Ball, &mut Transform, &mut Velocity), Without<Enemy>>,
    enemies: Query<(Entity, &Transform, &Enemy), Without<Ball>>,
) {
    let half = ENEMY_SIZE / 2.0;
    let mut killed: Vec<Entity> = Vec::new();
    for (ball, mut transform, mut velocity) in &mut balls {
        if ball.stuck {
            continue;
        }
        for (entity, enemy_t, enemy) in &enemies {
            if killed.contains(&entity) {
                continue;
            }
            if !ball_box_bounce(
                &mut transform.translation,
                &mut velocity.0,
                enemy_t.translation.truncate(),
                Vec2::splat(half),
            ) {
                continue;
            }
            commands.entity(entity).despawn();
            killed.push(entity);
            commands.trigger(EnemyDestroyed {
                position: enemy_t.translation.truncate(),
                points: enemy.kind.points(),
            });
            break;
        }
    }
}

/// Laser bolts destroy the first enemy they overlap; the bolt is consumed.
fn laser_enemy_collision(
    mut commands: Commands,
    bolts: Query<(Entity, &Transform), With<Laser>>,
    enemies: Query<(Entity, &Transform, &Enemy)>,
) {
    let reach = Vec2::new(
        (ENEMY_SIZE + LASER_WIDTH) / 2.0,
        (ENEMY_SIZE + LASER_HEIGHT) / 2.0,
    );
    let mut killed: Vec<Entity> = Vec::new();
    for (bolt, bolt_t) in &bolts {
        for (entity, enemy_t, enemy) in &enemies {
            if killed.contains(&entity) {
                continue;
            }
            let delta = bolt_t.translation.truncate() - enemy_t.translation.truncate();
            if delta.x.abs() > reach.x || delta.y.abs() > reach.y {
                continue;
            }
            commands.entity(bolt).despawn();
            commands.entity(entity).despawn();
            killed.push(entity);
            commands.trigger(EnemyDestroyed {
                position: enemy_t.translation.truncate(),
                points: enemy.kind.points(),
            });
            break;
        }
    }
}

/// Spawn cue + opens the originating gate.
fn on_enemy_spawned(
    trigger: On<EnemySpawned>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut gates: Query<&mut SpawnGate>,
) {
    commands.spawn((
        AudioPlayer(assets.sfx.enemy_spawn.clone()),
        PlaybackSettings::DESPAWN,
    ));
    for mut gate in &mut gates {
        if gate.index == trigger.gate {
            gate.open_timer.reset();
        }
    }
}

/// Scores a killed enemy, bursts the destroy VFX, and plays the death cue.
fn on_enemy_destroyed(
    trigger: On<EnemyDestroyed>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut score: ResMut<Score>,
    mut changed: MessageWriter<ScoreChanged>,
) {
    score.add(trigger.points);
    changed.write(ScoreChanged);
    spawn_vfx(&mut commands, &assets.vfx.enemy_destroy, trigger.position);
    commands.spawn((
        AudioPlayer(assets.sfx.enemy_destroyed.clone()),
        PlaybackSettings::DESPAWN,
    ));
}
