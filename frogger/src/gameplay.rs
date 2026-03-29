use bevy::prelude::*;

use crate::collision;
use crate::lanes;
use crate::player;
use crate::states::AppState;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player::frog_input,
                player::hop_animation.after(player::frog_input),
                lanes::move_lane_objects,
                collision::ride_platforms
                    .after(player::hop_animation)
                    .after(lanes::move_lane_objects),
                collision::check_vehicle_collision.after(collision::ride_platforms),
                collision::check_water_support.after(collision::ride_platforms),
                collision::check_bounds.after(collision::ride_platforms),
                collision::check_home_bay.after(collision::ride_platforms),
                collision::tick_timer,
                collision::handle_frog_event
                    .after(collision::check_vehicle_collision)
                    .after(collision::check_water_support)
                    .after(collision::check_bounds)
                    .after(collision::check_home_bay)
                    .after(collision::tick_timer),
                collision::check_level_clear.after(collision::handle_frog_event),
                lanes::update_bay_visuals.after(collision::check_level_clear),
                player::score_forward_hop.after(collision::handle_frog_event),
                collision::advance_level_clear
                    .after(lanes::update_bay_visuals)
                    .after(player::score_forward_hop),
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}
