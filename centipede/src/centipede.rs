use bevy::prelude::*;

use crate::{
    components::{
        CentipedeDir, CentipedeHead, CentipedeSegment, GameplayEntity, GridPos, PoisonRushing,
    },
    constants::*,
    resources::{CentipedeChains, CentipedeTimer, MushroomGrid, NextChainId, Wave},
    states::AppState,
};

pub struct CentipedePlugin;

impl Plugin for CentipedePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_initial_centipede)
            .add_systems(
                Update,
                (centipede_tick, check_wave_clear).run_if(in_state(AppState::Playing)),
            );
    }
}

// ── Spawning ──────────────────────────────────────────────────────────────────

pub fn spawn_initial_centipede(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut chains: ResMut<CentipedeChains>,
    mut next_id: ResMut<NextChainId>,
    wave: Res<Wave>,
) {
    chains.0.clear();
    let interval =
        CENTIPEDE_INTERVAL_BASE * 0.92_f32.powi(wave.0 as i32);
    // Timer is handled in resources; we spawn chain from wave resources
    spawn_centipede_chain(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut chains,
        &mut next_id,
        CENTIPEDE_LENGTH,
        0,  // start row
        1,  // direction: right
    );
    let _ = interval; // interval used by main to reset the timer
}

pub fn spawn_centipede_chain(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    chains: &mut ResMut<CentipedeChains>,
    next_id: &mut ResMut<NextChainId>,
    length: usize,
    start_row: i32,
    dx: i32,
) {
    if length == 0 {
        return;
    }
    let chain_id = next_id.next();
    let head_mesh = meshes.add(Circle::new(CELL_SIZE * 0.45));
    let body_mesh = meshes.add(Rectangle::new(CELL_SIZE * 0.75, CELL_SIZE * 0.75));
    let head_mat = materials.add(Color::srgb(1.0, 0.3, 0.1));
    let body_mat = materials.add(Color::srgb(0.9, 0.6, 0.1));

    let mut entities = Vec::new();

    for i in 0..length {
        // Segments start stacked in row start_row, spaced horizontally
        // Head is leftmost when dx=1, rightmost when dx=-1
        let col = if dx > 0 {
            (length - 1 - i) as i32 // head at col 0 when fully stacked
        } else {
            i as i32
        }
        .clamp(0, GRID_COLS - 1);

        let is_head = i == 0;
        let (mesh, mat, z) = if is_head {
            (head_mesh.clone(), head_mat.clone(), 2.0)
        } else {
            (body_mesh.clone(), body_mat.clone(), 1.5)
        };

        let mut ec = commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(grid_to_world_x(col), grid_to_world_y(start_row), z),
            GridPos {
                col,
                row: start_row,
            },
            CentipedeSegment { chain_id, index: i },
            GameplayEntity,
        ));
        if is_head {
            ec.insert(CentipedeHead);
            ec.insert(CentipedeDir { dx });
        }
        entities.push(ec.id());
    }

    chains.0.insert(chain_id, entities);
}

// ── Movement tick ─────────────────────────────────────────────────────────────

pub fn centipede_tick(
    mut commands: Commands,
    mut timer: ResMut<CentipedeTimer>,
    time: Res<Time>,
    mushroom_grid: Res<MushroomGrid>,
    mut chains: ResMut<CentipedeChains>,
    mut segment_q: Query<(
        &mut GridPos,
        &mut Transform,
        Option<&mut CentipedeDir>,
        Option<&PoisonRushing>,
        &CentipedeSegment,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut next_id: ResMut<NextChainId>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let chain_ids: Vec<u32> = chains.0.keys().copied().collect();

    for chain_id in chain_ids {
        let Some(entity_list) = chains.0.get(&chain_id).cloned() else {
            continue;
        };
        if entity_list.is_empty() {
            continue;
        }

        // Collect current positions
        let mut positions: Vec<(GridPos, i32)> = Vec::new(); // (pos, dx)
        let mut is_rushing = false;
        let mut head_dx = 1i32;

        for &entity in &entity_list {
            if let Ok((pos, _, dir, rushing, _)) = segment_q.get(entity) {
                let dx = dir.map(|d| d.dx).unwrap_or(head_dx);
                if let Some(d) = dir {
                    head_dx = d.dx;
                }
                if rushing.is_some() {
                    is_rushing = true;
                }
                positions.push((*pos, dx));
            }
        }

        if positions.is_empty() {
            continue;
        }

        let head_pos = positions[0].0;
        let dx = head_dx;

        // Determine head's new position
        let (new_head_col, new_head_row, new_dx) = compute_head_move(
            head_pos,
            dx,
            is_rushing,
            &mushroom_grid,
        );

        // Check if we need to wrap (went below row 29)
        let wrapped = new_head_row >= GRID_ROWS;

        if wrapped {
            // Despawn all and respawn at top
            for &entity in &entity_list {
                commands.entity(entity).despawn();
            }
            chains.0.remove(&chain_id);

            spawn_centipede_chain(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut chains,
                &mut next_id,
                entity_list.len(),
                0,
                new_dx,
            );
            continue;
        }

        // New positions: head gets new position, body follows
        let mut new_positions: Vec<GridPos> = vec![
            GridPos {
                col: new_head_col,
                row: new_head_row,
            };
            entity_list.len()
        ];
        for i in 1..entity_list.len() {
            new_positions[i] = positions[i - 1].0;
        }

        // Apply to entities
        for (i, &entity) in entity_list.iter().enumerate() {
            if let Ok((mut pos, mut transform, mut dir, _, _)) = segment_q.get_mut(entity) {
                *pos = new_positions[i];
                transform.translation.x = grid_to_world_x(pos.col);
                transform.translation.y = grid_to_world_y(pos.row);
                if i == 0 {
                    if let Some(ref mut d) = dir {
                        d.dx = new_dx;
                    }
                }
            }
        }

        // Update PoisonRushing: if the centipede stepped through a poisoned mushroom
        // (checked in collision.rs when bullet kills — here we clear if head turned)
        // PoisonRushing is set by collision system
    }
}

fn compute_head_move(
    pos: GridPos,
    dx: i32,
    rushing: bool,
    mushroom_grid: &MushroomGrid,
) -> (i32, i32, i32) {
    if rushing {
        // Rush straight down
        let new_row = pos.row + 1;
        (pos.col, new_row, dx)
    } else {
        let next_col = pos.col + dx;
        let blocked = next_col < 0
            || next_col >= GRID_COLS
            || mushroom_grid.0.contains_key(&(next_col, pos.row));

        if blocked {
            let new_row = pos.row + 1;
            let new_dx = -dx;
            // Move down, reverse
            (pos.col, new_row, new_dx)
        } else {
            (next_col, pos.row, dx)
        }
    }
}

// ── Wave clear check ──────────────────────────────────────────────────────────

fn check_wave_clear(
    mut commands: Commands,
    mut chains_res: ResMut<CentipedeChains>,
    mut wave: ResMut<Wave>,
    mut timer: ResMut<CentipedeTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut next_id: ResMut<NextChainId>,
    mushroom_q: Query<(Entity, &GridPos), With<crate::components::Mushroom>>,
    mut mushroom_grid: ResMut<MushroomGrid>,
) {
    if !chains_res.0.is_empty() {
        return;
    }

    wave.0 += 1;
    let new_interval = CENTIPEDE_INTERVAL_BASE * 0.92_f32.powi(wave.0 as i32);
    timer.0 = Timer::from_seconds(new_interval, TimerMode::Repeating);

    // Remove mushrooms in player zone
    for (entity, pos) in &mushroom_q {
        if pos.row >= PLAYER_ZONE_ROW_START {
            mushroom_grid.0.remove(&(pos.col, pos.row));
            commands.entity(entity).despawn();
        }
    }

    spawn_centipede_chain(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut chains_res,
        &mut next_id,
        CENTIPEDE_LENGTH,
        0,
        1,
    );
}
