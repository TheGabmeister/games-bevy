use bevy::prelude::*;

use crate::components::Decoration;
use crate::constants::*;
use crate::level::{tile_to_world, LevelGrid, LEVEL_HEIGHT, LEVEL_WIDTH};
use crate::states::AppState;

pub struct DecorationPlugin;

impl Plugin for DecorationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_decoration_assets);
    }
}

// ── Assets ──

#[derive(Resource)]
pub struct DecorationAssets {
    pub cloud_mesh: Handle<Mesh>,
    pub cloud_mat: Handle<ColorMaterial>,
    pub bush_mesh: Handle<Mesh>,
    pub bush_mat: Handle<ColorMaterial>,
    pub small_hill_mesh: Handle<Mesh>,
    pub big_hill_mesh: Handle<Mesh>,
    pub hill_mat: Handle<ColorMaterial>,
}

fn init_decoration_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(DecorationAssets {
        cloud_mesh: meshes.add(Circle::new(CLOUD_CIRCLE_RADIUS)),
        cloud_mat: materials.add(ColorMaterial::from_color(Color::srgba(1.0, 1.0, 1.0, 0.9))),
        bush_mesh: meshes.add(Circle::new(BUSH_CIRCLE_RADIUS)),
        bush_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.15, 0.55, 0.15))),
        small_hill_mesh: meshes.add(Ellipse::new(SMALL_HILL_WIDTH / 2.0, SMALL_HILL_HEIGHT / 2.0)),
        big_hill_mesh: meshes.add(Ellipse::new(BIG_HILL_WIDTH / 2.0, BIG_HILL_HEIGHT / 2.0)),
        hill_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.1, 0.45, 0.1))),
    });
}

// ── Public spawn function (called from spawn_level) ──

pub fn spawn_decorations(commands: &mut Commands, grid: &LevelGrid, assets: &DecorationAssets) {
    let period = DECORATION_PERIOD;

    for base_col in (0..LEVEL_WIDTH).step_by(period) {
        // ── Hills ──
        spawn_hill(commands, assets, grid, base_col, false);
        spawn_hill(commands, assets, grid, base_col + 16, true);

        // ── Clouds (don't need ground check) ──
        spawn_cloud(commands, assets, base_col + 8, 2, 2);
        spawn_cloud(commands, assets, base_col + 19, 1, 3);
        spawn_cloud(commands, assets, base_col + 36, 3, 2);

        // ── Bushes (need ground at position) ──
        spawn_bush(commands, assets, grid, base_col + 11, 2);
        spawn_bush(commands, assets, grid, base_col + 23, 3);
        spawn_bush(commands, assets, grid, base_col + 41, 2);
    }
}

// ── Helpers ──

fn has_ground(grid: &LevelGrid, col: usize) -> bool {
    col < LEVEL_WIDTH && grid.get_char(col as i32, (LEVEL_HEIGHT - 2) as i32) == '#'
}

fn spawn_cloud(commands: &mut Commands, assets: &DecorationAssets, col: usize, row: usize, count: usize) {
    if col >= LEVEL_WIDTH {
        return;
    }
    let (cx, cy) = tile_to_world(col, row);
    let z = Z_DECORATION + 0.2;
    let spacing = CLOUD_CIRCLE_RADIUS * 1.2;
    let offset = -(count as f32 - 1.0) * spacing / 2.0;

    for i in 0..count {
        commands.spawn((
            Decoration,
            Mesh2d(assets.cloud_mesh.clone()),
            MeshMaterial2d(assets.cloud_mat.clone()),
            Transform::from_xyz(cx + offset + i as f32 * spacing, cy, z),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

fn spawn_bush(commands: &mut Commands, assets: &DecorationAssets, grid: &LevelGrid, col: usize, count: usize) {
    if !has_ground(grid, col) {
        return;
    }
    let (_, ground_y) = tile_to_world(col, LEVEL_HEIGHT - 2);
    let cy = ground_y + TILE_SIZE / 2.0 + BUSH_CIRCLE_RADIUS * 0.5;
    let (cx, _) = tile_to_world(col, 0);
    let z = Z_DECORATION + 0.1;
    let spacing = BUSH_CIRCLE_RADIUS * 1.2;
    let offset = -(count as f32 - 1.0) * spacing / 2.0;

    for i in 0..count {
        commands.spawn((
            Decoration,
            Mesh2d(assets.bush_mesh.clone()),
            MeshMaterial2d(assets.bush_mat.clone()),
            Transform::from_xyz(cx + offset + i as f32 * spacing, cy, z),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

fn spawn_hill(commands: &mut Commands, assets: &DecorationAssets, grid: &LevelGrid, col: usize, big: bool) {
    if col >= LEVEL_WIDTH || !has_ground(grid, col) {
        return;
    }
    let (cx, ground_y) = tile_to_world(col, LEVEL_HEIGHT - 2);
    let half_h = if big { BIG_HILL_HEIGHT } else { SMALL_HILL_HEIGHT } / 2.0;
    let cy = ground_y + TILE_SIZE / 2.0 + half_h * 0.5;
    let mesh = if big {
        assets.big_hill_mesh.clone()
    } else {
        assets.small_hill_mesh.clone()
    };

    commands.spawn((
        Decoration,
        Mesh2d(mesh),
        MeshMaterial2d(assets.hill_mat.clone()),
        Transform::from_xyz(cx, cy, Z_DECORATION),
        DespawnOnExit(AppState::Playing),
    ));
}
