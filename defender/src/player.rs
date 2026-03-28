use bevy::prelude::*;

use crate::camera::wrap_x;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::scheduling::GameplaySet;
use crate::spawning::*;
use crate::states::AppState;

#[derive(Resource, Default)]
pub struct SmartBombTriggered(pub bool);

#[derive(Resource, Default)]
pub struct HyperspaceTriggered(pub bool);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_input.in_set(GameplaySet::Input))
            .add_systems(Update, player_movement.in_set(GameplaySet::Movement))
            .add_systems(
                Update,
                (explosion_system, smart_bomb_system, hyperspace_system).in_set(GameplaySet::Post),
            )
            .add_systems(
                Update,
                explosion_system.run_if(in_state(AppState::PlayerDeath)),
            );
    }
}

pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_q: Query<
        (
            &mut Velocity,
            &mut FacingDirection,
            &mut FireCooldown,
            &WorldPosition,
            &Transform,
        ),
        With<Player>,
    >,
    mut commands: Commands,
    assets: Res<GameplayAssets>,
    mut game_state: ResMut<GameState>,
    mut smart_bomb: ResMut<SmartBombTriggered>,
    mut hyperspace: ResMut<HyperspaceTriggered>,
) {
    let Ok((mut vel, mut facing, mut cooldown, wp, tf)) = player_q.single_mut() else {
        return;
    };

    // Thrust
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        vel.0.x += PLAYER_THRUST * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        vel.0.x -= PLAYER_THRUST * time.delta_secs();
    }

    // Vertical
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        vel.0.y = PLAYER_VERTICAL_SPEED;
    } else if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        vel.0.y = -PLAYER_VERTICAL_SPEED;
    } else {
        vel.0.y *= 0.9;
    }

    // Reverse direction
    if keyboard.just_pressed(KeyCode::ShiftLeft) || keyboard.just_pressed(KeyCode::ShiftRight) {
        facing.0 = -facing.0;
    }

    // Clamp horizontal speed
    vel.0.x = vel.0.x.clamp(-PLAYER_MAX_SPEED, PLAYER_MAX_SPEED);

    // Apply friction to horizontal
    vel.0.x *= PLAYER_FRICTION;

    // Fire
    cooldown.0.tick(time.delta());
    let dir = facing.0;
    if (keyboard.pressed(KeyCode::Space) || keyboard.pressed(KeyCode::KeyF))
        && cooldown.0.is_finished()
    {
        cooldown.0.reset();
        spawn_player_projectile(
            &mut commands,
            &assets,
            wrap_x(wp.0 + dir * 25.0),
            tf.translation.y,
            dir,
        );
    }

    // Smart bomb
    if (keyboard.just_pressed(KeyCode::KeyE) || keyboard.just_pressed(KeyCode::ControlLeft))
        && game_state.smart_bombs > 0
    {
        game_state.smart_bombs -= 1;
        smart_bomb.0 = true;
    }

    // Hyperspace
    if keyboard.just_pressed(KeyCode::KeyH) || keyboard.just_pressed(KeyCode::Enter) {
        hyperspace.0 = true;
    }
}

pub fn player_movement(
    time: Res<Time>,
    terrain: Res<TerrainData>,
    mut query: Query<
        (
            &Velocity,
            &mut WorldPosition,
            &mut Transform,
            &FacingDirection,
        ),
        With<Player>,
    >,
) {
    let Ok((vel, mut wp, mut tf, facing)) = query.single_mut() else {
        return;
    };

    wp.0 += vel.0.x * time.delta_secs();
    tf.translation.y += vel.0.y * time.delta_secs();

    // Get terrain height at player position
    let terrain_y = crate::terrain::get_terrain_y_at(&terrain, wp.0) + 15.0;

    // Clamp Y
    tf.translation.y = tf.translation.y.clamp(terrain_y, CEILING_Y);

    // Flip sprite based on facing direction
    tf.scale.x = facing.0.abs() * facing.0.signum();
}

pub fn smart_bomb_system(
    mut smart_bomb: ResMut<SmartBombTriggered>,
    mut commands: Commands,
    enemies: Query<(Entity, &WorldPosition, &Transform), With<Enemy>>,
    cam_pos: Res<CameraWorldPos>,
    mut game_state: ResMut<GameState>,
    assets: Res<GameplayAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !smart_bomb.0 {
        return;
    }
    smart_bomb.0 = false;

    for (entity, wp, tf) in &enemies {
        let dx = crate::camera::world_dx(wp.0, cam_pos.0);
        if dx.abs() < HALF_SCREEN_W {
            game_state.score += 200;
            let wx = wp.0;
            let y = tf.translation.y;
            commands.entity(entity).despawn();
            spawn_explosion(
                &mut commands,
                &assets,
                &mut materials,
                wx,
                y,
                Color::srgb(1.0, 0.8, 0.2),
            );
        }
    }
}

pub fn hyperspace_system(
    mut hyperspace: ResMut<HyperspaceTriggered>,
    mut player_q: Query<(&mut WorldPosition, &mut Transform), With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut rng: ResMut<GameRng>,
) {
    if !hyperspace.0 {
        return;
    }
    hyperspace.0 = false;

    let Ok((mut wp, mut tf)) = player_q.single_mut() else {
        return;
    };

    wp.0 = rng.world_x();
    tf.translation.y = 0.0;

    if rng.f32() < HYPERSPACE_DEATH_CHANCE {
        next_state.set(AppState::PlayerDeath);
    }
}

pub fn explosion_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Explosion, &mut Transform)>,
) {
    for (entity, mut explosion, mut tf) in &mut query {
        explosion.0.tick(time.delta());
        let progress = explosion.0.elapsed_secs() / explosion.0.duration().as_secs_f32();
        tf.scale = Vec3::splat(1.0 + progress * 3.0);
        if explosion.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
