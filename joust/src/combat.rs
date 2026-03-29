use bevy::prelude::*;
use std::collections::HashSet;

use crate::components::*;
use crate::constants::*;
use crate::enemy::spawn_enemy_at;
use crate::physics::*;
use crate::rendering::*;
use crate::resources::*;
use crate::states::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            invincibility_tick_system
                .in_set(GameSet::Combat)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            (
                joust_combat_system,
                bevy::ecs::schedule::ApplyDeferred,
                egg_collection_system,
                bevy::ecs::schedule::ApplyDeferred,
                egg_hatch_system,
                bevy::ecs::schedule::ApplyDeferred,
                lava_kill_system,
                bevy::ecs::schedule::ApplyDeferred,
                handle_score_messages,
                handle_player_died_messages,
                check_game_over,
            )
                .chain()
                .in_set(GameSet::Combat)
                .run_if(in_state(PlayState::WaveActive)),
        );
    }
}

#[derive(Clone, Copy)]
struct RiderSnapshot {
    entity: Entity,
    position: Vec2,
    player_id: Option<u8>,
    enemy_tier: Option<EnemyTier>,
    invincible: bool,
}

#[derive(Clone, Copy)]
struct KillOutcome {
    winner: Entity,
    loser: Entity,
    loser_position: Vec2,
    winner_player_id: Option<u8>,
    loser_player_id: Option<u8>,
    loser_tier: Option<EnemyTier>,
}

#[derive(Clone, Copy)]
struct BounceOutcome {
    a: Entity,
    b: Entity,
    pos_a: Vec2,
    pos_b: Vec2,
}

fn invincibility_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invincible, &mut Visibility)>,
) {
    for (entity, mut inv, mut vis) in &mut query {
        inv.0.tick(time.delta());
        if inv.0.is_finished() {
            commands.entity(entity).remove::<Invincible>();
            *vis = Visibility::Inherited;
        } else {
            let blink_on = ((inv.0.elapsed_secs() * 10.0) as u32).is_multiple_of(2);
            *vis = if blink_on {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn joust_combat_system(
    mut commands: Commands,
    riders: Query<
        (
            Entity,
            &Transform,
            Option<&Player>,
            Option<&EnemyTier>,
            Option<&Invincible>,
        ),
        With<Rider>,
    >,
    mut velocities: Query<&mut Velocity>,
    mut kill_writer: MessageWriter<JoustKillMessage>,
    mut score_writer: MessageWriter<ScoreMessage>,
    mut died_writer: MessageWriter<PlayerDiedMessage>,
    meshes: Res<SharedMeshes>,
    materials: Res<SharedMaterials>,
    game_state: Res<GameState>,
    mut respawn_timers: ResMut<RespawnTimers>,
) {
    let mut riders_vec: Vec<_> = riders
        .iter()
        .map(|(entity, transform, player, tier, invincible)| RiderSnapshot {
            entity,
            position: transform.translation.truncate(),
            player_id: player.map(|player| player.id),
            enemy_tier: tier.copied(),
            invincible: invincible.is_some(),
        })
        .collect();
    riders_vec.sort_by_key(|rider| rider.entity.to_bits());

    let mut losers = HashSet::new();
    let mut kills = Vec::new();
    let mut bounces = Vec::new();

    for i in 0..riders_vec.len() {
        for j in (i + 1)..riders_vec.len() {
            let a = riders_vec[i];
            let b = riders_vec[j];

            if a.invincible || b.invincible {
                continue;
            }

            let dx = wrapped_distance(a.position.x, b.position.x);
            let dy = (a.position.y - b.position.y).abs();
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > RIDER_RADIUS * 2.0 {
                continue;
            }

            let diff = (a.position.y + JOUST_POINT_Y) - (b.position.y + JOUST_POINT_Y);
            if diff > JOUST_DEAD_ZONE {
                losers.insert(b.entity);
                kills.push(KillOutcome {
                    winner: a.entity,
                    loser: b.entity,
                    loser_position: b.position,
                    winner_player_id: a.player_id,
                    loser_player_id: b.player_id,
                    loser_tier: b.enemy_tier,
                });
            } else if diff < -JOUST_DEAD_ZONE {
                losers.insert(a.entity);
                kills.push(KillOutcome {
                    winner: b.entity,
                    loser: a.entity,
                    loser_position: a.position,
                    winner_player_id: b.player_id,
                    loser_player_id: a.player_id,
                    loser_tier: a.enemy_tier,
                });
            } else {
                bounces.push(BounceOutcome {
                    a: a.entity,
                    b: b.entity,
                    pos_a: a.position,
                    pos_b: b.position,
                });
            }
        }
    }

    let mut processed_losers = HashSet::new();
    let mut invincible_entities = HashSet::new();

    for kill in kills {
        if !processed_losers.insert(kill.loser) {
            continue;
        }

        if let Some(tier) = kill.loser_tier {
            if let Some(player_id) = kill.winner_player_id {
                score_writer.write(ScoreMessage {
                    player_id,
                    points: tier.score(),
                });
            }

            let hatch_time = get_wave_def(game_state.wave).egg_hatch_time;
            spawn_egg_entity(
                &mut commands,
                &meshes,
                &materials,
                kill.loser_position,
                tier,
                hatch_time,
            );
        }

        if let Some(player_id) = kill.loser_player_id {
            died_writer.write(PlayerDiedMessage { player_id });
            schedule_respawn(&mut respawn_timers, player_id);
        }

        kill_writer.write(JoustKillMessage {
            loser_position: kill.loser_position,
            loser_tier: kill.loser_tier,
        });
        commands.entity(kill.loser).despawn();

        if !losers.contains(&kill.winner) {
            invincible_entities.insert(kill.winner);
        }
    }

    let mut bounced_entities = HashSet::new();
    for bounce in bounces {
        if losers.contains(&bounce.a) || losers.contains(&bounce.b) {
            continue;
        }

        let dx = wrapped_dx(bounce.pos_a.x, bounce.pos_b.x);
        let direction = if dx.abs() <= f32::EPSILON {
            1.0
        } else {
            dx.signum()
        };

        if bounced_entities.insert(bounce.a)
            && let Ok(mut velocity) = velocities.get_mut(bounce.a)
        {
            velocity.0.x = -direction * BOUNCE_HORIZONTAL;
            velocity.0.y = BOUNCE_VERTICAL;
        }

        if bounced_entities.insert(bounce.b)
            && let Ok(mut velocity) = velocities.get_mut(bounce.b)
        {
            velocity.0.x = direction * BOUNCE_HORIZONTAL;
            velocity.0.y = BOUNCE_VERTICAL;
        }
    }

    invincible_entities.extend(bounced_entities);
    for entity in invincible_entities {
        commands.entity(entity).insert(Invincible(Timer::from_seconds(
            BOUNCE_INVINCIBILITY,
            TimerMode::Once,
        )));
    }
}

fn schedule_respawn(respawn_timers: &mut RespawnTimers, player_id: u8) {
    if respawn_timers
        .timers
        .iter()
        .any(|(queued_player_id, _)| *queued_player_id == player_id)
    {
        return;
    }

    respawn_timers.timers.push((
        player_id,
        Timer::from_seconds(PLAYER_RESPAWN_DELAY, TimerMode::Once),
    ));
}

fn egg_collection_system(
    mut commands: Commands,
    players: Query<(&Player, &Transform), Without<Egg>>,
    eggs: Query<(Entity, &Transform), With<Egg>>,
    mut score_writer: MessageWriter<ScoreMessage>,
) {
    for (player, p_transform) in &players {
        for (egg_entity, e_transform) in &eggs {
            let dx = wrapped_distance(p_transform.translation.x, e_transform.translation.x);
            let dy = (p_transform.translation.y - e_transform.translation.y).abs();
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < RIDER_RADIUS + EGG_RADIUS {
                score_writer.write(ScoreMessage {
                    player_id: player.id,
                    points: SCORE_COLLECT_EGG,
                });
                commands.entity(egg_entity).despawn();
            }
        }
    }
}

fn egg_hatch_system(
    mut commands: Commands,
    time: Res<Time>,
    mut eggs: Query<(Entity, &mut Egg, &Transform)>,
    meshes: Res<SharedMeshes>,
    materials: Res<SharedMaterials>,
) {
    for (entity, mut egg, transform) in &mut eggs {
        egg.hatch_timer.tick(time.delta());
        if egg.hatch_timer.just_finished() {
            let position = transform.translation.truncate();
            let new_tier = egg.tier.hatched_tier();
            spawn_enemy_at(&mut commands, &meshes, &materials, position, new_tier);
            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::type_complexity)]
fn lava_kill_system(
    mut commands: Commands,
    query: Query<
        (Entity, &Transform, Option<&Player>, Option<&EnemyTier>, Option<&Egg>),
        (With<Velocity>, Without<Particle>),
    >,
    mut kill_writer: MessageWriter<JoustKillMessage>,
    mut died_writer: MessageWriter<PlayerDiedMessage>,
    mut respawn_timers: ResMut<RespawnTimers>,
) {
    for (entity, transform, player, enemy_tier, egg) in &query {
        let radius = if egg.is_some() { EGG_RADIUS } else { RIDER_RADIUS };
        let position = transform.translation.truncate();

        if position.y - radius < LAVA_Y {
            kill_writer.write(JoustKillMessage {
                loser_position: position,
                loser_tier: enemy_tier.copied().or_else(|| egg.map(|egg| egg.tier)),
            });

            if let Some(player) = player {
                died_writer.write(PlayerDiedMessage {
                    player_id: player.id,
                });
                schedule_respawn(&mut respawn_timers, player.id);
            }

            commands.entity(entity).despawn();
        }
    }
}

fn handle_score_messages(
    mut reader: MessageReader<ScoreMessage>,
    mut game_state: ResMut<GameState>,
) {
    for msg in reader.read() {
        let player_index = msg.player_id as usize;
        let old_score = game_state.scores[player_index];
        game_state.scores[player_index] += msg.points;
        let new_score = game_state.scores[player_index];

        let old_lives_earned = old_score / EXTRA_LIFE_INTERVAL;
        let new_lives_earned = new_score / EXTRA_LIFE_INTERVAL;
        if new_lives_earned > old_lives_earned {
            game_state.lives[player_index] = (game_state.lives[player_index] + 1).min(MAX_LIVES);
        }
    }
}

fn handle_player_died_messages(
    mut reader: MessageReader<PlayerDiedMessage>,
    mut game_state: ResMut<GameState>,
) {
    for msg in reader.read() {
        let player_index = msg.player_id as usize;
        if game_state.lives[player_index] > 0 {
            game_state.lives[player_index] -= 1;
        }
    }
}

fn check_game_over(
    game_state: Res<GameState>,
    players: Query<(), With<Player>>,
    respawn_timers: Res<RespawnTimers>,
    player_count: Res<PlayerCount>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !players.is_empty() || !respawn_timers.timers.is_empty() {
        return;
    }

    let all_out = (0..player_count.0).all(|i| game_state.lives[i as usize] == 0);
    if all_out {
        next_state.set(AppState::GameOver);
    }
}
