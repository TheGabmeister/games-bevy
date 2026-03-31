use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use serde::Deserialize;

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

// ── Room types ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub enum RoomType {
    #[default]
    Overworld,
    Cave,
    Dungeon,
    Shop,
    Hint,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub enum RoomId {
    #[default]
    OverworldCenter,
    OverworldNorth,
    OverworldSouth,
    OverworldEast,
    OverworldWest,
    CaveSword,
    CaveShop,
    CaveHint,
    Dungeon1Entry,
    Dungeon1North,
    Dungeon1East,
    Dungeon1Boss,
    Dungeon1Triforce,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub enum DoorKind {
    #[default]
    Open,
    Locked,
    Shutter,
    Bombable,
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

// ── Persistence ────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RoomPersistenceCategory {
    ResetOnRoomLoad,
    UniquePickup,
    Secret,
    DungeonDoor,
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
    dungeon_doors: HashSet<PersistentRoomKey>,
}

impl RoomPersistence {
    pub fn contains(&self, category: RoomPersistenceCategory, key: PersistentRoomKey) -> bool {
        match category {
            RoomPersistenceCategory::ResetOnRoomLoad => false,
            RoomPersistenceCategory::UniquePickup => self.unique_pickups.contains(&key),
            RoomPersistenceCategory::Secret => self.revealed_secrets.contains(&key),
            RoomPersistenceCategory::DungeonDoor => self.dungeon_doors.contains(&key),
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
            RoomPersistenceCategory::DungeonDoor => {
                self.dungeon_doors.insert(key);
            }
        }
    }
}

// ── Dungeon ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum DungeonId {
    Dungeon1,
}

#[derive(Resource, Default, Debug)]
pub struct DungeonState {
    pub current_dungeon: Option<DungeonId>,
    pub dungeon_keys: HashMap<DungeonId, u8>,
    pub has_map: HashSet<DungeonId>,
    pub has_compass: HashSet<DungeonId>,
    pub triforce_pieces: HashSet<DungeonId>,
    pub boss_defeated: HashSet<DungeonId>,
    pub rooms_cleared: HashSet<RoomId>,
}

impl DungeonState {
    pub fn keys_for_current(&self) -> u8 {
        self.current_dungeon
            .and_then(|d| self.dungeon_keys.get(&d).copied())
            .unwrap_or(0)
    }

    pub fn spend_key(&mut self) -> bool {
        if let Some(dungeon) = self.current_dungeon {
            let count = self.dungeon_keys.entry(dungeon).or_insert(0);
            if *count > 0 {
                *count -= 1;
                return true;
            }
        }
        false
    }
}

// ── Inventory ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EquippedItem {
    Bomb,
}

impl EquippedItem {
    pub const ALL: &[EquippedItem] = &[EquippedItem::Bomb];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Bomb => "BOMB",
        }
    }

    pub fn next(self) -> Option<EquippedItem> {
        let all = Self::ALL;
        let idx = all.iter().position(|&e| e == self).unwrap();
        if idx + 1 < all.len() {
            Some(all[idx + 1])
        } else {
            None
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct Inventory {
    pub rupees: u16,
    pub bombs: u8,
    pub keys: u8,
    pub has_sword: bool,
    pub equipped: Option<EquippedItem>,
}

// ── Dialogue ───────────────────────────────────────────────────────────

#[derive(Resource, Default, Debug)]
pub struct DialogueState {
    pub lines: Vec<String>,
    pub current_line: usize,
}

impl DialogueState {
    pub fn is_active(&self) -> bool {
        !self.lines.is_empty()
    }

    pub fn start(&mut self, lines: Vec<String>) {
        self.lines = lines;
        self.current_line = 0;
    }

    pub fn advance(&mut self) {
        if self.current_line + 1 < self.lines.len() {
            self.current_line += 1;
        } else {
            self.lines.clear();
            self.current_line = 0;
        }
    }

    pub fn current_text(&self) -> &str {
        if self.is_active() {
            &self.lines[self.current_line]
        } else {
            ""
        }
    }
}
