use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerSide {
    #[default]
    Left,
    Right,
}

impl PlayerSide {
    pub const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Left => "Left Player",
            Self::Right => "Right Player",
        }
    }
}

#[derive(Component)]
pub struct Paddle {
    pub side: PlayerSide,
}

#[derive(Component)]
pub struct Ball;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone, Copy)]
pub struct Collider {
    pub half_size: Vec2,
}

#[derive(Component)]
pub struct PlayerControl {
    pub up: KeyCode,
    pub down: KeyCode,
}

#[derive(Component)]
pub struct GameplayEntity;

#[derive(Component)]
pub struct MenuEntity;

#[derive(Component)]
pub struct WinnerEntity;

#[derive(Component)]
pub struct ScoreText;

#[derive(Resource, Debug, Clone, Copy)]
pub struct MatchConfig {
    pub win_score: u8,
    pub paddle_speed: f32,
    pub ball_speed: f32,
    pub speed_gain_per_hit: f32,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            win_score: 5,
            paddle_speed: 560.0,
            ball_speed: 360.0,
            speed_gain_per_hit: 26.0,
        }
    }
}

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Score {
    pub left: u8,
    pub right: u8,
}

impl Score {
    pub fn reset(&mut self) {
        self.left = 0;
        self.right = 0;
    }

    pub fn formatted(self) -> String {
        format!("{} : {}", self.left, self.right)
    }
}

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Winner {
    pub side: Option<PlayerSide>,
}

#[derive(Event)]
pub struct PaddleHitEvent;
