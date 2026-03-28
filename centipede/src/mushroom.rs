use bevy::prelude::*;
use rand::Rng;

use crate::{
    components::{GridPos, Mushroom, Poisoned},
    constants::*,
    resources::MushroomGrid,
    states::AppState,
};

pub struct MushroomPlugin;

impl Plugin for MushroomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_mushrooms);
    }
}

pub fn mushroom_color(hits: u8, poisoned: bool) -> Color {
    if poisoned {
        return Color::srgb(0.7, 0.2, 0.9);
    }
    match hits {
        0 => Color::srgb(0.2, 0.8, 0.2),
        1 => Color::srgb(0.6, 0.8, 0.1),
        2 => Color::srgb(0.9, 0.8, 0.1),
        _ => Color::srgb(0.9, 0.4, 0.1),
    }
}

pub fn spawn_mushrooms(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut grid: ResMut<MushroomGrid>,
) {
    grid.0.clear();
    let mut rng = rand::thread_rng();
    let mesh = meshes.add(Rectangle::new(CELL_SIZE * 0.72, CELL_SIZE * 0.72));

    let mut placed = 0;
    let mut attempts = 0;
    while placed < INITIAL_MUSHROOM_COUNT && attempts < 1000 {
        attempts += 1;
        let col = rng.gen_range(0..GRID_COLS);
        let row = rng.gen_range(1..PLAYER_ZONE_ROW_START); // rows 1–24
        let key = (col, row);
        if grid.0.contains_key(&key) {
            continue;
        }
        let entity = spawn_mushroom_at(&mut commands, &mut materials, &mesh, col, row, 0, false);
        grid.0.insert(key, entity);
        placed += 1;
    }
}

pub fn spawn_mushroom_at(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    mesh: &Handle<Mesh>,
    col: i32,
    row: i32,
    hits: u8,
    poisoned: bool,
) -> Entity {
    let color = mushroom_color(hits, poisoned);
    let mut ec = commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(grid_to_world_x(col), grid_to_world_y(row), 0.0),
        GridPos { col, row },
        Mushroom { hits },
        DespawnOnExit(AppState::Playing),
    ));
    if poisoned {
        ec.insert(Poisoned);
    }
    ec.id()
}
