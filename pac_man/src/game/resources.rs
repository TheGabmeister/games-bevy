use bevy::prelude::*;

use crate::game::constants::{
    BACKGROUND_COLOR, BLINKY_COLOR, CHASE_DURATION, CLYDE_COLOR, EYE_PUPIL_COLOR, EYE_WHITE_COLOR,
    FRIGHTENED_COLOR, FRIGHTENED_DURATION, GHOST_RADIUS, INKY_COLOR, PACMAN_COLOR, PACMAN_RADIUS,
    PELLET_COLOR, PELLET_RADIUS, PINKY_COLOR, PLAYER_LIVES, POWER_PELLET_RADIUS, READY_DURATION,
    SCATTER_DURATION, TILE_SIZE, WALL_COLOR,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoundPhase {
    Ready,
    Playing,
    Won,
    GameOver,
}

#[derive(Resource)]
pub struct GameSession {
    pub score: u32,
    pub lives: u8,
    pub pellets_remaining: usize,
    pub phase: RoundPhase,
    pub phase_timer: Timer,
    pub frightened_seconds: f32,
    pub scatter_mode: bool,
    pub mode_seconds: f32,
    pub ghost_combo: u8,
}

impl GameSession {
    pub fn new(pellets_remaining: usize) -> Self {
        let mut session = Self {
            score: 0,
            lives: PLAYER_LIVES,
            pellets_remaining,
            phase: RoundPhase::Ready,
            phase_timer: Timer::from_seconds(READY_DURATION, TimerMode::Once),
            frightened_seconds: 0.0,
            scatter_mode: true,
            mode_seconds: SCATTER_DURATION,
            ghost_combo: 0,
        };
        session.begin_ready_phase();
        session
    }

    pub fn reset_for_new_game(&mut self, pellets_remaining: usize) {
        self.score = 0;
        self.lives = PLAYER_LIVES;
        self.pellets_remaining = pellets_remaining;
        self.begin_ready_phase();
    }

    pub fn begin_ready_phase(&mut self) {
        self.phase = RoundPhase::Ready;
        self.phase_timer = Timer::from_seconds(READY_DURATION, TimerMode::Once);
        self.frightened_seconds = 0.0;
        self.scatter_mode = true;
        self.mode_seconds = SCATTER_DURATION;
        self.ghost_combo = 0;
    }

    pub fn frightened_active(&self) -> bool {
        self.frightened_seconds > 0.0
    }

    pub fn advance_mode_cycle(&mut self, delta_seconds: f32) {
        self.mode_seconds -= delta_seconds;
        if self.mode_seconds <= 0.0 {
            self.scatter_mode = !self.scatter_mode;
            self.mode_seconds = if self.scatter_mode {
                SCATTER_DURATION
            } else {
                CHASE_DURATION
            };
        }
    }

    pub fn set_frightened(&mut self) {
        self.frightened_seconds = FRIGHTENED_DURATION;
        self.ghost_combo = 0;
    }
}

#[derive(Resource)]
pub struct GameMeshes {
    pub wall: Handle<Mesh>,
    pub pellet: Handle<Mesh>,
    pub power_pellet: Handle<Mesh>,
    pub actor: Handle<Mesh>,
    pub eye: Handle<Mesh>,
    pub pupil: Handle<Mesh>,
    pub mouth: Handle<Mesh>,
}

impl GameMeshes {
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        Self {
            wall: meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE)),
            pellet: meshes.add(Circle::new(PELLET_RADIUS)),
            power_pellet: meshes.add(Circle::new(POWER_PELLET_RADIUS)),
            actor: meshes.add(Circle::new(PACMAN_RADIUS)),
            eye: meshes.add(Circle::new(GHOST_RADIUS * 0.24)),
            pupil: meshes.add(Circle::new(GHOST_RADIUS * 0.09)),
            mouth: meshes.add(Triangle2d::new(
                Vec2::ZERO,
                Vec2::new(PACMAN_RADIUS, PACMAN_RADIUS * 0.52),
                Vec2::new(PACMAN_RADIUS, -PACMAN_RADIUS * 0.52),
            )),
        }
    }
}

#[derive(Resource)]
pub struct GameMaterials {
    pub wall: Handle<ColorMaterial>,
    pub pellet: Handle<ColorMaterial>,
    pub pacman: Handle<ColorMaterial>,
    pub frightened_ghost: Handle<ColorMaterial>,
    pub hidden_ghost: Handle<ColorMaterial>,
    pub eye_white: Handle<ColorMaterial>,
    pub eye_pupil: Handle<ColorMaterial>,
    pub mouth_cutout: Handle<ColorMaterial>,
    pub ghost_colors: [Handle<ColorMaterial>; 4],
}

impl GameMaterials {
    pub fn new(materials: &mut Assets<ColorMaterial>) -> Self {
        Self {
            wall: materials.add(WALL_COLOR),
            pellet: materials.add(PELLET_COLOR),
            pacman: materials.add(PACMAN_COLOR),
            frightened_ghost: materials.add(FRIGHTENED_COLOR),
            hidden_ghost: materials.add(BACKGROUND_COLOR),
            eye_white: materials.add(EYE_WHITE_COLOR),
            eye_pupil: materials.add(EYE_PUPIL_COLOR),
            mouth_cutout: materials.add(BACKGROUND_COLOR),
            ghost_colors: [
                materials.add(BLINKY_COLOR),
                materials.add(PINKY_COLOR),
                materials.add(INKY_COLOR),
                materials.add(CLYDE_COLOR),
            ],
        }
    }
}
