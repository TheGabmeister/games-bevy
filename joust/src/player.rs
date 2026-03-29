use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::rendering::*;
use crate::resources::*;
use crate::states::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_players)
            .add_systems(
                Update,
                player_input_system
                    .in_set(GameSet::Input)
                    .run_if(in_state(PlayState::WaveActive)),
            )
            .add_systems(
                Update,
                player_respawn_system.run_if(in_state(AppState::Playing)),
            );
    }
}

fn spawn_players(
    mut commands: Commands,
    meshes: Res<SharedMeshes>,
    materials: Res<SharedMaterials>,
    player_count: Res<PlayerCount>,
) {
    let spawns = [PLAYER1_SPAWN, PLAYER2_SPAWN];
    let body_mats = [materials.player1_body.clone(), materials.player2_body.clone()];

    for i in 0..player_count.0 {
        let pos = Vec2::new(spawns[i as usize].0, spawns[i as usize].1);
        let entity = spawn_rider_visual(
            &mut commands,
            &meshes,
            &materials,
            pos,
            Z_PLAYERS,
            body_mats[i as usize].clone(),
        );
        commands.entity(entity).insert((
            Player { id: i },
            Rider,
            Velocity::default(),
            FacingDirection::Right,
            FlapCooldown::ready(),
            PreviousPosition(pos),
            Invincible(Timer::from_seconds(RESPAWN_INVINCIBILITY, TimerMode::Once)),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

fn player_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Player,
        &mut Velocity,
        &mut FacingDirection,
        &mut FlapCooldown,
        Option<&Grounded>,
    )>,
) {
    for (entity, player, mut vel, mut facing, mut cooldown, grounded) in &mut query {
        cooldown.0.tick(time.delta());
        let is_grounded = grounded.is_some();
        let accel = if is_grounded { GROUND_ACCEL } else { AIR_ACCEL };
        let max_speed = if is_grounded { MAX_GROUND_SPEED } else { MAX_AIR_SPEED };

        let (left_keys, right_keys, flap_keys): (&[KeyCode], &[KeyCode], &[KeyCode]) =
            match player.id {
                0 => (
                    &[KeyCode::KeyA, KeyCode::ArrowLeft],
                    &[KeyCode::KeyD, KeyCode::ArrowRight],
                    &[KeyCode::KeyW, KeyCode::Space, KeyCode::ArrowUp],
                ),
                _ => (
                    &[KeyCode::KeyJ],
                    &[KeyCode::KeyL],
                    &[KeyCode::KeyI],
                ),
            };

        // Horizontal
        if left_keys.iter().any(|k| keys.pressed(*k)) {
            vel.0.x = (vel.0.x - accel * time.delta_secs()).max(-max_speed);
            *facing = FacingDirection::Left;
        }
        if right_keys.iter().any(|k| keys.pressed(*k)) {
            vel.0.x = (vel.0.x + accel * time.delta_secs()).min(max_speed);
            *facing = FacingDirection::Right;
        }

        // Flap
        let flap_pressed = flap_keys.iter().any(|k| keys.just_pressed(*k));
        if flap_pressed && cooldown.0.is_finished() {
            vel.0.y = (vel.0.y + FLAP_IMPULSE).min(MAX_RISE_SPEED);
            cooldown.0.reset();
            if is_grounded {
                commands.entity(entity).remove::<Grounded>();
            }
        }
    }
}

fn player_respawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut respawn_timers: ResMut<RespawnTimers>,
    meshes: Res<SharedMeshes>,
    materials: Res<SharedMaterials>,
    game_state: Res<GameState>,
) {
    let mut spawned = Vec::new();
    for (i, (player_id, timer)) in respawn_timers.timers.iter_mut().enumerate() {
        timer.tick(time.delta());
        if timer.is_finished() {
            let pid = *player_id;
            if game_state.lives[pid as usize] > 0 {
                let pos = if pid == 0 {
                    Vec2::new(PLAYER1_SPAWN.0, PLAYER1_SPAWN.1)
                } else {
                    Vec2::new(PLAYER2_SPAWN.0, PLAYER2_SPAWN.1)
                };
                let body_mat = if pid == 0 {
                    materials.player1_body.clone()
                } else {
                    materials.player2_body.clone()
                };
                let entity = spawn_rider_visual(
                    &mut commands,
                    &meshes,
                    &materials,
                    pos,
                    Z_PLAYERS,
                    body_mat,
                );
                commands.entity(entity).insert((
                    Player { id: pid },
                    Rider,
                    Velocity::default(),
                    FacingDirection::Right,
                    FlapCooldown::ready(),
                    PreviousPosition(pos),
                    Invincible(Timer::from_seconds(RESPAWN_INVINCIBILITY, TimerMode::Once)),
                    DespawnOnExit(AppState::Playing),
                ));
            }
            spawned.push(i);
        }
    }
    for i in spawned.into_iter().rev() {
        respawn_timers.timers.remove(i);
    }
}
