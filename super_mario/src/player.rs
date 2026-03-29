use bevy::prelude::*;

use crate::{
    components::{Collider, FacingDirection, Grounded, Player, PowerState, Solid, Velocity},
    constants::*,
    resources::LevelState,
    states::{AppState, PlayState},
};

pub struct PlayerPlugin;

type PlayerMovementQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut Velocity, &'static mut Grounded, &'static Collider),
    (With<Player>, Without<Solid>),
>;

type SolidColliderQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static Collider), (With<Solid>, Without<Player>)>;

#[derive(Clone, Copy)]
struct Aabb {
    center: Vec2,
    half_size: Vec2,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_player_if_missing.run_if(in_state(AppState::Playing)))
            .add_systems(
                Update,
                (
                    apply_player_input,
                    apply_player_gravity,
                    move_player_and_resolve_collisions,
                    update_player_visual_facing,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing))
                    .run_if(in_state(PlayState::Running)),
            );
    }
}

fn spawn_player_if_missing(
    mut commands: Commands,
    level_state: Res<LevelState>,
    player_query: Query<Entity, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !player_query.is_empty() || level_state.width_tiles == 0 {
        return;
    }

    let body_mesh = meshes.add(Rectangle::new(PLAYER_WIDTH * 0.58, PLAYER_HEIGHT * 0.48));
    let legs_mesh = meshes.add(Rectangle::new(PLAYER_WIDTH * 0.62, PLAYER_HEIGHT * 0.28));
    let cap_brim_mesh = meshes.add(Rectangle::new(PLAYER_WIDTH * 0.88, PLAYER_HEIGHT * 0.12));
    let head_mesh = meshes.add(Circle::new(PLAYER_WIDTH * 0.24));

    let body_material = materials.add(COLOR_MARIO_RED);
    let legs_material = materials.add(COLOR_MARIO_BLUE);
    let skin_material = materials.add(COLOR_MARIO_SKIN);

    commands
        .spawn((
            Player,
            DespawnOnExit(AppState::Playing),
            Velocity::default(),
            Grounded::default(),
            FacingDirection::default(),
            PowerState::Small,
            Collider {
                width: PLAYER_WIDTH,
                height: PLAYER_HEIGHT,
            },
            Transform::from_xyz(level_state.player_start.x, level_state.player_start.y, Z_PLAYER),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(body_mesh.clone()),
                MeshMaterial2d(body_material.clone()),
                Transform::from_xyz(0.0, PLAYER_HEIGHT * 0.02, 0.0),
            ));
            parent.spawn((
                Mesh2d(legs_mesh),
                MeshMaterial2d(legs_material),
                Transform::from_xyz(0.0, -PLAYER_HEIGHT * 0.28, 0.0),
            ));
            parent.spawn((
                Mesh2d(head_mesh),
                MeshMaterial2d(skin_material.clone()),
                Transform::from_xyz(0.0, PLAYER_HEIGHT * 0.26, 0.1),
            ));
            parent.spawn((
                Mesh2d(cap_brim_mesh),
                MeshMaterial2d(body_material),
                Transform::from_xyz(0.0, PLAYER_HEIGHT * 0.36, 0.2),
            ));
        });
}

fn apply_player_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Velocity, &Grounded, &mut FacingDirection), With<Player>>,
) {
    let Ok((mut velocity, grounded, mut facing)) = player_query.single_mut() else {
        return;
    };

    let left_pressed = keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA);
    let right_pressed = keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD);
    let input_axis = match (left_pressed, right_pressed) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };

    let dt = time.delta_secs();

    if input_axis != 0.0 {
        velocity.x += input_axis * PLAYER_ACCELERATION * dt;
        velocity.x = velocity.x.clamp(-PLAYER_MAX_SPEED, PLAYER_MAX_SPEED);
        *facing = if input_axis < 0.0 {
            FacingDirection::Left
        } else {
            FacingDirection::Right
        };
    } else {
        let deceleration = PLAYER_DECELERATION * dt;
        if velocity.x > 0.0 {
            velocity.x = (velocity.x - deceleration).max(0.0);
        } else if velocity.x < 0.0 {
            velocity.x = (velocity.x + deceleration).min(0.0);
        }
    }

    let jump_pressed = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW);
    let jump_released = keyboard.just_released(KeyCode::Space)
        || keyboard.just_released(KeyCode::ArrowUp)
        || keyboard.just_released(KeyCode::KeyW);

    if jump_pressed && grounded.0 {
        velocity.y = PLAYER_JUMP_FORCE;
    }

    if jump_released && velocity.y > 0.0 {
        velocity.y *= PLAYER_JUMP_CUT_MULTIPLIER;
    }
}

fn apply_player_gravity(time: Res<Time>, mut player_query: Query<&mut Velocity, With<Player>>) {
    let Ok(mut velocity) = player_query.single_mut() else {
        return;
    };

    velocity.y = (velocity.y + GRAVITY * time.delta_secs()).max(MAX_FALL_SPEED);
}

fn move_player_and_resolve_collisions(
    time: Res<Time>,
    mut player_query: PlayerMovementQuery,
    solid_query: SolidColliderQuery,
) {
    let Ok((mut player_transform, mut velocity, mut grounded, player_collider)) = player_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let half_width = player_collider.width * 0.5;
    let half_height = player_collider.height * 0.5;

    let mut next_x = player_transform.translation.x + velocity.x * dt;
    for (solid_transform, solid_collider) in &solid_query {
        if aabb_overlap(
            Aabb {
                center: Vec2::new(next_x, player_transform.translation.y),
                half_size: Vec2::new(half_width, half_height),
            },
            Aabb {
                center: solid_transform.translation.truncate(),
                half_size: Vec2::new(solid_collider.width * 0.5, solid_collider.height * 0.5),
            },
        ) {
            if velocity.x > 0.0 {
                next_x = solid_transform.translation.x - solid_collider.width * 0.5 - half_width;
            } else if velocity.x < 0.0 {
                next_x = solid_transform.translation.x + solid_collider.width * 0.5 + half_width;
            }
            velocity.x = 0.0;
        }
    }
    player_transform.translation.x = next_x;

    grounded.0 = false;
    let mut next_y = player_transform.translation.y + velocity.y * dt;
    for (solid_transform, solid_collider) in &solid_query {
        if aabb_overlap(
            Aabb {
                center: Vec2::new(player_transform.translation.x, next_y),
                half_size: Vec2::new(half_width, half_height),
            },
            Aabb {
                center: solid_transform.translation.truncate(),
                half_size: Vec2::new(solid_collider.width * 0.5, solid_collider.height * 0.5),
            },
        ) {
            if velocity.y > 0.0 {
                next_y = solid_transform.translation.y - solid_collider.height * 0.5 - half_height;
            } else if velocity.y < 0.0 {
                next_y = solid_transform.translation.y + solid_collider.height * 0.5 + half_height;
                grounded.0 = true;
            }
            velocity.y = 0.0;
        }
    }
    player_transform.translation.y = next_y;
}

fn update_player_visual_facing(
    mut player_query: Query<(&FacingDirection, &mut Transform), With<Player>>,
) {
    let Ok((facing, mut transform)) = player_query.single_mut() else {
        return;
    };

    transform.scale.x = match facing {
        FacingDirection::Left => -1.0,
        FacingDirection::Right => 1.0,
    };
}

fn aabb_overlap(a: Aabb, b: Aabb) -> bool {
    (a.center.x - b.center.x).abs() < a.half_size.x + b.half_size.x
        && (a.center.y - b.center.y).abs() < a.half_size.y + b.half_size.y
}
