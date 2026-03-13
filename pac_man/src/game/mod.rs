mod components;
mod constants;
mod level;
mod resources;
mod systems;

use bevy::prelude::*;

use constants::LEVEL_MAP;
use level::LevelLayout;

pub struct PacmanPlugin;

impl Plugin for PacmanPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(constants::BACKGROUND_COLOR))
            .insert_resource(LevelLayout::from_ascii(&LEVEL_MAP))
            .add_systems(Startup, systems::setup_game)
            .add_systems(
                Update,
                (
                    systems::handle_restart_input,
                    systems::tick_round_state,
                    systems::handle_player_input,
                    systems::plan_ghost_turns,
                    systems::move_player,
                    systems::move_ghosts,
                    systems::collect_pellets,
                    systems::resolve_ghost_collisions,
                    systems::sync_ghost_appearance,
                    systems::animate_pacman,
                    systems::animate_power_pellets,
                    systems::update_hud,
                )
                    .chain(),
            );
    }
}
