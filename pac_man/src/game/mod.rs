mod components;
mod constants;
mod level;
mod logic;
mod resources;
mod systems;

use bevy::{prelude::*, time::Fixed};

use constants::{BACKGROUND_COLOR, FIXED_TIMESTEP_HZ, LEVEL_MAP};
use level::LevelLayout;
use resources::{GameSession, RoundState};

pub struct PacmanPlugin;

impl Plugin for PacmanPlugin {
    fn build(&self, app: &mut App) {
        let layout = LevelLayout::from_ascii(&LEVEL_MAP);
        let pellets_total = layout.pellets_total();

        app.insert_resource(ClearColor(BACKGROUND_COLOR))
            .insert_resource(Time::<Fixed>::from_hz(FIXED_TIMESTEP_HZ))
            .insert_resource(layout)
            .insert_resource(GameSession::new(pellets_total))
            .insert_state(RoundState::Ready)
            .configure_sets(
                Startup,
                (
                    systems::StartupSet::Resources,
                    systems::StartupSet::Scene,
                    systems::StartupSet::Round,
                )
                    .chain(),
            )
            .configure_sets(
                Update,
                (
                    systems::InputSet::Restart,
                    systems::InputSet::Player,
                    systems::PresentationSet::Animation,
                    systems::PresentationSet::Hud,
                )
                    .chain(),
            )
            .configure_sets(
                FixedUpdate,
                (
                    systems::SimulationSet::RoundState,
                    systems::SimulationSet::GhostPlanning,
                    systems::SimulationSet::Movement,
                    systems::SimulationSet::Interaction,
                )
                    .chain(),
            )
            .add_systems(Startup, systems::setup_assets.in_set(systems::StartupSet::Resources))
            .add_systems(
                Startup,
                systems::setup_scene.in_set(systems::StartupSet::Scene),
            )
            .add_systems(
                Startup,
                systems::spawn_round.in_set(systems::StartupSet::Round),
            )
            .add_systems(OnEnter(RoundState::Ready), systems::enter_ready_phase)
            .add_systems(OnEnter(RoundState::Won), systems::clear_round_effects)
            .add_systems(OnEnter(RoundState::GameOver), systems::clear_round_effects)
            .add_systems(
                Update,
                systems::handle_restart_input
                    .in_set(systems::InputSet::Restart)
                    .run_if(in_state(RoundState::Won).or(in_state(RoundState::GameOver))),
            )
            .add_systems(
                Update,
                systems::handle_player_input
                    .in_set(systems::InputSet::Player)
                    .run_if(in_state(RoundState::Ready).or(in_state(RoundState::Playing))),
            )
            .add_systems(
                Update,
                (
                    systems::sync_ghost_appearance,
                    systems::animate_pacman,
                    systems::animate_power_pellets,
                )
                    .in_set(systems::PresentationSet::Animation),
            )
            .add_systems(
                Update,
                systems::update_hud
                    .in_set(systems::PresentationSet::Hud)
                    .run_if(resource_changed::<GameSession>.or(state_changed::<RoundState>)),
            )
            .add_systems(
                FixedUpdate,
                systems::advance_ready_timer
                    .in_set(systems::SimulationSet::RoundState)
                    .run_if(in_state(RoundState::Ready)),
            )
            .add_systems(
                FixedUpdate,
                systems::advance_playing_timers
                    .in_set(systems::SimulationSet::RoundState)
                    .run_if(in_state(RoundState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (systems::plan_ghost_turns,)
                    .in_set(systems::SimulationSet::GhostPlanning)
                    .run_if(in_state(RoundState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (systems::move_player, systems::move_ghosts)
                    .in_set(systems::SimulationSet::Movement)
                    .run_if(in_state(RoundState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (systems::collect_pellets, systems::resolve_ghost_collisions)
                    .in_set(systems::SimulationSet::Interaction)
                    .run_if(in_state(RoundState::Playing)),
            );
    }
}
