use std::f32::consts::FRAC_PI_2;

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
    LaneConfig { row: 1, is_river: false, direction: 1.0, base_speed: 60.0, object_width_cells: 1, object_count: 4, color_index: 0 },
    LaneConfig { row: 2, is_river: false, direction: -1.0, base_speed: 80.0, object_width_cells: 1, object_count: 3, color_index: 1 },
    LaneConfig { row: 3, is_river: false, direction: 1.0, base_speed: 120.0, object_width_cells: 2, object_count: 2, color_index: 2 },
    LaneConfig { row: 4, is_river: false, direction: -1.0, base_speed: 50.0, object_width_cells: 3, object_count: 2, color_index: 3 },
    LaneConfig { row: 5, is_river: false, direction: 1.0, base_speed: 110.0, object_width_cells: 1, object_count: 3, color_index: 4 },
    // River lanes
    LaneConfig { row: 7, is_river: true, direction: -1.0, base_speed: 55.0, object_width_cells: 4, object_count: 2, color_index: 0 },
    LaneConfig { row: 8, is_river: true, direction: 1.0, base_speed: 45.0, object_width_cells: 3, object_count: 3, color_index: 0 },
    LaneConfig { row: 9, is_river: true, direction: -1.0, base_speed: 80.0, object_width_cells: 4, object_count: 2, color_index: 0 },
    LaneConfig { row: 10, is_river: true, direction: 1.0, base_speed: 55.0, object_width_cells: 2, object_count: 4, color_index: 0 },
    LaneConfig { row: 11, is_river: true, direction: -1.0, base_speed: 65.0, object_width_cells: 3, object_count: 2, color_index: 0 },
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

    // Road lane markings (dashed white lines between road rows)
    for row in 1..5 {
        let y = (row_to_world_y(row) + row_to_world_y(row + 1)) / 2.0;
        let total_span = PLAYFIELD_WIDTH;
        let step = ROAD_MARKING_WIDTH + ROAD_MARKING_SPACING;
        let count = (total_span / step) as i32 + 1;
        let start_x = -total_span / 2.0 + ROAD_MARKING_WIDTH / 2.0;

        for i in 0..count {
            let x = start_x + i as f32 * step;
            commands.spawn((
                GameplayEntity,
                Sprite {
                    color: COLOR_ROAD_MARKING,
                    custom_size: Some(Vec2::new(ROAD_MARKING_WIDTH, ROAD_MARKING_HEIGHT)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, 0.1)),
            ));
        }
    }

    // Home bay openings
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

fn spawn_lane_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let wrap_left = -PLAYFIELD_WIDTH / 2.0 - WRAP_MARGIN;
    let wrap_right = PLAYFIELD_WIDTH / 2.0 + WRAP_MARGIN;
    let virtual_width = wrap_right - wrap_left;

    for config in LANE_CONFIGS {
        let y = row_to_world_y(config.row);
        let obj_width = config.object_width_cells as f32 * CELL_SIZE;
        let obj_height = CELL_SIZE - 4.0;
        let spacing = virtual_width / config.object_count as f32;

        for i in 0..config.object_count {
            let x = wrap_left + spacing * (i as f32 + 0.5);

            if config.is_river {
                spawn_log(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    x,
                    y,
                    obj_width,
                    obj_height,
                    config.direction * config.base_speed,
                );
            } else {
                spawn_vehicle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    x,
                    y,
                    obj_width,
                    obj_height,
                    config.direction,
                    config.base_speed,
                    config.color_index,
                    config.object_width_cells,
                );
            }
        }
    }
}

fn spawn_log(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    speed: f32,
) {
    let log_radius = height / 2.0;
    let capsule_length = (width - 2.0 * log_radius).max(0.0);

    commands
        .spawn((
            GameplayEntity,
            LaneObject,
            Platform,
            Velocity(Vec2::new(speed, 0.0)),
            ObjectWidth(width),
            Transform::from_translation(Vec3::new(x, y, 1.0)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Capsule body (rotated 90 deg for horizontal)
            parent.spawn((
                Mesh2d(meshes.add(Capsule2d::new(log_radius, capsule_length))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_LOG))),
                Transform::from_rotation(Quat::from_rotation_z(FRAC_PI_2)),
            ));

            // Bark stripes
            let stripe_count = (width / CELL_SIZE).ceil() as i32;
            for s in 0..stripe_count {
                let stripe_x =
                    -width / 2.0 + CELL_SIZE * 0.5 + s as f32 * CELL_SIZE - CELL_SIZE * 0.15;
                parent.spawn((
                    Sprite {
                        color: COLOR_LOG_BARK,
                        custom_size: Some(Vec2::new(3.0, height - 12.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(stripe_x, 0.0, 0.1)),
                ));
            }
        });
}

fn spawn_vehicle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    direction: f32,
    base_speed: f32,
    color_index: usize,
    width_cells: i32,
) {
    let color = VEHICLE_COLORS[color_index];

    let mut entity = commands.spawn((
        GameplayEntity,
        LaneObject,
        Vehicle,
        Velocity(Vec2::new(direction * base_speed, 0.0)),
        ObjectWidth(width),
        Sprite {
            color,
            custom_size: Some(Vec2::new(width, height)),
            ..default()
        },
        Transform::from_translation(Vec3::new(x, y, 1.0)),
    ));

    entity.with_children(|parent| {
        // Windshield
        let win_w = if width_cells > 1 {
            CELL_SIZE * 0.5
        } else {
            width * 0.3
        };
        let win_x = (width / 2.0 - win_w / 2.0 - 4.0) * direction;
        parent.spawn((
            Sprite {
                color: COLOR_VEHICLE_WINDOW,
                custom_size: Some(Vec2::new(win_w, height * 0.55)),
                ..default()
            },
            Transform::from_translation(Vec3::new(win_x, 0.0, 0.1)),
        ));

        // Headlights (circles at the front)
        let hl_x = (width / 2.0 - 3.0) * direction;
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(VEHICLE_HEADLIGHT_RADIUS))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_VEHICLE_HEADLIGHT))),
            Transform::from_translation(Vec3::new(hl_x, height * 0.28, 0.2)),
        ));
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(VEHICLE_HEADLIGHT_RADIUS))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_VEHICLE_HEADLIGHT))),
            Transform::from_translation(Vec3::new(hl_x, -height * 0.28, 0.2)),
        ));

        // Cab section for trucks
        if width_cells > 1 {
            let cab_w = CELL_SIZE * 0.8;
            let cab_x = (width / 2.0 - cab_w / 2.0 - 1.0) * direction;
            parent.spawn((
                Sprite {
                    color: VEHICLE_CAB_COLORS[color_index],
                    custom_size: Some(Vec2::new(cab_w, height - 2.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(cab_x, 0.0, 0.05)),
            ));
        }
    });
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
