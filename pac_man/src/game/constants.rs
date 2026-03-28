use bevy::prelude::*;

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;
pub const TILE_SIZE: f32 = 28.0;
pub const PLAYER_SPEED: f32 = 108.0;
pub const GHOST_SPEED: f32 = 92.0;
pub const GHOST_FRIGHTENED_SPEED: f32 = 70.0;
pub const GHOST_EATEN_SPEED: f32 = 138.0;
pub const PACMAN_RADIUS: f32 = 11.0;
pub const GHOST_RADIUS: f32 = 11.0;
pub const PELLET_RADIUS: f32 = 2.4;
pub const POWER_PELLET_RADIUS: f32 = 5.5;
pub const TURN_TOLERANCE: f32 = 3.0;
pub const GHOST_COLLISION_RADIUS: f32 = 18.0;
pub const READY_DURATION: f32 = 1.5;
pub const FRIGHTENED_DURATION: f32 = 6.0;
pub const SCATTER_DURATION: f32 = 6.0;
pub const CHASE_DURATION: f32 = 14.0;
pub const PLAYER_SCORE_PELLET: u32 = 10;
pub const PLAYER_SCORE_POWER: u32 = 50;
pub const PLAYER_SCORE_GHOST: u32 = 200;
pub const PLAYER_LIVES: u8 = 3;
pub const WALL_Z: f32 = 0.0;
pub const PELLET_Z: f32 = 1.0;
pub const ACTOR_Z: f32 = 2.0;
pub const DETAIL_Z: f32 = 3.0;

pub const BACKGROUND_COLOR: Color = Color::srgb(0.03, 0.03, 0.06);
pub const WALL_COLOR: Color = Color::srgb(0.08, 0.25, 0.95);
pub const PELLET_COLOR: Color = Color::srgb(0.97, 0.87, 0.72);
pub const PACMAN_COLOR: Color = Color::srgb(1.0, 0.91, 0.08);
pub const BLINKY_COLOR: Color = Color::srgb(0.97, 0.16, 0.21);
pub const PINKY_COLOR: Color = Color::srgb(1.0, 0.65, 0.78);
pub const INKY_COLOR: Color = Color::srgb(0.24, 0.93, 1.0);
pub const CLYDE_COLOR: Color = Color::srgb(1.0, 0.67, 0.19);
pub const FRIGHTENED_COLOR: Color = Color::srgb(0.16, 0.33, 0.96);
pub const EYE_WHITE_COLOR: Color = Color::srgb(0.98, 0.98, 1.0);
pub const EYE_PUPIL_COLOR: Color = Color::srgb(0.1, 0.24, 0.95);
pub const HUD_COLOR: Color = Color::srgb(0.94, 0.95, 1.0);
pub const MESSAGE_COLOR: Color = Color::srgb(1.0, 0.92, 0.2);

pub const LEVEL_MAP: [&str; 21] = [
    "#####################",
    "#o........#........o#",
    "#.####.##.#.##.####.#",
    "#...................#",
    "#.####.#.#####.#.####",
    "#......#...#...#....#",
    "######.###.#.###.####",
    "_____#.#.....#.#_____",
    "######.#.###.#.######",
    "#........#.#........#",
    "#.####.#_GG__#.####.#",
    "#.####.#_____#.####.#",
    "#......___P___......#",
    "#.####.#_____#.####.#",
    "#o...#.#__G__#.#...o#",
    "###.#.#.#####.#.#.###",
    "#...#...........#...#",
    "#.#####.##.#.##.#####",
    "#.......#..G..#.....#",
    "#...........#.......#",
    "#####################",
];
