use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum FacingDirection {
    Left,
    #[default]
    Right,
}

#[derive(Component, Default)]
pub struct Grounded(pub bool);

#[derive(Component)]
pub struct Tile;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Ground,
    Brick,
    QuestionBlock,
    Empty,
    Solid,
    PipeTopLeft,
    PipeTopRight,
    PipeBodyLeft,
    PipeBodyRight,
}

// Enemy
#[derive(Component)]
pub struct Goomba;

#[derive(Component)]
pub struct EnemyWalker {
    pub speed: f32,
    pub direction: f32,
}

#[derive(Component)]
pub struct EnemyActive;

#[derive(Component)]
pub struct Squished(pub Timer);

#[derive(Component)]
pub struct ScorePopup(pub Timer);

// HUD markers
#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct CoinText;

#[derive(Component)]
pub struct TimerText;

// Tile grid position (for block hit lookup)
#[derive(Component)]
pub struct TilePos {
    pub col: i32,
    pub row: i32,
}

// Block bounce animation
#[derive(Component)]
pub struct BlockBounce {
    pub timer: Timer,
    pub original_y: f32,
}

// Block has been used (? became empty)
#[derive(Component)]
pub struct BlockUsed;

// Coin popping out of block
#[derive(Component)]
pub struct CoinPop {
    pub vel_y: f32,
    pub timer: Timer,
}

// Brick break particle
#[derive(Component)]
pub struct BrickParticle {
    pub vel_x: f32,
    pub vel_y: f32,
    pub timer: Timer,
}

// Floating coin in level (collectible)
#[derive(Component)]
pub struct FloatingCoin;

// Collision size for entities with AABB collision
#[derive(Component)]
pub struct CollisionSize {
    pub width: f32,
    pub height: f32,
}

// Mushroom power-up
#[derive(Component)]
pub struct Mushroom;

// Mushroom emerging from block
#[derive(Component)]
pub struct MushroomEmerging {
    pub remaining: f32,
}

// Player size state
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerSize {
    #[default]
    Small,
    Big,
    Fire,
}

// Growth/shrink animation
#[derive(Component)]
pub struct GrowthAnimation {
    pub timer: Timer,
    pub flash_timer: Timer,
    pub growing: bool,
}

// Invincibility after taking damage
#[derive(Component)]
pub struct Invincible {
    pub timer: Timer,
}

// Ducking marker (Big Mario only)
#[derive(Component)]
pub struct Ducking;

// Koopa Troopa
#[derive(Component)]
pub struct KoopaTroopa;

// Shell states
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShellState {
    Stationary,
    Moving,
}

// Shell component
#[derive(Component)]
pub struct Shell {
    pub state: ShellState,
    pub chain_kills: u32,
}

// Fire Flower power-up
#[derive(Component)]
pub struct FireFlower;

#[derive(Component)]
pub struct FireFlowerEmerging {
    pub remaining: f32,
}

// Fireball projectile
#[derive(Component)]
pub struct Fireball {
    pub direction: f32,
}

// Flagpole flag (slides down during level complete)
#[derive(Component)]
pub struct FlagpoleFlag;

// Castle marker
#[derive(Component)]
pub struct Castle;

// Starman power-up (bouncing item)
#[derive(Component)]
pub struct Starman;

#[derive(Component)]
pub struct StarmanEmerging {
    pub remaining: f32,
}

// Star power (invincibility granted by starman)
#[derive(Component)]
pub struct StarPower {
    pub timer: Timer,
    pub flash_timer: Timer,
    pub color_index: usize,
}

// 1-Up Mushroom (green, grants extra life)
#[derive(Component)]
pub struct OneUpMushroom;

// Player face (child entity for visual detail)
#[derive(Component)]
pub struct PlayerFace;

// Skidding visual (reversing direction at speed)
#[derive(Component)]
pub struct Skidding;

// Decoration marker
#[derive(Component)]
pub struct Decoration;

// Warp pipe fade overlay (full-screen black rect for transitions)
#[derive(Component)]
pub struct WarpFadeOverlay;
