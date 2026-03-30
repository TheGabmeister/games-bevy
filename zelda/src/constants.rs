use bevy::prelude::*;

pub const ROOM_WIDTH: f32 = 256.0;
pub const ROOM_HEIGHT: f32 = 176.0;
pub const HUD_HEIGHT: f32 = 64.0;
pub const LOGICAL_SCREEN_WIDTH: f32 = ROOM_WIDTH;
pub const LOGICAL_SCREEN_HEIGHT: f32 = ROOM_HEIGHT + HUD_HEIGHT;
pub const COLLISION_UNIT_SIZE: f32 = 8.0;

pub const WINDOW_SCALE: u32 = 4;
pub const WINDOW_WIDTH: u32 = LOGICAL_SCREEN_WIDTH as u32 * WINDOW_SCALE;
pub const WINDOW_HEIGHT: u32 = LOGICAL_SCREEN_HEIGHT as u32 * WINDOW_SCALE;

pub const ROOM_HALF_WIDTH: f32 = ROOM_WIDTH * 0.5;
pub const ROOM_HALF_HEIGHT: f32 = ROOM_HEIGHT * 0.5;

// Screen origin stays at the center of the full frame while the gameplay room
// is shifted downward to leave a consistent HUD strip at the top.
pub const ROOM_ORIGIN: Vec2 = Vec2::new(0.0, -HUD_HEIGHT * 0.5);
pub const HUD_CENTER: Vec2 =
    Vec2::new(0.0, ROOM_ORIGIN.y + ROOM_HALF_HEIGHT + HUD_HEIGHT * 0.5);

pub const NORTH_DOOR_ANCHOR: Vec2 = Vec2::new(0.0, ROOM_ORIGIN.y + ROOM_HALF_HEIGHT);
pub const SOUTH_DOOR_ANCHOR: Vec2 = Vec2::new(0.0, ROOM_ORIGIN.y - ROOM_HALF_HEIGHT);
pub const EAST_DOOR_ANCHOR: Vec2 = Vec2::new(ROOM_HALF_WIDTH, ROOM_ORIGIN.y);
pub const WEST_DOOR_ANCHOR: Vec2 = Vec2::new(-ROOM_HALF_WIDTH, ROOM_ORIGIN.y);

pub const DOOR_ENTRY_DEPTH: f32 = 24.0;
pub const NORTH_ENTRY_OFFSET: Vec2 = Vec2::new(0.0, NORTH_DOOR_ANCHOR.y - DOOR_ENTRY_DEPTH);
pub const SOUTH_ENTRY_OFFSET: Vec2 = Vec2::new(0.0, SOUTH_DOOR_ANCHOR.y + DOOR_ENTRY_DEPTH);
pub const EAST_ENTRY_OFFSET: Vec2 = Vec2::new(EAST_DOOR_ANCHOR.x - DOOR_ENTRY_DEPTH, ROOM_ORIGIN.y);
pub const WEST_ENTRY_OFFSET: Vec2 = Vec2::new(WEST_DOOR_ANCHOR.x + DOOR_ENTRY_DEPTH, ROOM_ORIGIN.y);

pub mod render_layers {
    pub const BACKGROUND: f32 = -20.0;
    pub const FLOOR: f32 = 0.0;
    pub const WALLS: f32 = 10.0;
    pub const ENTITIES: f32 = 20.0;
    pub const PICKUPS: f32 = 30.0;
    pub const PROJECTILES: f32 = 40.0;
    pub const UI_BACKGROUND: f32 = 90.0;
    pub const UI: f32 = 100.0;
    pub const DEBUG: f32 = 200.0;
}
