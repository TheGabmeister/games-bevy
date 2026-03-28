use bevy::prelude::*;
use bevy::ecs::system::SystemParam;

use crate::game::{
    components::{Direction, GridMover, Player, RoundEntity},
    level::LevelLayout,
    resources::{GameMaterials, GameMeshes, GameSession, RoundState},
    systems::setup::{cleanup_round_entities, spawn_round_entities},
};

#[derive(SystemParam)]
pub struct RestartRoundContext<'w, 's> {
    commands: Commands<'w, 's>,
    round_roots: Query<'w, 's, Entity, (With<RoundEntity>, Without<ChildOf>)>,
    layout: Res<'w, LevelLayout>,
    meshes: Res<'w, GameMeshes>,
    materials: Res<'w, GameMaterials>,
    session: ResMut<'w, GameSession>,
    next_state: ResMut<'w, NextState<RoundState>>,
}

pub fn handle_restart_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ctx: RestartRoundContext,
) {
    if !(keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter)) {
        return;
    }

    cleanup_round_entities(&mut ctx.commands, &ctx.round_roots);
    ctx.session.reset_for_new_game(ctx.layout.pellets_total());
    spawn_round_entities(&mut ctx.commands, &ctx.layout, &ctx.meshes, &ctx.materials);
    ctx.next_state.set(RoundState::Ready);
}

pub fn handle_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Single<&mut GridMover, With<Player>>,
) {
    let Some(direction) = latest_direction_input(&keyboard) else {
        return;
    };

    player.desired = Some(direction);
}

fn latest_direction_input(keyboard: &ButtonInput<KeyCode>) -> Option<Direction> {
    for (key, direction) in [
        (KeyCode::ArrowUp, Direction::Up),
        (KeyCode::ArrowLeft, Direction::Left),
        (KeyCode::ArrowDown, Direction::Down),
        (KeyCode::ArrowRight, Direction::Right),
        (KeyCode::KeyW, Direction::Up),
        (KeyCode::KeyA, Direction::Left),
        (KeyCode::KeyS, Direction::Down),
        (KeyCode::KeyD, Direction::Right),
    ] {
        if keyboard.just_pressed(key) {
            return Some(direction);
        }
    }

    for (key, direction) in [
        (KeyCode::ArrowUp, Direction::Up),
        (KeyCode::ArrowLeft, Direction::Left),
        (KeyCode::ArrowDown, Direction::Down),
        (KeyCode::ArrowRight, Direction::Right),
        (KeyCode::KeyW, Direction::Up),
        (KeyCode::KeyA, Direction::Left),
        (KeyCode::KeyS, Direction::Down),
        (KeyCode::KeyD, Direction::Right),
    ] {
        if keyboard.pressed(key) {
            return Some(direction);
        }
    }

    None
}
