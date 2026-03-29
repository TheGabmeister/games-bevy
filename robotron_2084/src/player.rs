use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::GameAssets;
use crate::states::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(OnEnter(PlayState::WaveActive), respawn_player_if_needed)
            .add_systems(
                Update,
                (player_movement, player_aim, player_fire).in_set(GameSet::Input),
            )
            .add_systems(
                Update,
                tick_invincibility.run_if(in_state(AppState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    spawn_player_entity(&mut commands, &assets);
}

pub fn spawn_player_entity(commands: &mut Commands, assets: &GameAssets) {
    commands.spawn((
        Player,
        Confined,
        Mesh2d(assets.player_mesh.clone()),
        MeshMaterial2d(assets.player_material.clone()),
        Transform::from_xyz(0.0, 0.0, 1.0),
        Velocity(Vec2::ZERO),
        Facing(Vec2::Y),
        CollisionRadius(PLAYER_RADIUS),
        FireCooldown(Timer::from_seconds(PLAYER_FIRE_COOLDOWN, TimerMode::Once)),
        Invincible(Timer::from_seconds(
            PLAYER_INVINCIBILITY_DURATION,
            TimerMode::Once,
        )),
        DespawnOnExit(AppState::Playing),
    ));
}

fn respawn_player_if_needed(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_q: Query<(), With<Player>>,
) {
    if player_q.is_empty() {
        spawn_player_entity(&mut commands, &assets);
    }
}

fn player_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let Ok(mut vel) = query.single_mut() else {
        return;
    };
    let mut dir = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    vel.0 = dir.normalize_or_zero() * PLAYER_SPEED;
}

fn player_aim(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Facing, &mut Transform), With<Player>>,
) {
    let Ok((mut facing, mut tf)) = query.single_mut() else {
        return;
    };
    let mut dir = Vec2::ZERO;
    if input.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if input.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if input.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if dir != Vec2::ZERO {
        let dir = dir.normalize();
        facing.0 = dir;
        tf.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
    }
}

fn player_fire(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    assets: Res<GameAssets>,
    mut player_q: Query<(&Transform, &Facing, &mut FireCooldown), With<Player>>,
    bullet_q: Query<(), With<PlayerBullet>>,
) {
    let Ok((tf, facing, mut cooldown)) = player_q.single_mut() else {
        return;
    };
    cooldown.0.tick(time.delta());
    let aiming = input.pressed(KeyCode::ArrowUp)
        || input.pressed(KeyCode::ArrowDown)
        || input.pressed(KeyCode::ArrowLeft)
        || input.pressed(KeyCode::ArrowRight);
    if !aiming || !cooldown.0.is_finished() {
        return;
    }
    if bullet_q.iter().count() >= MAX_PLAYER_BULLETS as usize {
        return;
    }
    cooldown.0.reset();
    let pos = tf.translation.truncate();
    commands.spawn((
        PlayerBullet,
        Mesh2d(assets.bullet_mesh.clone()),
        MeshMaterial2d(assets.bullet_material.clone()),
        Transform::from_xyz(pos.x, pos.y, 0.5),
        Velocity(facing.0 * BULLET_SPEED),
        CollisionRadius(BULLET_RADIUS),
        DespawnOnExit(AppState::Playing),
    ));
}

fn tick_invincibility(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invincible, &mut Visibility)>,
) {
    for (entity, mut inv, mut vis) in &mut query {
        inv.0.tick(time.delta());
        let elapsed = inv.0.elapsed_secs();
        let blink = ((elapsed / PLAYER_BLINK_INTERVAL) as u32).is_multiple_of(2);
        *vis = if blink {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if inv.0.is_finished() {
            commands.entity(entity).remove::<Invincible>();
            *vis = Visibility::Inherited;
        }
    }
}
