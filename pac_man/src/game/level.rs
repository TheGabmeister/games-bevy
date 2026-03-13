use bevy::prelude::*;

use crate::game::components::{Direction, PelletKind};

#[derive(Resource)]
pub struct LevelLayout {
    pub width: i32,
    pub height: i32,
    walls: Vec<bool>,
    pub pellet_spawns: Vec<(IVec2, PelletKind)>,
    pub player_spawn: IVec2,
    pub ghost_spawns: Vec<IVec2>,
}

impl LevelLayout {
    pub fn from_ascii(rows: &[&str]) -> Self {
        let height = rows.len() as i32;
        let width = rows
            .first()
            .map(|row| row.chars().count() as i32)
            .expect("level must contain at least one row");

        let mut walls = vec![false; (width * height) as usize];
        let mut pellet_spawns = Vec::new();
        let mut player_spawn = None;
        let mut ghost_spawns = Vec::new();

        for (y, row) in rows.iter().enumerate() {
            let row_width = row.chars().count() as i32;
            assert_eq!(row_width, width, "every level row must have the same width");

            for (x, ch) in row.chars().enumerate() {
                let tile = IVec2::new(x as i32, y as i32);
                let index = (y as i32 * width + x as i32) as usize;

                match ch {
                    '#' => walls[index] = true,
                    '.' => pellet_spawns.push((tile, PelletKind::Dot)),
                    'o' => pellet_spawns.push((tile, PelletKind::Power)),
                    'P' => player_spawn = Some(tile),
                    'G' => ghost_spawns.push(tile),
                    '_' | ' ' => {}
                    other => panic!("unsupported level marker: {other}"),
                }
            }
        }

        Self {
            width,
            height,
            walls,
            pellet_spawns,
            player_spawn: player_spawn.expect("level must contain a player spawn"),
            ghost_spawns,
        }
    }

    pub fn pellets_total(&self) -> usize {
        self.pellet_spawns.len()
    }

    pub fn tile_to_world(&self, tile: IVec2) -> Vec2 {
        let half_width = (self.width - 1) as f32 / 2.0;
        let half_height = (self.height - 1) as f32 / 2.0;

        Vec2::new(
            (tile.x as f32 - half_width) * crate::game::constants::TILE_SIZE,
            (half_height - tile.y as f32) * crate::game::constants::TILE_SIZE,
        )
    }

    pub fn world_to_tile(&self, position: Vec2) -> IVec2 {
        let half_width = (self.width - 1) as f32 / 2.0;
        let half_height = (self.height - 1) as f32 / 2.0;

        IVec2::new(
            (position.x / crate::game::constants::TILE_SIZE + half_width).round() as i32,
            (half_height - position.y / crate::game::constants::TILE_SIZE).round() as i32,
        )
    }

    pub fn in_bounds(&self, tile: IVec2) -> bool {
        tile.x >= 0 && tile.x < self.width && tile.y >= 0 && tile.y < self.height
    }

    pub fn is_wall(&self, tile: IVec2) -> bool {
        if !self.in_bounds(tile) {
            return true;
        }

        self.walls[(tile.y * self.width + tile.x) as usize]
    }

    pub fn is_walkable(&self, tile: IVec2) -> bool {
        self.in_bounds(tile) && !self.is_wall(tile)
    }

    pub fn can_move(&self, tile: IVec2, direction: Direction) -> bool {
        let next_tile = tile + direction.ivec2();

        if self.in_bounds(next_tile) {
            return !self.is_wall(next_tile);
        }

        direction.is_horizontal() && self.row_has_tunnel(tile.y)
    }

    pub fn row_has_tunnel(&self, row: i32) -> bool {
        if row < 0 || row >= self.height {
            return false;
        }

        self.is_walkable(IVec2::new(0, row)) && self.is_walkable(IVec2::new(self.width - 1, row))
    }

    pub fn wrap_translation(&self, translation: &mut Vec3, row: i32) {
        if !self.row_has_tunnel(row) {
            return;
        }

        let left_edge =
            self.tile_to_world(IVec2::new(0, row)).x - crate::game::constants::TILE_SIZE / 2.0;
        let right_edge = self.tile_to_world(IVec2::new(self.width - 1, row)).x
            + crate::game::constants::TILE_SIZE / 2.0;

        if translation.x < left_edge {
            translation.x = right_edge;
        } else if translation.x > right_edge {
            translation.x = left_edge;
        }
    }

    pub fn clamp_target(&self, tile: IVec2) -> IVec2 {
        IVec2::new(
            tile.x.clamp(0, self.width - 1),
            tile.y.clamp(0, self.height - 1),
        )
    }
}
