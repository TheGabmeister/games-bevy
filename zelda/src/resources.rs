use std::collections::HashSet;

use bevy::prelude::*;

use crate::constants;

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Resource, Clone, Copy, Debug)]
pub struct PlayerVitals {
    pub current_health: u8,
    pub max_health: u8,
}

impl Default for PlayerVitals {
    fn default() -> Self {
        Self {
            current_health: 6,
            max_health: 6,
        }
    }
}

impl PlayerVitals {
    pub fn continue_health(self) -> u8 {
        self.max_health.min(6)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RoomId {
    #[default]
    OverworldCenter,
    OverworldNorth,
    OverworldSouth,
    OverworldEast,
    OverworldWest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExitDirection {
    North,
    South,
    East,
    West,
}

impl ExitDirection {
    pub const fn target_spawn_offset(self) -> Vec2 {
        match self {
            Self::North => constants::SOUTH_ENTRY_OFFSET,
            Self::South => constants::NORTH_ENTRY_OFFSET,
            Self::East => constants::WEST_ENTRY_OFFSET,
            Self::West => constants::EAST_ENTRY_OFFSET,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct CurrentRoom {
    pub id: RoomId,
}

impl Default for CurrentRoom {
    fn default() -> Self {
        Self {
            id: RoomId::OverworldCenter,
        }
    }
}

#[derive(Resource, Debug)]
pub struct RoomTransitionState {
    pub locked: bool,
    pub direction: Option<ExitDirection>,
    pub timer: Timer,
}

impl Default for RoomTransitionState {
    fn default() -> Self {
        Self {
            locked: false,
            direction: None,
            timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RoomPersistenceCategory {
    ResetOnRoomLoad,
    UniquePickup,
    Secret,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PersistentRoomKey {
    pub room: RoomId,
    pub key: &'static str,
}

#[derive(Resource, Default, Debug)]
pub struct RoomPersistence {
    unique_pickups: HashSet<PersistentRoomKey>,
    revealed_secrets: HashSet<PersistentRoomKey>,
}

impl RoomPersistence {
    pub fn contains(&self, category: RoomPersistenceCategory, key: PersistentRoomKey) -> bool {
        match category {
            RoomPersistenceCategory::ResetOnRoomLoad => false,
            RoomPersistenceCategory::UniquePickup => self.unique_pickups.contains(&key),
            RoomPersistenceCategory::Secret => self.revealed_secrets.contains(&key),
        }
    }

    pub fn record(&mut self, category: RoomPersistenceCategory, key: PersistentRoomKey) {
        match category {
            RoomPersistenceCategory::ResetOnRoomLoad => {}
            RoomPersistenceCategory::UniquePickup => {
                self.unique_pickups.insert(key);
            }
            RoomPersistenceCategory::Secret => {
                self.revealed_secrets.insert(key);
            }
        }
    }
}
