use bevy::prelude::*;
use serde::Deserialize;

use crate::components::{Health, PickupKind};
use crate::resources::{DungeonState, Inventory, PlayerVitals};

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load_item_table())
            .insert_resource(load_drop_table());
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum PickupEffect {
    AddRupees(u16),
    RestoreHealth(u8),
    AddBombs(u8),
    AddKeys(u8),
    HeartContainer,
    AddDungeonKey,
    GrantDungeonMap,
    GrantCompass,
    GrantTriforce,
    GrantSword,
}

#[derive(Deserialize)]
struct ItemDataRon {
    kind: PickupKind,
    label: String,
    description: String,
    color: (f32, f32, f32),
    radius: f32,
    effect: PickupEffect,
}

#[derive(Clone, Debug)]
pub struct ItemData {
    pub kind: PickupKind,
    pub label: String,
    pub description: String,
    pub color: Color,
    pub radius: f32,
    pub effect: PickupEffect,
}

#[derive(Resource)]
pub struct ItemTable {
    items: Vec<ItemData>,
}

impl ItemTable {
    pub fn lookup(&self, kind: PickupKind) -> &ItemData {
        self.items
            .iter()
            .find(|item| item.kind == kind)
            .expect("missing item data entry")
    }
}

fn load_item_table() -> ItemTable {
    let ron_str = std::fs::read_to_string("assets/data/items.ron")
        .expect("failed to read assets/data/items.ron");
    let entries: Vec<ItemDataRon> =
        ron::from_str(&ron_str).expect("failed to parse items.ron");
    let items = entries
        .into_iter()
        .map(|e| ItemData {
            kind: e.kind,
            label: e.label,
            description: e.description,
            color: Color::srgb(e.color.0, e.color.1, e.color.2),
            radius: e.radius,
            effect: e.effect,
        })
        .collect();
    ItemTable { items }
}

// ── Drop table ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DropEntry {
    pub kind: PickupKind,
    pub chance: f32,
}

#[derive(Resource)]
pub struct DropTable {
    pub entries: Vec<DropEntry>,
}

fn load_drop_table() -> DropTable {
    let ron_str =
        std::fs::read_to_string("assets/data/drops.ron").expect("failed to read assets/data/drops.ron");
    let entries: Vec<DropEntry> = ron::from_str(&ron_str).expect("failed to parse drops.ron");
    DropTable { entries }
}

// ── Pickup effects ─────────────────────────────────────────────────────

pub fn apply_pickup_effect(
    effect: &PickupEffect,
    inventory: &mut Inventory,
    health: &mut Health,
    vitals: &mut PlayerVitals,
    dungeon_state: &mut DungeonState,
) {
    match effect {
        PickupEffect::AddRupees(amount) => {
            inventory.rupees = (inventory.rupees + amount).min(999);
        }
        PickupEffect::RestoreHealth(amount) => {
            health.current = (health.current + amount).min(health.max);
            vitals.current_health = health.current;
        }
        PickupEffect::AddBombs(amount) => {
            inventory.bombs = (inventory.bombs + amount).min(99);
        }
        PickupEffect::AddKeys(amount) => {
            inventory.keys = (inventory.keys + amount).min(99);
        }
        PickupEffect::HeartContainer => {
            health.max += 2;
            health.current = health.max;
            vitals.max_health = health.max;
            vitals.current_health = health.current;
        }
        PickupEffect::AddDungeonKey => {
            if let Some(dungeon) = dungeon_state.current_dungeon {
                *dungeon_state.dungeon_keys.entry(dungeon).or_insert(0) += 1;
            }
        }
        PickupEffect::GrantDungeonMap => {
            if let Some(dungeon) = dungeon_state.current_dungeon {
                dungeon_state.has_map.insert(dungeon);
            }
        }
        PickupEffect::GrantCompass => {
            if let Some(dungeon) = dungeon_state.current_dungeon {
                dungeon_state.has_compass.insert(dungeon);
            }
        }
        PickupEffect::GrantTriforce => {
            if let Some(dungeon) = dungeon_state.current_dungeon {
                dungeon_state.triforce_pieces.insert(dungeon);
            }
        }
        PickupEffect::GrantSword => {
            inventory.has_sword = true;
        }
    }
}
