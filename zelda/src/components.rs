use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct RoomEntity;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Door;

#[derive(Component)]
pub struct StaticBlocker;

#[derive(Component, Clone, Copy, Debug)]
pub struct MoveSpeed(pub f32);

#[derive(Component, Clone, Copy, Debug)]
pub struct Health {
    pub current: u8,
    pub max: u8,
}

impl Health {
    pub const fn new(max: u8) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Facing {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Hitbox {
    pub half_size: Vec2,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Hurtbox {
    pub half_size: Vec2,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct SolidBody {
    pub half_size: Vec2,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Damage(pub u8);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Knockback {
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct SwordAttack;

#[derive(Component, Deref, DerefMut)]
pub struct InvulnerabilityTimer(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct Lifetime(pub Timer);

#[derive(Component)]
pub struct Label(pub &'static str);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    Rupee,
    FiveRupees,
    Heart,
    Bomb,
    Key,
    HeartContainer,
}

impl PickupKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rupee => "rupee",
            Self::FiveRupees => "5 rupee",
            Self::Heart => "heart",
            Self::Bomb => "bomb",
            Self::Key => "key",
            Self::HeartContainer => "heart+",
        }
    }
}
