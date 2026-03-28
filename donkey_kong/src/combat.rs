use std::collections::HashSet;

use bevy::prelude::*;
use bevy::audio::PlaybackSettings;

use crate::components::{Enemy, Laser, Player};
use crate::constants::{ENEMY_SCORE, ENEMY_SIZE, LASER_SIZE, LASER_SPEED, WINDOW_HEIGHT};
use crate::resources::GameData;
use crate::states::AppState;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_shoot,
                move_lasers.after(player_shoot),
                laser_enemy_collision.after(move_lasers),
                check_win.after(laser_enemy_collision),
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(OnExit(AppState::Playing), cleanup_lasers);
    }
}

fn player_shoot(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    query: Query<&Transform, With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let Ok(player_transform) = query.single() else {
        return;
    };

    commands.spawn((
        Laser,
        Sprite::from_image(asset_server.load("player_laser.png")),
        Transform::from_translation(player_transform.translation),
    ));
    commands.spawn((
        AudioPlayer::new(asset_server.load("sfx_laser1.ogg")),
        PlaybackSettings::DESPAWN,
    ));
}

fn move_lasers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform), With<Laser>>,
) {
    let top = WINDOW_HEIGHT / 2.0;

    for (entity, mut transform) in &mut query {
        transform.translation.y += LASER_SPEED * time.delta_secs();

        if transform.translation.y > top {
            commands.entity(entity).despawn();
        }
    }
}

fn laser_enemy_collision(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    laser_query: Query<(Entity, &Transform), With<Laser>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    let collision_dist = (ENEMY_SIZE + LASER_SIZE) / 2.0;
    let mut hit_lasers = HashSet::new();
    let mut hit_enemies = HashSet::new();

    for (laser_entity, laser_transform) in &laser_query {
        if hit_lasers.contains(&laser_entity) {
            continue;
        }

        for (enemy_entity, enemy_transform) in &enemy_query {
            if hit_enemies.contains(&enemy_entity) {
                continue;
            }

            let distance = laser_transform
                .translation
                .truncate()
                .distance(enemy_transform.translation.truncate());

            if distance < collision_dist {
                hit_lasers.insert(laser_entity);
                hit_enemies.insert(enemy_entity);
                commands.entity(laser_entity).despawn();
                commands.entity(enemy_entity).despawn();
                game_data.score += ENEMY_SCORE;
                break;
            }
        }
    }
}

fn check_win(
    enemy_query: Query<&Enemy>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if enemy_query.iter().count() == 0 {
        next_state.set(AppState::GameOver);
    }
}

fn cleanup_lasers(mut commands: Commands, query: Query<Entity, With<Laser>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
