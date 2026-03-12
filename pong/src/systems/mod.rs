pub mod gameplay;
pub mod menu;
pub mod winner;

use bevy::prelude::*;

use crate::{
    components::{GameplayEntity, MenuEntity, WinnerEntity},
    state::Phase,
};

pub struct PongSystemsPlugin;

impl Plugin for PongSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(OnEnter(Phase::Menu), menu::spawn_menu)
            .add_systems(Update, menu::menu_input.run_if(in_state(Phase::Menu)))
            .add_systems(OnExit(Phase::Menu), cleanup_menu)
            .add_systems(OnEnter(Phase::Playing), gameplay::spawn_gameplay)
            .add_systems(
                Update,
                (
                    gameplay::move_paddles,
                    gameplay::move_ball,
                    gameplay::bounce_from_bounds,
                    gameplay::bounce_from_paddles,
                    gameplay::handle_score_and_win,
                    gameplay::update_score_text,
                )
                    .chain()
                    .run_if(in_state(Phase::Playing)),
            )
            .add_systems(OnExit(Phase::Playing), cleanup_gameplay)
            .add_systems(OnEnter(Phase::Winner), winner::spawn_winner_screen)
            .add_systems(Update, winner::winner_input.run_if(in_state(Phase::Winner)))
            .add_systems(OnExit(Phase::Winner), cleanup_winner);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_gameplay(mut commands: Commands, query: Query<Entity, With<GameplayEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_winner(mut commands: Commands, query: Query<Entity, With<WinnerEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
