use bevy::prelude::*;

use crate::{
    rendering::WorldColor,
    resources::{DialogueState, DungeonState, EquippedItem, Inventory, PlayerVitals},
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

#[derive(Component)]
struct DialogueOverlay;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), spawn_title_screen)
            .add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(
                Update,
                (update_hud, update_dialogue_overlay).run_if(in_state(AppState::Playing)),
            )
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

fn spawn_pause_overlay(
    mut commands: Commands,
    inventory: Res<Inventory>,
    dungeon_state: Res<DungeonState>,
) {
    let mut items_parts: Vec<&str> = Vec::new();
    if inventory.has_sword {
        items_parts.push("SWORD");
    }
    let items_line = if items_parts.is_empty() {
        format!("RUPEES: {}  BOMBS: {}  KEYS: {}", inventory.rupees, inventory.bombs, inventory.keys)
    } else {
        format!(
            "{}  |  RUPEES: {}  BOMBS: {}  KEYS: {}",
            items_parts.join("  "),
            inventory.rupees,
            inventory.bombs,
            inventory.keys,
        )
    };

    let dungeon_line = if let Some(dungeon) = dungeon_state.current_dungeon {
        let mut parts = vec![format!("DUNGEON KEYS: {}", dungeon_state.keys_for_current())];
        if dungeon_state.has_map.contains(&dungeon) {
            parts.push("MAP".to_string());
        }
        if dungeon_state.has_compass.contains(&dungeon) {
            parts.push("COMPASS".to_string());
        }
        Some(parts.join("  "))
    } else {
        None
    };

    let triforce_count = dungeon_state.triforce_pieces.len();

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
                Text::new("INVENTORY"),
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
                Text::new(items_line),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(WorldColor::UiText.color()),
            ));
            if let Some(line) = dungeon_line {
                parent.spawn((
                    Text::new(line),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(WorldColor::Accent.color()),
                ));
            }
            if triforce_count > 0 {
                parent.spawn((
                    Text::new(format!("TRIFORCE: {}", triforce_count)),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(WorldColor::Accent.color()),
                ));
            }
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

// === Dialogue Overlay ===

fn update_dialogue_overlay(
    mut commands: Commands,
    dialogue: Res<DialogueState>,
    existing: Query<Entity, With<DialogueOverlay>>,
) {
    if dialogue.is_active() {
        let dialogue_text = dialogue.current_text().to_string();
        if let Ok(entity) = existing.single() {
            // Update existing overlay text
            commands.entity(entity).despawn();
        }
        commands
            .spawn((
                DialogueOverlay,
                DespawnOnExit(AppState::Playing),
                Node {
                    width: percent(100),
                    height: px(80),
                    position_type: PositionType::Absolute,
                    bottom: px(0),
                    left: px(0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.02, 0.08, 0.9)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(dialogue_text),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(WorldColor::UiText.color()),
                ));
            });
    } else {
        // Dialogue not active — despawn overlay if it exists
        if let Ok(entity) = existing.single() {
            commands.entity(entity).despawn();
        }
    }
}
