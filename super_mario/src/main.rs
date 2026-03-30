use bevy::prelude::*;

mod components;
mod constants;
mod level;

use components::*;
use constants::*;
use level::{LEVEL_HEIGHT, LEVEL_WIDTH, LevelGrid, level_1_1, tile_to_world, world_to_col, world_to_row};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(
                    WINDOW_WIDTH as u32,
                    WINDOW_HEIGHT as u32,
                ),
                title: "Super Mario Bros".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.36, 0.53, 0.95)))
        .add_systems(Startup, setup)
        .add_systems(Update, (player_input, apply_gravity, apply_velocity, tile_collision, camera_follow).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = CAMERA_SCALE;
    commands.spawn((
        Camera2d,
        Projection::from(projection),
        Transform::from_xyz(CAMERA_MIN_X, CAMERA_FIXED_Y, 0.0),
    ));

    // Pre-allocate shared mesh/material handles for each tile type
    let tile_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE));
    let pipe_top_mesh = meshes.add(Rectangle::new(TILE_SIZE + PIPE_LIP_OVERHANG, TILE_SIZE));
    let ground_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.55, 0.27, 0.07)));
    let brick_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.72, 0.40, 0.10)));
    let question_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.90, 0.75, 0.10)));
    let solid_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.45, 0.30, 0.15)));
    let pipe_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.0, 0.65, 0.15)));
    let player_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.8, 0.1, 0.1)));

    let grid = level_1_1();
    commands.insert_resource(LevelGrid { grid });
    let mut spawn_pos = (0.0_f32, 0.0_f32);

    for row in 0..LEVEL_HEIGHT {
        for col in 0..LEVEL_WIDTH {
            let ch = grid[row][col];
            let (wx, wy) = tile_to_world(col, row);

            match ch {
                '#' => {
                    commands.spawn((
                        Tile,
                        TileType::Ground,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(ground_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_TILE),
                    ));
                }
                'B' => {
                    commands.spawn((
                        Tile,
                        TileType::Brick,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(brick_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_TILE),
                    ));
                }
                '?' | 'M' => {
                    commands.spawn((
                        Tile,
                        TileType::QuestionBlock,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(question_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_TILE),
                    ));
                }
                'X' => {
                    commands.spawn((
                        Tile,
                        TileType::Solid,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(solid_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_TILE),
                    ));
                }
                '[' => {
                    commands.spawn((
                        Tile,
                        TileType::PipeTopLeft,
                        Mesh2d(pipe_top_mesh.clone()),
                        MeshMaterial2d(pipe_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_PIPE),
                    ));
                }
                ']' => {
                    commands.spawn((
                        Tile,
                        TileType::PipeTopRight,
                        Mesh2d(pipe_top_mesh.clone()),
                        MeshMaterial2d(pipe_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_PIPE),
                    ));
                }
                '{' => {
                    commands.spawn((
                        Tile,
                        TileType::PipeBodyLeft,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(pipe_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_PIPE),
                    ));
                }
                '}' => {
                    commands.spawn((
                        Tile,
                        TileType::PipeBodyRight,
                        Mesh2d(tile_mesh.clone()),
                        MeshMaterial2d(pipe_mat.clone()),
                        Transform::from_xyz(wx, wy, Z_PIPE),
                    ));
                }
                'S' => {
                    spawn_pos = (wx, wy);
                }
                _ => {} // '.', 'G', 'K', 'F' — ignored for now
            }
        }
    }

    // Mario — spawned at the S tile position
    commands.spawn((
        Player,
        Velocity::default(),
        FacingDirection::default(),
        Grounded::default(),
        Mesh2d(meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_SMALL_HEIGHT))),
        MeshMaterial2d(player_mat),
        Transform::from_xyz(spawn_pos.0, spawn_pos.1, Z_PLAYER),
    ));
}

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut FacingDirection, &Grounded), With<Player>>,
) {
    let Ok((mut vel, mut facing, grounded)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // --- Horizontal movement with acceleration/deceleration ---
    let mut dir = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dir -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dir += 1.0;
    }

    if dir != 0.0 {
        *facing = if dir < 0.0 {
            FacingDirection::Left
        } else {
            FacingDirection::Right
        };
    }

    let running = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let max_speed = if running { PLAYER_RUN_SPEED } else { PLAYER_WALK_SPEED };
    let accel = if grounded.0 { PLAYER_ACCELERATION } else { PLAYER_AIR_ACCELERATION };

    if dir != 0.0 {
        vel.x += dir * accel * dt;
        vel.x = vel.x.clamp(-max_speed, max_speed);
    } else if grounded.0 {
        let decel = PLAYER_DECELERATION * dt;
        if vel.x.abs() < decel {
            vel.x = 0.0;
        } else {
            vel.x -= decel * vel.x.signum();
        }
    }

    // --- Jumping ---
    if (keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::ArrowUp))
        && grounded.0
    {
        vel.y = PLAYER_JUMP_IMPULSE;
    }

    // Variable-height jump: cut upward velocity on early release
    if (keyboard.just_released(KeyCode::Space) || keyboard.just_released(KeyCode::ArrowUp))
        && vel.y > 0.0
    {
        vel.y *= JUMP_CUT_MULTIPLIER;
    }
}

fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Grounded), With<Player>>,
) {
    let Ok((mut vel, grounded)) = query.single_mut() else {
        return;
    };

    if grounded.0 {
        return;
    }

    let gravity = if vel.y > 0.0 {
        GRAVITY_ASCENDING
    } else {
        GRAVITY_DESCENDING
    };

    vel.y -= gravity * time.delta_secs();
    vel.y = vel.y.max(-TERMINAL_VELOCITY);
}

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<Player>>,
) {
    let Ok((vel, mut transform)) = query.single_mut() else {
        return;
    };

    transform.translation.x += vel.x * time.delta_secs();
    transform.translation.y += vel.y * time.delta_secs();
}

fn tile_collision(
    level: Res<LevelGrid>,
    mut query: Query<(&mut Velocity, &mut Transform, &mut Grounded), With<Player>>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok((mut vel, mut transform, mut grounded)) = query.single_mut() else {
        return;
    };

    let half_w = PLAYER_WIDTH / 2.0;
    let half_h = PLAYER_SMALL_HEIGHT / 2.0;
    let tile_half = TILE_SIZE / 2.0;

    // Camera acts as a left wall (like original SMB)
    if let Ok(camera_tf) = camera_query.single() {
        let camera_left = camera_tf.translation.x - CAMERA_VISIBLE_WIDTH / 2.0;
        let min_x = camera_left + half_w;
        if transform.translation.x < min_x {
            transform.translation.x = min_x;
            if vel.x < 0.0 {
                vel.x = 0.0;
            }
        }
    }

    // Neighborhood: columns and rows that could overlap Mario's AABB (with 1-tile margin)
    let col_min = world_to_col(transform.translation.x - half_w) - 1;
    let col_max = world_to_col(transform.translation.x + half_w) + 1;
    let row_min = world_to_row(transform.translation.y + half_h) - 1;
    let row_max = world_to_row(transform.translation.y - half_h) + 1;

    for row in row_min..=row_max {
        for col in col_min..=col_max {
            if !level.is_solid(col, row) {
                continue;
            }

            let (tile_cx, tile_cy) = tile_to_world(col as usize, row as usize);

            // Recompute player pos (may shift from prior resolution this frame)
            let px = transform.translation.x;
            let py = transform.translation.y;

            let overlap_x = (half_w + tile_half) - (px - tile_cx).abs();
            let overlap_y = (half_h + tile_half) - (py - tile_cy).abs();

            if overlap_x <= 0.0 || overlap_y <= 0.0 {
                continue;
            }

            // Push out on the axis with the smallest penetration
            if overlap_y < overlap_x {
                if py > tile_cy {
                    // Landing — push up
                    transform.translation.y += overlap_y;
                    if vel.y < 0.0 {
                        vel.y = 0.0;
                    }
                } else {
                    // Ceiling bump — push down
                    transform.translation.y -= overlap_y;
                    if vel.y > 0.0 {
                        vel.y = 0.0;
                    }
                }
            } else {
                if px > tile_cx {
                    transform.translation.x += overlap_x;
                } else {
                    transform.translation.x -= overlap_x;
                }
                vel.x = 0.0;
            }
        }
    }

    // Grounded probe: check 1 pixel below Mario's feet
    let probe_y = transform.translation.y - half_h - 1.0;
    let probe_row = world_to_row(probe_y);
    let left_col = world_to_col(transform.translation.x - half_w + 1.0);
    let right_col = world_to_col(transform.translation.x + half_w - 1.0);
    grounded.0 = level.is_solid(left_col, probe_row) || level.is_solid(right_col, probe_row);
}

fn camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.single() else { return };
    let Ok(mut camera_tf) = camera_query.single_mut() else { return };

    // Target: keep Mario at the dead zone line (2/3 from left edge of view)
    let desired_x = player_tf.translation.x - CAMERA_DEAD_ZONE_OFFSET;

    // One-way: never scroll left
    let target_x = desired_x.max(camera_tf.translation.x);

    // Clamp to level bounds
    let target_x = target_x.clamp(CAMERA_MIN_X, CAMERA_MAX_X);

    // Smooth lerp (frame-rate independent)
    let t = 1.0 - (-CAMERA_LERP_SPEED * time.delta_secs()).exp();
    camera_tf.translation.x += (target_x - camera_tf.translation.x) * t;

    // Fixed vertical position
    camera_tf.translation.y = CAMERA_FIXED_Y;
}
