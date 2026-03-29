#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::effects::{spawn_particles, spawn_score_popup};
use crate::resources::*;
use crate::states::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Rescue resolves before brain conversion (spec requirement)
                player_vs_human,
                bullet_collisions,
                // Damage systems run after bullets and rescue
                (
                    damage_vs_player,
                    electrode_vs_killable,
                    hulk_vs_human,
                    brain_vs_human,
                ),
                check_wave_clear,
            )
                .chain()
                .in_set(GameSet::Combat),
        );
    }
}

fn player_vs_human(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    assets: Res<GameAssets>,
    player_q: Query<(&Transform, &CollisionRadius), With<Player>>,
    human_q: Query<(Entity, &Transform, &CollisionRadius), With<Human>>,
) {
    let Ok((p_tf, p_radius)) = player_q.single() else {
        return;
    };
    let p_pos = p_tf.translation.truncate();
    for (h_entity, h_tf, h_radius) in &human_q {
        let h_pos = h_tf.translation.truncate();
        let radii = p_radius.0 + h_radius.0;
        if p_pos.distance_squared(h_pos) < radii * radii {
            commands.entity(h_entity).despawn();
            game.rescue_count_this_wave += 1;
            let bonus = match game.rescue_count_this_wave {
                1 => 1000,
                2 => 2000,
                3 => 3000,
                4 => 4000,
                _ => 5000,
            };
            game.award_score(bonus);
            spawn_particles(
                &mut commands,
                &assets,
                h_pos,
                RESCUE_PARTICLE_COUNT,
                &assets.particle_material_rescue,
            );
            spawn_score_popup(&mut commands, h_pos, bonus);
        }
    }
}

fn bullet_collisions(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    mut shake: ResMut<ScreenShake>,
    assets: Res<GameAssets>,
    bullet_q: Query<(Entity, &Transform, &Velocity, &CollisionRadius), With<PlayerBullet>>,
    killable_q: Query<(Entity, &Transform, &CollisionRadius, &PointValue), With<Killable>>,
    mut hulk_q: Query<(&Transform, &CollisionRadius, &mut Knockback), With<Hulk>>,
    electrode_q: Query<(Entity, &Transform, &CollisionRadius), With<Electrode>>,
) {
    for (b_entity, b_tf, b_vel, b_radius) in &bullet_q {
        let b_pos = b_tf.translation.truncate();
        let mut hit = false;

        // Check killable enemies
        for (e_entity, e_tf, e_radius, points) in &killable_q {
            let e_pos = e_tf.translation.truncate();
            let radii = b_radius.0 + e_radius.0;
            if b_pos.distance_squared(e_pos) < radii * radii {
                commands.entity(b_entity).despawn();
                commands.entity(e_entity).despawn();
                game.award_score(points.0);
                spawn_particles(
                    &mut commands,
                    &assets,
                    e_pos,
                    EXPLOSION_PARTICLE_COUNT,
                    &assets.particle_material_explosion,
                );
                spawn_score_popup(&mut commands, e_pos, points.0);
                hit = true;
                break;
            }
        }

        // Check hulks (knockback, not kill)
        if !hit {
            for (h_tf, h_radius, mut kb) in &mut hulk_q {
                let h_pos = h_tf.translation.truncate();
                let radii = b_radius.0 + h_radius.0;
                if b_pos.distance_squared(h_pos) < radii * radii {
                    commands.entity(b_entity).despawn();
                    kb.0 += b_vel.0.normalize_or_zero() * HULK_KNOCKBACK_STRENGTH;
                    shake.trauma = (shake.trauma + 0.1).min(1.0);
                    hit = true;
                    break;
                }
            }
        }

        // Check electrodes
        if !hit {
            for (el_entity, el_tf, el_radius) in &electrode_q {
                let el_pos = el_tf.translation.truncate();
                let radii = b_radius.0 + el_radius.0;
                if b_pos.distance_squared(el_pos) < radii * radii {
                    commands.entity(b_entity).despawn();
                    commands.entity(el_entity).despawn();
                    game.award_score(25);
                    spawn_particles(
                        &mut commands,
                        &assets,
                        el_pos,
                        8,
                        &assets.particle_material_electrode,
                    );
                    break;
                }
            }
        }
    }
}

// FIX: Combined hazard_vs_player and proj_vs_player into one system to prevent
// double-death bug where both systems could fire in the same frame
fn damage_vs_player(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    mut next_play: ResMut<NextState<PlayState>>,
    mut shake: ResMut<ScreenShake>,
    assets: Res<GameAssets>,
    player_q: Query<(Entity, &Transform, &CollisionRadius, Option<&Invincible>), With<Player>>,
    hazard_q: Query<
        (&Transform, &CollisionRadius),
        (With<DamagesPlayer>, Without<EnemyProjectile>),
    >,
    proj_q: Query<(Entity, &Transform, &CollisionRadius), With<EnemyProjectile>>,
) {
    let Ok((p_entity, p_tf, p_radius, invincible)) = player_q.single() else {
        return;
    };
    if invincible.is_some() {
        return;
    }
    let p_pos = p_tf.translation.truncate();

    // Check enemy body collisions
    for (h_tf, h_radius) in &hazard_q {
        let h_pos = h_tf.translation.truncate();
        let radii = p_radius.0 + h_radius.0;
        if p_pos.distance_squared(h_pos) < radii * radii {
            commands.entity(p_entity).despawn();
            game.lives = game.lives.saturating_sub(1);
            shake.trauma = 1.0;
            spawn_particles(
                &mut commands,
                &assets,
                p_pos,
                DEATH_PARTICLE_COUNT,
                &assets.particle_material_death,
            );
            next_play.set(PlayState::PlayerDeath);
            return;
        }
    }

    // Check enemy projectile collisions
    for (proj_entity, proj_tf, proj_r) in &proj_q {
        let proj_pos = proj_tf.translation.truncate();
        let radii = p_radius.0 + proj_r.0;
        if p_pos.distance_squared(proj_pos) < radii * radii {
            commands.entity(proj_entity).despawn();
            commands.entity(p_entity).despawn();
            game.lives = game.lives.saturating_sub(1);
            shake.trauma = 1.0;
            spawn_particles(
                &mut commands,
                &assets,
                p_pos,
                DEATH_PARTICLE_COUNT,
                &assets.particle_material_death,
            );
            next_play.set(PlayState::PlayerDeath);
            return;
        }
    }
}

fn electrode_vs_killable(
    mut commands: Commands,
    electrode_q: Query<(&Transform, &CollisionRadius), With<Electrode>>,
    killable_q: Query<(Entity, &Transform, &CollisionRadius), With<Killable>>,
) {
    for (el_tf, el_radius) in &electrode_q {
        let el_pos = el_tf.translation.truncate();
        for (k_entity, k_tf, k_radius) in &killable_q {
            let k_pos = k_tf.translation.truncate();
            let radii = el_radius.0 + k_radius.0;
            if el_pos.distance_squared(k_pos) < radii * radii {
                commands.entity(k_entity).despawn();
            }
        }
    }
}

fn hulk_vs_human(
    mut commands: Commands,
    hulk_q: Query<(&Transform, &CollisionRadius), With<Hulk>>,
    human_q: Query<(Entity, &Transform, &CollisionRadius), With<Human>>,
) {
    for (hulk_tf, hulk_r) in &hulk_q {
        let hulk_pos = hulk_tf.translation.truncate();
        for (h_entity, h_tf, h_r) in &human_q {
            let h_pos = h_tf.translation.truncate();
            let radii = hulk_r.0 + h_r.0;
            if hulk_pos.distance_squared(h_pos) < radii * radii {
                commands.entity(h_entity).despawn();
            }
        }
    }
}

fn brain_vs_human(
    mut commands: Commands,
    assets: Res<GameAssets>,
    brain_q: Query<(&Transform, &CollisionRadius), With<Brain>>,
    human_q: Query<(Entity, &Transform, &CollisionRadius), With<Human>>,
) {
    for (br_tf, br_r) in &brain_q {
        let br_pos = br_tf.translation.truncate();
        for (h_entity, h_tf, h_r) in &human_q {
            let h_pos = h_tf.translation.truncate();
            let radii = br_r.0 + h_r.0;
            if br_pos.distance_squared(h_pos) < radii * radii {
                commands.entity(h_entity).despawn();
                // Convert human to Prog
                commands.spawn((
                    Enemy,
                    Prog,
                    Killable,
                    DamagesPlayer,
                    Confined,
                    WaveEntity,
                    Mesh2d(assets.prog_mesh.clone()),
                    MeshMaterial2d(assets.prog_material.clone()),
                    Transform::from_xyz(h_pos.x, h_pos.y, 1.0),
                    Velocity(Vec2::ZERO),
                    CollisionRadius(PROG_RADIUS),
                    PointValue(100),
                    DespawnOnExit(AppState::Playing),
                ));
            }
        }
    }
}

// FIX: Now transitions to WaveClear instead of skipping straight to WaveIntro
fn check_wave_clear(
    mut next_state: ResMut<NextState<PlayState>>,
    mut shake: ResMut<ScreenShake>,
    killable_q: Query<(), With<Killable>>,
) {
    if killable_q.is_empty() {
        shake.trauma = (shake.trauma + 0.3).min(1.0);
        next_state.set(PlayState::WaveClear);
    }
}
