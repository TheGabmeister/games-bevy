use bevy::prelude::*;

use crate::{
    components::{
        CentipedeDir, CentipedeHead, CentipedeSegment, GridPos, Mushroom, PoisonRushing, Poisoned,
    },
    constants::*,
    resources::{CentipedeChains, CentipedeTimer, MushroomGrid, NextChainId, Wave},
    scheduling::GameplaySet,
    states::AppState,
};

pub struct CentipedePlugin;

impl Plugin for CentipedePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_initial_centipede)
            .add_systems(
                Update,
                (
                    centipede_tick.in_set(GameplaySet::Movement),
                    check_wave_clear.in_set(GameplaySet::Cleanup),
                ),
            );
    }
}

struct ChainAssets<'a> {
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<ColorMaterial>,
}

struct ChainState<'a> {
    chains: &'a mut CentipedeChains,
    next_id: &'a mut NextChainId,
}

struct ChainSpawn {
    length: usize,
    start_row: i32,
    dx: i32,
}

pub fn spawn_initial_centipede(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut chains: ResMut<CentipedeChains>,
    mut next_id: ResMut<NextChainId>,
    _wave: Res<Wave>,
) {
    chains.0.clear();

    let mut assets = ChainAssets {
        meshes: &mut meshes,
        materials: &mut materials,
    };
    let mut state = ChainState {
        chains: &mut chains,
        next_id: &mut next_id,
    };

    spawn_centipede_chain(
        &mut commands,
        &mut assets,
        &mut state,
        ChainSpawn {
            length: CENTIPEDE_LENGTH,
            start_row: 0,
            dx: 1,
        },
    );
}

fn spawn_centipede_chain(
    commands: &mut Commands,
    assets: &mut ChainAssets,
    state: &mut ChainState,
    spawn: ChainSpawn,
) {
    if spawn.length == 0 {
        return;
    }

    let chain_id = state.next_id.next();
    let head_mesh = assets.meshes.add(Circle::new(CELL_SIZE * 0.45));
    let body_mesh = assets
        .meshes
        .add(Rectangle::new(CELL_SIZE * 0.75, CELL_SIZE * 0.75));
    let head_mat = assets.materials.add(Color::srgb(1.0, 0.3, 0.1));
    let body_mat = assets.materials.add(Color::srgb(0.9, 0.6, 0.1));

    let mut entities = Vec::with_capacity(spawn.length);

    for i in 0..spawn.length {
        let col = if spawn.dx > 0 {
            (spawn.length - 1 - i) as i32
        } else {
            i as i32
        }
        .clamp(0, GRID_COLS - 1);

        let is_head = i == 0;
        let (mesh, material, z) = if is_head {
            (head_mesh.clone(), head_mat.clone(), 2.0)
        } else {
            (body_mesh.clone(), body_mat.clone(), 1.5)
        };

        let mut entity = commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(grid_to_world_x(col), grid_to_world_y(spawn.start_row), z),
            GridPos {
                col,
                row: spawn.start_row,
            },
            CentipedeSegment { chain_id, index: i },
            DespawnOnExit(AppState::Playing),
        ));

        if is_head {
            entity.insert((CentipedeHead, CentipedeDir { dx: spawn.dx }));
        }

        entities.push(entity.id());
    }

    state.chains.0.insert(chain_id, entities);
}

#[allow(clippy::too_many_arguments)]
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
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut next_id: ResMut<NextChainId>,
    poisoned_mushrooms: Query<&GridPos, (With<Mushroom>, With<Poisoned>)>,
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

        let mut positions = Vec::with_capacity(entity_list.len());
        let mut is_rushing = false;
        let mut head_dx = 1;

        for &entity in &entity_list {
            if let Ok((pos, _, dir, rushing)) = segment_q.get(entity) {
                if let Some(dir) = dir {
                    head_dx = dir.dx;
                }
                if rushing.is_some() {
                    is_rushing = true;
                }
                positions.push(*pos);
            }
        }

        if positions.is_empty() {
            continue;
        }

        let (new_head_col, new_head_row, new_dx) =
            compute_head_move(positions[0], head_dx, is_rushing, &mushroom_grid);
        let wrapped = new_head_row >= GRID_ROWS;

        if wrapped {
            for &entity in &entity_list {
                commands.entity(entity).despawn();
            }
            chains.0.remove(&chain_id);

            let mut assets = ChainAssets {
                meshes: &mut meshes,
                materials: &mut materials,
            };
            let mut state = ChainState {
                chains: &mut chains,
                next_id: &mut next_id,
            };

            spawn_centipede_chain(
                &mut commands,
                &mut assets,
                &mut state,
                ChainSpawn {
                    length: entity_list.len(),
                    start_row: 0,
                    dx: new_dx,
                },
            );
            continue;
        }

        let mut new_positions = vec![
            GridPos {
                col: new_head_col,
                row: new_head_row,
            };
            entity_list.len()
        ];
        if entity_list.len() > 1 {
            new_positions[1..entity_list.len()]
                .copy_from_slice(&positions[..(entity_list.len() - 1)]);
        }

        for (i, &entity) in entity_list.iter().enumerate() {
            if let Ok((mut pos, mut transform, mut dir, _)) = segment_q.get_mut(entity) {
                *pos = new_positions[i];
                transform.translation.x = grid_to_world_x(pos.col);
                transform.translation.y = grid_to_world_y(pos.row);
                if i == 0
                    && let Some(ref mut dir) = dir
                {
                    dir.dx = new_dx;
                }
            }
        }

        if !is_rushing
            && poisoned_mushrooms
                .iter()
                .any(|pos| *pos == new_positions[0])
        {
            commands.entity(entity_list[0]).insert(PoisonRushing);
        }
    }
}

fn compute_head_move(
    pos: GridPos,
    dx: i32,
    rushing: bool,
    mushroom_grid: &MushroomGrid,
) -> (i32, i32, i32) {
    if rushing {
        return (pos.col, pos.row + 1, dx);
    }

    let next_col = pos.col + dx;
    let blocked =
        !(0..GRID_COLS).contains(&next_col) || mushroom_grid.0.contains_key(&(next_col, pos.row));

    if blocked {
        (pos.col, pos.row + 1, -dx)
    } else {
        (next_col, pos.row, dx)
    }
}

#[allow(clippy::too_many_arguments)]
fn check_wave_clear(
    mut commands: Commands,
    mut chains: ResMut<CentipedeChains>,
    mut wave: ResMut<Wave>,
    mut timer: ResMut<CentipedeTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut next_id: ResMut<NextChainId>,
    mushroom_q: Query<(Entity, &GridPos), With<Mushroom>>,
    mut mushroom_grid: ResMut<MushroomGrid>,
) {
    if !chains.0.is_empty() {
        return;
    }

    wave.0 += 1;
    let new_interval = CENTIPEDE_INTERVAL_BASE * 0.92_f32.powi(wave.0 as i32);
    timer.0 = Timer::from_seconds(new_interval, TimerMode::Repeating);

    for (entity, pos) in &mushroom_q {
        if pos.row >= PLAYER_ZONE_ROW_START {
            mushroom_grid.0.remove(&(pos.col, pos.row));
            commands.entity(entity).despawn();
        }
    }

    let mut assets = ChainAssets {
        meshes: &mut meshes,
        materials: &mut materials,
    };
    let mut state = ChainState {
        chains: &mut chains,
        next_id: &mut next_id,
    };

    spawn_centipede_chain(
        &mut commands,
        &mut assets,
        &mut state,
        ChainSpawn {
            length: CENTIPEDE_LENGTH,
            start_row: 0,
            dx: 1,
        },
    );
}
