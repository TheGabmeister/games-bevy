use std::collections::HashSet;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RoomId {
    #[default]
    OverworldTest,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct CurrentRoom {
    pub id: RoomId,
}

impl Default for CurrentRoom {
    fn default() -> Self {
        Self {
            id: RoomId::OverworldTest,
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
