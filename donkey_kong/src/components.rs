use bevy::prelude::*;

// --- Entity Markers ---

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct DonkeyKong;

#[derive(Component)]
pub struct PaulineEntity;

#[derive(Component)]
pub struct OilDrumEntity;

#[derive(Component)]
pub struct GirderEntity;

#[derive(Component)]
pub struct LadderEntity(pub usize);

#[derive(Component)]
pub struct GoalZoneEntity;

/// All gameplay entities carry this for bulk cleanup.
#[derive(Component)]
pub struct StageEntity;

// --- Player State ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Locomotion {
    Walking,
    Jumping,
    Falling,
    Climbing,
    Dying,
}

#[derive(Component)]
pub struct PlayerState {
    pub locomotion: Locomotion,
    pub facing: f32,
    pub vel_y: f32,
    pub jump_dx: f32,
    pub last_supported_y: f32,
    pub current_girder: Option<usize>,
    pub current_ladder: Option<usize>,
    pub hammer_timer: Option<f32>,
    pub jump_scored: Vec<Entity>,
    pub jump_score_count: u32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            locomotion: Locomotion::Walking,
            facing: 1.0,
            vel_y: 0.0,
            jump_dx: 0.0,
            last_supported_y: 0.0,
            current_girder: Some(0),
            current_ladder: None,
            hammer_timer: None,
            jump_scored: Vec::new(),
            jump_score_count: 0,
        }
    }
}

// --- DK State ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DkAnimState {
    Idle,
    WindUp,
    Throwing,
}

#[derive(Component)]
pub struct DkState {
    pub anim: DkAnimState,
    pub timer: f32,
    pub throw_timer: f32,
    pub barrels_thrown: u32,
}

// --- Barrel ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarrelMoveState {
    Rolling,
    Falling { target_girder: usize },
    Descending { ladder_index: usize },
}

#[derive(Component)]
pub struct Barrel {
    pub is_blue: bool,
    pub is_wild: bool,
    pub phase: f32,
    pub state: BarrelMoveState,
    pub direction: f32,
    pub girder: usize,
}

// --- Fireball ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FireballMoveState {
    Patrolling,
    Climbing { target_girder: usize },
}

#[derive(Component)]
pub struct Fireball {
    pub state: FireballMoveState,
    pub direction: f32,
    pub girder: usize,
}

// --- Pickups ---

#[derive(Component)]
pub struct HammerPickup(pub usize);

#[derive(Component)]
pub struct BonusItemEntity(pub usize);

// --- UI Markers ---

#[derive(Component)]
pub struct StartScreenUI;

#[derive(Component)]
pub struct GameHudUI;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct HighScoreText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct BonusTimerText;

#[derive(Component)]
pub struct GameOverUI;

#[derive(Component)]
pub struct WinScreenUI;

#[derive(Component)]
pub struct WaveTallyUI;
