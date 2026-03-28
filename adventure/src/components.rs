use bevy::prelude::*;

// ---- Enums ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyColor {
    Gold,
    Red,
    Blue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum ItemKind {
    GoldKey,
    RedKey,
    BlueKey,
    Sword,
    Bridge,
    Chalice,
    Magnet,
    Dot,
}

impl ItemKind {
    pub fn name(&self) -> &'static str {
        match self {
            ItemKind::GoldKey => "Gold Key",
            ItemKind::RedKey => "Red Key",
            ItemKind::BlueKey => "Blue Key",
            ItemKind::Sword => "Sword",
            ItemKind::Bridge => "Bridge",
            ItemKind::Chalice => "Chalice",
            ItemKind::Magnet => "Magnet",
            ItemKind::Dot => "Dot",
        }
    }

    pub fn key_color(&self) -> Option<KeyColor> {
        match self {
            ItemKind::GoldKey => Some(KeyColor::Gold),
            ItemKind::RedKey => Some(KeyColor::Red),
            ItemKind::BlueKey => Some(KeyColor::Blue),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum DragonKind {
    Yorgle,
    Grundle,
    Rhindle,
}

impl DragonKind {
    pub fn speed(&self) -> f32 {
        match self {
            DragonKind::Yorgle => 80.0,
            DragonKind::Grundle => 60.0,
            DragonKind::Rhindle => 100.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            DragonKind::Yorgle => Color::srgb(1.0, 0.5, 0.0),
            DragonKind::Grundle => Color::srgb(0.0, 0.7, 0.2),
            DragonKind::Rhindle => Color::srgb(0.8, 0.1, 0.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitDir {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

// ---- Marker components ----

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub struct Dragon;

#[derive(Component)]
pub struct DragonAlive;

#[derive(Component)]
pub struct DragonBody;

#[derive(Component)]
pub struct DragonHead;

#[derive(Component)]
pub struct Bat;

#[derive(Component)]
pub struct Gate;

#[derive(Component)]
pub struct RoomWallMarker;

/// Marks item as currently carried by player
#[derive(Component)]
pub struct Carried;

/// UI text showing current room name
#[derive(Component)]
pub struct RoomNameText;

/// UI text showing carried item
#[derive(Component)]
pub struct InventoryText;

/// Marker for title screen entities
#[derive(Component)]
pub struct TitleScreen;

/// Marker for game over screen entities
#[derive(Component)]
pub struct GameOverScreen;

/// Marker for win screen entities
#[derive(Component)]
pub struct WinScreen;

/// Marker for in-game UI entities
#[derive(Component)]
pub struct GameUi;

// ---- Data components ----

/// Which room this entity belongs to (255 = carried/held by bat)
#[derive(Component)]
pub struct InRoom(pub u8);

#[derive(Component)]
pub struct GateData {
    pub key_color: KeyColor,
    pub exit_dir: ExitDir,
    pub open: bool,
}

#[derive(Component)]
pub struct DragonData {
    pub kind: DragonKind,
    pub alive: bool,
}

#[derive(Component)]
pub struct BatData {
    pub held_item: Option<Entity>,
    pub wander_timer: Timer,
    pub grab_timer: Timer,
}

/// Tracks the dragon swallow animation before game over
#[derive(Resource)]
pub struct SwallowInfo {
    pub dragon: Entity,
    pub timer: Timer,
}

/// Wall bypass info computed each frame (bridge + easter egg)
#[derive(Resource, Default)]
pub struct WallBypass {
    pub bridge: Option<crate::world::WallRect>,
    pub easter_egg_north: bool,
}
