use bevy::prelude::*;

// ── Player ──────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Player;

// ── Projectile ──────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Bullet;

// ── Grid position (shared by mushrooms and centipede segments) ───────────────

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct GridPos {
    pub col: i32,
    pub row: i32,
}

// ── Mushrooms ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Mushroom {
    /// Number of hits received so far (destroyed when hits == MUSHROOM_MAX_HITS)
    pub hits: u8,
}

/// Marker: this mushroom was poisoned by a scorpion
#[derive(Component)]
pub struct Poisoned;

// ── Centipede ────────────────────────────────────────────────────────────────

#[derive(Component, Clone)]
pub struct CentipedeSegment {
    pub chain_id: u32,
    pub index: usize,
}

/// Marker on the first segment of a chain
#[derive(Component)]
pub struct CentipedeHead;

/// Horizontal direction of a centipede chain head (+1 = right, -1 = left)
#[derive(Component, Clone, Copy)]
pub struct CentipedeDir {
    pub dx: i32,
}

/// Marker: this centipede head hit a poisoned mushroom and rushes straight down
#[derive(Component)]
pub struct PoisonRushing;

// ── Enemies ──────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Flea {
    pub hits: u8,
}

#[derive(Component)]
pub struct Spider {
    pub dir: Vec2,
    pub change_timer: f32,
}

#[derive(Component)]
pub struct Scorpion {
    pub dx: f32, // +1.0 or -1.0
}

// ── UI markers ───────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct MenuScreen;

#[derive(Component)]
pub struct GameOverScreen;
