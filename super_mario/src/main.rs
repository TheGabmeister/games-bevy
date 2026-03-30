use bevy::prelude::*;

mod components;
mod constants;
mod level;

use components::*;
use constants::*;

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
        .add_systems(Update, (player_input, apply_gravity, apply_velocity, ground_collision).chain())
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
    ));

    // Mario — red rectangle
    commands.spawn((
        Player,
        Velocity::default(),
        FacingDirection::default(),
        Grounded::default(),
        Mesh2d(meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_SMALL_HEIGHT))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.8, 0.1, 0.1)))),
        Transform::from_xyz(0.0, 50.0, 0.0),
    ));

    // Ground — brown rectangle
    commands.spawn((
        Ground,
        Mesh2d(meshes.add(Rectangle::new(GROUND_WIDTH, GROUND_HEIGHT))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.55, 0.27, 0.07)))),
        Transform::from_xyz(0.0, GROUND_Y, 0.0),
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

fn ground_collision(
    mut query: Query<(&mut Velocity, &mut Transform, &mut Grounded), With<Player>>,
) {
    let Ok((mut vel, mut transform, mut grounded)) = query.single_mut() else {
        return;
    };

    let ground_surface = GROUND_Y + GROUND_HEIGHT / 2.0;
    let player_bottom = transform.translation.y - PLAYER_SMALL_HEIGHT / 2.0;

    if player_bottom <= ground_surface && vel.y <= 0.0 {
        transform.translation.y = ground_surface + PLAYER_SMALL_HEIGHT / 2.0;
        vel.y = 0.0;
        grounded.0 = true;
    } else {
        grounded.0 = false;
    }
}
