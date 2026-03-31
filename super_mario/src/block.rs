use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::aabb_overlap;
use crate::components::*;
use crate::constants::*;
use crate::level::*;
use crate::resources::*;
use crate::states::*;
use crate::ui;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingBlockHit>()
            .add_systems(
                Update,
                (process_block_hits, floating_coin_collection)
                    .in_set(GameplaySet::Late),
            )
            .add_systems(
                Update,
                (block_bounce_system, coin_pop_system, brick_particle_system)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn process_block_hits(
    mut commands: Commands,
    mut pending: ResMut<PendingBlockHit>,
    mut level: ResMut<LevelGrid>,
    mut score_events: MessageWriter<ScoreEvent>,
    mut coin_events: MessageWriter<CoinEvent>,
    assets: Res<GameAssets>,
    tile_query: Query<(Entity, &TilePos, &Transform), With<Tile>>,
    enemy_query: Query<
        (Entity, &Transform),
        (With<EnemyActive>, Without<Squished>, Without<Tile>),
    >,
) {
    let Some(hit) = pending.hit.take() else { return };

    let col = hit.col;
    let row = hit.row;
    let ch = level.get_char(col, row);

    // Find the tile entity at this position
    let tile_data = tile_query
        .iter()
        .find(|(_, tp, _)| tp.col == col && tp.row == row);
    let Some((tile_entity, _, tile_tf)) = tile_data else {
        return;
    };
    let tile_pos = tile_tf.translation;

    match ch {
        '?' | 'M' => {
            // Bounce animation
            commands.entity(tile_entity).insert(BlockBounce {
                timer: Timer::from_seconds(BLOCK_BOUNCE_DURATION, TimerMode::Once),
                original_y: tile_pos.y,
            });
            commands.entity(tile_entity).insert(BlockUsed);

            // Change to used appearance
            commands
                .entity(tile_entity)
                .insert(MeshMaterial2d(assets.tile.empty_block_mat.clone()));

            // Spawn content
            if ch == 'M' {
                if hit.player_size == PlayerSize::Small {
                    assets.mushroom.spawn(&mut commands, tile_pos);
                } else {
                    assets.fire_flower.spawn(&mut commands, tile_pos);
                }
            } else {
                assets.coin_pop.spawn(&mut commands, tile_pos);
                coin_events.write(CoinEvent);
                score_events.write(ScoreEvent { points: COIN_SCORE });
            }

            // Mark grid as used (solid but not hittable)
            level.set_char(col, row, 'E');
        }
        'B' => {
            if hit.player_size != PlayerSize::Small {
                // Big Mario breaks brick
                commands.entity(tile_entity).despawn();
                level.set_char(col, row, '.');

                let velocities = [
                    (-BRICK_PARTICLE_SPEED, BRICK_PARTICLE_SPEED * 1.5),
                    (BRICK_PARTICLE_SPEED, BRICK_PARTICLE_SPEED * 1.5),
                    (-BRICK_PARTICLE_SPEED * 0.7, BRICK_PARTICLE_SPEED),
                    (BRICK_PARTICLE_SPEED * 0.7, BRICK_PARTICLE_SPEED),
                ];

                for (vx, vy) in velocities {
                    assets.brick_particle.spawn(&mut commands, tile_pos, vx, vy);
                }
            } else {
                // Small Mario bumps brick
                commands.entity(tile_entity).insert(BlockBounce {
                    timer: Timer::from_seconds(BLOCK_BOUNCE_DURATION, TimerMode::Once),
                    original_y: tile_pos.y,
                });
            }
        }
        _ => return,
    }

    // Kill enemies standing on top of the hit block
    let (block_cx, block_cy) = tile_to_world(col as usize, row as usize);
    let kill_y = block_cy + TILE_SIZE;

    for (entity, transform) in &enemy_query {
        let dx = (transform.translation.x - block_cx).abs();
        let dy = (transform.translation.y - kill_y).abs();

        if dx < BRICK_BUMP_KILL_RANGE && dy < TILE_SIZE / 2.0 {
            commands.entity(entity).despawn();
            score_events.write(ScoreEvent { points: STOMP_SCORE });

            ui::spawn_score_popup(
                &mut commands, STOMP_SCORE,
                transform.translation.x,
                transform.translation.y + 10.0,
            );
        }
    }
}


// ── Animation Systems ──

fn block_bounce_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BlockBounce, &mut Transform)>,
) {
    for (entity, mut bounce, mut transform) in &mut query {
        bounce.timer.tick(time.delta());

        let fraction = bounce.timer.fraction();
        let offset = (fraction * std::f32::consts::PI).sin() * BLOCK_BOUNCE_HEIGHT;
        transform.translation.y = bounce.original_y + offset;

        if bounce.timer.is_finished() {
            transform.translation.y = bounce.original_y;
            commands.entity(entity).remove::<BlockBounce>();
        }
    }
}

fn coin_pop_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut CoinPop, &mut Transform)>,
) {
    for (entity, mut coin, mut transform) in &mut query {
        coin.timer.tick(time.delta());

        let dt = time.delta_secs();
        coin.vel_y -= GRAVITY_DESCENDING * dt;
        transform.translation.y += coin.vel_y * dt;

        if coin.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn brick_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BrickParticle, &mut Transform)>,
) {
    for (entity, mut particle, mut transform) in &mut query {
        particle.timer.tick(time.delta());

        let dt = time.delta_secs();
        particle.vel_y -= GRAVITY_DESCENDING * dt;
        transform.translation.x += particle.vel_x * dt;
        transform.translation.y += particle.vel_y * dt;
        transform.rotate_z(5.0 * dt);

        if particle.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Floating Coin Collection ──

fn floating_coin_collection(
    mut commands: Commands,
    mut score_events: MessageWriter<ScoreEvent>,
    mut coin_events: MessageWriter<CoinEvent>,
    player_query: Query<(&Transform, &CollisionSize), With<Player>>,
    coin_query: Query<(Entity, &Transform), With<FloatingCoin>>,
) {
    let Ok((player_tf, player_size)) = player_query.single() else {
        return;
    };

    let coin_half = FLOATING_COIN_SIZE / 2.0;

    for (entity, coin_tf) in &coin_query {
        if aabb_overlap(
            player_tf.translation.x, player_tf.translation.y,
            player_size.width / 2.0, player_size.height / 2.0,
            coin_tf.translation.x, coin_tf.translation.y,
            coin_half, coin_half,
        ).is_none() {
            continue;
        }

        commands.entity(entity).despawn();
        score_events.write(ScoreEvent { points: FLOATING_COIN_SCORE });
        coin_events.write(CoinEvent);

        ui::spawn_score_popup_colored(
            &mut commands, FLOATING_COIN_SCORE,
            coin_tf.translation.x,
            coin_tf.translation.y + 10.0,
            Color::srgb(1.0, 0.85, 0.0),
        );
    }
}
