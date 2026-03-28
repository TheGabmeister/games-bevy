use bevy::prelude::*;

use crate::components::KeyColor;

pub const ROOM_W: f32 = 800.0;
pub const ROOM_H: f32 = 600.0;
pub const WALL_T: f32 = 16.0;
pub const PASSAGE_W: f32 = 64.0;
pub const PLAYER_HW: f32 = 6.0;
pub const PLAYER_HH: f32 = 6.0;
pub const PLAYER_SPEED: f32 = 150.0;
pub const PASSAGE_THRESHOLD: f32 = WALL_T + 4.0;

#[derive(Debug, Clone, Copy)]
pub struct WallRect {
    pub x: f32,
    pub y: f32,
    pub hw: f32,
    pub hh: f32,
}

impl WallRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x,
            y,
            hw: w / 2.0,
            hh: h / 2.0,
        }
    }

    pub fn overlaps(&self, px: f32, py: f32, phw: f32, phh: f32) -> bool {
        (px - self.x).abs() < self.hw + phw && (py - self.y).abs() < self.hh + phh
    }
}

pub struct RoomDef {
    pub name: &'static str,
    pub color: Color,
    pub exits: [Option<u8>; 4],
    pub gates: [Option<KeyColor>; 4],
    pub interior_walls: Vec<[f32; 4]>,
}

#[derive(Resource)]
pub struct WorldMap {
    pub rooms: Vec<RoomDef>,
}

impl Default for WorldMap {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldMap {
    pub fn new() -> Self {
        let rooms = vec![
            RoomDef {
                name: "GOLDEN CASTLE",
                color: Color::srgb(0.45, 0.38, 0.05),
                exits: [None, Some(1), None, None],
                gates: [None, Some(KeyColor::Gold), None, None],
                interior_walls: vec![[-200.0, 0.0, 40.0, 200.0], [200.0, 0.0, 40.0, 200.0]],
            },
            RoomDef {
                name: "ANTECHAMBER",
                color: Color::srgb(0.35, 0.35, 0.35),
                exits: [Some(0), Some(3), Some(2), Some(6)],
                gates: [None, None, None, None],
                interior_walls: vec![],
            },
            RoomDef {
                name: "OPEN PLAINS",
                color: Color::srgb(0.15, 0.45, 0.1),
                exits: [None, Some(5), Some(11), Some(1)],
                gates: [None, None, None, None],
                interior_walls: vec![[0.0, 80.0, 200.0, 20.0], [0.0, -80.0, 200.0, 20.0]],
            },
            RoomDef {
                name: "DARK WOODS",
                color: Color::srgb(0.05, 0.2, 0.05),
                exits: [Some(1), Some(4), Some(12), None],
                gates: [None, None, None, None],
                interior_walls: vec![
                    [-150.0, 50.0, 30.0, 120.0],
                    [50.0, -50.0, 30.0, 120.0],
                    [150.0, 100.0, 30.0, 80.0],
                ],
            },
            RoomDef {
                name: "RED COURTYARD",
                color: Color::srgb(0.4, 0.2, 0.05),
                exits: [Some(3), None, Some(5), None],
                gates: [None, None, Some(KeyColor::Red), None],
                interior_walls: vec![[-100.0, 0.0, 20.0, 150.0], [100.0, 0.0, 20.0, 150.0]],
            },
            RoomDef {
                name: "RED CASTLE",
                color: Color::srgb(0.3, 0.05, 0.05),
                exits: [None, None, None, Some(4)],
                gates: [None, None, None, None],
                interior_walls: vec![[-180.0, 0.0, 40.0, 180.0], [180.0, 0.0, 40.0, 180.0]],
            },
            RoomDef {
                name: "CRYSTAL CAVE",
                color: Color::srgb(0.1, 0.3, 0.45),
                exits: [None, Some(7), Some(1), Some(12)],
                gates: [None, None, None, None],
                interior_walls: vec![[0.0, 0.0, 20.0, 150.0]],
            },
            RoomDef {
                name: "BLUE MAZE",
                color: Color::srgb(0.05, 0.1, 0.4),
                exits: [Some(6), None, Some(8), None],
                gates: [None, None, None, None],
                interior_walls: vec![
                    [-100.0, 80.0, 160.0, 20.0],
                    [80.0, -40.0, 160.0, 20.0],
                    [-80.0, -120.0, 160.0, 20.0],
                ],
            },
            RoomDef {
                name: "UNDERGROUND VAULT",
                color: Color::srgb(0.15, 0.15, 0.15),
                exits: [None, Some(9), None, Some(7)],
                gates: [None, None, None, None],
                interior_walls: vec![[0.0, 60.0, 200.0, 20.0]],
            },
            RoomDef {
                name: "BLACK COURTYARD",
                color: Color::srgb(0.15, 0.18, 0.22),
                exits: [Some(8), Some(10), None, None],
                gates: [None, Some(KeyColor::Blue), None, None],
                interior_walls: vec![[-120.0, 0.0, 20.0, 120.0], [120.0, 0.0, 20.0, 120.0]],
            },
            RoomDef {
                name: "BLACK CASTLE",
                color: Color::srgb(0.04, 0.04, 0.06),
                exits: [Some(9), None, None, None],
                gates: [None, None, None, None],
                interior_walls: vec![[-160.0, 0.0, 40.0, 200.0], [160.0, 0.0, 40.0, 200.0]],
            },
            RoomDef {
                name: "OPEN SEA",
                color: Color::srgb(0.0, 0.45, 0.5),
                exits: [None, None, None, Some(2)],
                gates: [None, None, None, None],
                interior_walls: vec![],
            },
            RoomDef {
                name: "HIDDEN PASSAGE",
                color: Color::srgb(0.2, 0.05, 0.3),
                exits: [None, Some(3), Some(6), None],
                gates: [None, None, None, None],
                interior_walls: vec![[0.0, 0.0, 20.0, 120.0]],
            },
            RoomDef {
                name: "CREATED BY WARREN ROBINETT",
                color: Color::srgb(0.8, 0.7, 0.2),
                exits: [None, Some(6), None, None],
                gates: [None, None, None, None],
                interior_walls: vec![],
            },
        ];

        Self { rooms }
    }

    pub fn room(&self, id: u8) -> &RoomDef {
        &self.rooms[id as usize]
    }

    pub fn room_count(&self) -> u8 {
        self.rooms.len() as u8
    }
}

pub fn build_room_walls(room: &RoomDef) -> Vec<WallRect> {
    let mut walls = Vec::new();

    let half_w = ROOM_W / 2.0;
    let half_h = ROOM_H / 2.0;
    let half_t = WALL_T / 2.0;
    let half_p = PASSAGE_W / 2.0;

    let top_y = half_h - half_t;
    if room.exits[0].is_none() {
        walls.push(WallRect::new(0.0, top_y, ROOM_W, WALL_T));
    } else {
        let lw = half_w - half_p;
        walls.push(WallRect::new(-(half_p + lw / 2.0), top_y, lw, WALL_T));
        walls.push(WallRect::new(half_p + lw / 2.0, top_y, lw, WALL_T));
    }

    let bot_y = -(half_h - half_t);
    if room.exits[1].is_none() {
        walls.push(WallRect::new(0.0, bot_y, ROOM_W, WALL_T));
    } else {
        let lw = half_w - half_p;
        walls.push(WallRect::new(-(half_p + lw / 2.0), bot_y, lw, WALL_T));
        walls.push(WallRect::new(half_p + lw / 2.0, bot_y, lw, WALL_T));
    }

    let right_x = half_w - half_t;
    if room.exits[2].is_none() {
        walls.push(WallRect::new(right_x, 0.0, WALL_T, ROOM_H));
    } else {
        let lh = half_h - half_p;
        walls.push(WallRect::new(right_x, -(half_p + lh / 2.0), WALL_T, lh));
        walls.push(WallRect::new(right_x, half_p + lh / 2.0, WALL_T, lh));
    }

    let left_x = -(half_w - half_t);
    if room.exits[3].is_none() {
        walls.push(WallRect::new(left_x, 0.0, WALL_T, ROOM_H));
    } else {
        let lh = half_h - half_p;
        walls.push(WallRect::new(left_x, -(half_p + lh / 2.0), WALL_T, lh));
        walls.push(WallRect::new(left_x, half_p + lh / 2.0, WALL_T, lh));
    }

    for &[x, y, w, h] in &room.interior_walls {
        walls.push(WallRect::new(x, y, w, h));
    }

    walls
}

#[derive(Resource)]
pub struct CurrentRoom(pub u8);

impl Default for CurrentRoom {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Resource, Default)]
pub struct PlayerInventory {
    pub item: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct RoomWalls(pub Vec<WallRect>);

#[derive(Resource)]
pub struct DeadDragonMaterial(pub Handle<ColorMaterial>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ExitDir;

    #[test]
    fn gate_definitions_only_exist_on_real_exits() {
        let world = WorldMap::new();

        for room in &world.rooms {
            for direction in ExitDir::ALL {
                let index = direction.index();
                if room.gates[index].is_some() {
                    assert!(
                        room.exits[index].is_some(),
                        "gate defined on missing exit in room {} direction {}",
                        room.name,
                        index
                    );
                }
            }
        }
    }

    #[test]
    fn room_exits_stay_within_bounds() {
        let world = WorldMap::new();
        let room_count = world.room_count();

        for room in &world.rooms {
            for exit in room.exits.into_iter().flatten() {
                assert!(
                    exit < room_count,
                    "room {} points to invalid room {}",
                    room.name,
                    exit
                );
            }
        }
    }

    #[test]
    fn antechamber_walls_split_for_each_exit() {
        let world = WorldMap::new();
        let walls = build_room_walls(world.room(1));

        assert_eq!(walls.len(), 8);
    }

    #[test]
    fn castle_room_has_boundary_and_interior_walls() {
        let world = WorldMap::new();
        let walls = build_room_walls(world.room(0));

        assert_eq!(walls.len(), 7);
        assert!(walls.iter().any(|wall| wall.x == -200.0 && wall.hw == 20.0));
        assert!(walls.iter().any(|wall| wall.x == 200.0 && wall.hw == 20.0));
    }
}
