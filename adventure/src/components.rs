use bevy::{ecs::system::SystemParam, prelude::*};

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

impl ExitDir {
    pub const ALL: [Self; 4] = [Self::North, Self::South, Self::East, Self::West];

    pub const fn index(self) -> usize {
        self as usize
    }
}

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

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct Carried;

#[derive(Component)]
pub struct RoomNameText;

#[derive(Component)]
pub struct InventoryText;

#[derive(Component)]
pub struct TitleScreen;

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct WinScreen;

#[derive(Component)]
pub struct GameUi;

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
}

#[derive(Component)]
pub struct BatData {
    pub held_item: Option<Entity>,
    pub wander_timer: Timer,
    pub grab_timer: Timer,
}

#[derive(Resource)]
pub struct SwallowInfo {
    pub dragon: Entity,
    pub timer: Timer,
}

#[derive(Resource, Default)]
pub struct WallBypass {
    pub bridge: Option<crate::world::WallRect>,
    pub easter_egg_north: bool,
}

#[derive(SystemParam)]
pub struct InventoryItems<'w, 's> {
    pub inventory: Res<'w, crate::world::PlayerInventory>,
    pub item_kinds: Query<'w, 's, &'static ItemKind, With<Item>>,
}

impl<'w, 's> InventoryItems<'w, 's> {
    pub fn carried_entity(&self) -> Option<Entity> {
        self.inventory.item
    }

    pub fn carried_kind(&self) -> Option<ItemKind> {
        self.carried_entity()
            .and_then(|entity| self.item_kinds.get(entity).ok())
            .copied()
    }

    pub fn carried_key_color(&self) -> Option<KeyColor> {
        self.carried_kind().and_then(|kind| kind.key_color())
    }

    pub fn is_carrying(&self, kind: ItemKind) -> bool {
        self.carried_kind() == Some(kind)
    }
}
