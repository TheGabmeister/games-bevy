mod flow;
mod input;
mod presentation;
mod setup;
mod simulation;

use bevy::prelude::*;

pub use flow::{advance_playing_timers, advance_ready_timer, clear_round_effects, enter_ready_phase};
pub use input::{handle_player_input, handle_restart_input};
pub use presentation::{animate_pacman, animate_power_pellets, sync_ghost_appearance, update_hud};
pub use setup::{setup_assets, setup_scene, spawn_round};
pub use simulation::{collect_pellets, move_ghosts, move_player, plan_ghost_turns, resolve_ghost_collisions};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum StartupSet {
    Resources,
    Scene,
    Round,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InputSet {
    Restart,
    Player,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SimulationSet {
    RoundState,
    GhostPlanning,
    Movement,
    Interaction,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum PresentationSet {
    Animation,
    Hud,
}
