use std::collections::HashSet;

use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::effects::spawn_explosion;
use crate::enemy::{enemy_color_for_row, score_for_row};
use crate::resources::*;
use crate::states::AppState;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_shoot,
                move_player_bullets.after(player_shoot),
                laser_enemy_collision.after(move_player_bullets),
                enemy_bullet_player_collision.after(laser_enemy_collision),
                diving_enemy_player_collision.after(enemy_bullet_player_collision),
                check_stage_clear.after(diving_enemy_player_collision),
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(OnExit(AppState::Playing), cleanup_player_bullets);
    }
}

fn player_shoot(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    game_data: Res<GameData>,
    player_query: Query<&Transform, With<Player>>,
    bullet_query: Query<&PlayerBullet>,
) {
    if game_data.phase != WavePhase::Combat {
        return;
    }

    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    if bullet_query.iter().count() >= MAX_PLAYER_BULLETS {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    commands.spawn((
        PlayerBullet,
        Sprite {
            color: Color::srgb(0.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(LASER_WIDTH, LASER_HEIGHT)),
            ..default()
        },
        Transform::from_translation(
            player_transform.translation + Vec3::new(0.0, PLAYER_HEIGHT / 2.0, 0.0),
        ),
    ));
}

fn move_player_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform), With<PlayerBullet>>,
) {
    let top = WINDOW_HEIGHT / 2.0 + LASER_HEIGHT;
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
    laser_query: Query<(Entity, &Transform), With<PlayerBullet>>,
    enemy_query: Query<(Entity, &Transform, &FormationSlot), With<Enemy>>,
) {
    if game_data.phase != WavePhase::Combat && game_data.phase != WavePhase::Respawning {
        return;
    }

    let mut hit_lasers = HashSet::new();
    let mut hit_enemies = HashSet::new();

    for (laser_entity, laser_transform) in &laser_query {
        if hit_lasers.contains(&laser_entity) {
            continue;
        }

        for (enemy_entity, enemy_transform, slot) in &enemy_query {
            if hit_enemies.contains(&enemy_entity) {
                continue;
            }

            let dist = laser_transform
                .translation
                .truncate()
                .distance(enemy_transform.translation.truncate());

            if dist < LASER_COLLISION_RADIUS + ENEMY_COLLISION_RADIUS {
                hit_lasers.insert(laser_entity);
                hit_enemies.insert(enemy_entity);
                let pos = enemy_transform.translation;
                commands.entity(laser_entity).despawn();
                commands.entity(enemy_entity).despawn();
                game_data.score += score_for_row(slot.row);
                spawn_explosion(&mut commands, pos, enemy_color_for_row(slot.row));
                break;
            }
        }
    }
}

fn enemy_bullet_player_collision(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<AppState>>,
    bullet_query: Query<(Entity, &Transform), With<EnemyBullet>>,
    player_query: Query<(Entity, &Transform, Option<&Invulnerable>), With<Player>>,
) {
    if game_data.phase != WavePhase::Combat {
        return;
    }

    let Ok((player_entity, player_transform, invuln)) = player_query.single() else {
        return;
    };

    if invuln.is_some() {
        return;
    }

    for (bullet_entity, bullet_transform) in &bullet_query {
        let dist = player_transform
            .translation
            .truncate()
            .distance(bullet_transform.translation.truncate());

        if dist < PLAYER_COLLISION_RADIUS + ENEMY_BULLET_COLLISION_RADIUS {
            commands.entity(bullet_entity).despawn();
            handle_player_death(
                &mut commands,
                &mut game_data,
                &mut next_state,
                player_entity,
                player_transform.translation,
            );
            return;
        }
    }
}

#[allow(clippy::type_complexity)]
fn diving_enemy_player_collision(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<AppState>>,
    enemy_query: Query<(Entity, &Transform, &FormationSlot), (With<Enemy>, With<DivingEnemy>)>,
    player_query: Query<(Entity, &Transform, Option<&Invulnerable>), With<Player>>,
) {
    if game_data.phase != WavePhase::Combat {
        return;
    }

    let Ok((player_entity, player_transform, invuln)) = player_query.single() else {
        return;
    };

    if invuln.is_some() {
        return;
    }

    for (enemy_entity, enemy_transform, slot) in &enemy_query {
        let dist = player_transform
            .translation
            .truncate()
            .distance(enemy_transform.translation.truncate());

        if dist < PLAYER_COLLISION_RADIUS + ENEMY_COLLISION_RADIUS {
            let enemy_pos = enemy_transform.translation;
            commands.entity(enemy_entity).despawn();
            spawn_explosion(&mut commands, enemy_pos, enemy_color_for_row(slot.row));
            handle_player_death(
                &mut commands,
                &mut game_data,
                &mut next_state,
                player_entity,
                player_transform.translation,
            );
            return;
        }
    }
}

fn handle_player_death(
    commands: &mut Commands,
    game_data: &mut ResMut<GameData>,
    next_state: &mut ResMut<NextState<AppState>>,
    player_entity: Entity,
    position: Vec3,
) {
    commands.entity(player_entity).despawn();
    game_data.lives = game_data.lives.saturating_sub(1);
    game_data.phase = WavePhase::Respawning;

    spawn_explosion(commands, position, Color::srgb(0.2, 0.6, 1.0));

    if game_data.lives == 0 {
        commands.remove_resource::<RespawnTimer>();
        next_state.set(AppState::GameOver);
    } else {
        commands.insert_resource(RespawnTimer(Timer::from_seconds(
            RESPAWN_DELAY,
            TimerMode::Once,
        )));
    }
}

fn check_stage_clear(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    enemy_query: Query<(), With<Enemy>>,
    bullet_query: Query<Entity, With<EnemyBullet>>,
) {
    if game_data.phase != WavePhase::Combat && game_data.phase != WavePhase::Respawning {
        return;
    }
    if game_data.lives == 0 {
        return;
    }

    if enemy_query.is_empty() {
        game_data.phase = WavePhase::StageClear;
        commands.insert_resource(StageClearTimer(Timer::from_seconds(
            STAGE_CLEAR_DELAY,
            TimerMode::Once,
        )));
        commands.remove_resource::<RespawnTimer>();
        for entity in &bullet_query {
            commands.entity(entity).despawn();
        }
    }
}

fn cleanup_player_bullets(
    mut commands: Commands,
    query: Query<Entity, With<PlayerBullet>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
