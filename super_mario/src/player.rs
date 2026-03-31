use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::{self, WallAction};
use crate::components::*;
use crate::constants::*;
use crate::input::ActionInput;
use crate::level::LevelGrid;
use crate::resources::*;
use crate::states::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            player_input.in_set(GameplaySet::Input),
        )
        .add_systems(
            Update,
            (apply_gravity, apply_velocity, tile_collision)
                .chain()
                .in_set(GameplaySet::Physics),
        )
        .add_systems(
            Update,
            check_pit_death.in_set(GameplaySet::Late),
        )
        .add_systems(
            Update,
            (skid_system, update_player_face).run_if(in_state(PlayState::Running)),
        );
    }
}

fn player_input(
    action: Res<ActionInput>,
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut FacingDirection, &Grounded, Has<Ducking>), With<Player>>,
) {
    let Ok((mut vel, mut facing, grounded, is_ducking)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Ducking: decelerate and block horizontal input, but allow jumping
    if is_ducking {
        let decel = PLAYER_DECELERATION * dt;
        if vel.x.abs() < decel {
            vel.x = 0.0;
        } else {
            vel.x -= decel * vel.x.signum();
        }

        if action.jump_just_released && vel.y > 0.0 {
            vel.y *= JUMP_CUT_MULTIPLIER;
        }
        return;
    }

    let dir = action.move_x;

    if dir != 0.0 {
        *facing = if dir < 0.0 {
            FacingDirection::Left
        } else {
            FacingDirection::Right
        };
    }

    let max_speed = if action.running { PLAYER_RUN_SPEED } else { PLAYER_WALK_SPEED };
    let accel = if grounded.0 { PLAYER_ACCELERATION } else { PLAYER_AIR_ACCELERATION };

    if dir != 0.0 {
        vel.x += dir * accel * dt;
        vel.x = vel.x.clamp(-max_speed, max_speed);
    } else if grounded.0 {
        let decel = PLAYER_DECELERATION * dt;
        if vel.x.abs() < decel {
            vel.x = 0.0;
        } else {
            vel.x -= decel * vel.x.signum();
        }
    }

    if action.jump_just_pressed && grounded.0 {
        vel.y = PLAYER_JUMP_IMPULSE;
    }

    if action.jump_just_released && vel.y > 0.0 {
        vel.y *= JUMP_CUT_MULTIPLIER;
    }
}

fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Grounded), With<Player>>,
) {
    let Ok((mut vel, grounded)) = query.single_mut() else {
        return;
    };

    if grounded.0 {
        return;
    }

    let gravity = if vel.y > 0.0 {
        GRAVITY_ASCENDING
    } else {
        GRAVITY_DESCENDING
    };

    vel.y -= gravity * time.delta_secs();
    vel.y = vel.y.max(-TERMINAL_VELOCITY);
}

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<Player>>,
) {
    let Ok((vel, mut transform)) = query.single_mut() else {
        return;
    };

    transform.translation.x += vel.x * time.delta_secs();
    transform.translation.y += vel.y * time.delta_secs();
}

fn tile_collision(
    level: Res<LevelGrid>,
    mut pending_hit: ResMut<PendingBlockHit>,
    mut query: Query<
        (&mut Velocity, &mut Transform, &mut Grounded, &CollisionSize, &PlayerSize),
        With<Player>,
    >,
    camera_query: Query<&Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok((mut vel, mut transform, mut grounded, coll_size, player_size)) = query.single_mut()
    else {
        return;
    };

    let half_w = coll_size.width / 2.0;
    let half_h = coll_size.height / 2.0;

    // Clamp to left edge of camera
    if let Ok(camera_tf) = camera_query.single() {
        let camera_left = camera_tf.translation.x - CAMERA_VISIBLE_WIDTH / 2.0;
        let min_x = camera_left + half_w;
        if transform.translation.x < min_x {
            transform.translation.x = min_x;
            if vel.x < 0.0 {
                vel.x = 0.0;
            }
        }
    }

    let mut unused_dir = 0.0;
    let result = collision::resolve_tile_collisions(
        &level,
        &mut transform.translation,
        &mut vel,
        half_w,
        half_h,
        WallAction::Stop,
        &mut unused_dir,
    );

    grounded.0 = result.grounded;

    if let Some((col, row)) = result.head_hit {
        pending_hit.hit = Some(BlockHitInfo {
            col,
            row,
            player_size: *player_size,
        });
    }
}

fn check_pit_death(
    player_query: Query<&Transform, With<Player>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    if player_tf.translation.y < DEATH_Y {
        next_play_state.set(PlayState::Dying);
    }
}

fn skid_system(
    action: Res<ActionInput>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Velocity,
            &Grounded,
            &PlayerSize,
            &mut MeshMaterial2d<ColorMaterial>,
            Has<Skidding>,
            Has<StarPower>,
        ),
        With<Player>,
    >,
    assets: Res<GameAssets>,
) {
    let Ok((entity, vel, grounded, player_size, mut mat, is_skidding, has_star)) =
        query.single_mut()
    else {
        return;
    };

    // Don't override star power visuals
    if has_star {
        if is_skidding {
            commands.entity(entity).remove::<Skidding>();
        }
        return;
    }

    let should_skid = grounded.0
        && vel.x.abs() > SKID_SPEED_THRESHOLD
        && action.move_x != 0.0
        && action.move_x * vel.x < 0.0;

    if should_skid && !is_skidding {
        commands.entity(entity).insert(Skidding);
        mat.0 = assets.player.skid_mat.clone();
    } else if !should_skid && is_skidding {
        commands.entity(entity).remove::<Skidding>();
        mat.0 = match player_size {
            PlayerSize::Fire => assets.player.fire_mat.clone(),
            _ => assets.player.normal_mat.clone(),
        };
    }
}

fn update_player_face(
    player_query: Query<(&CollisionSize, &Children), With<Player>>,
    mut face_query: Query<&mut Transform, With<PlayerFace>>,
) {
    let Ok((coll, children)) = player_query.single() else {
        return;
    };
    for child in children.iter() {
        if let Ok(mut tf) = face_query.get_mut(child) {
            tf.translation.y = coll.height / 2.0 - 4.0;
        }
    }
}

