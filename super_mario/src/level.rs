use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::{
    components::{BlockContents, BrickBlock, Castle, Collider, Flagpole, HardBlock, Pipe, Player, QuestionBlock, Solid},
    constants::*,
    resources::LevelState,
    states::AppState,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, enter_playing_on_boot)
            .add_systems(OnEnter(AppState::Playing), (spawn_level, position_camera_at_level_start).chain())
            .add_systems(Update, update_camera_follow.run_if(in_state(AppState::Playing)));
    }
}

#[derive(Component)]
struct LevelEntity;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LevelTile {
    Empty,
    Ground,
    Brick,
    Question(BlockContents),
    HardBlock,
}

#[derive(Clone, Copy)]
struct PipeData {
    x: usize,
    height_tiles: usize,
}

#[derive(Clone, Copy)]
struct FlagpoleData {
    x: usize,
    height_tiles: usize,
}

#[derive(Clone, Copy)]
struct CastleData {
    x: usize,
    width_tiles: usize,
    height_tiles: usize,
}

struct LevelDefinition {
    width: usize,
    height: usize,
    tiles: Vec<LevelTile>,
    pipes: Vec<PipeData>,
    flagpole: FlagpoleData,
    castle: CastleData,
    player_start: Vec2,
}

struct LevelGrid {
    width: usize,
    height: usize,
    tiles: Vec<LevelTile>,
}

impl LevelGrid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![LevelTile::Empty; width * height],
        }
    }

    fn set(&mut self, x: usize, y: usize, tile: LevelTile) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    fn fill_rect(&mut self, start_x: usize, start_y: usize, width: usize, height: usize, tile: LevelTile) {
        for y in start_y..start_y.saturating_add(height).min(self.height) {
            for x in start_x..start_x.saturating_add(width).min(self.width) {
                self.set(x, y, tile);
            }
        }
    }
}

fn enter_playing_on_boot(mut next_state: ResMut<NextState<AppState>>) {
    // Temporary bootstrap until the start screen phase is implemented.
    next_state.set(AppState::Playing);
}

fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut level_state: ResMut<LevelState>,
) {
    let level = build_world_1_1();

    let tile_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE));
    let horizontal_border_mesh = meshes.add(Rectangle::new(TILE_SIZE * 0.78, TILE_SIZE * 0.08));
    let vertical_border_mesh = meshes.add(Rectangle::new(TILE_SIZE * 0.08, TILE_SIZE * 0.78));
    let pipe_body_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE));
    let pipe_cap_mesh = meshes.add(Rectangle::new(TILE_SIZE * 2.2, TILE_SIZE * 0.42));
    let flagpole_mesh = meshes.add(Rectangle::new(TILE_SIZE * 0.12, level.flagpole.height_tiles as f32 * TILE_SIZE));
    let flag_mesh = meshes.add(RegularPolygon::new(TILE_SIZE * 0.42, 3));
    let castle_body_mesh =
        meshes.add(Rectangle::new(level.castle.width_tiles as f32 * TILE_SIZE, level.castle.height_tiles as f32 * TILE_SIZE));
    let turret_mesh = meshes.add(Rectangle::new(TILE_SIZE * 1.1, TILE_SIZE * 1.2));

    let ground_material = materials.add(COLOR_GROUND);
    let brick_material = materials.add(COLOR_BRICK);
    let brick_border_material = materials.add(COLOR_MARIO_BROWN);
    let question_material = materials.add(Color::srgb(1.2, 1.0, 0.18));
    let question_detail_material = materials.add(COLOR_MARIO_BROWN);
    let hard_block_material = materials.add(COLOR_HARD_BLOCK);
    let pipe_material = materials.add(COLOR_PIPE_GREEN);
    let pipe_shadow_material = materials.add(COLOR_PIPE_GREEN_DARK);
    let flagpole_material = materials.add(COLOR_FLAGPOLE);
    let flag_material = materials.add(Color::srgb(0.14, 0.95, 0.22));
    let castle_material = materials.add(COLOR_CASTLE);
    let castle_shadow_material = materials.add(Color::srgb(0.38, 0.34, 0.32));

    for tile_y in 0..level.height {
        for tile_x in 0..level.width {
            let tile = level.tiles[tile_y * level.width + tile_x];
            if tile == LevelTile::Empty {
                continue;
            }

            let world_position = tile_to_world(tile_x, tile_y);
            let mut entity = commands.spawn((
                LevelEntity,
                DespawnOnExit(AppState::Playing),
                Mesh2d(tile_mesh.clone()),
                Transform::from_xyz(world_position.x, world_position.y, Z_TILES),
            ));

            match tile {
                LevelTile::Ground => {
                    entity.insert((
                        MeshMaterial2d(ground_material.clone()),
                        Solid,
                        Collider {
                            width: TILE_SIZE,
                            height: TILE_SIZE,
                        },
                    ));
                }
                LevelTile::Brick => {
                    entity.insert((
                        MeshMaterial2d(brick_material.clone()),
                        Solid,
                        BrickBlock,
                        Collider {
                            width: TILE_SIZE,
                            height: TILE_SIZE,
                        },
                    ));
                    entity.with_children(|parent| {
                        spawn_block_frame(
                            parent,
                            horizontal_border_mesh.clone(),
                            vertical_border_mesh.clone(),
                            brick_border_material.clone(),
                        );
                    });
                }
                LevelTile::Question(contents) => {
                    entity.insert((
                        MeshMaterial2d(question_material.clone()),
                        Solid,
                        QuestionBlock {
                            contents,
                            spent: false,
                        },
                        Collider {
                            width: TILE_SIZE,
                            height: TILE_SIZE,
                        },
                    ));
                    entity.with_children(|parent| {
                        spawn_block_frame(
                            parent,
                            horizontal_border_mesh.clone(),
                            vertical_border_mesh.clone(),
                            question_detail_material.clone(),
                        );
                        parent.spawn((
                            Text2d::new("?"),
                            TextFont {
                                font_size: TILE_SIZE * 0.7,
                                ..default()
                            },
                            TextColor(COLOR_MARIO_BROWN),
                            TextLayout::new_with_justify(Justify::Center),
                            Transform::from_xyz(0.0, TILE_SIZE * 0.03, 1.0),
                        ));
                    });
                }
                LevelTile::HardBlock => {
                    entity.insert((
                        MeshMaterial2d(hard_block_material.clone()),
                        Solid,
                        HardBlock,
                        Collider {
                            width: TILE_SIZE,
                            height: TILE_SIZE,
                        },
                    ));
                }
                LevelTile::Empty => {}
            }
        }
    }

    for pipe in &level.pipes {
        let width = TILE_SIZE * 2.0;
        let height = pipe.height_tiles as f32 * TILE_SIZE;
        let center = Vec2::new(
            pipe.x as f32 * TILE_SIZE + width * 0.5,
            world_bottom_y() + GROUND_TILE_ROWS as f32 * TILE_SIZE + height * 0.5,
        );

        commands
            .spawn((
                LevelEntity,
                DespawnOnExit(AppState::Playing),
                Pipe,
                Solid,
                Collider { width, height },
                Transform::from_xyz(center.x, center.y, Z_TILES + 0.2),
            ))
            .with_children(|parent| {
                let body_height = (height - TILE_SIZE * 0.32).max(TILE_SIZE * 0.75);
                let body_y = -height * 0.5 + body_height * 0.5;
                let body_offset_x = TILE_SIZE * 0.5;
                parent.spawn((
                    Mesh2d(pipe_body_mesh.clone()),
                    MeshMaterial2d(pipe_material.clone()),
                    Transform::from_xyz(-body_offset_x, body_y, 0.0)
                        .with_scale(Vec3::new(1.0, body_height / TILE_SIZE, 1.0)),
                ));
                parent.spawn((
                    Mesh2d(pipe_body_mesh.clone()),
                    MeshMaterial2d(pipe_shadow_material.clone()),
                    Transform::from_xyz(body_offset_x, body_y, 0.0)
                        .with_scale(Vec3::new(1.0, body_height / TILE_SIZE, 1.0)),
                ));
                parent.spawn((
                    Mesh2d(pipe_cap_mesh.clone()),
                    MeshMaterial2d(pipe_material.clone()),
                    Transform::from_xyz(0.0, height * 0.5 - TILE_SIZE * 0.21, 0.2),
                ));
            });
    }

    let flag_center = Vec2::new(
        level.flagpole.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        world_bottom_y()
            + GROUND_TILE_ROWS as f32 * TILE_SIZE
            + level.flagpole.height_tiles as f32 * TILE_SIZE * 0.5,
    );
    commands
        .spawn((
            LevelEntity,
            DespawnOnExit(AppState::Playing),
            Flagpole,
            Collider {
                width: TILE_SIZE * 0.4,
                height: level.flagpole.height_tiles as f32 * TILE_SIZE,
            },
            Transform::from_xyz(flag_center.x, flag_center.y, Z_TILES + 0.3),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(flagpole_mesh),
                MeshMaterial2d(flagpole_material),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            parent.spawn((
                Mesh2d(flag_mesh),
                MeshMaterial2d(flag_material),
                Transform::from_xyz(TILE_SIZE * 0.35, TILE_SIZE * 2.5, 0.1)
                    .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            ));
        });

    let castle_center = Vec2::new(
        level.castle.x as f32 * TILE_SIZE + level.castle.width_tiles as f32 * TILE_SIZE * 0.5,
        world_bottom_y()
            + GROUND_TILE_ROWS as f32 * TILE_SIZE
            + level.castle.height_tiles as f32 * TILE_SIZE * 0.5,
    );
    commands
        .spawn((
            LevelEntity,
            DespawnOnExit(AppState::Playing),
            Castle,
            Transform::from_xyz(castle_center.x, castle_center.y, Z_TILES + 0.25),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(castle_body_mesh),
                MeshMaterial2d(castle_material.clone()),
                Transform::default(),
            ));
            parent.spawn((
                Mesh2d(turret_mesh.clone()),
                MeshMaterial2d(castle_shadow_material.clone()),
                Transform::from_xyz(-TILE_SIZE * 2.0, level.castle.height_tiles as f32 * TILE_SIZE * 0.5, 0.1),
            ));
            parent.spawn((
                Mesh2d(turret_mesh.clone()),
                MeshMaterial2d(castle_shadow_material.clone()),
                Transform::from_xyz(TILE_SIZE * 2.0, level.castle.height_tiles as f32 * TILE_SIZE * 0.5, 0.1),
            ));
            parent.spawn((
                Mesh2d(turret_mesh),
                MeshMaterial2d(castle_shadow_material),
                Transform::from_xyz(0.0, level.castle.height_tiles as f32 * TILE_SIZE * 0.5 + TILE_SIZE * 0.3, 0.1),
            ));
        });

    let level_width_world = level.width as f32 * TILE_SIZE;
    level_state.width_tiles = level.width;
    level_state.height_tiles = level.height;
    level_state.player_start = level.player_start;
    level_state.camera_min_x = WINDOW_WIDTH * 0.5;
    level_state.camera_max_x = (level_width_world - WINDOW_WIDTH * 0.5).max(level_state.camera_min_x);
    level_state.camera_y = 0.0;
}

fn position_camera_at_level_start(
    level_state: Res<LevelState>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation.x = level_state.camera_min_x;
    camera_transform.translation.y = level_state.camera_y;
}

fn update_camera_follow(
    time: Res<Time>,
    level_state: Res<LevelState>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let Some(player_transform) = player_query.iter().next() else {
        camera_transform.translation.x = level_state.camera_min_x;
        camera_transform.translation.y = level_state.camera_y;
        return;
    };

    let target_x = player_transform
        .translation
        .x
        .clamp(level_state.camera_min_x, level_state.camera_max_x)
        .max(camera_transform.translation.x);
    let smoothing = (CAMERA_FOLLOW_LERP * time.delta_secs()).clamp(0.0, 1.0);

    camera_transform.translation.x += (target_x - camera_transform.translation.x) * smoothing;
    camera_transform.translation.y = level_state.camera_y;
}

fn spawn_block_frame(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands<'_>,
    horizontal_border_mesh: Handle<Mesh>,
    vertical_border_mesh: Handle<Mesh>,
    border_material: Handle<ColorMaterial>,
) {
    let edge_offset = TILE_SIZE * 0.38;
    parent.spawn((
        Mesh2d(horizontal_border_mesh.clone()),
        MeshMaterial2d(border_material.clone()),
        Transform::from_xyz(0.0, edge_offset, 0.2),
    ));
    parent.spawn((
        Mesh2d(horizontal_border_mesh),
        MeshMaterial2d(border_material.clone()),
        Transform::from_xyz(0.0, -edge_offset, 0.2),
    ));
    parent.spawn((
        Mesh2d(vertical_border_mesh.clone()),
        MeshMaterial2d(border_material.clone()),
        Transform::from_xyz(-edge_offset, 0.0, 0.2),
    ));
    parent.spawn((
        Mesh2d(vertical_border_mesh),
        MeshMaterial2d(border_material),
        Transform::from_xyz(edge_offset, 0.0, 0.2),
    ));
}

fn build_world_1_1() -> LevelDefinition {
    let width = 96;
    let height = LEVEL_HEIGHT_TILES;
    let mut grid = LevelGrid::new(width, height);

    grid.fill_rect(0, 0, width, GROUND_TILE_ROWS, LevelTile::Ground);

    carve_gap(&mut grid, 22, 2);
    carve_gap(&mut grid, 37, 2);
    carve_gap(&mut grid, 54, 3);

    place_blocks(
        &mut grid,
        &[
            (12, 4, LevelTile::Brick),
            (13, 4, LevelTile::Question(BlockContents::Coin)),
            (14, 4, LevelTile::Brick),
            (20, 4, LevelTile::Question(BlockContents::Coin)),
            (21, 4, LevelTile::Brick),
            (22, 4, LevelTile::Question(BlockContents::Mushroom)),
            (23, 4, LevelTile::Brick),
            (33, 5, LevelTile::Brick),
            (34, 5, LevelTile::Brick),
            (35, 5, LevelTile::Question(BlockContents::Coin)),
            (36, 5, LevelTile::Brick),
            (47, 4, LevelTile::Question(BlockContents::Coin)),
            (48, 4, LevelTile::Brick),
            (49, 4, LevelTile::Question(BlockContents::Coin)),
            (61, 5, LevelTile::Brick),
            (62, 5, LevelTile::Brick),
            (63, 5, LevelTile::Brick),
        ],
    );

    grid.fill_rect(42, 7, 3, 1, LevelTile::HardBlock);
    grid.fill_rect(65, 4, 4, 1, LevelTile::Brick);

    for step in 0..6 {
        for height_step in 0..=step {
            grid.set(74 + step, GROUND_TILE_ROWS + height_step, LevelTile::HardBlock);
        }
    }

    let pipes = vec![
        PipeData {
            x: 27,
            height_tiles: 2,
        },
        PipeData {
            x: 39,
            height_tiles: 3,
        },
        PipeData {
            x: 58,
            height_tiles: 4,
        },
    ];

    let player_start = tile_to_world(4, GROUND_TILE_ROWS + 3);

    LevelDefinition {
        width,
        height,
        tiles: grid.tiles,
        pipes,
        flagpole: FlagpoleData {
            x: 88,
            height_tiles: 8,
        },
        castle: CastleData {
            x: 91,
            width_tiles: 4,
            height_tiles: 4,
        },
        player_start,
    }
}

fn carve_gap(grid: &mut LevelGrid, start_x: usize, width: usize) {
    for y in 0..GROUND_TILE_ROWS {
        for x in start_x..start_x + width {
            grid.set(x, y, LevelTile::Empty);
        }
    }
}

fn place_blocks(grid: &mut LevelGrid, blocks: &[(usize, usize, LevelTile)]) {
    for (x, y, tile) in blocks {
        grid.set(*x, *y, *tile);
    }
}

fn tile_to_world(tile_x: usize, tile_y: usize) -> Vec2 {
    Vec2::new(
        tile_x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        world_bottom_y() + tile_y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
    )
}

fn world_bottom_y() -> f32 {
    -WINDOW_HEIGHT * 0.5
}
