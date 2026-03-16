use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;

pub fn setup_scanner(mut commands: Commands) {
    // Scanner bar background
    commands
        .spawn((
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
    existing_dots: Query<(Entity, &ScannerDot)>,
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

    // Remove old dots
    for (dot_entity, _) in &existing_dots {
        commands.entity(dot_entity).despawn();
    }

    // Helper to spawn a dot
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

    // Player dots (white)
    for (entity, wp) in &players {
        spawn_dot(&mut commands, wp.0, Color::WHITE, entity);
    }

    // Human dots (green)
    for (entity, wp) in &humans {
        spawn_dot(&mut commands, wp.0, COLOR_HUMAN, entity);
    }

    // Enemy dots
    for (entity, wp) in &landers {
        spawn_dot(&mut commands, wp.0, COLOR_LANDER, entity);
    }
    for (entity, wp) in &mutants {
        spawn_dot(&mut commands, wp.0, COLOR_MUTANT, entity);
    }
    for (entity, wp) in &bombers {
        spawn_dot(&mut commands, wp.0, COLOR_BOMBER, entity);
    }
    for (entity, wp) in &pods {
        spawn_dot(&mut commands, wp.0, COLOR_POD, entity);
    }
    for (entity, wp) in &swarmers {
        spawn_dot(&mut commands, wp.0, COLOR_SWARMER, entity);
    }
    for (entity, wp) in &baiters {
        spawn_dot(&mut commands, wp.0, COLOR_BAITER, entity);
    }
}
