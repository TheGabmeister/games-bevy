use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::AppState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_frog);
    }
}

fn spawn_frog(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let pos = grid_to_world(FROG_SPAWN_COL, FROG_SPAWN_ROW);

    commands
        .spawn((
            Frog,
            GridPosition {
                col: FROG_SPAWN_COL,
                row: FROG_SPAWN_ROW,
            },
            HopState::default(),
            GameplayEntity,
            Mesh2d(meshes.add(Circle::new(FROG_BODY_RADIUS))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_FROG))),
            Transform::from_translation(pos.extend(10.0)),
        ))
        .with_children(|parent| {
            // Left eye
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(FROG_EYE_RADIUS))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_FROG_EYE))),
                Transform::from_translation(Vec3::new(
                    -FROG_EYE_OFFSET_X,
                    FROG_EYE_OFFSET_Y,
                    0.1,
                )),
            ));
            // Left pupil
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(FROG_PUPIL_RADIUS))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_FROG_PUPIL))),
                Transform::from_translation(Vec3::new(
                    -FROG_EYE_OFFSET_X,
                    FROG_EYE_OFFSET_Y + FROG_PUPIL_OFFSET_Y,
                    0.2,
                )),
            ));
            // Right eye
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(FROG_EYE_RADIUS))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_FROG_EYE))),
                Transform::from_translation(Vec3::new(
                    FROG_EYE_OFFSET_X,
                    FROG_EYE_OFFSET_Y,
                    0.1,
                )),
            ));
            // Right pupil
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(FROG_PUPIL_RADIUS))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_FROG_PUPIL))),
                Transform::from_translation(Vec3::new(
                    FROG_EYE_OFFSET_X,
                    FROG_EYE_OFFSET_Y + FROG_PUPIL_OFFSET_Y,
                    0.2,
                )),
            ));
        });
}

pub fn frog_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut GridPosition, &mut HopState, &Transform), With<Frog>>,
    frog_event: Res<FrogEvent>,
    level_state: Res<LevelState>,
) {
    if *frog_event != FrogEvent::None || level_state.celebrating {
        return;
    }

    let Ok((mut grid_pos, mut hop_state, transform)) = query.single_mut() else {
        return;
    };
    if hop_state.active {
        return;
    }

    let (dc, dr) = if keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
    {
        (0, 1)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        (0, -1)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        (-1, 0)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        (1, 0)
    } else {
        return;
    };

    let effective_col = world_x_to_col(transform.translation.x);
    let new_col = effective_col + dc;
    let new_row = grid_pos.row + dr;

    if !(0..GRID_COLS).contains(&new_col) || !(0..=HOME_ROW).contains(&new_row) {
        return;
    }

    let from = transform.translation.truncate();
    let to = grid_to_world(new_col, new_row);

    grid_pos.col = new_col;
    grid_pos.row = new_row;

    hop_state.active = true;
    hop_state.from = from;
    hop_state.to = to;
    hop_state.progress = 0.0;
}

pub fn hop_animation(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut HopState), With<Frog>>,
) {
    let Ok((mut transform, mut hop)) = query.single_mut() else {
        return;
    };
    if !hop.active {
        return;
    }

    hop.progress += time.delta_secs() / HOP_DURATION;

    if hop.progress >= 1.0 {
        transform.translation.x = hop.to.x;
        transform.translation.y = hop.to.y;
        transform.scale = Vec3::ONE;
        hop.active = false;
    } else {
        let t = hop.progress;
        let pos = hop.from.lerp(hop.to, t);
        let arc = HOP_ARC_HEIGHT * (t * PI).sin();
        transform.translation.x = pos.x;
        transform.translation.y = pos.y + arc;

        // Squash/stretch during hop
        let stretch = 1.0 + 0.2 * (t * PI).sin();
        let squash = 1.0 - 0.12 * (t * PI).sin();
        transform.scale = Vec3::new(squash, stretch, 1.0);
    }

    // Face movement direction
    let dir = (hop.to - hop.from).normalize_or_zero();
    if dir != Vec2::ZERO {
        let angle = dir.y.atan2(dir.x) - FRAC_PI_2;
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

pub fn score_forward_hop(
    mut game_data: ResMut<GameData>,
    query: Query<&GridPosition, (With<Frog>, Changed<GridPosition>)>,
) {
    let Ok(grid_pos) = query.single() else {
        return;
    };
    if grid_pos.row > game_data.max_row_this_life {
        let gained = (grid_pos.row - game_data.max_row_this_life) as u32;
        game_data.add_score(gained * SCORE_FORWARD_HOP);
        game_data.max_row_this_life = grid_pos.row;
    }
}
