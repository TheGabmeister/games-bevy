use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::AppState;

pub struct LanesPlugin;

impl Plugin for LanesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Playing),
            (spawn_world, spawn_lane_objects),
        )
        .add_systems(OnExit(AppState::Playing), cleanup_gameplay);
    }
}

// --- Lane configuration ---

struct LaneConfig {
    row: i32,
    is_river: bool,
    direction: f32,
    base_speed: f32,
    object_width_cells: i32,
    object_count: i32,
    color_index: usize,
}

const LANE_CONFIGS: &[LaneConfig] = &[
    // Road lanes
    LaneConfig {
        row: 1,
        is_river: false,
        direction: 1.0,
        base_speed: 60.0,
        object_width_cells: 1,
        object_count: 4,
        color_index: 0,
    },
    LaneConfig {
        row: 2,
        is_river: false,
        direction: -1.0,
        base_speed: 80.0,
        object_width_cells: 1,
        object_count: 3,
        color_index: 1,
    },
    LaneConfig {
        row: 3,
        is_river: false,
        direction: 1.0,
        base_speed: 120.0,
        object_width_cells: 2,
        object_count: 2,
        color_index: 2,
    },
    LaneConfig {
        row: 4,
        is_river: false,
        direction: -1.0,
        base_speed: 50.0,
        object_width_cells: 3,
        object_count: 2,
        color_index: 3,
    },
    LaneConfig {
        row: 5,
        is_river: false,
        direction: 1.0,
        base_speed: 110.0,
        object_width_cells: 1,
        object_count: 3,
        color_index: 4,
    },
    // River lanes
    LaneConfig {
        row: 7,
        is_river: true,
        direction: -1.0,
        base_speed: 55.0,
        object_width_cells: 4,
        object_count: 2,
        color_index: 0,
    },
    LaneConfig {
        row: 8,
        is_river: true,
        direction: 1.0,
        base_speed: 45.0,
        object_width_cells: 3,
        object_count: 3,
        color_index: 0,
    },
    LaneConfig {
        row: 9,
        is_river: true,
        direction: -1.0,
        base_speed: 80.0,
        object_width_cells: 4,
        object_count: 2,
        color_index: 0,
    },
    LaneConfig {
        row: 10,
        is_river: true,
        direction: 1.0,
        base_speed: 55.0,
        object_width_cells: 2,
        object_count: 4,
        color_index: 0,
    },
    LaneConfig {
        row: 11,
        is_river: true,
        direction: -1.0,
        base_speed: 65.0,
        object_width_cells: 3,
        object_count: 2,
        color_index: 0,
    },
];

// --- Spawning ---

fn spawn_world(mut commands: Commands) {
    // Background rows
    for row in 0..PLAYFIELD_ROWS {
        let y = row_to_world_y(row);
        let color = match row {
            0 | 6 => COLOR_SAFE_ZONE,
            1..=5 => COLOR_ROAD,
            7..=11 => COLOR_RIVER,
            12 => COLOR_HOME_WALL,
            _ => COLOR_BACKGROUND,
        };

        commands.spawn((
            GameplayEntity,
            Sprite {
                color,
                custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, CELL_SIZE)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y, 0.0)),
        ));
    }

    // Home bay openings (dark slots on top of the green home wall)
    for (i, &col) in HOME_BAY_COLS.iter().enumerate() {
        let pos = grid_to_world(col, HOME_ROW);
        commands.spawn((
            GameplayEntity,
            HomeBay { index: i },
            Sprite {
                color: COLOR_HOME_BAY_OPEN,
                custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                ..default()
            },
            Transform::from_translation(pos.extend(0.5)),
        ));
    }
}

fn spawn_lane_objects(mut commands: Commands) {
    let wrap_left = -PLAYFIELD_WIDTH / 2.0 - WRAP_MARGIN;
    let wrap_right = PLAYFIELD_WIDTH / 2.0 + WRAP_MARGIN;
    let virtual_width = wrap_right - wrap_left;

    for config in LANE_CONFIGS {
        let y = row_to_world_y(config.row);
        let obj_width = config.object_width_cells as f32 * CELL_SIZE;
        let spacing = virtual_width / config.object_count as f32;

        let color = if config.is_river {
            COLOR_LOG
        } else {
            VEHICLE_COLORS[config.color_index]
        };

        for i in 0..config.object_count {
            let x = wrap_left + spacing * (i as f32 + 0.5);

            let mut entity = commands.spawn((
                GameplayEntity,
                LaneObject,
                Velocity(Vec2::new(config.direction * config.base_speed, 0.0)),
                ObjectWidth(obj_width),
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(obj_width, CELL_SIZE - 4.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 1.0)),
            ));

            if config.is_river {
                entity.insert(Platform);
            } else {
                entity.insert(Vehicle);
            }
        }
    }
}

// --- Update systems ---

pub fn move_lane_objects(
    time: Res<Time>,
    level_state: Res<LevelState>,
    mut query: Query<(&Velocity, &mut Transform), With<LaneObject>>,
) {
    let wrap_left = -PLAYFIELD_WIDTH / 2.0 - WRAP_MARGIN;
    let wrap_right = PLAYFIELD_WIDTH / 2.0 + WRAP_MARGIN;
    let virtual_width = wrap_right - wrap_left;

    for (velocity, mut transform) in &mut query {
        transform.translation.x += velocity.0.x * level_state.speed_multiplier * time.delta_secs();

        if transform.translation.x > wrap_right {
            transform.translation.x -= virtual_width;
        } else if transform.translation.x < wrap_left {
            transform.translation.x += virtual_width;
        }
    }
}

pub fn update_bay_visuals(game_data: Res<GameData>, mut query: Query<(&HomeBay, &mut Sprite)>) {
    for (bay, mut sprite) in &mut query {
        sprite.color = if game_data.filled_bays[bay.index] {
            COLOR_FILLED_BAY
        } else {
            COLOR_HOME_BAY_OPEN
        };
    }
}

// --- Cleanup ---

fn cleanup_gameplay(mut commands: Commands, query: Query<Entity, With<GameplayEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
