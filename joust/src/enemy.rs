use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::physics::*;
use crate::rendering::*;
use crate::resources::*;
use crate::states::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayState::WaveIntro), spawn_wave_enemies)
            .add_systems(
                Update,
                enemy_ai_system
                    .in_set(GameSet::Ai)
                    .run_if(in_state(PlayState::WaveActive)),
            );
    }
}

fn spawn_wave_enemies(
    mut commands: Commands,
    meshes: Res<SharedMeshes>,
    materials: Res<SharedMaterials>,
    game_state: Res<GameState>,
) {
    let wave_def = get_wave_def(game_state.wave);
    let mut spawn_index = 0usize;
    let positions = &ENEMY_SPAWN_POSITIONS;

    let mut spawn_one = |tier: EnemyTier, commands: &mut Commands| {
        let pos_idx = spawn_index % positions.len();
        spawn_index += 1;
        let pos = Vec2::new(positions[pos_idx].0, positions[pos_idx].1);
        spawn_enemy_at(commands, &meshes, &materials, pos, tier);
    };

    for _ in 0..wave_def.bounders {
        spawn_one(EnemyTier::Bounder, &mut commands);
    }
    for _ in 0..wave_def.hunters {
        spawn_one(EnemyTier::Hunter, &mut commands);
    }
    for _ in 0..wave_def.shadow_lords {
        spawn_one(EnemyTier::ShadowLord, &mut commands);
    }
}

pub fn spawn_enemy_at(
    commands: &mut Commands,
    meshes: &SharedMeshes,
    materials: &SharedMaterials,
    position: Vec2,
    tier: EnemyTier,
) {
    let body_mat = match tier {
        EnemyTier::Bounder => materials.bounder_body.clone(),
        EnemyTier::Hunter => materials.hunter_body.clone(),
        EnemyTier::ShadowLord => materials.shadow_lord_body.clone(),
    };
    let entity = spawn_rider_visual(commands, meshes, materials, position, Z_ENEMIES, body_mat);

    let mut rng = rand::rng();
    let facing = if rng.random_bool(0.5) {
        FacingDirection::Left
    } else {
        FacingDirection::Right
    };

    commands.entity(entity).insert((
        Enemy,
        tier,
        EnemyAiState::Wander,
        Rider,
        Velocity::default(),
        facing,
        FlapCooldown::ready(),
        PreviousPosition(position),
        DespawnOnExit(AppState::Playing),
        AiTimers {
            flap: Timer::from_seconds(
                AI_FLAP_INTERVAL_BASE + rng.random_range(0.0..AI_FLAP_RANDOMNESS),
                TimerMode::Repeating,
            ),
            direction: Timer::from_seconds(AI_DIRECTION_CHANGE_INTERVAL, TimerMode::Repeating),
        },
        Invincible(Timer::from_seconds(RESPAWN_INVINCIBILITY, TimerMode::Once)),
    ));
}

#[allow(clippy::type_complexity)]
fn enemy_ai_system(
    time: Res<Time>,
    mut commands: Commands,
    players: Query<&Transform, With<Player>>,
    mut enemies: Query<(
        Entity,
        &mut Velocity,
        &mut FacingDirection,
        &mut EnemyAiState,
        &mut AiTimers,
        &mut FlapCooldown,
        &EnemyTier,
        &Transform,
        Option<&Grounded>,
    )>,
) {
    let mut rng = rand::rng();
    let player_positions: Vec<Vec2> = players
        .iter()
        .map(|t| t.translation.truncate())
        .collect();

    for (entity, mut vel, mut facing, mut ai_state, mut timers, mut cooldown, tier, transform, grounded) in
        &mut enemies
    {
        let dt = time.delta_secs();
        let speed_mult = tier.speed_multiplier();
        let pos = transform.translation.truncate();
        let is_grounded = grounded.is_some();
        let accel = if is_grounded { GROUND_ACCEL } else { AIR_ACCEL };
        let max_speed = if is_grounded { MAX_GROUND_SPEED } else { MAX_AIR_SPEED };

        cooldown.0.tick(time.delta());
        timers.flap.tick(time.delta());
        timers.direction.tick(time.delta());

        // Find nearest player
        let nearest_player = player_positions.iter().min_by(|a, b| {
            let da = wrapped_distance(pos.x, a.x).hypot(pos.y - a.y);
            let db = wrapped_distance(pos.x, b.x).hypot(pos.y - b.y);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Update AI state
        if let Some(target) = nearest_player {
            let dist = wrapped_distance(pos.x, target.x).hypot(pos.y - target.y);
            *ai_state = if dist < AI_PURSUE_RANGE {
                EnemyAiState::Pursue
            } else {
                EnemyAiState::Wander
            };
        }

        // Act on state
        match *ai_state {
            EnemyAiState::Wander => {
                if timers.direction.just_finished() {
                    *facing = if rng.random_bool(0.5) {
                        FacingDirection::Left
                    } else {
                        FacingDirection::Right
                    };
                }
                let dir = match *facing {
                    FacingDirection::Left => -1.0,
                    FacingDirection::Right => 1.0,
                };
                vel.0.x = (vel.0.x + dir * accel * speed_mult * dt * 0.5)
                    .clamp(-max_speed * speed_mult, max_speed * speed_mult);

                if timers.flap.just_finished() && cooldown.0.is_finished() {
                    vel.0.y = (vel.0.y + FLAP_IMPULSE * 0.7).min(MAX_RISE_SPEED);
                    cooldown.0.reset();
                    if is_grounded {
                        commands.entity(entity).remove::<Grounded>();
                    }
                }
            }
            EnemyAiState::Pursue => {
                if let Some(target) = nearest_player {
                    let dx = wrapped_dx(pos.x, target.x);
                    let dir = dx.signum();
                    *facing = if dir < 0.0 {
                        FacingDirection::Left
                    } else {
                        FacingDirection::Right
                    };
                    vel.0.x = (vel.0.x + dir * accel * speed_mult * dt)
                        .clamp(-max_speed * speed_mult, max_speed * speed_mult);

                    // Flap to get above target
                    let should_flap = timers.flap.just_finished()
                        || (pos.y < target.y + JOUST_POINT_Y + 20.0 && cooldown.0.is_finished());
                    if should_flap && cooldown.0.is_finished() {
                        vel.0.y = (vel.0.y + FLAP_IMPULSE).min(MAX_RISE_SPEED);
                        cooldown.0.reset();
                        if is_grounded {
                            commands.entity(entity).remove::<Grounded>();
                        }
                    }
                }
            }
        }
    }
}
