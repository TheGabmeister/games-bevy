use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::{self, entities_overlap, WallAction};
use crate::components::*;
use crate::constants::*;
use crate::level::LevelGrid;
use crate::resources::*;
use crate::states::*;
use crate::ui;

pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (mushroom_emerge, fire_flower_emerge).run_if(in_state(PlayState::Running)),
        )
        .add_systems(
            Update,
            (
                mushroom_collection,
                fire_flower_collection,
                fireball_enemy_collision,
            )
                .in_set(GameplaySet::Late),
        )
        .add_systems(
            Update,
            fireball_shoot.in_set(GameplaySet::Input),
        )
        .add_systems(
            Update,
            fireball_physics.in_set(GameplaySet::Physics),
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

// ── Fire Flower Emerge ──

fn fire_flower_emerge(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut FireFlowerEmerging, &mut Transform), With<FireFlower>>,
) {
    for (entity, mut emerging, mut transform) in &mut query {
        let move_amount = MUSHROOM_EMERGE_SPEED * time.delta_secs();
        emerging.remaining -= move_amount;
        transform.translation.y += move_amount;

        if emerging.remaining <= 0.0 {
            commands.entity(entity).remove::<FireFlowerEmerging>();
        }
    }
}

// ── Mushroom Collection ──

fn mushroom_collection(
    mut commands: Commands,
    mut score_events: MessageWriter<ScoreEvent>,
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
        if entities_overlap(player_tf, player_coll, mush_tf, mush_coll) {
            commands.entity(mush_entity).despawn();
            score_events.write(ScoreEvent { points: MUSHROOM_SCORE });

            ui::spawn_score_popup(
                &mut commands, MUSHROOM_SCORE,
                mush_tf.translation.x,
                mush_tf.translation.y + 10.0,
            );

            if player_size == PlayerSize::Small {
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

// ── Fire Flower Collection ──

fn fire_flower_collection(
    mut commands: Commands,
    mut score_events: MessageWriter<ScoreEvent>,
    mut player_query: Query<
        (Entity, &Transform, &CollisionSize, &mut PlayerSize, &mut MeshMaterial2d<ColorMaterial>),
        With<Player>,
    >,
    flower_query: Query<
        (Entity, &Transform, &CollisionSize),
        (With<FireFlower>, Without<FireFlowerEmerging>, Without<Player>),
    >,
    assets: Res<GameAssets>,
) {
    let Ok((_entity, player_tf, player_coll, mut player_size, mut player_mat)) =
        player_query.single_mut()
    else {
        return;
    };

    for (flower_entity, flower_tf, flower_coll) in &flower_query {
        if entities_overlap(player_tf, player_coll, flower_tf, flower_coll) {
            commands.entity(flower_entity).despawn();
            score_events.write(ScoreEvent { points: FIRE_FLOWER_SCORE });

            ui::spawn_score_popup(
                &mut commands, FIRE_FLOWER_SCORE,
                flower_tf.translation.x,
                flower_tf.translation.y + 10.0,
            );

            if *player_size == PlayerSize::Big {
                *player_size = PlayerSize::Fire;
                player_mat.0 = assets.player.fire_mat.clone();
            }

            break;
        }
    }
}

// ── Fireball Shooting ──

fn fireball_shoot(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    player_query: Query<(&Transform, &FacingDirection, &PlayerSize), With<Player>>,
    fireball_query: Query<&Fireball>,
    assets: Res<GameAssets>,
) {
    if !(keyboard.just_pressed(KeyCode::KeyJ) || keyboard.just_pressed(KeyCode::KeyE)) {
        return;
    }

    let Ok((player_tf, facing, player_size)) = player_query.single() else {
        return;
    };

    if *player_size != PlayerSize::Fire {
        return;
    }

    if fireball_query.iter().count() >= MAX_FIREBALLS {
        return;
    }

    let dir = match facing {
        FacingDirection::Left => -1.0,
        FacingDirection::Right => 1.0,
    };

    let spawn_x = player_tf.translation.x + dir * (PLAYER_WIDTH / 2.0 + FIREBALL_RADIUS + 2.0);
    let spawn_y = player_tf.translation.y;

    assets.fireball.spawn(&mut commands, spawn_x, spawn_y, dir);
}

// ── Fireball Physics ──

fn fireball_physics(
    mut commands: Commands,
    time: Res<Time>,
    level: Res<LevelGrid>,
    mut query: Query<(Entity, &Fireball, &mut Velocity, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, fireball, mut vel, mut transform) in &mut query {
        vel.y -= GRAVITY_DESCENDING * dt;
        vel.y = vel.y.max(-TERMINAL_VELOCITY);

        vel.x = FIREBALL_SPEED * fireball.direction;

        transform.translation.x += vel.x * dt;
        transform.translation.y += vel.y * dt;

        let half = FIREBALL_RADIUS;
        let mut dummy_dir = fireball.direction;
        let result = collision::resolve_tile_collisions(
            &level,
            &mut transform.translation,
            &mut vel,
            half,
            half,
            WallAction::Stop,
            &mut dummy_dir,
        );

        if vel.x.abs() < 0.1 {
            commands.entity(entity).despawn();
            continue;
        }

        if result.grounded && vel.y <= 0.0 {
            vel.y = FIREBALL_BOUNCE_IMPULSE;
        }

        if transform.translation.y < DEATH_Y {
            commands.entity(entity).despawn();
        }
    }
}

// ── Fireball–Enemy Collision ──

fn fireball_enemy_collision(
    mut commands: Commands,
    fireball_query: Query<(Entity, &Transform, &CollisionSize), With<Fireball>>,
    goomba_query: Query<
        (Entity, &Transform, &CollisionSize),
        (With<Goomba>, With<EnemyActive>, Without<Squished>, Without<Fireball>),
    >,
    koopa_query: Query<
        (Entity, &Transform, &CollisionSize),
        (With<KoopaTroopa>, With<EnemyActive>, Without<Fireball>, Without<Goomba>),
    >,
    mut score_events: MessageWriter<ScoreEvent>,
    assets: Res<GameAssets>,
) {
    for (fb_entity, fb_tf, fb_coll) in &fireball_query {
        let mut hit = false;

        for (goomba_entity, goomba_tf, goomba_coll) in &goomba_query {
            if entities_overlap(fb_tf, fb_coll, goomba_tf, goomba_coll) {
                commands.entity(goomba_entity).despawn();
                commands.entity(fb_entity).despawn();
                score_events.write(ScoreEvent { points: FIREBALL_SCORE });

                ui::spawn_score_popup(
                    &mut commands, FIREBALL_SCORE,
                    goomba_tf.translation.x,
                    goomba_tf.translation.y + 10.0,
                );

                hit = true;
                break;
            }
        }

        if hit {
            continue;
        }

        for (koopa_entity, koopa_tf, koopa_coll) in &koopa_query {
            if entities_overlap(fb_tf, fb_coll, koopa_tf, koopa_coll) {
                commands.entity(koopa_entity).despawn();
                commands.entity(fb_entity).despawn();
                score_events.write(ScoreEvent { points: FIREBALL_SCORE });

                let shell_y = koopa_tf.translation.y - (KOOPA_HEIGHT - SHELL_HEIGHT) / 2.0;

                assets.shell.spawn(&mut commands, koopa_tf.translation.x, shell_y);

                ui::spawn_score_popup(
                    &mut commands, FIREBALL_SCORE,
                    koopa_tf.translation.x,
                    koopa_tf.translation.y + 10.0,
                );

                break;
            }
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
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        With<Player>,
    >,
    mut next_play_state: ResMut<NextState<PlayState>>,
    assets: Res<GameAssets>,
) {
    let Ok((entity, mut growth, mut transform, mut coll_size, mut player_size, mut mesh, mut mat)) =
        query.single_mut()
    else {
        return;
    };

    growth.timer.tick(time.delta());
    growth.flash_timer.tick(time.delta());

    if growth.flash_timer.is_finished() {
        growth.flash_timer.reset();

        if coll_size.height == PLAYER_SMALL_HEIGHT {
            transform.translation.y += (PLAYER_BIG_HEIGHT - PLAYER_SMALL_HEIGHT) / 2.0;
            coll_size.height = PLAYER_BIG_HEIGHT;
            mesh.0 = assets.player.big_mesh.clone();
        } else {
            transform.translation.y -= (PLAYER_BIG_HEIGHT - PLAYER_SMALL_HEIGHT) / 2.0;
            coll_size.height = PLAYER_SMALL_HEIGHT;
            mesh.0 = assets.player.small_mesh.clone();
        }
    }

    if growth.timer.is_finished() {
        let target_height = if growth.growing {
            PLAYER_BIG_HEIGHT
        } else {
            PLAYER_SMALL_HEIGHT
        };

        let height_diff = target_height - coll_size.height;
        transform.translation.y += height_diff / 2.0;

        coll_size.height = target_height;
        *player_size = if growth.growing {
            PlayerSize::Big
        } else {
            PlayerSize::Small
        };
        mesh.0 = if growth.growing {
            assets.player.big_mesh.clone()
        } else {
            assets.player.small_mesh.clone()
        };

        if !growth.growing {
            commands.entity(entity).insert(Invincible {
                timer: Timer::from_seconds(INVINCIBILITY_DURATION, TimerMode::Once),
            });
            mat.0 = assets.player.normal_mat.clone();
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
    assets: Res<GameAssets>,
) {
    let Ok((entity, player_size, grounded, mut coll_size, mut transform, mut mesh, is_ducking)) =
        query.single_mut()
    else {
        return;
    };

    let wants_duck =
        keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS);

    if *player_size != PlayerSize::Small && grounded.0 && wants_duck && !is_ducking {
        commands.entity(entity).insert(Ducking);
        let old_height = coll_size.height;
        coll_size.height = PLAYER_SMALL_HEIGHT;
        mesh.0 = assets.player.small_mesh.clone();
        transform.translation.y -= (old_height - PLAYER_SMALL_HEIGHT) / 2.0;
    } else if is_ducking && (!wants_duck || !grounded.0 || *player_size == PlayerSize::Small) {
        commands.entity(entity).remove::<Ducking>();
        if *player_size != PlayerSize::Small {
            coll_size.height = PLAYER_BIG_HEIGHT;
            mesh.0 = assets.player.big_mesh.clone();
            transform.translation.y += (PLAYER_BIG_HEIGHT - PLAYER_SMALL_HEIGHT) / 2.0;
        }
    }
}
