use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;

use crate::components::HolePhase;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tile {
    Empty,
    Brick,
    Concrete,
    Ladder,
    Bar,
    HiddenLadder,
}

impl Tile {
    pub fn is_solid(self) -> bool {
        matches!(self, Tile::Brick | Tile::Concrete)
    }
}

#[derive(Resource, Clone)]
pub struct LevelGrid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl LevelGrid {
    pub fn get(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return Tile::Concrete;
        }
        self.tiles[y as usize * self.width + x as usize]
    }

    /// Effective tile at (x,y), accounting for open holes turning bricks into empty.
    pub fn effective_tile(&self, x: i32, y: i32, holes: &HoleMap) -> Tile {
        let base = self.get(x, y);
        if base == Tile::Brick
            && let Some(phase) = holes.get(x, y)
        {
            return match phase {
                HolePhase::Open => Tile::Empty,
                HolePhase::Closing => Tile::Brick,
            };
        }
        base
    }

    fn effective_is_solid(&self, x: i32, y: i32, holes: &HoleMap) -> bool {
        self.effective_tile(x, y, holes).is_solid()
    }

    pub fn is_supported(&self, x: i32, y: i32, holes: &HoleMap) -> bool {
        let here = self.effective_tile(x, y, holes);
        if here == Tile::Ladder {
            return true;
        }
        if here == Tile::Bar {
            return true;
        }
        if y == 0 {
            return true;
        }
        let below = self.effective_tile(x, y - 1, holes);
        if below.is_solid() {
            return true;
        }
        if below == Tile::Ladder {
            return true;
        }
        false
    }

    pub fn can_enter(&self, x: i32, y: i32, holes: &HoleMap) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        !self.effective_is_solid(x, y, holes)
    }

    pub fn can_move_horizontal(&self, x: i32, y: i32, dx: i32, holes: &HoleMap) -> bool {
        if dx == 0 || !self.can_enter(x + dx, y, holes) {
            return false;
        }

        let here = self.effective_tile(x, y, holes);
        here == Tile::Ladder || here == Tile::Bar || self.is_supported(x, y, holes)
    }

    pub fn can_climb_up(&self, x: i32, y: i32, holes: &HoleMap) -> bool {
        let here = self.effective_tile(x, y, holes);
        if here != Tile::Ladder {
            return false;
        }
        !self.effective_is_solid(x, y + 1, holes)
    }

    pub fn can_climb_down(&self, x: i32, y: i32, holes: &HoleMap) -> bool {
        if y <= 0 {
            return false;
        }
        let here = self.effective_tile(x, y, holes);
        let below = self.effective_tile(x, y - 1, holes);
        if here == Tile::Ladder {
            return !below.is_solid();
        }
        if below == Tile::Ladder {
            return true;
        }
        if here == Tile::Bar {
            return !below.is_solid();
        }
        false
    }

    /// Convert all HiddenLadder tiles to Ladder (called when exit is unlocked).
    pub fn reveal_hidden_ladders(&mut self) {
        for tile in &mut self.tiles {
            if *tile == Tile::HiddenLadder {
                *tile = Tile::Ladder;
            }
        }
    }

    /// Can the player dig the brick at target (dx direction from player)?
    /// Player is at `pos`, digging toward `pos + (dx, -1)`.
    pub fn can_dig(&self, pos: IVec2, dx: i32, holes: &HoleMap) -> bool {
        let target = IVec2::new(pos.x + dx, pos.y - 1);
        // Target must be a base brick, not already a hole
        if self.get(target.x, target.y) != Tile::Brick {
            return false;
        }
        if holes.has(target.x, target.y) {
            return false;
        }
        // Side cell at player height must be enterable (reach space)
        let side = IVec2::new(pos.x + dx, pos.y);
        if self.effective_is_solid(side.x, side.y, holes) {
            return false;
        }
        true
    }
}

/// Tracks active holes as a separate overlay on the base tile grid.
/// Keeps base `LevelGrid` unchanged for easy level reset.
#[derive(Resource, Default)]
pub struct HoleMap {
    holes: HashMap<(i32, i32), HolePhase>,
}

impl HoleMap {
    pub fn get(&self, x: i32, y: i32) -> Option<HolePhase> {
        self.holes.get(&(x, y)).copied()
    }

    pub fn has(&self, x: i32, y: i32) -> bool {
        self.holes.contains_key(&(x, y))
    }

    pub fn insert(&mut self, x: i32, y: i32, phase: HolePhase) {
        self.holes.insert((x, y), phase);
    }

    pub fn remove(&mut self, x: i32, y: i32) {
        self.holes.remove(&(x, y));
    }
}

/// BFS from `start` toward `goal` on the traversable grid.
/// Returns the first step direction, or None if no path exists.
pub fn bfs_next_step(
    grid: &LevelGrid,
    holes: &HoleMap,
    start: IVec2,
    goal: IVec2,
) -> Option<IVec2> {
    if start == goal {
        return None;
    }

    let mut visited = HashSet::new();
    // (position, first_step) — we track what the first move was
    let mut queue: VecDeque<(IVec2, IVec2)> = VecDeque::new();
    visited.insert(start);

    // Expand neighbors of start
    for neighbor in bfs_neighbors(grid, holes, start) {
        if visited.insert(neighbor) {
            queue.push_back((neighbor, neighbor));
        }
    }

    while let Some((pos, first_step)) = queue.pop_front() {
        if pos == goal {
            return Some(first_step);
        }
        for neighbor in bfs_neighbors(grid, holes, pos) {
            if visited.insert(neighbor) {
                queue.push_back((neighbor, first_step));
            }
        }
    }

    None
}

fn bfs_neighbors(grid: &LevelGrid, holes: &HoleMap, pos: IVec2) -> Vec<IVec2> {
    let mut out = Vec::with_capacity(4);

    // If unsupported, the only option is falling
    if !grid.is_supported(pos.x, pos.y, holes) {
        let below = pos - IVec2::Y;
        if grid.can_enter(below.x, below.y, holes) {
            out.push(below);
        }
        return out;
    }

    // Left / right
    if grid.can_enter(pos.x - 1, pos.y, holes) {
        out.push(pos - IVec2::X);
    }
    if grid.can_enter(pos.x + 1, pos.y, holes) {
        out.push(pos + IVec2::X);
    }
    // Climb up
    if grid.can_climb_up(pos.x, pos.y, holes) {
        out.push(pos + IVec2::Y);
    }
    // Climb down / drop
    if grid.can_climb_down(pos.x, pos.y, holes) {
        out.push(pos - IVec2::Y);
    }

    out
}

pub fn grid_to_world(pos: IVec2, width: usize, height: usize, cell_size: f32) -> Vec2 {
    let offset_x = -(width as f32 * cell_size) * 0.5 + cell_size * 0.5;
    let offset_y = -(height as f32 * cell_size) * 0.5 + cell_size * 0.5;
    Vec2::new(
        offset_x + pos.x as f32 * cell_size,
        offset_y + pos.y as f32 * cell_size,
    )
}

pub struct ParsedLevel {
    pub grid: LevelGrid,
    pub player_spawn: IVec2,
    pub guard_spawns: Vec<IVec2>,
    pub gold_positions: Vec<IVec2>,
    pub hidden_ladder_positions: Vec<IVec2>,
}

pub fn parse_level(source: &str) -> ParsedLevel {
    let lines: Vec<&str> = source.lines().collect();
    assert!(
        !lines.is_empty(),
        "Level source must contain at least one row"
    );

    let height = lines.len();
    let width = lines[0].len();
    assert!(width > 0, "Level rows must not be empty");

    for (i, line) in lines.iter().enumerate() {
        assert_eq!(
            line.len(),
            width,
            "Row {i} has width {}, expected {width}",
            line.len()
        );
    }

    let mut tiles = vec![Tile::Empty; width * height];
    let mut player_spawn = None;
    let mut guard_spawns = Vec::new();
    let mut gold_positions = Vec::new();
    let mut hidden_ladder_positions = Vec::new();

    for (row_from_top, line) in lines.iter().enumerate() {
        let y = height - 1 - row_from_top;
        for (x, ch) in line.chars().enumerate() {
            let tile = match ch {
                '.' => Tile::Empty,
                '#' => Tile::Brick,
                '=' => Tile::Concrete,
                'H' => Tile::Ladder,
                '-' => Tile::Bar,
                '^' => {
                    hidden_ladder_positions.push(IVec2::new(x as i32, y as i32));
                    Tile::HiddenLadder
                }
                '$' => {
                    gold_positions.push(IVec2::new(x as i32, y as i32));
                    Tile::Empty
                }
                'S' => {
                    gold_positions.push(IVec2::new(x as i32, y as i32));
                    Tile::Ladder
                }
                'P' => {
                    assert!(
                        player_spawn.is_none(),
                        "Level must contain exactly one player spawn"
                    );
                    player_spawn = Some(IVec2::new(x as i32, y as i32));
                    Tile::Empty
                }
                'G' => {
                    guard_spawns.push(IVec2::new(x as i32, y as i32));
                    Tile::Empty
                }
                _ => panic!("Unsupported level character '{ch}' at ({x}, {y})"),
            };
            tiles[y * width + x] = tile;
        }
    }

    let player_spawn = player_spawn.expect("Level must contain exactly one player spawn");

    ParsedLevel {
        grid: LevelGrid {
            width,
            height,
            tiles,
        },
        player_spawn,
        guard_spawns,
        gold_positions,
        hidden_ladder_positions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_holes() -> HoleMap {
        HoleMap::default()
    }

    fn test_grid() -> LevelGrid {
        // 5x4 grid:
        // row 3 (top):  ..P..
        // row 2:        .H-..
        // row 1:        ##.##
        // row 0 (bot):  =====
        parse_level(
            "\
..P..\n\
.H-..\n\
##.##\n\
=====",
        )
        .grid
    }

    #[test]
    fn solid_tiles() {
        let g = test_grid();
        assert!(g.get(0, 0).is_solid());
        assert!(g.get(0, 1).is_solid());
        assert!(!g.get(1, 2).is_solid());
        assert!(!g.get(2, 2).is_solid());
        assert!(!g.get(2, 1).is_solid());
    }

    #[test]
    fn out_of_bounds_is_solid() {
        let g = test_grid();
        assert!(g.get(-1, 0).is_solid());
        assert!(g.get(0, -1).is_solid());
        assert!(g.get(5, 0).is_solid());
        assert!(g.get(0, 4).is_solid());
    }

    #[test]
    fn support_on_solid() {
        let g = test_grid();
        let h = empty_holes();
        assert!(g.is_supported(0, 2, &h));
        assert!(g.is_supported(4, 2, &h));
    }

    #[test]
    fn support_on_ladder() {
        let g = test_grid();
        let h = empty_holes();
        assert!(g.is_supported(1, 2, &h));
    }

    #[test]
    fn support_on_bar() {
        let g = test_grid();
        let h = empty_holes();
        assert!(g.is_supported(2, 2, &h));
    }

    #[test]
    fn support_on_floor() {
        let g = test_grid();
        let h = empty_holes();
        assert!(g.is_supported(0, 0, &h));
    }

    #[test]
    fn no_support_in_air() {
        let g = test_grid();
        let h = empty_holes();
        assert!(!g.is_supported(3, 3, &h));
        assert!(!g.is_supported(4, 3, &h));
    }

    #[test]
    fn support_above_ladder() {
        let g = test_grid();
        let h = empty_holes();
        assert!(g.is_supported(1, 3, &h));
    }

    #[test]
    fn can_enter_checks() {
        let g = test_grid();
        let h = empty_holes();
        assert!(g.can_enter(2, 1, &h));
        assert!(g.can_enter(1, 2, &h));
        assert!(g.can_enter(2, 2, &h));
        assert!(!g.can_enter(0, 1, &h));
        assert!(!g.can_enter(0, 0, &h));
        assert!(!g.can_enter(-1, 0, &h));
    }

    #[test]
    fn unsupported_cells_cannot_move_sideways() {
        let g = parse_level(
            "\
.....\n\
.P...\n\
.....\n\
=====",
        )
        .grid;
        let h = empty_holes();
        assert!(!g.can_move_horizontal(1, 2, -1, &h));
        assert!(!g.can_move_horizontal(1, 2, 1, &h));
    }

    #[test]
    fn ladder_cells_can_move_sideways_without_floor() {
        let g = parse_level(
            "\
.....\n\
.P...\n\
.H...\n\
=====",
        )
        .grid;
        let h = empty_holes();
        assert!(g.can_move_horizontal(1, 2, 1, &h));
    }

    #[test]
    fn climb_up_from_ladder() {
        let g = test_grid();
        let h = empty_holes();
        assert!(g.can_climb_up(1, 2, &h));
    }

    #[test]
    fn cannot_climb_up_from_non_ladder() {
        let g = test_grid();
        let h = empty_holes();
        assert!(!g.can_climb_up(0, 2, &h));
    }

    #[test]
    fn climb_down_onto_ladder() {
        let g = test_grid();
        let h = empty_holes();
        assert!(g.can_climb_down(1, 3, &h));
    }

    #[test]
    fn climb_down_on_ladder() {
        let g = test_grid();
        let h = empty_holes();
        assert!(!g.can_climb_down(1, 2, &h));
    }

    #[test]
    fn drop_from_bar() {
        let g = test_grid();
        let h = empty_holes();
        assert!(g.can_climb_down(2, 2, &h));
    }

    #[test]
    fn parse_level_basics() {
        let parsed = parse_level(
            "\
.P.\n\
.$.\n\
===",
        );
        assert_eq!(parsed.grid.width, 3);
        assert_eq!(parsed.grid.height, 3);
        assert_eq!(parsed.player_spawn, IVec2::new(1, 2));
        assert_eq!(parsed.gold_positions, vec![IVec2::new(1, 1)]);
    }

    #[test]
    fn parse_guard_and_gold_on_ladder() {
        let parsed = parse_level(
            "\
.GP\n\
.S.\n\
===",
        );
        assert_eq!(parsed.guard_spawns, vec![IVec2::new(1, 2)]);
        assert_eq!(parsed.gold_positions, vec![IVec2::new(1, 1)]);
        assert_eq!(parsed.grid.get(1, 1), Tile::Ladder);
    }

    // --- Hole-related tests ---

    fn dig_grid() -> LevelGrid {
        // 5x3 grid for dig tests:
        // row 2 (top):  .P...
        // row 1:        #####
        // row 0 (bot):  =====
        parse_level(
            "\
.P...\n\
#####\n\
=====",
        )
        .grid
    }

    #[test]
    fn can_dig_basic() {
        let g = dig_grid();
        let h = empty_holes();
        let player = IVec2::new(1, 2);
        // Dig left: target (0,1) is brick, side (0,2) is empty — ok
        assert!(g.can_dig(player, -1, &h));
        // Dig right: target (2,1) is brick, side (2,2) is empty — ok
        assert!(g.can_dig(player, 1, &h));
    }

    #[test]
    fn cannot_dig_non_brick() {
        // 3x3: player on concrete row
        let g = parse_level(
            "\
.P.\n\
===\n\
===",
        )
        .grid;
        let h = empty_holes();
        let player = IVec2::new(1, 2);
        // Target (0,1) is concrete — cannot dig
        assert!(!g.can_dig(player, -1, &h));
    }

    #[test]
    fn cannot_dig_already_open_hole() {
        let g = dig_grid();
        let mut h = empty_holes();
        h.insert(0, 1, HolePhase::Open);
        let player = IVec2::new(1, 2);
        // Target (0,1) already has a hole
        assert!(!g.can_dig(player, -1, &h));
    }

    #[test]
    fn cannot_dig_when_side_blocked() {
        // Player at (1,2), brick at (0,2) blocks reach
        let g = parse_level(
            "\
#P...\n\
#####\n\
=====",
        )
        .grid;
        let h = empty_holes();
        let player = IVec2::new(1, 2);
        // Side (0,2) is brick — blocked
        assert!(!g.can_dig(player, -1, &h));
    }

    #[test]
    fn hole_removes_support() {
        let g = dig_grid();
        let mut h = empty_holes();
        // Before hole: (1,2) supported by brick at (1,1)
        assert!(g.is_supported(1, 2, &h));
        // Open hole at (1,1)
        h.insert(1, 1, HolePhase::Open);
        // Now (1,2) is not supported — brick became empty
        assert!(!g.is_supported(1, 2, &h));
    }

    #[test]
    fn closing_hole_is_solid() {
        let g = dig_grid();
        let mut h = empty_holes();
        h.insert(1, 1, HolePhase::Closing);
        // Closing hole acts as brick — still solid
        assert!(g.is_supported(1, 2, &h));
        assert!(!g.can_enter(1, 1, &h));
    }

    #[test]
    fn open_hole_is_enterable() {
        let g = dig_grid();
        let mut h = empty_holes();
        h.insert(1, 1, HolePhase::Open);
        assert!(g.can_enter(1, 1, &h));
    }

    // --- Hidden ladder tests ---

    #[test]
    fn parse_hidden_ladders() {
        let parsed = parse_level(
            "\
.^.\n\
P..\n\
===",
        );
        assert_eq!(parsed.hidden_ladder_positions, vec![IVec2::new(1, 2)]);
        assert_eq!(parsed.grid.get(1, 2), Tile::HiddenLadder);
    }

    #[test]
    fn hidden_ladder_not_climbable() {
        let parsed = parse_level(
            "\
.^.\n\
P..\n\
===",
        );
        let h = empty_holes();
        // HiddenLadder is not Ladder, so can't climb up from it
        assert!(!parsed.grid.can_climb_up(1, 1, &h));
    }

    #[test]
    fn reveal_makes_hidden_ladder_climbable() {
        let mut parsed = parse_level(
            "\
..P\n\
.^.\n\
.H.\n\
===",
        );
        let h = empty_holes();
        // (1,2) is HiddenLadder — can't climb up FROM it
        assert!(!parsed.grid.can_climb_up(1, 2, &h));

        parsed.grid.reveal_hidden_ladders();
        assert_eq!(parsed.grid.get(1, 2), Tile::Ladder);
        // Now it's a real ladder — can climb up to (1,3) which is empty
        assert!(parsed.grid.can_climb_up(1, 2, &h));
    }

    // --- BFS tests ---

    #[test]
    fn bfs_simple_horizontal() {
        // .P..G
        // =====
        let g = parse_level(
            "\
.P..G\n\
=====",
        )
        .grid;
        let h = empty_holes();
        let start = IVec2::new(1, 1);
        let goal = IVec2::new(4, 1);
        let step = bfs_next_step(&g, &h, start, goal);
        // Should move right
        assert_eq!(step, Some(IVec2::new(2, 1)));
    }

    #[test]
    fn bfs_climb_to_reach() {
        // ..G
        // .H.
        // PH.
        // ===
        let g = parse_level(
            "\
..G\n\
.H.\n\
PH.\n\
===",
        )
        .grid;
        let h = empty_holes();
        let start = IVec2::new(0, 1);
        let goal = IVec2::new(2, 3);
        let step = bfs_next_step(&g, &h, start, goal);
        // Should move right to reach the ladder
        assert_eq!(step, Some(IVec2::new(1, 1)));
    }

    #[test]
    fn bfs_no_path() {
        // P#G
        // ===
        let g = parse_level(
            "\
P#G\n\
===",
        )
        .grid;
        let h = empty_holes();
        let start = IVec2::new(0, 1);
        let goal = IVec2::new(2, 1);
        assert_eq!(bfs_next_step(&g, &h, start, goal), None);
    }

    #[test]
    fn bfs_same_position() {
        let g = parse_level(
            "\
.P.\n\
===",
        )
        .grid;
        let h = empty_holes();
        let pos = IVec2::new(1, 1);
        assert_eq!(bfs_next_step(&g, &h, pos, pos), None);
    }

    #[test]
    #[should_panic(expected = "Unsupported level character")]
    fn parse_level_rejects_unknown_characters() {
        let _ = parse_level(
            "\
.@.\n\
===",
        );
    }

    #[test]
    #[should_panic(expected = "exactly one player spawn")]
    fn parse_level_requires_a_single_player_spawn() {
        let _ = parse_level(
            "\
P.P\n\
===",
        );
    }
}
