use bevy::prelude::*;

use crate::components::{Invulnerable, Player, Velocity};
use crate::constants::*;
use crate::resources::{GameData, RespawnTimer, WavePhase};
use crate::states::AppState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(OnExit(AppState::Playing), cleanup_player)
            .add_systems(
                Update,
                (
                    player_input,
                    player_movement.after(player_input),
                    invulnerability_tick,
                    respawn_player,
                    auto_respawn_player,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn player_mesh(meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    meshes.add(Triangle2d::new(
        Vec2::new(0.0, PLAYER_HEIGHT / 2.0),
        Vec2::new(-PLAYER_WIDTH / 2.0, -PLAYER_HEIGHT / 2.0),
        Vec2::new(PLAYER_WIDTH / 2.0, -PLAYER_HEIGHT / 2.0),
    ))
}

fn player_material(materials: &mut Assets<ColorMaterial>) -> Handle<ColorMaterial> {
    materials.add(Color::srgb(0.2, 0.6, 1.0))
}

fn spawn_player_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    invulnerable: bool,
) {
    let mut entity = commands.spawn((
        Player,
        Velocity::default(),
        Mesh2d(player_mesh(meshes)),
        MeshMaterial2d(player_material(materials)),
        Transform::from_translation(Vec3::new(0.0, PLAYER_Y, 1.0)),
    ));

    if invulnerable {
        entity.insert(Invulnerable(Timer::from_seconds(
            INVULNERABLE_DURATION,
            TimerMode::Once,
        )));
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_player_entity(&mut commands, &mut meshes, &mut materials, false);
}

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let Ok(mut velocity) = query.single_mut() else {
        return;
    };

    let mut direction = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    velocity.0 = Vec2::new(direction * PLAYER_SPEED, 0.0);
}

fn player_movement(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<Player>>,
) {
    let Ok((velocity, mut transform)) = query.single_mut() else {
        return;
    };

    transform.translation.x += velocity.0.x * time.delta_secs();
    let half_w = WINDOW_WIDTH / 2.0 - PLAYER_WIDTH / 2.0;
    transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
}

fn invulnerability_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invulnerable, &mut Visibility), With<Player>>,
) {
    for (entity, mut invuln, mut visibility) in &mut query {
        invuln.0.tick(time.delta());

        *visibility = if ((invuln.0.elapsed_secs() * 10.0) as u32).is_multiple_of(2) {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };

        if invuln.0.just_finished() {
            *visibility = Visibility::Visible;
            commands.entity(entity).remove::<Invulnerable>();
        }
    }
}

fn respawn_player(
    mut commands: Commands,
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    respawn_timer: Option<ResMut<RespawnTimer>>,
) {
    if game_data.phase != WavePhase::Respawning {
        return;
    }

    let Some(mut timer) = respawn_timer else {
        return;
    };

    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        spawn_player_entity(&mut commands, &mut meshes, &mut materials, true);
        game_data.phase = WavePhase::Combat;
        commands.remove_resource::<RespawnTimer>();
    }
}

fn auto_respawn_player(
    mut commands: Commands,
    game_data: Res<GameData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<&Player>,
) {
    if game_data.phase != WavePhase::Combat {
        return;
    }
    if !player_query.is_empty() {
        return;
    }
    if game_data.lives == 0 {
        return;
    }

    spawn_player_entity(&mut commands, &mut meshes, &mut materials, true);
}

fn cleanup_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<RespawnTimer>();
}
