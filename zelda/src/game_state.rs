use bevy::prelude::*;

use crate::{
    input::InputActions,
    resources::{EquippedItem, Inventory},
    states::AppState,
};

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<Inventory>()
            .add_systems(OnEnter(AppState::Boot), advance_to_title)
            .add_systems(Update, start_from_title.run_if(in_state(AppState::Title)))
            .add_systems(Update, handle_playing_input.run_if(in_state(AppState::Playing)))
            .add_systems(
                Update,
                handle_paused_input.run_if(in_state(AppState::PausedInventory)),
            )
            .add_systems(Update, handle_game_over_input.run_if(in_state(AppState::GameOver)));
    }
}

fn advance_to_title(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Title);
}

fn start_from_title(actions: Res<InputActions>, mut next_state: ResMut<NextState<AppState>>) {
    if actions.confirm || actions.attack {
        next_state.set(AppState::Playing);
    }
}

fn handle_playing_input(
    actions: Res<InputActions>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if actions.pause {
        next_state.set(AppState::PausedInventory);
    } else if actions.cancel {
        next_state.set(AppState::Title);
    }
}

fn handle_paused_input(
    actions: Res<InputActions>,
    mut next_state: ResMut<NextState<AppState>>,
    mut inventory: ResMut<Inventory>,
) {
    if actions.pause || actions.confirm || actions.cancel {
        next_state.set(AppState::Playing);
    } else if actions.attack {
        inventory.equipped = match inventory.equipped {
            None => EquippedItem::ALL.first().copied(),
            Some(item) => item.next(),
        };
    }
}

fn handle_game_over_input(
    actions: Res<InputActions>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if actions.confirm || actions.attack {
        next_state.set(AppState::Playing);
    } else if actions.cancel {
        next_state.set(AppState::Title);
    }
}
