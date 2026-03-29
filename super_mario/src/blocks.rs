use bevy::prelude::*;
use bevy::ecs::system::SystemParam;

use crate::{
    components::{
        BlockContents, BrickBlock, Coin, Collider, Mushroom, Player, PowerState, QuestionBlock,
        QuestionMarkVisual, Solid, Velocity,
    },
    constants::*,
    messages::BlockHit,
    resources::GameData,
    states::{AppState, PlayState},
};

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_block_hits,
                animate_bumping_blocks,
                animate_coin_pops,
                animate_score_popups,
                animate_brick_debris,
                animate_mushroom_emergence,
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            (move_mushrooms, collect_mushrooms, despawn_fallen_mushrooms)
                .chain()
                .run_if(in_state(AppState::Playing))
                .run_if(in_state(PlayState::Running)),
        );
    }
}

#[derive(Component)]
struct BlockBumpAnimation {
    base_y: f32,
    timer: Timer,
}

#[derive(Component)]
struct CoinPopAnimation {
    start_y: f32,
    timer: Timer,
}

#[derive(Component)]
struct FloatingScore {
    start_y: f32,
    timer: Timer,
}

#[derive(Component)]
struct BrickDebris {
    velocity: Vec2,
    timer: Timer,
}

#[derive(Component)]
struct MushroomEmergence {
    start_y: f32,
    timer: Timer,
}

#[derive(SystemParam)]
struct BlockHitParams<'w, 's> {
    question_blocks: Query<
        'w,
        's,
        (
            &'static mut QuestionBlock,
            &'static Transform,
            &'static Children,
            &'static mut MeshMaterial2d<ColorMaterial>,
        ),
    >,
    brick_blocks: Query<'w, 's, &'static Transform, With<BrickBlock>>,
    hard_blocks: Query<'w, 's, &'static Transform, (With<crate::components::HardBlock>, Without<QuestionBlock>)>,
    question_mark_query: Query<'w, 's, (), With<QuestionMarkVisual>>,
    game_data: ResMut<'w, GameData>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    meshes: ResMut<'w, Assets<Mesh>>,
}

fn handle_block_hits(
    mut commands: Commands,
    mut block_hits: MessageReader<BlockHit>,
    player_query: Query<&PowerState>,
    mut params: BlockHitParams,
) {
    let player_power = player_query
        .single()
        .copied()
        .unwrap_or(PowerState::Small);

    for hit in block_hits.read() {
        if let Ok((mut question_block, block_transform, children, mut material)) = params.question_blocks.get_mut(hit.entity) {
            commands.entity(hit.entity).insert(BlockBumpAnimation {
                base_y: block_transform.translation.y,
                timer: Timer::from_seconds(BLOCK_BUMP_DURATION, TimerMode::Once),
            });

            if question_block.spent {
                continue;
            }

            question_block.spent = true;
            *material = MeshMaterial2d(params.materials.add(COLOR_QUESTION_BLOCK_SPENT));

            for child in children.iter() {
                if params.question_mark_query.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }

            match question_block.contents {
                BlockContents::Coin => {
                    params.game_data.add_coin();
                    spawn_coin_pop(&mut commands, &mut params.meshes, &mut params.materials, block_transform.translation);
                    spawn_score_popup(&mut commands, block_transform.translation, 200);
                }
                BlockContents::Mushroom => {
                    spawn_mushroom_emergence(
                        &mut commands,
                        &mut params.meshes,
                        &mut params.materials,
                        block_transform.translation,
                    );
                }
            }
            continue;
        }

        if let Ok(block_transform) = params.brick_blocks.get(hit.entity) {
            if player_power == PowerState::Big {
                params.game_data.score += SCORE_BRICK;
                spawn_score_popup(&mut commands, block_transform.translation, SCORE_BRICK);
                spawn_brick_debris(
                    &mut commands,
                    &mut params.meshes,
                    &mut params.materials,
                    block_transform.translation,
                );
                commands.entity(hit.entity).despawn();
            } else {
                commands.entity(hit.entity).insert(BlockBumpAnimation {
                    base_y: block_transform.translation.y,
                    timer: Timer::from_seconds(BLOCK_BUMP_DURATION, TimerMode::Once),
                });
            }
            continue;
        }

        if let Ok(block_transform) = params.hard_blocks.get(hit.entity) {
            commands.entity(hit.entity).insert(BlockBumpAnimation {
                base_y: block_transform.translation.y,
                timer: Timer::from_seconds(BLOCK_BUMP_DURATION, TimerMode::Once),
            });
        }
    }
}

fn animate_bumping_blocks(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BlockBumpAnimation)>,
) {
    for (entity, mut transform, mut animation) in &mut query {
        animation.timer.tick(time.delta());
        let progress = animation.timer.fraction();
        let arc = (progress * std::f32::consts::PI).sin();
        transform.translation.y = animation.base_y + arc * BLOCK_BUMP_HEIGHT;

        if animation.timer.is_finished() {
            transform.translation.y = animation.base_y;
            commands.entity(entity).remove::<BlockBumpAnimation>();
        }
    }
}

fn animate_coin_pops(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &MeshMaterial2d<ColorMaterial>,
            &mut CoinPopAnimation,
        ),
        With<Coin>,
    >,
) {
    for (entity, mut transform, material_handle, mut animation) in &mut query {
        animation.timer.tick(time.delta());
        let progress = animation.timer.fraction();
        let rise = (1.0 - (2.0 * progress - 1.0).powi(2)).max(0.0);
        transform.translation.y = animation.start_y + rise * COIN_POP_HEIGHT;
        transform.scale = Vec3::splat(1.0 - progress * 0.25);

        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.color.set_alpha(1.0 - progress);
        }

        if animation.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_score_popups(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut FloatingScore)>,
) {
    for (entity, mut transform, mut text_color, mut popup) in &mut query {
        popup.timer.tick(time.delta());
        let progress = popup.timer.fraction();
        transform.translation.y = popup.start_y + progress * SCORE_POP_RISE;
        text_color.0.set_alpha(1.0 - progress);

        if popup.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_brick_debris(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BrickDebris)>,
) {
    for (entity, mut transform, mut debris) in &mut query {
        debris.timer.tick(time.delta());
        debris.velocity.y += BRICK_DEBRIS_GRAVITY * time.delta_secs();
        transform.translation += debris.velocity.extend(0.0) * time.delta_secs();
        transform.rotate_z(6.0 * time.delta_secs());

        if debris.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_mushroom_emergence(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut MushroomEmergence), With<Mushroom>>,
) {
    for (entity, mut transform, mut emergence) in &mut query {
        emergence.timer.tick(time.delta());
        let progress = emergence.timer.fraction();
        transform.translation.y = emergence.start_y + progress * MUSHROOM_EMERGE_HEIGHT;

        if emergence.timer.is_finished() {
            commands.entity(entity).remove::<MushroomEmergence>();
            commands.entity(entity).insert((
                Velocity {
                    x: MUSHROOM_SPEED,
                    y: 0.0,
                },
                Collider {
                    width: MUSHROOM_COLLIDER_WIDTH,
                    height: MUSHROOM_COLLIDER_HEIGHT,
                },
            ));
        }
    }
}

fn spawn_coin_pop(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    block_translation: Vec3,
) {
    commands.spawn((
        Coin,
        DespawnOnExit(AppState::Playing),
        CoinPopAnimation {
            start_y: block_translation.y + TILE_SIZE * 0.25,
            timer: Timer::from_seconds(COIN_POP_DURATION, TimerMode::Once),
        },
        Mesh2d(meshes.add(Circle::new(TILE_SIZE * 0.22))),
        MeshMaterial2d(materials.add(Color::srgba(1.3, 1.05, 0.18, 1.0))),
        Transform::from_xyz(
            block_translation.x,
            block_translation.y + TILE_SIZE * 0.25,
            Z_ITEMS + 1.0,
        ),
    ));
}

fn spawn_score_popup(commands: &mut Commands, block_translation: Vec3, score: u32) {
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        FloatingScore {
            start_y: block_translation.y + TILE_SIZE * 0.4,
            timer: Timer::from_seconds(SCORE_POP_DURATION, TimerMode::Once),
        },
        Text2d::new(format!("+{score}")),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.98, 0.85, 1.0)),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(
            block_translation.x,
            block_translation.y + TILE_SIZE * 0.4,
            Z_PARTICLES,
        ),
    ));
}

fn spawn_brick_debris(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    block_translation: Vec3,
) {
    let fragment_mesh = meshes.add(Rectangle::new(TILE_SIZE * 0.28, TILE_SIZE * 0.22));
    let fragment_material = materials.add(COLOR_BRICK);
    let velocities = [
        Vec2::new(-160.0, 280.0),
        Vec2::new(-80.0, 360.0),
        Vec2::new(80.0, 360.0),
        Vec2::new(160.0, 280.0),
    ];
    let offsets = [
        Vec2::new(-8.0, 6.0),
        Vec2::new(-8.0, -6.0),
        Vec2::new(8.0, 6.0),
        Vec2::new(8.0, -6.0),
    ];

    for (velocity, offset) in velocities.into_iter().zip(offsets) {
        commands.spawn((
            DespawnOnExit(AppState::Playing),
            BrickDebris {
                velocity,
                timer: Timer::from_seconds(BRICK_DEBRIS_LIFETIME, TimerMode::Once),
            },
            Mesh2d(fragment_mesh.clone()),
            MeshMaterial2d(fragment_material.clone()),
            Transform::from_xyz(
                block_translation.x + offset.x,
                block_translation.y + offset.y,
                Z_PARTICLES,
            ),
        ));
    }
}

fn spawn_mushroom_emergence(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    block_translation: Vec3,
) {
    let body_mesh = meshes.add(Rectangle::new(TILE_SIZE * 0.5, TILE_SIZE * 0.38));
    let cap_mesh = meshes.add(Circle::new(TILE_SIZE * 0.28));
    let spot_mesh = meshes.add(Circle::new(TILE_SIZE * 0.06));
    let stem_material = materials.add(COLOR_MUSHROOM_SPOTS);
    let cap_material = materials.add(COLOR_MUSHROOM_RED);
    let spot_material = materials.add(COLOR_MUSHROOM_SPOTS);

    commands
        .spawn((
            Mushroom,
            DespawnOnExit(AppState::Playing),
            MushroomEmergence {
                start_y: block_translation.y + TILE_SIZE * 0.3,
                timer: Timer::from_seconds(MUSHROOM_EMERGE_DURATION, TimerMode::Once),
            },
            Transform::from_xyz(
                block_translation.x,
                block_translation.y + TILE_SIZE * 0.3,
                Z_ITEMS + 0.5,
            ),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(body_mesh),
                MeshMaterial2d(stem_material),
                Transform::from_xyz(0.0, -TILE_SIZE * 0.08, 0.0),
            ));
            parent.spawn((
                Mesh2d(cap_mesh.clone()),
                MeshMaterial2d(cap_material),
                Transform::from_xyz(0.0, TILE_SIZE * 0.12, 0.1)
                    .with_scale(Vec3::new(1.0, 0.75, 1.0)),
            ));
            parent.spawn((
                Mesh2d(spot_mesh.clone()),
                MeshMaterial2d(spot_material.clone()),
                Transform::from_xyz(-TILE_SIZE * 0.1, TILE_SIZE * 0.18, 0.2),
            ));
            parent.spawn((
                Mesh2d(spot_mesh),
                MeshMaterial2d(spot_material),
                Transform::from_xyz(TILE_SIZE * 0.1, TILE_SIZE * 0.18, 0.2),
            ));
        });
}

fn move_mushrooms(
    time: Res<Time>,
    mut mushroom_query: Query<
        (&mut Transform, &mut Velocity, &Collider),
        (With<Mushroom>, Without<MushroomEmergence>, Without<Solid>, Without<Player>),
    >,
    solid_query: Query<(&Transform, &Collider), (With<Solid>, Without<Mushroom>)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity, collider) in &mut mushroom_query {
        velocity.y = (velocity.y + GRAVITY * dt).max(MAX_FALL_SPEED);

        let half_w = collider.width * 0.5;
        let half_h = collider.height * 0.5;

        // Horizontal
        let next_x = transform.translation.x + velocity.x * dt;
        let mut blocked = false;
        for (solid_tf, solid_col) in &solid_query {
            if aabb_overlap(
                next_x,
                transform.translation.y,
                half_w,
                half_h,
                solid_tf.translation.x,
                solid_tf.translation.y,
                solid_col.width * 0.5,
                solid_col.height * 0.5,
            ) {
                blocked = true;
                break;
            }
        }
        if blocked {
            velocity.x = -velocity.x;
        } else {
            transform.translation.x = next_x;
        }

        // Vertical
        let next_y = transform.translation.y + velocity.y * dt;
        let mut landed = false;
        let mut land_y = next_y;
        for (solid_tf, solid_col) in &solid_query {
            if aabb_overlap(
                transform.translation.x,
                next_y,
                half_w,
                half_h,
                solid_tf.translation.x,
                solid_tf.translation.y,
                solid_col.width * 0.5,
                solid_col.height * 0.5,
            ) {
                if velocity.y < 0.0 {
                    land_y = solid_tf.translation.y + solid_col.height * 0.5 + half_h;
                    landed = true;
                }
                break;
            }
        }
        if landed {
            transform.translation.y = land_y;
            velocity.y = 0.0;
        } else {
            transform.translation.y = next_y;
        }
    }
}

fn collect_mushrooms(
    mut commands: Commands,
    mut player_query: Query<
        (&mut Transform, &mut Collider, &mut PowerState),
        (With<Player>, Without<Mushroom>),
    >,
    mushroom_query: Query<
        (Entity, &Transform, &Collider),
        (With<Mushroom>, Without<MushroomEmergence>, Without<Player>),
    >,
    mut game_data: ResMut<GameData>,
) {
    let Ok((mut player_tf, mut player_col, mut power_state)) = player_query.single_mut() else {
        return;
    };

    let p_hw = player_col.width * 0.5;
    let p_hh = player_col.height * 0.5;

    for (mushroom_entity, mushroom_tf, mushroom_col) in &mushroom_query {
        let m_hw = mushroom_col.width * 0.5;
        let m_hh = mushroom_col.height * 0.5;

        if (player_tf.translation.x - mushroom_tf.translation.x).abs() < p_hw + m_hw
            && (player_tf.translation.y - mushroom_tf.translation.y).abs() < p_hh + m_hh
        {
            commands.entity(mushroom_entity).despawn();
            game_data.score += 1000;

            if *power_state == PowerState::Small {
                *power_state = PowerState::Big;
                player_tf.translation.y += (PLAYER_BIG_HEIGHT - PLAYER_HEIGHT) * 0.5;
                player_tf.scale.y = PLAYER_BIG_HEIGHT / PLAYER_HEIGHT;
                player_col.width = PLAYER_BIG_WIDTH;
                player_col.height = PLAYER_BIG_HEIGHT;
            }
            break;
        }
    }
}

fn despawn_fallen_mushrooms(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<Mushroom>, Without<MushroomEmergence>)>,
) {
    for (entity, transform) in &query {
        if transform.translation.y < DEATH_Y {
            commands.entity(entity).despawn();
        }
    }
}

fn aabb_overlap(
    ax: f32,
    ay: f32,
    ahw: f32,
    ahh: f32,
    bx: f32,
    by: f32,
    bhw: f32,
    bhh: f32,
) -> bool {
    (ax - bx).abs() < ahw + bhw && (ay - by).abs() < ahh + bhh
}
