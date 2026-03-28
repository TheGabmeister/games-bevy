pub mod gameplay;
pub mod menu;
pub mod winner;

use bevy::prelude::*;

use crate::{
    components::{GameplayEntity, MenuEntity, WinnerEntity},
    state::Phase,
};

pub struct PongSystemsPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameplaySet {
    Movement,
    Collision,
    Scoring,
    Presentation,
}

impl Plugin for PongSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, gameplay::setup_gameplay_assets))
            .configure_sets(
                Update,
                (
                    GameplaySet::Movement,
                    GameplaySet::Collision,
                    GameplaySet::Scoring,
                    GameplaySet::Presentation,
                )
                    .chain(),
            )
            .add_systems(OnEnter(Phase::Menu), menu::spawn_menu)
            .add_systems(Update, menu::menu_input.run_if(in_state(Phase::Menu)))
            .add_systems(OnExit(Phase::Menu), cleanup_menu)
            .add_systems(OnEnter(Phase::Playing), gameplay::spawn_gameplay)
            .add_systems(
                Update,
                (gameplay::move_paddles, gameplay::move_ball)
                    .in_set(GameplaySet::Movement)
                    .run_if(in_state(Phase::Playing)),
            )
            .add_systems(
                Update,
                (gameplay::bounce_from_bounds, gameplay::bounce_from_paddles)
                    .in_set(GameplaySet::Collision)
                    .run_if(in_state(Phase::Playing)),
            )
            .add_systems(
                Update,
                gameplay::handle_score_and_win
                    .in_set(GameplaySet::Scoring)
                    .run_if(in_state(Phase::Playing)),
            )
            .add_systems(
                Update,
                gameplay::update_score_text
                    .in_set(GameplaySet::Presentation)
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
