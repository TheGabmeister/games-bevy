use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Generate terrain points
    let mut points = Vec::with_capacity(TERRAIN_SEGMENTS + 1);
    for i in 0..=TERRAIN_SEGMENTS {
        let x = (i as f32 / TERRAIN_SEGMENTS as f32) * WORLD_WIDTH;
        let freq1 = 2.0 * std::f32::consts::PI * 3.0 / WORLD_WIDTH;
        let freq2 = 2.0 * std::f32::consts::PI * 7.0 / WORLD_WIDTH;
        let freq3 = 2.0 * std::f32::consts::PI * 13.0 / WORLD_WIDTH;
        let y = GROUND_Y
            + TERRAIN_AMPLITUDE_1 * (x * freq1).sin()
            + TERRAIN_AMPLITUDE_2 * (x * freq2 + 1.0).sin()
            + TERRAIN_AMPLITUDE_3 * (x * freq3 + 2.5).sin();
        points.push(Vec2::new(x, y));
    }

    commands.insert_resource(TerrainData {
        points: points.clone(),
    });

    // Split terrain into chunks for proper world wrapping
    let chunk_width = WORLD_WIDTH / TERRAIN_CHUNKS as f32;
    let points_per_chunk = TERRAIN_SEGMENTS / TERRAIN_CHUNKS;

    let material = materials.add(ColorMaterial::from_color(COLOR_TERRAIN));

    for chunk_idx in 0..TERRAIN_CHUNKS {
        let start = chunk_idx * points_per_chunk;
        let end = (start + points_per_chunk).min(TERRAIN_SEGMENTS);

        let chunk_center_x = (chunk_idx as f32 + 0.5) * chunk_width;
        let chunk_points: Vec<Vec2> = (start..=end)
            .map(|i| {
                let p = points[i];
                Vec2::new(p.x - chunk_center_x, p.y)
            })
            .collect();

        let mesh = build_terrain_chunk_mesh(&chunk_points);
        commands.spawn((
            TerrainChunk,
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
            WorldPosition(chunk_center_x),
        ));
    }
}

fn build_terrain_chunk_mesh(points: &[Vec2]) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for (i, pair) in points.windows(2).enumerate() {
        let base = (i as u32) * 4;
        positions.push([pair[0].x, pair[0].y, 0.0]);
        positions.push([pair[1].x, pair[1].y, 0.0]);
        positions.push([pair[1].x, TERRAIN_BOTTOM_Y, 0.0]);
        positions.push([pair[0].x, TERRAIN_BOTTOM_Y, 0.0]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn get_terrain_y_at(terrain: &TerrainData, world_x: f32) -> f32 {
    let x = ((world_x % WORLD_WIDTH) + WORLD_WIDTH) % WORLD_WIDTH;
    let segment_width = WORLD_WIDTH / TERRAIN_SEGMENTS as f32;
    let idx = (x / segment_width) as usize;
    let idx = idx.min(TERRAIN_SEGMENTS - 1);
    let next = idx + 1;
    let t = (x - terrain.points[idx].x) / segment_width;
    terrain.points[idx].y * (1.0 - t) + terrain.points[next].y * t
}
