use bevy::prelude::*;

use crate::components::*;
use crate::world::*;
use crate::{AppState, PlayingEnterSet, PlayingSet};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), spawn_title)
            .add_systems(OnExit(AppState::Title), despawn_title)
            .add_systems(Update, title_to_playing.run_if(in_state(AppState::Title)))
            .add_systems(
                OnEnter(AppState::Playing),
                spawn_ui.in_set(PlayingEnterSet::Ui),
            )
            .add_systems(OnExit(AppState::Playing), despawn_ui)
            .add_systems(
                Update,
                update_ui
                    .in_set(PlayingSet::Presentation)
                    .run_if(ui_needs_update),
            )
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over)
            .add_systems(OnExit(AppState::GameOver), despawn_game_over)
            .add_systems(Update, restart_game.run_if(in_state(AppState::GameOver)))
            .add_systems(OnEnter(AppState::Win), spawn_win_screen)
            .add_systems(OnExit(AppState::Win), despawn_win_screen)
            .add_systems(Update, restart_game.run_if(in_state(AppState::Win)));
    }
}

pub fn ui_needs_update(current_room: Res<CurrentRoom>, inventory: Res<PlayerInventory>) -> bool {
    current_room.is_changed() || inventory.is_changed()
}

pub fn title_to_playing(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

pub fn spawn_title(mut commands: Commands) {
    commands.spawn((
        Text::new(
            "ADVENTURE\n\nPress SPACE to Start\n\nFind the Enchanted Chalice\nand return it to the Golden Castle",
        ),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(180.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_self: JustifySelf::Center,
            ..default()
        },
        TitleScreen,
    ));
}

pub fn despawn_title(mut commands: Commands, q: Query<Entity, With<TitleScreen>>) {
    for entity in q.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("Room: ANTECHAMBER"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(10.0),
            ..default()
        },
        RoomNameText,
        GameUi,
    ));

    commands.spawn((
        Text::new("Carrying: nothing"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(8.0),
            left: Val::Px(10.0),
            ..default()
        },
        InventoryText,
        GameUi,
    ));

    commands.spawn((
        Text::new("WASD/Arrows: Move  Space/E: Drop"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(8.0),
            right: Val::Px(10.0),
            ..default()
        },
        GameUi,
    ));
}

pub fn despawn_ui(mut commands: Commands, q: Query<Entity, With<GameUi>>) {
    for entity in q.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn update_ui(
    current_room: Res<CurrentRoom>,
    world: Res<WorldMap>,
    inventory_items: InventoryItems<'_, '_>,
    mut room_text_q: Query<&mut Text, (With<RoomNameText>, Without<InventoryText>)>,
    mut inv_text_q: Query<&mut Text, (With<InventoryText>, Without<RoomNameText>)>,
) {
    if let Ok(mut text) = room_text_q.single_mut() {
        *text = Text::new(format!("Room: {}", world.room(current_room.0).name));
    }

    if let Ok(mut text) = inv_text_q.single_mut() {
        let carrying = inventory_items
            .carried_kind()
            .map(|kind| kind.name())
            .unwrap_or("nothing");
        *text = Text::new(format!("Carrying: {}", carrying));
    }
}

pub fn spawn_game_over(mut commands: Commands) {
    commands.spawn((
        Text::new("EATEN BY A DRAGON!\n\nPress SPACE to try again"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.1, 0.1)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(220.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_self: JustifySelf::Center,
            ..default()
        },
        GameOverScreen,
    ));
}

pub fn despawn_game_over(mut commands: Commands, q: Query<Entity, With<GameOverScreen>>) {
    for entity in q.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_win_screen(mut commands: Commands) {
    commands.spawn((
        Text::new("YOU WIN!\n\nThe Enchanted Chalice is yours!\n\nPress SPACE to play again"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(200.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_self: JustifySelf::Center,
            ..default()
        },
        WinScreen,
    ));
}

pub fn despawn_win_screen(mut commands: Commands, q: Query<Entity, With<WinScreen>>) {
    for entity in q.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn restart_game(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next_state.set(crate::AppState::Title);
    }
}
