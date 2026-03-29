use bevy::prelude::*;

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
        app.add_event::<JoustKillEvent>()
            .add_event::<JoustBounceEvent>()
            .add_event::<EggCollectedEvent>()
            .add_event::<PlayerDiedEvent>()
            .add_event::<ScoreEvent>()
            .add_systems(
                Update,
                (
                    invincibility_tick_system,
                    joust_combat_system,
                    egg_collection_system,
                    egg_hatch_system,
                    lava_kill_system,
                    handle_score_events,
                    check_game_over,
                )
                    .chain()
                    .in_set(GameSet::Combat)
                    .run_if(in_state(PlayState::WaveActive)),
            );
    }
}

fn invincibility_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invincible)>,
) {
    for (entity, mut inv) in &mut query {
        inv.0.tick(time.delta());
        if inv.0.finished() {
            commands.entity(entity).remove::<Invincible>();
        }
    }
}

fn joust_combat_system(
    mut commands: Commands,
    riders: Query<(
        Entity,
        &Transform,
        &Velocity,
        Option<&Player>,
        Option<&Enemy>,
        Option<&EnemyTier>,
        Option<&Invincible>,
    ), With<Rider>>,
    mut kill_events: EventWriter<JoustKillEvent>,
    mut bounce_events: EventWriter<JoustBounceEvent>,
    mut player_died_events: EventWriter<PlayerDiedEvent>,
    mut score_events: EventWriter<ScoreEvent>,
    meshes: Res<SharedMeshes>,
    materials: Res<SharedMaterials>,
    game_state: Res<GameState>,
    mut respawn_timers: ResMut<RespawnTimers>,
) {
    let riders_vec: Vec<_> = riders.iter().collect();
    let mut dead: Vec<Entity> = Vec::new();

    for i in 0..riders_vec.len() {
        for j in (i + 1)..riders_vec.len() {
            let (e_a, t_a, _, p_a, _, tier_a, inv_a) = riders_vec[i];
            let (e_b, t_b, _, p_b, _, tier_b, inv_b) = riders_vec[j];

            if inv_a.is_some() || inv_b.is_some() {
                continue;
            }
            if dead.contains(&e_a) || dead.contains(&e_b) {
                continue;
            }

            let pos_a = t_a.translation.truncate();
            let pos_b = t_b.translation.truncate();
            let dx = wrapped_distance(pos_a.x, pos_b.x);
            let dy = (pos_a.y - pos_b.y).abs();
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > RIDER_RADIUS * 2.0 {
                continue;
            }

            let joust_a = pos_a.y + JOUST_POINT_Y;
            let joust_b = pos_b.y + JOUST_POINT_Y;
            let diff = joust_a - joust_b;

            if diff > JOUST_DEAD_ZONE {
                // A wins
                handle_kill(
                    &mut commands, e_a, e_b, pos_b, p_a, p_b, tier_b,
                    &mut kill_events, &mut player_died_events, &mut score_events,
                    &meshes, &materials, &game_state, &mut respawn_timers, &mut dead,
                );
            } else if diff < -JOUST_DEAD_ZONE {
                // B wins
                handle_kill(
                    &mut commands, e_b, e_a, pos_a, p_b, p_a, tier_a,
                    &mut kill_events, &mut player_died_events, &mut score_events,
                    &meshes, &materials, &game_state, &mut respawn_timers, &mut dead,
                );
            } else {
                // Bounce both
                bounce_events.send(JoustBounceEvent {
                    entity_a: e_a,
                    entity_b: e_b,
                });
            }
        }
    }

    // Apply bounces
    for event in bounce_events.send_buffer() {
        let _ = event;
    }
    // Bounce handling done via separate pass below
}

#[allow(clippy::too_many_arguments)]
fn handle_kill(
    commands: &mut Commands,
    winner: Entity,
    loser: Entity,
    loser_pos: Vec2,
    winner_player: Option<&Player>,
    loser_player: Option<&Player>,
    loser_tier: Option<&EnemyTier>,
    kill_events: &mut EventWriter<JoustKillEvent>,
    player_died_events: &mut EventWriter<PlayerDiedEvent>,
    score_events: &mut EventWriter<ScoreEvent>,
    meshes: &SharedMeshes,
    materials: &SharedMaterials,
    game_state: &GameState,
    respawn_timers: &mut RespawnTimers,
    dead: &mut Vec<Entity>,
) {
    dead.push(loser);

    // Award points to winner if they're a player
    if let Some(player) = winner_player {
        if let Some(tier) = loser_tier {
            score_events.send(ScoreEvent {
                player_id: player.id,
                points: tier.score(),
            });
        }
    }

    // Spawn egg if loser is enemy
    if let Some(tier) = loser_tier {
        let hatch_time = get_wave_def(game_state.wave).egg_hatch_time;
        spawn_egg_entity(commands, meshes, materials, loser_pos, *tier, hatch_time);
    }

    // Handle player death
    if let Some(player) = loser_player {
        player_died_events.send(PlayerDiedEvent {
            player_id: player.id,
            position: loser_pos,
        });
        respawn_timers.timers.push((
            player.id,
            Timer::from_seconds(PLAYER_RESPAWN_DELAY, TimerMode::Once),
        ));
    }

    kill_events.send(JoustKillEvent {
        winner,
        loser_position: loser_pos,
        loser_tier: loser_tier.copied(),
        winner_player_id: winner_player.map(|p| p.id),
    });

    commands.entity(loser).despawn();

    // Give winner brief invincibility and bounce
    commands
        .entity(winner)
        .insert(Invincible(Timer::from_seconds(
            BOUNCE_INVINCIBILITY,
            TimerMode::Once,
        )));
}

fn egg_collection_system(
    mut commands: Commands,
    players: Query<(&Player, &Transform), Without<Egg>>,
    eggs: Query<(Entity, &Transform), With<Egg>>,
    mut score_events: EventWriter<ScoreEvent>,
    mut collected_events: EventWriter<EggCollectedEvent>,
) {
    for (player, p_transform) in &players {
        for (egg_entity, e_transform) in &eggs {
            let dx = wrapped_distance(
                p_transform.translation.x,
                e_transform.translation.x,
            );
            let dy = (p_transform.translation.y - e_transform.translation.y).abs();
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < RIDER_RADIUS + EGG_RADIUS {
                score_events.send(ScoreEvent {
                    player_id: player.id,
                    points: SCORE_COLLECT_EGG,
                });
                collected_events.send(EggCollectedEvent {
                    player_id: player.id,
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
            let pos = transform.translation.truncate();
            let new_tier = egg.tier.hatched_tier();
            spawn_enemy_at(&mut commands, &meshes, &materials, pos, new_tier);
            commands.entity(entity).despawn();
        }
    }
}

fn lava_kill_system(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Transform,
        Option<&Player>,
        Option<&Enemy>,
        Option<&Egg>,
    ), With<Velocity>>,
    mut player_died_events: EventWriter<PlayerDiedEvent>,
    mut respawn_timers: ResMut<RespawnTimers>,
    mut game_state: ResMut<GameState>,
) {
    for (entity, transform, player, enemy, egg) in &query {
        if transform.translation.y - RIDER_RADIUS < LAVA_Y {
            if let Some(p) = player {
                player_died_events.send(PlayerDiedEvent {
                    player_id: p.id,
                    position: transform.translation.truncate(),
                });
                if game_state.lives[p.id as usize] > 0 {
                    game_state.lives[p.id as usize] -= 1;
                    respawn_timers.timers.push((
                        p.id,
                        Timer::from_seconds(PLAYER_RESPAWN_DELAY, TimerMode::Once),
                    ));
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

fn handle_score_events(
    mut events: EventReader<ScoreEvent>,
    mut game_state: ResMut<GameState>,
) {
    for event in events.read() {
        let pid = event.player_id as usize;
        let old_score = game_state.scores[pid];
        game_state.scores[pid] += event.points;
        let new_score = game_state.scores[pid];

        // Extra life check
        let old_lives_earned = old_score / EXTRA_LIFE_INTERVAL;
        let new_lives_earned = new_score / EXTRA_LIFE_INTERVAL;
        if new_lives_earned > old_lives_earned {
            game_state.lives[pid] = (game_state.lives[pid] + 1).min(MAX_LIVES);
        }
    }
}

fn handle_player_died(
    mut events: EventReader<PlayerDiedEvent>,
    mut game_state: ResMut<GameState>,
) {
    for event in events.read() {
        let pid = event.player_id as usize;
        if game_state.lives[pid] > 0 {
            game_state.lives[pid] -= 1;
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
