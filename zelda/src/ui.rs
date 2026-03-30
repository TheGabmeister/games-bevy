use bevy::prelude::*;

use crate::{rendering::WorldColor, states::AppState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), spawn_title_screen)
            .add_systems(OnEnter(AppState::Playing), spawn_playing_overlay)
            .add_systems(OnEnter(AppState::PausedInventory), spawn_pause_overlay);
    }
}

fn spawn_title_screen(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(AppState::Title),
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("THE LEGEND OF ZELDA"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(WorldColor::Accent.color()),
            ));
            parent.spawn((
                Text::new("Primitive prototype bootstrap"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(WorldColor::UiText.color()),
            ));
            parent.spawn((
                Text::new("Press Enter, Space, or Gamepad South to start"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(WorldColor::UiText.color()),
            ));
        });
}

fn spawn_playing_overlay(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        Text::new(
            "Move through open doorways to transition rooms | Transition lock keeps motion frozen briefly between rooms",
        ),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(WorldColor::UiText.color()),
        Node {
            position_type: PositionType::Absolute,
            top: px(14),
            left: px(16),
            ..default()
        },
    ));

    commands.spawn((
        DespawnOnExit(AppState::Playing),
        Text::new(
            "Yellow pickups persist per room. Orange pickups reset on reload. Center room bush still hides the secret.",
        ),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(WorldColor::Accent.color()),
        Node {
            position_type: PositionType::Absolute,
            top: px(38),
            left: px(16),
            ..default()
        },
    ));
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(AppState::PausedInventory),
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.06, 0.08, 0.82)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED\nPress Tab, Enter, or Esc to return"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(WorldColor::UiText.color()),
            ));
        });
}
