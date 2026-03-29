use bevy::prelude::*;

// --- Gameplay components ---

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub enum FacingDirection {
    Left,
    #[default]
    Right,
}

#[derive(Component)]
pub struct Grounded;

#[derive(Component)]
pub struct FlapCooldown(pub Timer);

impl FlapCooldown {
    /// Creates a cooldown that is immediately ready (first flap has no delay).
    pub fn ready() -> Self {
        let mut t = Timer::from_seconds(crate::constants::FLAP_COOLDOWN, TimerMode::Once);
        t.finish();
        Self(t)
    }
}

#[derive(Component, Default)]
pub struct PreviousPosition(pub Vec2);

#[derive(Component)]
pub struct Invincible(pub Timer);

#[derive(Component)]
pub struct Rider;

// --- Player ---

#[derive(Component)]
pub struct Player {
    pub id: u8,
}

// --- Enemy ---

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum EnemyTier {
    Bounder,
    Hunter,
    ShadowLord,
}

impl EnemyTier {
    pub fn hatched_tier(self) -> Self {
        match self {
            Self::Bounder => Self::Hunter,
            Self::Hunter => Self::ShadowLord,
            Self::ShadowLord => Self::ShadowLord,
        }
    }

    pub fn score(self) -> u32 {
        use crate::constants::*;
        match self {
            Self::Bounder => SCORE_KILL_BOUNDER,
            Self::Hunter => SCORE_KILL_HUNTER,
            Self::ShadowLord => SCORE_KILL_SHADOW_LORD,
        }
    }

    pub fn speed_multiplier(self) -> f32 {
        match self {
            Self::Bounder => 0.6,
            Self::Hunter => 0.85,
            Self::ShadowLord => 1.1,
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum EnemyAiState {
    Wander,
    Pursue,
}

#[derive(Component)]
pub struct AiTimers {
    pub flap: Timer,
    pub direction: Timer,
}

// --- Egg ---

#[derive(Component)]
pub struct Egg {
    pub tier: EnemyTier,
    pub hatch_timer: Timer,
}

// --- Rendering ---

#[derive(Component)]
pub struct Wing {
    pub base_y: f32,
}

// --- UI markers ---

#[derive(Component)]
pub struct ScoreText(pub u8);

#[derive(Component)]
pub struct LivesText(pub u8);

#[derive(Component)]
pub struct WaveText;

// --- Effects ---

#[derive(Component)]
pub struct Particle {
    pub lifetime: Timer,
}

// --- Messages ---

#[derive(Message)]
pub struct JoustKillMessage {
    pub loser_position: Vec2,
    pub loser_tier: Option<EnemyTier>,
}

#[derive(Message)]
pub struct ScoreMessage {
    pub player_id: u8,
    pub points: u32,
}

#[derive(Message)]
pub struct PlayerDiedMessage {
    pub player_id: u8,
}
