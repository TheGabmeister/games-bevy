use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Guard;

#[derive(Component)]
pub struct Gold;

#[derive(Component, Copy, Clone, Debug)]
pub struct GridPosition(pub IVec2);

#[derive(Component)]
pub struct SpawnPoint(pub IVec2);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizontalDir {
    Left,
    Right,
}

#[derive(Component, Debug)]
pub enum MovementState {
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
}

impl Default for MovementState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Marker for a hole entity that visually represents a dug-out brick.
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

/// Marker on tile entities that are hidden ladders (invisible until exit unlocked).
#[derive(Component)]
pub struct HiddenLadderTile;

/// Marker on tile entities representing the top-row exit zone.
#[derive(Component)]
pub struct ExitZone;
