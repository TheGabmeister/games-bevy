use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::*;
use crate::constants::*;
use crate::input::ActionInput;
use crate::level::{tile_to_world, world_to_col, world_to_row, LevelGrid, WarpPipes};
use crate::states::*;

// ── Warp Animation Resource ──

pub enum WarpPhase {
    EnterPipe,
    FadeOut,
    Teleport,
    FadeIn,
    ExitPipe,
}

#[derive(Resource)]
pub struct WarpAnimation {
    phase: WarpPhase,
    timer: Timer,
    entry_center_x: f32,
    entry_pipe_top_y: f32,
    exit_center_x: f32,
    /// For pipe exits: top edge of the exit pipe tile.
    /// For non-pipe exits: tile center y of destination (player stands here).
    exit_y: f32,
    has_exit_pipe: bool,
    fade_entity: Option<Entity>,
}

// ── Plugin ──

pub struct WarpPlugin;

impl Plugin for WarpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            check_warp_pipe.in_set(GameplaySet::Late),
        )
        .add_systems(
            Update,
            warp_animation_system.run_if(in_state(PlayState::Warping)),
        );
    }
}

// ── Detection System ──

fn check_warp_pipe(
    action: Res<ActionInput>,
    level: Res<LevelGrid>,
    warp_pipes: Option<Res<WarpPipes>>,
    player_query: Query<(&Transform, &CollisionSize, &Velocity, &Grounded), With<Player>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut commands: Commands,
) {
    let Some(warp_pipes) = warp_pipes else { return };
    if warp_pipes.destinations.is_empty() || !action.duck {
        return;
    }

    let Ok((tf, coll, vel, grounded)) = player_query.single() else { return };
    if !grounded.0 || vel.x.abs() > 10.0 {
        return;
    }

    // Find the tile the player is standing on (just below their feet)
    let feet_y = tf.translation.y - coll.height / 2.0 - 1.0;
    let feet_col = world_to_col(tf.translation.x);
    let feet_row = world_to_row(feet_y);

    let ch = level.get_char(feet_col, feet_row);
    if ch != '[' && ch != ']' {
        return;
    }

    // Find pipe top-left column
    let pipe_col = if ch == ']' { feet_col - 1 } else { feet_col };
    let pipe_row = feet_row;

    let Some(dest) = warp_pipes.destinations.get(&(pipe_col, pipe_row)) else { return };

    // Compute entry pipe geometry
    let (entry_wx, entry_wy) = tile_to_world(pipe_col as usize, pipe_row as usize);
    let entry_center_x = entry_wx + TILE_SIZE / 2.0;
    let entry_pipe_top_y = entry_wy + TILE_SIZE / 2.0;

    // Compute exit geometry
    let (dest_wx, dest_wy) = tile_to_world(dest.target_col, dest.target_row);
    let dest_char = level.get_char(dest.target_col as i32, dest.target_row as i32);
    let has_exit_pipe = dest_char == '[';

    let exit_center_x = if has_exit_pipe {
        dest_wx + TILE_SIZE / 2.0
    } else {
        dest_wx
    };
    let exit_y = if has_exit_pipe {
        dest_wy + TILE_SIZE / 2.0 // top edge of pipe tile
    } else {
        dest_wy // tile center — player stands at ground level
    };

    commands.insert_resource(WarpAnimation {
        phase: WarpPhase::EnterPipe,
        timer: Timer::from_seconds(WARP_FADE_DURATION, TimerMode::Once),
        entry_center_x,
        entry_pipe_top_y,
        exit_center_x,
        exit_y,
        has_exit_pipe,
        fade_entity: None,
    });

    next_play_state.set(PlayState::Warping);
}

// ── Animation System ──

fn warp_animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut anim: ResMut<WarpAnimation>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut CollisionSize,
            &mut Mesh2d,
            &PlayerSize,
            Entity,
            Has<Ducking>,
        ),
        With<Player>,
    >,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    mut fade_query: Query<&mut BackgroundColor, With<WarpFadeOverlay>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    assets: Res<GameAssets>,
) {
    let Ok((mut player_tf, mut vel, mut coll_size, mut mesh, player_size, player_entity, is_ducking)) =
        player_query.single_mut()
    else {
        return;
    };
    let dt = time.delta_secs();

    vel.x = 0.0;
    vel.y = 0.0;

    match anim.phase {
        WarpPhase::EnterPipe => {
            let half_h = coll_size.height / 2.0;

            // Snap player to pipe center, render behind pipe
            player_tf.translation.x = anim.entry_center_x;
            player_tf.translation.z = Z_PIPE - 0.5;
            player_tf.translation.y -= WARP_SLIDE_SPEED * dt;

            // Done when player's top edge is below pipe top
            if player_tf.translation.y + half_h < anim.entry_pipe_top_y {
                anim.phase = WarpPhase::FadeOut;
                anim.timer = Timer::from_seconds(WARP_FADE_DURATION, TimerMode::Once);

                // Spawn fade overlay
                let entity = commands
                    .spawn((
                        WarpFadeOverlay,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        GlobalZIndex(100),
                    ))
                    .id();
                anim.fade_entity = Some(entity);

                // Hide player during fade
                commands.entity(player_entity).insert(Visibility::Hidden);
            }
        }

        WarpPhase::FadeOut => {
            anim.timer.tick(time.delta());
            let progress = anim.timer.fraction();

            if let Some(fade_entity) = anim.fade_entity {
                if let Ok(mut bg) = fade_query.get_mut(fade_entity) {
                    bg.0 = Color::srgba(0.0, 0.0, 0.0, progress);
                }
            }

            if anim.timer.is_finished() {
                anim.phase = WarpPhase::Teleport;
            }
        }

        WarpPhase::Teleport => {
            // Undo ducking so player exits at full height
            if is_ducking {
                commands.entity(player_entity).remove::<Ducking>();
                if *player_size != PlayerSize::Small {
                    coll_size.height = PLAYER_BIG_HEIGHT;
                    mesh.0 = assets.player.big_mesh.clone();
                }
            }

            if anim.has_exit_pipe {
                // Start below the exit pipe top
                player_tf.translation.x = anim.exit_center_x;
                player_tf.translation.y = anim.exit_y - coll_size.height;
                player_tf.translation.z = Z_PIPE - 0.5;
            } else {
                // No exit pipe — place at destination tile center (ground level)
                player_tf.translation.x = anim.exit_center_x;
                player_tf.translation.y = anim.exit_y;
                player_tf.translation.z = Z_PLAYER;
            }

            // Jump camera to destination
            if let Ok(mut camera_tf) = camera_query.single_mut() {
                let camera_x = (player_tf.translation.x - CAMERA_DEAD_ZONE_OFFSET)
                    .clamp(CAMERA_MIN_X, CAMERA_MAX_X);
                camera_tf.translation.x = camera_x;
            }

            anim.phase = WarpPhase::FadeIn;
            anim.timer = Timer::from_seconds(WARP_FADE_DURATION, TimerMode::Once);
        }

        WarpPhase::FadeIn => {
            anim.timer.tick(time.delta());
            let progress = anim.timer.fraction();

            if let Some(fade_entity) = anim.fade_entity {
                if let Ok(mut bg) = fade_query.get_mut(fade_entity) {
                    bg.0 = Color::srgba(0.0, 0.0, 0.0, 1.0 - progress);
                }
            }

            if anim.timer.is_finished() {
                if anim.has_exit_pipe {
                    anim.phase = WarpPhase::ExitPipe;
                    commands.entity(player_entity).insert(Visibility::Inherited);
                } else {
                    // No exit pipe — done
                    cleanup_warp(&mut commands, &anim, player_entity);
                    next_play_state.set(PlayState::Running);
                }
            }
        }

        WarpPhase::ExitPipe => {
            let half_h = coll_size.height / 2.0;
            player_tf.translation.y += WARP_SLIDE_SPEED * dt;

            let standing_y = anim.exit_y + half_h;
            if player_tf.translation.y >= standing_y {
                player_tf.translation.y = standing_y;
                player_tf.translation.z = Z_PLAYER;

                cleanup_warp(&mut commands, &anim, player_entity);
                next_play_state.set(PlayState::Running);
            }
        }
    }
}

fn cleanup_warp(commands: &mut Commands, anim: &WarpAnimation, player_entity: Entity) {
    if let Some(fade_entity) = anim.fade_entity {
        commands.entity(fade_entity).despawn();
    }
    commands.entity(player_entity).insert(Visibility::Inherited);
    commands.remove_resource::<WarpAnimation>();
}
