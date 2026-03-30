use bevy::prelude::*;

use crate::collision::aabb_overlap;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::*;

pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            mushroom_emerge.run_if(in_state(PlayState::Running)),
        )
        .add_systems(
            Update,
            mushroom_collection.in_set(GameplaySet::Late),
        )
        .add_systems(
            Update,
            growth_animation_system.run_if(in_state(PlayState::Growing)),
        )
        .add_systems(
            Update,
            invincibility_system.run_if(in_state(PlayState::Running)),
        )
        .add_systems(
            Update,
            ducking_system.run_if(in_state(PlayState::Running)),
        );
    }
}

// ── Mushroom Emerge ──

fn mushroom_emerge(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut MushroomEmerging, &mut Transform), With<Mushroom>>,
) {
    for (entity, mut emerging, mut transform) in &mut query {
        let move_amount = MUSHROOM_EMERGE_SPEED * time.delta_secs();
        emerging.remaining -= move_amount;
        transform.translation.y += move_amount;

        if emerging.remaining <= 0.0 {
            commands.entity(entity).remove::<MushroomEmerging>();
            commands.entity(entity).insert((
                EnemyWalker {
                    speed: MUSHROOM_SPEED,
                    direction: 1.0,
                },
                EnemyActive,
            ));
        }
    }
}

// ── Mushroom Collection ──

fn mushroom_collection(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    player_query: Query<
        (Entity, &Transform, &CollisionSize, &PlayerSize),
        With<Player>,
    >,
    mushroom_query: Query<
        (Entity, &Transform, &CollisionSize),
        (With<Mushroom>, Without<MushroomEmerging>, Without<Player>),
    >,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    let Ok((player_entity, player_tf, player_coll, &player_size)) = player_query.single()
    else {
        return;
    };

    for (mush_entity, mush_tf, mush_coll) in &mushroom_query {
        if aabb_overlap(
            player_tf.translation.x, player_tf.translation.y,
            player_coll.width / 2.0, player_coll.height / 2.0,
            mush_tf.translation.x, mush_tf.translation.y,
            mush_coll.width / 2.0, mush_coll.height / 2.0,
        ).is_some() {
            commands.entity(mush_entity).despawn();
            game_data.score += MUSHROOM_SCORE;

            // Score popup
            commands.spawn((
                ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
                Text2d::new(format!("+{MUSHROOM_SCORE}")),
                TextFont {
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(
                    mush_tf.translation.x,
                    mush_tf.translation.y + 10.0,
                    Z_PLAYER + 1.0,
                ),
                DespawnOnExit(AppState::Playing),
            ));

            if player_size == PlayerSize::Small {
                // Start growth animation
                commands.entity(player_entity).insert(GrowthAnimation {
                    timer: Timer::from_seconds(GROWTH_DURATION, TimerMode::Once),
                    flash_timer: Timer::from_seconds(
                        GROWTH_FLASH_INTERVAL,
                        TimerMode::Repeating,
                    ),
                    growing: true,
                });
                next_play_state.set(PlayState::Growing);
            }

            break;
        }
    }
}

// ── Growth Animation ──

fn growth_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut GrowthAnimation,
            &mut Transform,
            &mut CollisionSize,
            &mut PlayerSize,
            &mut Mesh2d,
        ),
        With<Player>,
    >,
    mut next_play_state: ResMut<NextState<PlayState>>,
    player_meshes: Option<Res<PlayerMeshes>>,
) {
    let Ok((entity, mut growth, mut transform, mut coll_size, mut player_size, mut mesh)) =
        query.single_mut()
    else {
        return;
    };
    let Some(meshes) = player_meshes else { return };

    growth.timer.tick(time.delta());
    growth.flash_timer.tick(time.delta());

    // Flash between sizes on each interval
    if growth.flash_timer.is_finished() {
        growth.flash_timer.reset();

        if coll_size.height == PLAYER_SMALL_HEIGHT {
            // Switch to big visual
            transform.translation.y += (PLAYER_BIG_HEIGHT - PLAYER_SMALL_HEIGHT) / 2.0;
            coll_size.height = PLAYER_BIG_HEIGHT;
            mesh.0 = meshes.big.clone();
        } else {
            // Switch to small visual
            transform.translation.y -= (PLAYER_BIG_HEIGHT - PLAYER_SMALL_HEIGHT) / 2.0;
            coll_size.height = PLAYER_SMALL_HEIGHT;
            mesh.0 = meshes.small.clone();
        }
    }

    if growth.timer.is_finished() {
        let target_height = if growth.growing {
            PLAYER_BIG_HEIGHT
        } else {
            PLAYER_SMALL_HEIGHT
        };

        // Adjust position from current to target
        let height_diff = target_height - coll_size.height;
        transform.translation.y += height_diff / 2.0;

        coll_size.height = target_height;
        *player_size = if growth.growing {
            PlayerSize::Big
        } else {
            PlayerSize::Small
        };
        mesh.0 = if growth.growing {
            meshes.big.clone()
        } else {
            meshes.small.clone()
        };

        if !growth.growing {
            // Shrink complete — grant invincibility
            commands.entity(entity).insert(Invincible {
                timer: Timer::from_seconds(INVINCIBILITY_DURATION, TimerMode::Once),
            });
        }

        commands.entity(entity).remove::<GrowthAnimation>();
        next_play_state.set(PlayState::Running);
    }
}

// ── Invincibility ──

fn invincibility_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invincible, &mut Visibility), With<Player>>,
) {
    let Ok((entity, mut invincible, mut vis)) = query.single_mut() else {
        return;
    };

    invincible.timer.tick(time.delta());

    if invincible.timer.is_finished() {
        *vis = Visibility::Inherited;
        commands.entity(entity).remove::<Invincible>();
    } else {
        let flash = ((invincible.timer.elapsed_secs() / 0.05) as u32) % 2 == 0;
        *vis = if flash {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

// ── Ducking ──

fn ducking_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &PlayerSize,
            &Grounded,
            &mut CollisionSize,
            &mut Transform,
            &mut Mesh2d,
            Has<Ducking>,
        ),
        With<Player>,
    >,
    player_meshes: Option<Res<PlayerMeshes>>,
) {
    let Ok((entity, player_size, grounded, mut coll_size, mut transform, mut mesh, is_ducking)) =
        query.single_mut()
    else {
        return;
    };
    let Some(meshes) = player_meshes else { return };

    let wants_duck =
        keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS);

    if *player_size == PlayerSize::Big && grounded.0 && wants_duck && !is_ducking {
        // Start ducking
        commands.entity(entity).insert(Ducking);
        let old_height = coll_size.height;
        coll_size.height = PLAYER_SMALL_HEIGHT;
        mesh.0 = meshes.small.clone();
        transform.translation.y -= (old_height - PLAYER_SMALL_HEIGHT) / 2.0;
    } else if is_ducking && (!wants_duck || !grounded.0 || *player_size != PlayerSize::Big) {
        // Stop ducking
        commands.entity(entity).remove::<Ducking>();
        if *player_size == PlayerSize::Big {
            coll_size.height = PLAYER_BIG_HEIGHT;
            mesh.0 = meshes.big.clone();
            transform.translation.y += (PLAYER_BIG_HEIGHT - PLAYER_SMALL_HEIGHT) / 2.0;
        }
    }
}
