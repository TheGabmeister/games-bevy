mod collision;
mod components;
mod constants;
mod lanes;
mod player;
mod resources;
mod states;
mod ui;

use bevy::prelude::*;

use constants::*;
use resources::*;
use states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                resolution: bevy::window::WindowResolution::new(
                    WINDOW_WIDTH as u32,
                    WINDOW_HEIGHT as u32,
                ),
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .init_resource::<GameData>()
        .init_resource::<FrogTimer>()
        .init_resource::<LevelState>()
        .init_resource::<FrogEvent>()
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .add_plugins((player::PlayerPlugin, lanes::LanesPlugin, ui::UiPlugin))
        // All gameplay systems with explicit ordering
        .add_systems(
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
                lanes::update_bay_visuals.after(collision::handle_frog_event),
                player::score_forward_hop.after(collision::handle_frog_event),
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
