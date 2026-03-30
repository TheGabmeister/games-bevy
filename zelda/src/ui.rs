use bevy::prelude::*;

use crate::{
    rendering::WorldColor,
    resources::{EquippedItem, Inventory, PlayerVitals},
    states::AppState,
};

pub struct UiPlugin;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum HudElement {
    Hearts,
    Rupees,
    Bombs,
    Keys,
    Equipped,
}

#[derive(Component)]
struct PauseEquippedText;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), spawn_title_screen)
            .add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(Update, update_hud.run_if(in_state(AppState::Playing)))
            .add_systems(OnEnter(AppState::PausedInventory), spawn_pause_overlay)
            .add_systems(
                Update,
                update_pause_equipped.run_if(in_state(AppState::PausedInventory)),
            )
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over_overlay);
    }
}

// === Title Screen ===

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

// === HUD ===

fn spawn_hud(mut commands: Commands, vitals: Res<PlayerVitals>, inventory: Res<Inventory>) {
    // Life label
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        Text::new("-LIFE-"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(WorldColor::UiText.color()),
        Node {
            position_type: PositionType::Absolute,
            top: px(20),
            left: px(50),
            ..default()
        },
    ));

    // Hearts
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        HudElement::Hearts,
        Text::new(hearts_string(vitals.current_health, vitals.max_health)),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.2, 0.25)),
        Node {
            position_type: PositionType::Absolute,
            top: px(50),
            left: px(50),
            ..default()
        },
    ));

    // USE label
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        Text::new("USE"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(WorldColor::UiText.color()),
        Node {
            position_type: PositionType::Absolute,
            top: px(20),
            left: px(450),
            ..default()
        },
    ));

    // Equipped item
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        HudElement::Equipped,
        Text::new(equipped_string(inventory.equipped)),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(WorldColor::Accent.color()),
        Node {
            position_type: PositionType::Absolute,
            top: px(50),
            left: px(440),
            ..default()
        },
    ));

    // Rupee counter
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        HudElement::Rupees,
        Text::new(format!("RUPEES  x{:03}", inventory.rupees)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(WorldColor::Pickup.color()),
        Node {
            position_type: PositionType::Absolute,
            top: px(35),
            left: px(650),
            ..default()
        },
    ));

    // Bomb counter
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        HudElement::Bombs,
        Text::new(format!("BOMBS   x{:02}", inventory.bombs)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(WorldColor::UiText.color()),
        Node {
            position_type: PositionType::Absolute,
            top: px(85),
            left: px(650),
            ..default()
        },
    ));

    // Key counter
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        HudElement::Keys,
        Text::new(format!("KEYS    x{:02}", inventory.keys)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(WorldColor::UiText.color()),
        Node {
            position_type: PositionType::Absolute,
            top: px(135),
            left: px(650),
            ..default()
        },
    ));
}

fn update_hud(
    vitals: Res<PlayerVitals>,
    inventory: Res<Inventory>,
    mut elements: Query<(&HudElement, &mut Text)>,
) {
    for (element, mut text) in &mut elements {
        match element {
            HudElement::Hearts => {
                *text = Text::new(hearts_string(vitals.current_health, vitals.max_health));
            }
            HudElement::Rupees => {
                *text = Text::new(format!("RUPEES  x{:03}", inventory.rupees));
            }
            HudElement::Bombs => {
                *text = Text::new(format!("BOMBS   x{:02}", inventory.bombs));
            }
            HudElement::Keys => {
                *text = Text::new(format!("KEYS    x{:02}", inventory.keys));
            }
            HudElement::Equipped => {
                *text = Text::new(equipped_string(inventory.equipped));
            }
        }
    }
}

fn hearts_string(current: u8, max: u8) -> String {
    let heart_slots = (max + 1) / 2;
    let mut s = String::new();
    let mut hp_left = current;
    for i in 0..heart_slots {
        if i > 0 && i % 8 == 0 {
            s.push('\n');
        } else if i > 0 {
            s.push(' ');
        }
        if hp_left >= 2 {
            s.push_str("<3");
            hp_left -= 2;
        } else if hp_left == 1 {
            s.push_str("<:");
            hp_left = 0;
        } else {
            s.push_str("<>");
        }
    }
    s
}

fn equipped_string(equipped: Option<EquippedItem>) -> String {
    match equipped {
        Some(item) => item.display_name().to_string(),
        None => "----".to_string(),
    }
}

// === Pause ===

fn spawn_pause_overlay(mut commands: Commands, inventory: Res<Inventory>) {
    commands
        .spawn((
            DespawnOnExit(AppState::PausedInventory),
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.06, 0.08, 0.82)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(WorldColor::UiText.color()),
            ));
            parent.spawn((
                PauseEquippedText,
                Text::new(format!(
                    "Equipped: {}",
                    equipped_string(inventory.equipped)
                )),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(WorldColor::Accent.color()),
            ));
            parent.spawn((
                Text::new("Z: cycle item  |  Tab/Enter/Esc: resume"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(WorldColor::UiText.color()),
            ));
        });
}

fn update_pause_equipped(
    inventory: Res<Inventory>,
    mut text_query: Query<&mut Text, With<PauseEquippedText>>,
) {
    for mut text in &mut text_query {
        *text = Text::new(format!(
            "Equipped: {}",
            equipped_string(inventory.equipped)
        ));
    }
}

// === Game Over ===

fn spawn_game_over_overlay(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(AppState::GameOver),
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.02, 0.02, 0.84)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("YOU DIED\nPress Enter or Z to continue\nPress Esc for title"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(WorldColor::UiText.color()),
            ));
        });
}
