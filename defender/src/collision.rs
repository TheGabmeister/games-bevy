use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::camera::*;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::scheduling::GameplaySet;
use crate::spawning::*;
use crate::states::AppState;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, collision_detection.in_set(GameplaySet::Collision));
    }
}

#[derive(SystemParam)]
pub struct CollisionQueries<'w, 's> {
    player_q: Query<
        'w,
        's,
        (
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        With<Player>,
    >,
    player_projectiles: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        With<PlayerProjectile>,
    >,
    enemy_projectiles: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        (With<EnemyProjectile>, Without<PlayerProjectile>),
    >,
    mines: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        (
            With<Mine>,
            Without<PlayerProjectile>,
            Without<EnemyProjectile>,
        ),
    >,
    landers: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
            Option<&'static LanderState>,
        ),
        With<Lander>,
    >,
    mutants: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        (With<Mutant>, Without<Lander>),
    >,
    bombers: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        (With<Bomber>, Without<Lander>, Without<Mutant>),
    >,
    pods: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        (With<Pod>, Without<Lander>, Without<Mutant>, Without<Bomber>),
    >,
    swarmers: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        (
            With<Swarmer>,
            Without<Lander>,
            Without<Mutant>,
            Without<Bomber>,
            Without<Pod>,
        ),
    >,
    baiters: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        (
            With<Baiter>,
            Without<Lander>,
            Without<Mutant>,
            Without<Bomber>,
            Without<Pod>,
            Without<Swarmer>,
        ),
    >,
    falling_humans: Query<
        'w,
        's,
        (
            Entity,
            &'static WorldPosition,
            &'static Transform,
            &'static CollisionRadius,
        ),
        (With<Human>, With<HumanFalling>, Without<CaughtByPlayer>),
    >,
}

pub fn collision_detection(
    mut commands: Commands,
    assets: Res<GameplayAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
    queries: CollisionQueries,
) {
    let Ok((p_wp, p_tf, p_rad)) = queries.player_q.single() else {
        return;
    };

    // --- Player projectiles vs enemies ---
    for (proj_entity, proj_wp, proj_tf, proj_rad) in &queries.player_projectiles {
        let mut hit = false;

        // vs Landers
        for (enemy_e, enemy_wp, enemy_tf, enemy_rad, lander_state) in &queries.landers {
            if collides(
                proj_wp.0,
                proj_tf.translation.y,
                proj_rad.0,
                enemy_wp.0,
                enemy_tf.translation.y,
                enemy_rad.0,
            ) {
                game_state.score += SCORE_LANDER;
                let wx = enemy_wp.0;
                let y = enemy_tf.translation.y;

                // If carrying a human, free it
                if let Some(LanderState::Ascending(human_entity)) = lander_state {
                    commands.entity(*human_entity).remove::<GrabbedBy>();
                    commands.entity(*human_entity).insert(HumanFalling);
                }

                commands.entity(enemy_e).despawn();
                spawn_explosion(&mut commands, &assets, &mut materials, wx, y, COLOR_LANDER);
                hit = true;
                break;
            }
        }
        if hit {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // vs Mutants
        for (enemy_e, enemy_wp, enemy_tf, enemy_rad) in &queries.mutants {
            if collides(
                proj_wp.0,
                proj_tf.translation.y,
                proj_rad.0,
                enemy_wp.0,
                enemy_tf.translation.y,
                enemy_rad.0,
            ) {
                game_state.score += SCORE_MUTANT;
                let wx = enemy_wp.0;
                let y = enemy_tf.translation.y;
                commands.entity(enemy_e).despawn();
                spawn_explosion(&mut commands, &assets, &mut materials, wx, y, COLOR_MUTANT);
                hit = true;
                break;
            }
        }
        if hit {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // vs Bombers
        for (enemy_e, enemy_wp, enemy_tf, enemy_rad) in &queries.bombers {
            if collides(
                proj_wp.0,
                proj_tf.translation.y,
                proj_rad.0,
                enemy_wp.0,
                enemy_tf.translation.y,
                enemy_rad.0,
            ) {
                game_state.score += SCORE_BOMBER;
                let wx = enemy_wp.0;
                let y = enemy_tf.translation.y;
                commands.entity(enemy_e).despawn();
                spawn_explosion(&mut commands, &assets, &mut materials, wx, y, COLOR_BOMBER);
                hit = true;
                break;
            }
        }
        if hit {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // vs Pods (spawn swarmers)
        for (enemy_e, enemy_wp, enemy_tf, enemy_rad) in &queries.pods {
            if collides(
                proj_wp.0,
                proj_tf.translation.y,
                proj_rad.0,
                enemy_wp.0,
                enemy_tf.translation.y,
                enemy_rad.0,
            ) {
                game_state.score += SCORE_POD;
                let wx = enemy_wp.0;
                let y = enemy_tf.translation.y;
                commands.entity(enemy_e).despawn();
                spawn_explosion(&mut commands, &assets, &mut materials, wx, y, COLOR_POD);
                // Spawn swarmers
                for i in 0..4 {
                    let offset = (i as f32 - 1.5) * 20.0;
                    spawn_swarmer(&mut commands, &assets, wrap_x(wx + offset), y);
                }
                hit = true;
                break;
            }
        }
        if hit {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // vs Swarmers
        for (enemy_e, enemy_wp, enemy_tf, enemy_rad) in &queries.swarmers {
            if collides(
                proj_wp.0,
                proj_tf.translation.y,
                proj_rad.0,
                enemy_wp.0,
                enemy_tf.translation.y,
                enemy_rad.0,
            ) {
                game_state.score += SCORE_SWARMER;
                let wx = enemy_wp.0;
                let y = enemy_tf.translation.y;
                commands.entity(enemy_e).despawn();
                spawn_explosion(&mut commands, &assets, &mut materials, wx, y, COLOR_SWARMER);
                hit = true;
                break;
            }
        }
        if hit {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // vs Baiters
        for (enemy_e, enemy_wp, enemy_tf, enemy_rad) in &queries.baiters {
            if collides(
                proj_wp.0,
                proj_tf.translation.y,
                proj_rad.0,
                enemy_wp.0,
                enemy_tf.translation.y,
                enemy_rad.0,
            ) {
                game_state.score += SCORE_BAITER;
                let wx = enemy_wp.0;
                let y = enemy_tf.translation.y;
                commands.entity(enemy_e).despawn();
                spawn_explosion(&mut commands, &assets, &mut materials, wx, y, COLOR_BAITER);
                hit = true;
                break;
            }
        }
        if hit {
            commands.entity(proj_entity).despawn();
            continue;
        }
    }

    // --- Player vs enemies ---
    let all_enemies: Vec<(Entity, f32, f32, f32)> = queries
        .landers
        .iter()
        .map(|(e, wp, tf, r, _)| (e, wp.0, tf.translation.y, r.0))
        .chain(
            queries
                .mutants
                .iter()
                .map(|(e, wp, tf, r)| (e, wp.0, tf.translation.y, r.0)),
        )
        .chain(
            queries
                .bombers
                .iter()
                .map(|(e, wp, tf, r)| (e, wp.0, tf.translation.y, r.0)),
        )
        .chain(
            queries
                .pods
                .iter()
                .map(|(e, wp, tf, r)| (e, wp.0, tf.translation.y, r.0)),
        )
        .chain(
            queries
                .swarmers
                .iter()
                .map(|(e, wp, tf, r)| (e, wp.0, tf.translation.y, r.0)),
        )
        .chain(
            queries
                .baiters
                .iter()
                .map(|(e, wp, tf, r)| (e, wp.0, tf.translation.y, r.0)),
        )
        .collect();

    for (_enemy_e, ex, ey, er) in &all_enemies {
        if collides(p_wp.0, p_tf.translation.y, p_rad.0, *ex, *ey, *er) {
            next_state.set(AppState::PlayerDeath);
            return;
        }
    }

    // --- Player vs enemy projectiles ---
    for (_entity, wp, tf, rad) in &queries.enemy_projectiles {
        if collides(
            p_wp.0,
            p_tf.translation.y,
            p_rad.0,
            wp.0,
            tf.translation.y,
            rad.0,
        ) {
            next_state.set(AppState::PlayerDeath);
            return;
        }
    }

    // --- Player vs mines ---
    for (_entity, wp, tf, rad) in &queries.mines {
        if collides(
            p_wp.0,
            p_tf.translation.y,
            p_rad.0,
            wp.0,
            tf.translation.y,
            rad.0,
        ) {
            next_state.set(AppState::PlayerDeath);
            return;
        }
    }

    // --- Player vs falling humans (catch them) ---
    for (h_entity, h_wp, h_tf, h_rad) in &queries.falling_humans {
        if collides(
            p_wp.0,
            p_tf.translation.y,
            p_rad.0,
            h_wp.0,
            h_tf.translation.y,
            h_rad.0,
        ) {
            commands.entity(h_entity).insert(CaughtByPlayer);
            game_state.score += SCORE_HUMAN_SAVED;
        }
    }

    // --- Check extra life ---
    if game_state.score >= game_state.next_extra_life_score {
        game_state.lives += 1;
        game_state.smart_bombs += 1;
        game_state.next_extra_life_score += EXTRA_LIFE_INTERVAL;
    }
}

fn collides(ax: f32, ay: f32, ar: f32, bx: f32, by: f32, br: f32) -> bool {
    world_distance(ax, ay, bx, by) < ar + br
}
