use bevy::prelude::*;

use crate::{
    components::{
        Bullet, CentipedeDir, CentipedeHead, CentipedeSegment, Flea, GridPos, Mushroom, Player,
        Poisoned, Scorpion, Spider,
    },
    constants::*,
    enemies::flea_drop_mushroom,
    mushroom::{mushroom_color, spawn_mushroom_at},
    resources::{CentipedeChains, Lives, MushroomGrid, NextChainId, RespawnTimer, Score},
    scheduling::GameplaySet,
    states::AppState,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                bullet_vs_mushroom,
                bullet_vs_centipede,
                bullet_vs_enemies,
                player_vs_enemies,
                centipede_vs_player,
            )
                .chain()
                .in_set(GameplaySet::Collision),
        );
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn overlaps(ax: f32, ay: f32, ar: f32, bx: f32, by: f32, br: f32) -> bool {
    (ax - bx).abs() < ar + br && (ay - by).abs() < ar + br
}

#[allow(clippy::type_complexity)]
fn chain_direction(
    chain: &[Entity],
    segment_q: &Query<(
        Entity,
        &Transform,
        &GridPos,
        &CentipedeSegment,
        Option<&CentipedeHead>,
        Option<&CentipedeDir>,
    )>,
) -> i32 {
    chain
        .first()
        .and_then(|entity| segment_q.get(*entity).ok())
        .and_then(|(_, _, _, _, _, dir)| dir)
        .map(|dir| dir.dx)
        .unwrap_or(1)
}

// ── bullet vs mushroom ────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
fn bullet_vs_mushroom(
    mut commands: Commands,
    bullet_q: Query<(Entity, &Transform), With<Bullet>>,
    mut mushroom_q: Query<(
        Entity,
        &GridPos,
        &mut Mushroom,
        &MeshMaterial2d<ColorMaterial>,
        Option<&Poisoned>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut grid: ResMut<MushroomGrid>,
) {
    for (bullet_entity, bullet_t) in &bullet_q {
        let bx = bullet_t.translation.x;
        let by = bullet_t.translation.y;

        for (m_entity, pos, mut mushroom, mat_handle, poisoned) in &mut mushroom_q {
            let mx = grid_to_world_x(pos.col);
            let my = grid_to_world_y(pos.row);

            if overlaps(bx, by, 7.0, mx, my, CELL_SIZE * 0.36) {
                mushroom.hits += 1;
                commands.entity(bullet_entity).despawn();

                if mushroom.hits >= MUSHROOM_MAX_HITS {
                    grid.0.remove(&(pos.col, pos.row));
                    commands.entity(m_entity).despawn();
                } else if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.color = mushroom_color(mushroom.hits, poisoned.is_some());
                }
                break;
            }
        }
    }
}

// ── bullet vs centipede ───────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn bullet_vs_centipede(
    mut commands: Commands,
    bullet_q: Query<(Entity, &Transform), With<Bullet>>,
    segment_q: Query<(
        Entity,
        &Transform,
        &GridPos,
        &CentipedeSegment,
        Option<&CentipedeHead>,
        Option<&CentipedeDir>,
    )>,
    mut chains: ResMut<CentipedeChains>,
    mut next_id: ResMut<NextChainId>,
    mut score: ResMut<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut grid: ResMut<MushroomGrid>,
    poisoned_mushrooms: Query<&GridPos, (With<Mushroom>, With<Poisoned>)>,
) {
    for (bullet_entity, bullet_t) in &bullet_q {
        let bx = bullet_t.translation.x;
        let by = bullet_t.translation.y;

        for (seg_entity, seg_t, seg_pos, seg_comp, is_head, _) in &segment_q {
            let sx = seg_t.translation.x;
            let sy = seg_t.translation.y;

            if !overlaps(bx, by, 7.0, sx, sy, CELL_SIZE * 0.45) {
                continue;
            }

            // Score
            if is_head.is_some() {
                score.value += 100;
            } else {
                score.value += 10;
            }

            commands.entity(bullet_entity).despawn();

            let chain_id = seg_comp.chain_id;
            let hit_index = seg_comp.index;

            // Spawn mushroom at segment's grid position
            let mesh = meshes.add(Rectangle::new(CELL_SIZE * 0.72, CELL_SIZE * 0.72));
            let is_pos_poisoned = poisoned_mushrooms
                .iter()
                .any(|p| p.col == seg_pos.col && p.row == seg_pos.row);
            let m_entity = spawn_mushroom_at(
                &mut commands,
                &mut materials,
                &mesh,
                seg_pos.col,
                seg_pos.row,
                0,
                is_pos_poisoned,
            );
            grid.0.insert((seg_pos.col, seg_pos.row), m_entity);

            // Despawn hit segment
            commands.entity(seg_entity).despawn();

            // Split the chain
            if let Some(chain) = chains.0.remove(&chain_id) {
                let chain_dx = chain_direction(&chain, &segment_q);
                let leading: Vec<Entity> = chain[..hit_index].to_vec();
                let trailing: Vec<Entity> = chain[hit_index + 1..].to_vec();

                // Leading part keeps chain_id
                if !leading.is_empty() {
                    chains.0.insert(chain_id, leading);
                }

                // Trailing part gets a new chain_id; its first entity becomes a head
                if !trailing.is_empty() {
                    let new_chain_id = next_id.next();
                    let new_head = trailing[0];
                    commands.entity(new_head).insert(CentipedeHead);
                    commands
                        .entity(new_head)
                        .insert(CentipedeDir { dx: chain_dx });
                    // Update CentipedeSegment chain_id for all trailing
                    for (new_idx, &e) in trailing.iter().enumerate() {
                        commands.entity(e).insert(CentipedeSegment {
                            chain_id: new_chain_id,
                            index: new_idx,
                        });
                    }
                    chains.0.insert(new_chain_id, trailing);
                }
            }

            break;
        }
    }
}

// ── bullet vs enemies ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn bullet_vs_enemies(
    mut commands: Commands,
    bullet_q: Query<(Entity, &Transform), With<Bullet>>,
    flea_q: Query<(Entity, &Transform, &Flea)>,
    spider_q: Query<(Entity, &Transform), With<Spider>>,
    scorpion_q: Query<(Entity, &Transform), With<Scorpion>>,
    player_q: Query<&Transform, With<Player>>,
    mut score: ResMut<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut grid: ResMut<MushroomGrid>,
) {
    let player_x = player_q.single().map(|t| t.translation.x).unwrap_or(0.0);

    'bullets: for (bullet_entity, bullet_t) in &bullet_q {
        let bx = bullet_t.translation.x;
        let by = bullet_t.translation.y;

        // vs Flea
        for (flea_entity, flea_t, flea) in &flea_q {
            if overlaps(
                bx,
                by,
                7.0,
                flea_t.translation.x,
                flea_t.translation.y,
                CELL_SIZE * 0.35,
            ) {
                commands.entity(bullet_entity).despawn();
                if flea.hits == 0 {
                    // First hit: drop a mushroom, keep flea alive with hits=1
                    commands.entity(flea_entity).insert(Flea { hits: 1 });
                    flea_drop_mushroom(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &mut grid,
                        flea_t.translation.x,
                        flea_t.translation.y,
                    );
                } else {
                    // Second hit: kill flea
                    score.value += 200;
                    commands.entity(flea_entity).despawn();
                }
                continue 'bullets;
            }
        }

        // vs Spider
        for (spider_entity, spider_t) in &spider_q {
            if overlaps(
                bx,
                by,
                7.0,
                spider_t.translation.x,
                spider_t.translation.y,
                CELL_SIZE * 0.4,
            ) {
                commands.entity(bullet_entity).despawn();
                let dist = (bx - player_x).abs();
                let pts = if dist < CELL_SIZE * 3.0 {
                    900
                } else if dist < CELL_SIZE * 6.0 {
                    600
                } else {
                    300
                };
                score.value += pts;
                commands.entity(spider_entity).despawn();
                continue 'bullets;
            }
        }

        // vs Scorpion
        for (scorpion_entity, scorpion_t) in &scorpion_q {
            if overlaps(
                bx,
                by,
                7.0,
                scorpion_t.translation.x,
                scorpion_t.translation.y,
                CELL_SIZE * 0.55,
            ) {
                commands.entity(bullet_entity).despawn();
                score.value += 1000;
                commands.entity(scorpion_entity).despawn();
                continue 'bullets;
            }
        }
    }
}

// ── player vs enemies ─────────────────────────────────────────────────────────

fn player_vs_enemies(
    mut commands: Commands,
    player_q: Query<(Entity, &Transform), With<Player>>,
    flea_q: Query<&Transform, With<Flea>>,
    spider_q: Query<&Transform, With<Spider>>,
    mut lives: ResMut<Lives>,
    mut respawn: ResMut<RespawnTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if respawn.0.is_some() {
        return;
    }
    let Ok((player_entity, player_t)) = player_q.single() else {
        return;
    };
    let px = player_t.translation.x;
    let py = player_t.translation.y;

    let hit = flea_q.iter().any(|t| {
        overlaps(
            px,
            py,
            12.0,
            t.translation.x,
            t.translation.y,
            CELL_SIZE * 0.35,
        )
    }) || spider_q.iter().any(|t| {
        overlaps(
            px,
            py,
            12.0,
            t.translation.x,
            t.translation.y,
            CELL_SIZE * 0.4,
        )
    });

    if hit {
        player_death(
            &mut commands,
            player_entity,
            &mut lives,
            &mut respawn,
            &mut next_state,
        );
    }
}

// ── centipede vs player ───────────────────────────────────────────────────────

fn centipede_vs_player(
    mut commands: Commands,
    player_q: Query<(Entity, &Transform), With<Player>>,
    segment_q: Query<&GridPos, With<CentipedeSegment>>,
    mut lives: ResMut<Lives>,
    mut respawn: ResMut<RespawnTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if respawn.0.is_some() {
        return;
    }
    let Ok((player_entity, player_t)) = player_q.single() else {
        return;
    };
    let px = player_t.translation.x;
    let py = player_t.translation.y;

    for seg_pos in &segment_q {
        let sx = grid_to_world_x(seg_pos.col);
        let sy = grid_to_world_y(seg_pos.row);
        if overlaps(px, py, 12.0, sx, sy, CELL_SIZE * 0.45) {
            player_death(
                &mut commands,
                player_entity,
                &mut lives,
                &mut respawn,
                &mut next_state,
            );
            return;
        }
    }
}

// ── shared death logic ────────────────────────────────────────────────────────

fn player_death(
    commands: &mut Commands,
    player_entity: Entity,
    lives: &mut ResMut<Lives>,
    respawn: &mut ResMut<RespawnTimer>,
    next_state: &mut ResMut<NextState<AppState>>,
) {
    commands.entity(player_entity).despawn();

    if lives.0 <= 1 {
        lives.0 = 0;
        next_state.set(AppState::GameOver);
        return;
    }

    lives.0 -= 1;
    respawn.0 = Some(Timer::from_seconds(RESPAWN_DELAY, TimerMode::Once));
}
