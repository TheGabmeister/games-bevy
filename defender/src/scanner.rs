use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::components::*;
use crate::constants::*;
use crate::scheduling::GameplaySet;

pub struct ScannerPlugin;

impl Plugin for ScannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scanner)
            .add_systems(Update, scanner_update.in_set(GameplaySet::Post));
    }
}

pub fn setup_scanner(mut commands: Commands) {
    // Scanner bar background
    commands.spawn((
        ScannerBar,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px((SCREEN_WIDTH as f32 - SCANNER_WIDTH) / 2.0),
            width: Val::Px(SCANNER_WIDTH),
            height: Val::Px(SCANNER_HEIGHT),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(COLOR_SCANNER_BG),
    ));
}

pub fn scanner_update(
    mut commands: Commands,
    scanner_bar: Query<Entity, With<ScannerBar>>,
    mut existing_dots: Query<(Entity, &ScannerDot, &mut Node, &mut BackgroundColor)>,
    // All trackable entities
    players: Query<(Entity, &WorldPosition), With<Player>>,
    humans: Query<(Entity, &WorldPosition), With<Human>>,
    landers: Query<(Entity, &WorldPosition), With<Lander>>,
    mutants: Query<(Entity, &WorldPosition), With<Mutant>>,
    bombers: Query<(Entity, &WorldPosition), With<Bomber>>,
    pods: Query<(Entity, &WorldPosition), With<Pod>>,
    swarmers: Query<(Entity, &WorldPosition), With<Swarmer>>,
    baiters: Query<(Entity, &WorldPosition), With<Baiter>>,
) {
    let Ok(bar_entity) = scanner_bar.single() else {
        return;
    };

    let mut trackables = HashMap::new();
    for (entity, wp) in &players {
        trackables.insert(entity, (wp.0, Color::WHITE));
    }
    for (entity, wp) in &humans {
        trackables.insert(entity, (wp.0, COLOR_HUMAN));
    }
    for (entity, wp) in &landers {
        trackables.insert(entity, (wp.0, COLOR_LANDER));
    }
    for (entity, wp) in &mutants {
        trackables.insert(entity, (wp.0, COLOR_MUTANT));
    }
    for (entity, wp) in &bombers {
        trackables.insert(entity, (wp.0, COLOR_BOMBER));
    }
    for (entity, wp) in &pods {
        trackables.insert(entity, (wp.0, COLOR_POD));
    }
    for (entity, wp) in &swarmers {
        trackables.insert(entity, (wp.0, COLOR_SWARMER));
    }
    for (entity, wp) in &baiters {
        trackables.insert(entity, (wp.0, COLOR_BAITER));
    }

    let spawn_dot = |commands: &mut Commands, world_x: f32, color: Color, tracked: Entity| {
        let left = (world_x / WORLD_WIDTH) * SCANNER_WIDTH;
        let dot = commands
            .spawn((
                ScannerDot(tracked),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(left),
                    top: Val::Px(SCANNER_HEIGHT / 2.0 - 1.5),
                    width: Val::Px(3.0),
                    height: Val::Px(3.0),
                    ..default()
                },
                BackgroundColor(color),
            ))
            .id();
        commands.entity(bar_entity).add_child(dot);
    };

    let mut seen = HashSet::new();
    for (dot_entity, tracked, mut node, mut bg) in &mut existing_dots {
        if let Some((world_x, color)) = trackables.get(&tracked.0) {
            node.left = Val::Px((world_x / WORLD_WIDTH) * SCANNER_WIDTH);
            *bg = BackgroundColor(*color);
            seen.insert(tracked.0);
        } else {
            commands.entity(dot_entity).despawn();
        }
    }

    for (entity, (world_x, color)) in trackables {
        if !seen.contains(&entity) {
            spawn_dot(&mut commands, world_x, color, entity);
        }
    }
}
