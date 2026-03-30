use bevy::prelude::Vec3;

use crate::components::Velocity;
use crate::constants::TILE_SIZE;
use crate::level::{LevelGrid, tile_to_world, world_to_col, world_to_row};

/// AABB overlap test between two axis-aligned boxes defined by center position
/// and half-extents. Returns `Some((overlap_x, overlap_y))` when overlapping.
pub fn aabb_overlap(
    ax: f32,
    ay: f32,
    a_half_w: f32,
    a_half_h: f32,
    bx: f32,
    by: f32,
    b_half_w: f32,
    b_half_h: f32,
) -> Option<(f32, f32)> {
    let ox = (a_half_w + b_half_w) - (ax - bx).abs();
    let oy = (a_half_h + b_half_h) - (ay - by).abs();
    if ox > 0.0 && oy > 0.0 {
        Some((ox, oy))
    } else {
        None
    }
}

/// Result of resolving tile collisions for a single entity.
pub struct TileCollisionResult {
    /// The hittable block closest to the entity center that was hit from below,
    /// if any: `(col, row)`.
    pub head_hit: Option<(i32, i32)>,
    /// Whether the entity is standing on solid ground.
    pub grounded: bool,
}

/// Horizontal wall-hit callback action.
pub enum WallAction {
    /// Zero velocity (used by player).
    Stop,
    /// Reverse the supplied direction value (used by enemies/shells).
    Reverse,
}

/// Resolve AABB-vs-tile-grid collisions for one entity.
///
/// * Pushes the entity out of overlapping solid tiles.
/// * Returns head-hit and grounded information.
/// * `wall_action` controls what happens on a horizontal collision.
pub fn resolve_tile_collisions(
    level: &LevelGrid,
    pos: &mut Vec3,
    vel: &mut Velocity,
    half_w: f32,
    half_h: f32,
    wall_action: WallAction,
    direction: &mut f32,
) -> TileCollisionResult {
    let tile_half = TILE_SIZE / 2.0;

    let col_min = world_to_col(pos.x - half_w) - 1;
    let col_max = world_to_col(pos.x + half_w) + 1;
    let row_min = world_to_row(pos.y + half_h) - 1;
    let row_max = world_to_row(pos.y - half_h) + 1;

    let mut best_head_hit: Option<(i32, i32, f32)> = None;

    for row in row_min..=row_max {
        for col in col_min..=col_max {
            if !level.is_solid(col, row) {
                continue;
            }

            let (tile_cx, tile_cy) = tile_to_world(col as usize, row as usize);

            let overlap_x = (half_w + tile_half) - (pos.x - tile_cx).abs();
            let overlap_y = (half_h + tile_half) - (pos.y - tile_cy).abs();

            if overlap_x <= 0.0 || overlap_y <= 0.0 {
                continue;
            }

            if overlap_y < overlap_x {
                if pos.y > tile_cy {
                    // Landing on top
                    pos.y += overlap_y;
                    if vel.y < 0.0 {
                        vel.y = 0.0;
                    }
                } else {
                    // Head hit
                    pos.y -= overlap_y;
                    if vel.y > 0.0 {
                        vel.y = 0.0;

                        if level.is_hittable(col, row) {
                            let dist = (pos.x - tile_cx).abs();
                            if best_head_hit.is_none() || dist < best_head_hit.unwrap().2 {
                                best_head_hit = Some((col, row, dist));
                            }
                        }
                    }
                }
            } else {
                // Horizontal push-out
                if pos.x > tile_cx {
                    pos.x += overlap_x;
                } else {
                    pos.x -= overlap_x;
                }
                match wall_action {
                    WallAction::Stop => {}
                    WallAction::Reverse => *direction = -*direction,
                }
                vel.x = 0.0;
            }
        }
    }

    // Grounded probe (1 pixel below feet)
    let probe_y = pos.y - half_h - 1.0;
    let probe_row = world_to_row(probe_y);
    let left_col = world_to_col(pos.x - half_w + 1.0);
    let right_col = world_to_col(pos.x + half_w - 1.0);
    let grounded = level.is_solid(left_col, probe_row) || level.is_solid(right_col, probe_row);

    TileCollisionResult {
        head_hit: best_head_hit.map(|(c, r, _)| (c, r)),
        grounded,
    }
}
