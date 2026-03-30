use bevy::prelude::*;

use crate::collision::aabb_overlap;
use crate::components::*;
use crate::constants::*;
use crate::level::*;
use crate::resources::*;
use crate::states::*;

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
    mut game_data: ResMut<GameData>,
    tile_query: Query<(Entity, &TilePos, &Transform), With<Tile>>,
    enemy_query: Query<
        (Entity, &Transform),
        (With<EnemyActive>, Without<Squished>, Without<Tile>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
            let empty_mat =
                materials.add(ColorMaterial::from_color(Color::srgb(0.35, 0.25, 0.15)));
            commands
                .entity(tile_entity)
                .insert(MeshMaterial2d(empty_mat));

            // Spawn content
            if ch == 'M' {
                spawn_mushroom(&mut commands, &mut meshes, &mut materials, tile_pos);
            } else {
                spawn_coin_pop(&mut commands, &mut meshes, &mut materials, tile_pos);
                game_data.add_coin();
                game_data.score += COIN_SCORE;
            }

            // Mark grid as used (solid but not hittable)
            level.set_char(col, row, 'E');
        }
        'B' => {
            if hit.is_big {
                // Big Mario breaks brick
                commands.entity(tile_entity).despawn();
                level.set_char(col, row, '.');

                // Spawn 4 particles
                let particle_mesh =
                    meshes.add(Rectangle::new(BRICK_PARTICLE_SIZE, BRICK_PARTICLE_SIZE));
                let particle_mat = materials
                    .add(ColorMaterial::from_color(Color::srgb(0.72, 0.40, 0.10)));

                let velocities = [
                    (-BRICK_PARTICLE_SPEED, BRICK_PARTICLE_SPEED * 1.5),
                    (BRICK_PARTICLE_SPEED, BRICK_PARTICLE_SPEED * 1.5),
                    (-BRICK_PARTICLE_SPEED * 0.7, BRICK_PARTICLE_SPEED),
                    (BRICK_PARTICLE_SPEED * 0.7, BRICK_PARTICLE_SPEED),
                ];

                for (vx, vy) in velocities {
                    commands.spawn((
                        BrickParticle {
                            vel_x: vx,
                            vel_y: vy,
                            timer: Timer::from_seconds(
                                BRICK_PARTICLE_DURATION,
                                TimerMode::Once,
                            ),
                        },
                        Mesh2d(particle_mesh.clone()),
                        MeshMaterial2d(particle_mat.clone()),
                        Transform::from_xyz(tile_pos.x, tile_pos.y, Z_ITEM),
                        DespawnOnExit(AppState::Playing),
                    ));
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
            game_data.score += STOMP_SCORE;

            commands.spawn((
                ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
                Text2d::new(format!("+{STOMP_SCORE}")),
                TextFont {
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y + 10.0,
                    Z_PLAYER + 1.0,
                ),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

fn spawn_coin_pop(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    block_pos: Vec3,
) {
    let coin_mesh = meshes.add(Circle::new(4.0));
    let coin_mat = materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.85, 0.0)));

    commands.spawn((
        CoinPop {
            vel_y: COIN_POP_IMPULSE,
            timer: Timer::from_seconds(COIN_POP_DURATION, TimerMode::Once),
        },
        Mesh2d(coin_mesh),
        MeshMaterial2d(coin_mat),
        Transform::from_xyz(block_pos.x, block_pos.y + TILE_SIZE, Z_ITEM),
        DespawnOnExit(AppState::Playing),
    ));
}

fn spawn_mushroom(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    block_pos: Vec3,
) {
    let cap_mesh = meshes.add(Ellipse::new(7.0, 5.0));
    let cap_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.8, 0.1, 0.1)));
    let stem_mesh = meshes.add(Rectangle::new(8.0, 6.0));
    let stem_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.95, 0.85, 0.7)));

    commands
        .spawn((
            Mushroom,
            MushroomEmerging { remaining: TILE_SIZE },
            CollisionSize {
                width: MUSHROOM_WIDTH,
                height: MUSHROOM_HEIGHT,
            },
            Velocity::default(),
            Grounded::default(),
            Mesh2d(cap_mesh),
            MeshMaterial2d(cap_mat),
            Transform::from_xyz(block_pos.x, block_pos.y, Z_ITEM),
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(stem_mesh),
                MeshMaterial2d(stem_mat),
                Transform::from_xyz(0.0, -5.0, 0.0),
            ));
        });
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
    mut game_data: ResMut<GameData>,
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
        game_data.score += FLOATING_COIN_SCORE;
        game_data.add_coin();

        commands.spawn((
            ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
            Text2d::new(format!("+{FLOATING_COIN_SCORE}")),
            TextFont {
                font_size: 8.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.85, 0.0)),
            Transform::from_xyz(
                coin_tf.translation.x,
                coin_tf.translation.y + 10.0,
                Z_PLAYER + 1.0,
            ),
            DespawnOnExit(AppState::Playing),
        ));
    }
}
