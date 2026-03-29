use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Guard;

#[derive(Component)]
pub struct Gold;

#[derive(Component, Copy, Clone, Debug)]
pub struct GridPosition(pub IVec2);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizontalDir {
    Left,
    Right,
}

#[derive(Component, Debug, Default)]
pub enum MovementState {
    #[default]
    Idle,
    Moving {
        from: IVec2,
        to: IVec2,
        progress: f32,
    },
    Falling {
        from: IVec2,
        to: IVec2,
        progress: f32,
    },
    Climbing {
        from: IVec2,
        to: IVec2,
        progress: f32,
    },
    Digging {
        side: HorizontalDir,
        timer: Timer,
    },
    Trapped {
        timer: Timer,
    },
}

#[derive(Component)]
pub struct Hole {
    pub cell: IVec2,
    pub phase: HolePhase,
    pub timer: Timer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HolePhase {
    Open,
    Closing,
}

#[derive(Component)]
pub struct HiddenLadderTile;

/// Guard AI recalculation timer.
#[derive(Component)]
pub struct AiTimer(pub Timer);

/// HUD marker components.
#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct HudScore;

#[derive(Component)]
pub struct HudLives;

#[derive(Component)]
pub struct HudLevel;

#[derive(Component)]
pub struct HudGold;

/// Pause overlay marker.
#[derive(Component)]
pub struct PauseOverlay;
