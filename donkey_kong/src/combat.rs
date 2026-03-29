use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::level::*;
use crate::resources::*;
use crate::states::AppState;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_hammer_pickup,
                hammer_vs_hazards.after(check_hammer_pickup),
                player_vs_hazards.after(hammer_vs_hazards),
                jump_over_scoring.after(player_vs_hazards),
                check_goal.after(jump_over_scoring),
                tick_bonus_timer.after(check_goal),
                bonus_items_system.after(tick_bonus_timer),
                check_extra_life.after(bonus_items_system),
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}

// --- AABB-circle overlap ---

fn aabb_circle_overlap(
    aabb_min: Vec2,
    aabb_max: Vec2,
    circle_center: Vec2,
    radius: f32,
) -> bool {
    let closest = Vec2::new(
        circle_center.x.clamp(aabb_min.x, aabb_max.x),
        circle_center.y.clamp(aabb_min.y, aabb_max.y),
    );
    closest.distance(circle_center) < radius
}

fn player_aabb(tf: &Transform) -> (Vec2, Vec2) {
    let half = Vec2::new(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0);
    let center = tf.translation.truncate();
    (center - half, center + half)
}

// --- Hammer Pickup ---

fn check_hammer_pickup(
    mut commands: Commands,
    game_mats: Res<GameMaterials>,
    mut player_q: Query<(&Transform, &mut PlayerState, &mut MeshMaterial2d<ColorMaterial>), With<Player>>,
    hammer_q: Query<(Entity, &Transform), With<HammerPickup>>,
) {
    let Ok((ptf, mut ps, mut mat)) = player_q.single_mut() else { return };
    if ps.hammer_timer.is_some() {
        return;
    }

    let (pmin, pmax) = player_aabb(ptf);

    for (entity, htf) in &hammer_q {
        let hmin = htf.translation.truncate() - Vec2::splat(HAMMER_PICKUP_SIZE / 2.0);
        let hmax = htf.translation.truncate() + Vec2::splat(HAMMER_PICKUP_SIZE / 2.0);

        if pmin.x < hmax.x && pmax.x > hmin.x && pmin.y < hmax.y && pmax.y > hmin.y {
            ps.hammer_timer = Some(HAMMER_DURATION);
            mat.0 = game_mats.player_hammer.clone();
            commands.entity(entity).despawn();
            return;
        }
    }
}

// --- Hammer vs Hazards ---

fn hammer_vs_hazards(
    mut commands: Commands,
    mut run_data: ResMut<RunData>,
    player_q: Query<(&Transform, &PlayerState), With<Player>>,
    barrel_q: Query<(Entity, &Transform), With<Barrel>>,
    fireball_q: Query<(Entity, &Transform), With<Fireball>>,
) {
    let Ok((ptf, ps)) = player_q.single() else { return };
    if ps.hammer_timer.is_none() {
        return;
    }

    // Hammer hit zone: forward from player facing
    let cx = ptf.translation.x + ps.facing * HAMMER_HIT_FORWARD / 2.0;
    let cy = ptf.translation.y + HAMMER_HIT_UP / 2.0;
    let hmin = Vec2::new(
        cx - HAMMER_HIT_FORWARD / 2.0,
        cy - HAMMER_HIT_UP / 2.0,
    );
    let hmax = Vec2::new(
        cx + HAMMER_HIT_FORWARD / 2.0,
        cy + HAMMER_HIT_UP / 2.0,
    );

    for (entity, btf) in &barrel_q {
        if aabb_circle_overlap(hmin, hmax, btf.translation.truncate(), BARREL_RADIUS) {
            commands.entity(entity).despawn();
            run_data.score += SCORE_SMASH_BARREL;
        }
    }

    for (entity, ftf) in &fireball_q {
        if aabb_circle_overlap(hmin, hmax, ftf.translation.truncate(), FIREBALL_RADIUS) {
            commands.entity(entity).despawn();
            run_data.score += SCORE_SMASH_FIREBALL;
        }
    }
}

// --- Player vs Hazards (death) ---

fn player_vs_hazards(
    mut commands: Commands,
    player_q: Query<(&Transform, &PlayerState), With<Player>>,
    barrel_q: Query<&Transform, With<Barrel>>,
    fireball_q: Query<&Transform, (With<Fireball>, Without<Player>, Without<Barrel>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok((ptf, ps)) = player_q.single() else { return };
    // Fall death detected by player locomotion (set in player_movement)
    if ps.locomotion == Locomotion::Dying {
        commands.insert_resource(DeathSequence {
            elapsed: 0.0,
            cause: DeathCause::Fall,
        });
        next_state.set(AppState::Dying);
        return;
    }

    if ps.hammer_timer.is_some() {
        return;
    }

    let (pmin, pmax) = player_aabb(ptf);

    for btf in &barrel_q {
        if aabb_circle_overlap(pmin, pmax, btf.translation.truncate(), BARREL_RADIUS) {
            commands.insert_resource(DeathSequence {
                elapsed: 0.0,
                cause: DeathCause::Barrel,
            });
            next_state.set(AppState::Dying);
            return;
        }
    }

    for ftf in &fireball_q {
        if aabb_circle_overlap(pmin, pmax, ftf.translation.truncate(), FIREBALL_RADIUS) {
            commands.insert_resource(DeathSequence {
                elapsed: 0.0,
                cause: DeathCause::Fireball,
            });
            next_state.set(AppState::Dying);
            return;
        }
    }
}

// --- Jump-Over Scoring ---

fn jump_over_scoring(
    mut run_data: ResMut<RunData>,
    mut player_q: Query<(&Transform, &mut PlayerState), With<Player>>,
    barrel_q: Query<(Entity, &Transform), With<Barrel>>,
) {
    let Ok((ptf, mut ps)) = player_q.single_mut() else { return };
    if !matches!(ps.locomotion, Locomotion::Jumping | Locomotion::Falling) {
        // Landing — award score if any barrels were jumped
        if ps.jump_score_count > 0 {
            // Already awarded during the jump
            ps.jump_score_count = 0;
        }
        ps.jump_scored.clear();
        return;
    }

    let feet_y = ptf.translation.y - PLAYER_HEIGHT / 2.0;
    let px = ptf.translation.x;

    for (entity, btf) in &barrel_q {
        if ps.jump_scored.contains(&entity) {
            continue;
        }

        let barrel_top = btf.translation.y + BARREL_RADIUS;
        let barrel_x = btf.translation.x;

        // Feet above barrel top, and barrel within horizontal scoring footprint
        if feet_y > barrel_top && (px - barrel_x).abs() < JUMP_SCORE_HALF_WIDTH {
            ps.jump_scored.push(entity);
            let prev_count = ps.jump_score_count;
            ps.jump_score_count += 1;

            // Score: first barrel = 100, two or more = 300 total
            if ps.jump_score_count == 1 {
                run_data.score += SCORE_JUMP_ONE;
            } else if ps.jump_score_count == 2 && prev_count == 1 {
                // Upgrade from 100 to 300 (add 200 more)
                run_data.score += SCORE_JUMP_MULTI - SCORE_JUMP_ONE;
            }
            // Additional barrels beyond 2 don't add more
        }
    }
}

// --- Goal Detection ---

fn check_goal(
    player_q: Query<(&Transform, &PlayerState), With<Player>>,
    stage: Res<StageData>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok((ptf, ps)) = player_q.single() else { return };
    if ps.locomotion == Locomotion::Dying {
        return;
    }

    let (pmin, pmax) = player_aabb(ptf);
    let goal = stage.goal_zone_center;
    let gmin = goal - Vec2::new(GOAL_ZONE_WIDTH / 2.0, GOAL_ZONE_HEIGHT / 2.0);
    let gmax = goal + Vec2::new(GOAL_ZONE_WIDTH / 2.0, GOAL_ZONE_HEIGHT / 2.0);

    if pmin.x < gmax.x && pmax.x > gmin.x && pmin.y < gmax.y && pmax.y > gmin.y {
        next_state.set(AppState::WaveTally);
    }
}

// --- Bonus Timer ---

fn tick_bonus_timer(
    time: Res<Time>,
    mut commands: Commands,
    mut wave_rt: ResMut<WaveRuntime>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    wave_rt.elapsed_wave_time += time.delta_secs();
    wave_rt.bonus_tick += time.delta_secs();

    if wave_rt.bonus_tick >= BONUS_TIMER_INTERVAL {
        wave_rt.bonus_tick -= BONUS_TIMER_INTERVAL;
        wave_rt.bonus_timer = (wave_rt.bonus_timer - BONUS_TIMER_DECREASE).max(0);

        if wave_rt.bonus_timer <= 0 {
            commands.insert_resource(DeathSequence {
                elapsed: 0.0,
                cause: DeathCause::Timer,
            });
            next_state.set(AppState::Dying);
        }
    }
}

// --- Bonus Items ---

fn bonus_items_system(
    mut commands: Commands,
    stage: Res<StageData>,
    game_meshes: Res<GameMeshes>,
    game_mats: Res<GameMaterials>,
    mut wave_rt: ResMut<WaveRuntime>,
    mut run_data: ResMut<RunData>,
    player_q: Query<&Transform, With<Player>>,
    bonus_q: Query<(Entity, &BonusItemEntity, &Transform)>,
) {
    let elapsed = wave_rt.elapsed_wave_time;

    for i in 0..3 {
        let spawn_time = BONUS_ITEM_TIMES[i];
        let expire_time = spawn_time + BONUS_ITEM_DURATION;

        match wave_rt.bonus_items[i] {
            BonusItemStatus::Pending => {
                if elapsed >= spawn_time {
                    wave_rt.bonus_items[i] = BonusItemStatus::Active;
                    commands.spawn((
                        StageEntity,
                        BonusItemEntity(i),
                        Mesh2d(game_meshes.bonus_item.clone()),
                        MeshMaterial2d(game_mats.bonus_item.clone()),
                        Transform::from_xyz(
                            stage.bonus_item_position.x,
                            stage.bonus_item_position.y,
                            4.0,
                        ),
                    ));
                }
            }
            BonusItemStatus::Active => {
                if elapsed >= expire_time {
                    wave_rt.bonus_items[i] = BonusItemStatus::Expired;
                    for (entity, bi, _) in &bonus_q {
                        if bi.0 == i {
                            commands.entity(entity).despawn();
                        }
                    }
                } else {
                    // Check player pickup
                    if let Ok(ptf) = player_q.single() {
                        let (pmin, pmax) = player_aabb(ptf);
                        for (entity, bi, btf) in &bonus_q {
                            if bi.0 != i {
                                continue;
                            }
                            let bmin = btf.translation.truncate()
                                - Vec2::splat(BONUS_ITEM_SIZE / 2.0);
                            let bmax = btf.translation.truncate()
                                + Vec2::splat(BONUS_ITEM_SIZE / 2.0);
                            if pmin.x < bmax.x
                                && pmax.x > bmin.x
                                && pmin.y < bmax.y
                                && pmax.y > bmin.y
                            {
                                wave_rt.bonus_items[i] = BonusItemStatus::Collected;
                                run_data.score += BONUS_ITEM_VALUES[i];
                                commands.entity(entity).despawn();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// --- Extra Life ---

fn check_extra_life(mut run_data: ResMut<RunData>) {
    if !run_data.extra_life_awarded && run_data.score >= EXTRA_LIFE_SCORE {
        run_data.extra_life_awarded = true;
        if run_data.lives < MAX_LIVES {
            run_data.lives += 1;
        }
    }
}
