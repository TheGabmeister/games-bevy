use bevy::prelude::*;

use crate::constants::*;

#[derive(Resource)]
pub struct RenderAssets {
    pub tile_mesh: Handle<Mesh>,
    pub bar_mesh: Handle<Mesh>,
    pub brick_material: Handle<ColorMaterial>,
    pub concrete_material: Handle<ColorMaterial>,
    pub ladder_material: Handle<ColorMaterial>,
    pub bar_material: Handle<ColorMaterial>,
    pub hidden_ladder_material: Handle<ColorMaterial>,
    pub gold_mesh: Handle<Mesh>,
    pub gold_material: Handle<ColorMaterial>,
    pub player_mesh: Handle<Mesh>,
    pub player_material: Handle<ColorMaterial>,
    pub guard_mesh: Handle<Mesh>,
    pub guard_material: Handle<ColorMaterial>,
    pub hole_mesh: Handle<Mesh>,
    pub hole_material: Handle<ColorMaterial>,
}

pub fn setup_render_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tile_size = CELL_SIZE - 2.0;
    commands.insert_resource(RenderAssets {
        tile_mesh: meshes.add(Rectangle::new(tile_size, tile_size)),
        bar_mesh: meshes.add(Rectangle::new(tile_size, 4.0)),
        brick_material: materials.add(ColorMaterial::from_color(COLOR_BRICK)),
        concrete_material: materials.add(ColorMaterial::from_color(COLOR_CONCRETE)),
        ladder_material: materials.add(ColorMaterial::from_color(COLOR_LADDER)),
        bar_material: materials.add(ColorMaterial::from_color(COLOR_BAR)),
        hidden_ladder_material: materials.add(ColorMaterial::from_color(COLOR_HIDDEN_LADDER)),
        gold_mesh: meshes.add(Circle::new(CELL_SIZE * 0.25)),
        gold_material: materials.add(ColorMaterial::from_color(COLOR_GOLD)),
        player_mesh: meshes.add(Rectangle::new(CELL_SIZE * 0.7, CELL_SIZE * 0.9)),
        player_material: materials.add(ColorMaterial::from_color(COLOR_PLAYER)),
        guard_mesh: meshes.add(Rectangle::new(CELL_SIZE * 0.7, CELL_SIZE * 0.9)),
        guard_material: materials.add(ColorMaterial::from_color(COLOR_GUARD)),
        hole_mesh: meshes.add(Rectangle::new(tile_size, tile_size)),
        hole_material: materials.add(ColorMaterial::from_color(COLOR_BACKGROUND)),
    });
}
