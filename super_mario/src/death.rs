use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::*;

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayState::Dying), start_death_animation)
            .add_systems(
                Update,
                death_animation_system.run_if(in_state(PlayState::Dying)),
            );
    }
}

fn start_death_animation(
    mut commands: Commands,
    mut player_query: Query<(&mut Velocity, &Transform), With<Player>>,
) {
    let Ok((mut vel, transform)) = player_query.single_mut() else {
        return;
    };
    vel.x = 0.0;
    vel.y = 0.0;

    let pit_death = transform.translation.y < DEATH_Y;

    commands.insert_resource(DeathAnimation {
        phase: DeathPhase::Pause,
        timer: Timer::from_seconds(
            if pit_death { 1.0 } else { DEATH_PAUSE_DURATION },
            TimerMode::Once,
        ),
        pit_death,
    });
}

fn death_animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut death_anim: ResMut<DeathAnimation>,
    mut player_query: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut Grounded,
            &mut CollisionSize,
            &mut PlayerSize,
            &mut Mesh2d,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        With<Player>,
    >,
    mut game_data: ResMut<GameData>,
    mut game_timer: ResMut<GameTimer>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    spawn_point: Res<SpawnPoint>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    assets: Res<GameAssets>,
) {
    death_anim.timer.tick(time.delta());

    let Ok((
        mut vel,
        mut player_tf,
        mut grounded,
        mut coll_size,
        mut player_size,
        mut mesh,
        mut mat,
    )) = player_query.single_mut()
    else {
        return;
    };

    let mut animation_complete = false;

    match death_anim.phase {
        DeathPhase::Pause => {
            if death_anim.timer.is_finished() {
                if death_anim.pit_death {
                    animation_complete = true;
                } else {
                    death_anim.phase = DeathPhase::Bouncing;
                    death_anim.timer =
                        Timer::from_seconds(DEATH_FALL_DURATION, TimerMode::Once);
                    vel.y = DEATH_BOUNCE_IMPULSE;
                }
            }
        }
        DeathPhase::Bouncing => {
            let gravity = if vel.y > 0.0 {
                GRAVITY_ASCENDING
            } else {
                GRAVITY_DESCENDING
            };
            vel.y -= gravity * time.delta_secs();
            vel.y = vel.y.max(-TERMINAL_VELOCITY);
            player_tf.translation.y += vel.y * time.delta_secs();

            if death_anim.timer.is_finished() {
                animation_complete = true;
            }
        }
    }

    if animation_complete {
        commands.remove_resource::<DeathAnimation>();
        game_data.lives = game_data.lives.saturating_sub(1);

        if game_data.lives == 0 {
            next_app_state.set(AppState::GameOver);
        } else {
            player_tf.translation.x = spawn_point.x;
            player_tf.translation.y = spawn_point.y;
            vel.x = 0.0;
            vel.y = 0.0;
            grounded.0 = false;
            game_timer.time = TIMER_START;

            // Reset to small Mario with normal appearance
            *player_size = PlayerSize::Small;
            coll_size.height = PLAYER_SMALL_HEIGHT;
            mesh.0 = assets.player.small_mesh.clone();
            mat.0 = assets.player.normal_mat.clone();

            if let Ok(mut camera_tf) = camera_query.single_mut() {
                camera_tf.translation.x = CAMERA_MIN_X;
            }

            next_play_state.set(PlayState::Running);
        }
    }
}
