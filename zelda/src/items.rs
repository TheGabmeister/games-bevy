use bevy::prelude::*;

use crate::components::{Health, PickupKind};
use crate::resources::{Inventory, PlayerVitals};

pub struct ItemData {
    pub kind: PickupKind,
    pub label: &'static str,
    pub description: &'static str,
    pub color: Color,
    pub radius: f32,
}

const BASE_RADIUS: f32 = 7.0;

pub const ITEM_TABLE: &[ItemData] = &[
    ItemData {
        kind: PickupKind::Rupee,
        label: "rupee",
        description: "A single green rupee worth 1.",
        color: Color::srgb(0.97, 0.89, 0.44),
        radius: BASE_RADIUS,
    },
    ItemData {
        kind: PickupKind::FiveRupees,
        label: "5 rupee",
        description: "A blue rupee worth 5.",
        color: Color::srgb(0.97, 0.89, 0.44),
        radius: BASE_RADIUS + 2.0,
    },
    ItemData {
        kind: PickupKind::Heart,
        label: "heart",
        description: "Restores one heart of health.",
        color: Color::srgb(0.85, 0.2, 0.25),
        radius: BASE_RADIUS,
    },
    ItemData {
        kind: PickupKind::Bomb,
        label: "bomb",
        description: "Adds 4 bombs to inventory.",
        color: Color::srgb(0.86, 0.47, 0.16),
        radius: BASE_RADIUS,
    },
    ItemData {
        kind: PickupKind::Key,
        label: "key",
        description: "Opens one locked door.",
        color: Color::srgb(0.94, 0.86, 0.51),
        radius: BASE_RADIUS,
    },
    ItemData {
        kind: PickupKind::HeartContainer,
        label: "heart+",
        description: "Increases max health by one heart and fully heals.",
        color: Color::srgb(0.85, 0.2, 0.25),
        radius: BASE_RADIUS + 2.0,
    },
];

pub fn lookup(kind: PickupKind) -> &'static ItemData {
    ITEM_TABLE
        .iter()
        .find(|item| item.kind == kind)
        .expect("missing item data entry")
}

pub fn apply_pickup_effect(
    kind: PickupKind,
    inventory: &mut Inventory,
    health: &mut Health,
    vitals: &mut PlayerVitals,
) {
    match kind {
        PickupKind::Rupee => inventory.rupees = (inventory.rupees + 1).min(999),
        PickupKind::FiveRupees => inventory.rupees = (inventory.rupees + 5).min(999),
        PickupKind::Heart => {
            health.current = (health.current + 2).min(health.max);
            vitals.current_health = health.current;
        }
        PickupKind::Bomb => inventory.bombs = (inventory.bombs + 4).min(99),
        PickupKind::Key => inventory.keys = (inventory.keys + 1).min(99),
        PickupKind::HeartContainer => {
            health.max += 2;
            health.current = health.max;
            vitals.max_health = health.max;
            vitals.current_health = health.current;
        }
    }
}
