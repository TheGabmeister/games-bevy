use bevy::prelude::*;
use rand::Rng;

use crate::{
    components::{Flea, GridPos, Mushroom, Poisoned, Scorpion, Spider},
    constants::*,
    mushroom::{mushroom_color, spawn_mushroom_at},
    resources::{FleaSpawnTimer, MushroomGrid, ScorpionSpawnTimer, SpiderSpawnTimer},
    scheduling::GameplaySet,
    states::AppState,
};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                flea_spawn_check.in_set(GameplaySet::Movement),
                flea_movement.in_set(GameplaySet::Movement),
                spider_spawn.in_set(GameplaySet::Movement),
                spider_movement.in_set(GameplaySet::Movement),
                spider_mushroom_destroy.in_set(GameplaySet::Collision),
                scorpion_spawn.in_set(GameplaySet::Movement),
                scorpion_movement.in_set(GameplaySet::Movement),
                scorpion_poison_mushrooms.in_set(GameplaySet::Collision),
                despawn_offscreen_enemies.in_set(GameplaySet::Cleanup),
            ),
        );
    }
}

// ── Flea ──────────────────────────────────────────────────────────────────────

fn flea_spawn_check(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut timer: ResMut<FleaSpawnTimer>,
    time: Res<Time>,
    flea_q: Query<(), With<Flea>>,
    mushroom_q: Query<&GridPos, With<Mushroom>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    if !flea_q.is_empty() {
        return; // only one flea at a time
    }

    // Count mushrooms in player zone
    let count = mushroom_q
        .iter()
        .filter(|p| p.row >= PLAYER_ZONE_ROW_START)
        .count();

    if count >= MIN_MUSHROOMS_IN_PLAYER_ZONE {
        return;
    }

    let mut rng = rand::thread_rng();
    let col = rng.gen_range(0..GRID_COLS);
    let mesh = meshes.add(Circle::new(CELL_SIZE * 0.35));

    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(Color::srgb(0.9, 0.9, 0.1))),
        Transform::from_xyz(grid_to_world_x(col), grid_to_world_y(0) + CELL_SIZE, 2.0),
        Flea { hits: 0 },
        DespawnOnExit(AppState::Playing),
    ));
}

fn flea_movement(mut query: Query<&mut Transform, With<Flea>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation.y -= FLEA_SPEED * time.delta_secs();
    }
}

// Drop mushrooms as flea falls (handled in collision.rs when flea is hit;
// here we randomly drop while alive)
pub fn flea_drop_mushroom(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: &mut ResMut<MushroomGrid>,
    flea_x: f32,
    flea_y: f32,
) {
    let col = world_to_grid_col(flea_x);
    let row = world_to_grid_row(flea_y);
    if !(0..GRID_ROWS).contains(&row) {
        return;
    }
    let key = (col, row);
    if grid.0.contains_key(&key) {
        return;
    }
    let mesh = meshes.add(Rectangle::new(CELL_SIZE * 0.72, CELL_SIZE * 0.72));
    let entity = spawn_mushroom_at(commands, materials, &mesh, col, row, 0, false);
    grid.0.insert(key, entity);
}

// ── Spider ────────────────────────────────────────────────────────────────────

fn spider_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut timer: ResMut<SpiderSpawnTimer>,
    time: Res<Time>,
    spider_q: Query<(), With<Spider>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    if !spider_q.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    // Spawn from left or right edge in player zone
    let from_left = rng.gen_bool(0.5);
    let x = if from_left {
        -WINDOW_WIDTH / 2.0 + CELL_SIZE
    } else {
        WINDOW_WIDTH / 2.0 - CELL_SIZE
    };
    let row = rng.gen_range(PLAYER_ZONE_ROW_START..GRID_ROWS);
    let y = grid_to_world_y(row);

    let initial_dir = if from_left {
        Vec2::new(1.0, 0.5).normalize()
    } else {
        Vec2::new(-1.0, 0.5).normalize()
    };

    // Diamond shape: rotated square
    let mesh = meshes.add(Rectangle::new(CELL_SIZE * 0.7, CELL_SIZE * 0.7));
    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.8))),
        Transform::from_xyz(x, y, 2.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        Spider {
            dir: initial_dir,
            change_timer: 0.5,
        },
        DespawnOnExit(AppState::Playing),
    ));
}

fn spider_movement(mut query: Query<(&mut Transform, &mut Spider)>, time: Res<Time>) {
    let mut rng = rand::thread_rng();
    for (mut transform, mut spider) in &mut query {
        spider.change_timer -= time.delta_secs();
        if spider.change_timer <= 0.0 {
            // Random new direction within player zone constraints
            let angle = rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI);
            spider.dir = Vec2::new(angle.cos(), angle.sin()).normalize();
            spider.change_timer = rng.gen_range(0.3f32..0.8);
        }

        transform.translation.x += spider.dir.x * SPIDER_SPEED * time.delta_secs();
        transform.translation.y += spider.dir.y * SPIDER_SPEED * time.delta_secs();

        // Bounce off player zone boundaries
        let min_y = player_zone_min_y();
        let max_y = player_zone_max_y();
        let half_w = WINDOW_WIDTH / 2.0 - CELL_SIZE / 2.0;

        if transform.translation.x < -half_w || transform.translation.x > half_w {
            spider.dir.x = -spider.dir.x;
            transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
        }
        if transform.translation.y < min_y || transform.translation.y > max_y {
            spider.dir.y = -spider.dir.y;
            transform.translation.y = transform.translation.y.clamp(min_y, max_y);
        }
    }
}

fn spider_mushroom_destroy(
    mut commands: Commands,
    spider_q: Query<&Transform, With<Spider>>,
    mushroom_q: Query<(Entity, &GridPos), With<Mushroom>>,
    mut grid: ResMut<MushroomGrid>,
) {
    for spider_t in &spider_q {
        let sx = spider_t.translation.x;
        let sy = spider_t.translation.y;
        let radius = CELL_SIZE * 0.5;

        for (m_entity, m_pos) in &mushroom_q {
            let mx = grid_to_world_x(m_pos.col);
            let my = grid_to_world_y(m_pos.row);
            if (sx - mx).abs() < radius && (sy - my).abs() < radius {
                grid.0.remove(&(m_pos.col, m_pos.row));
                commands.entity(m_entity).despawn();
            }
        }
    }
}

// ── Scorpion ──────────────────────────────────────────────────────────────────

fn scorpion_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut timer: ResMut<ScorpionSpawnTimer>,
    time: Res<Time>,
    scorpion_q: Query<(), With<Scorpion>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    if !scorpion_q.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let from_left = rng.gen_bool(0.5);
    let dx = if from_left { 1.0 } else { -1.0 };
    let x = if from_left {
        -WINDOW_WIDTH / 2.0 - CELL_SIZE
    } else {
        WINDOW_WIDTH / 2.0 + CELL_SIZE
    };
    let row = rng.gen_range(1..PLAYER_ZONE_ROW_START);
    let y = grid_to_world_y(row);

    let mesh = meshes.add(Rectangle::new(CELL_SIZE * 1.1, CELL_SIZE * 0.55));
    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.2, 0.8))),
        Transform::from_xyz(x, y, 2.0),
        Scorpion { dx },
        DespawnOnExit(AppState::Playing),
    ));
}

fn scorpion_movement(mut query: Query<(&mut Transform, &Scorpion)>, time: Res<Time>) {
    for (mut transform, scorpion) in &mut query {
        transform.translation.x += scorpion.dx * SCORPION_SPEED * time.delta_secs();
    }
}

fn scorpion_poison_mushrooms(
    scorpion_q: Query<&Transform, With<Scorpion>>,
    mut mushroom_q: Query<
        (
            Entity,
            &GridPos,
            &mut Mushroom,
            &MeshMaterial2d<ColorMaterial>,
        ),
        Without<Poisoned>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    for scorpion_t in &scorpion_q {
        let sx = scorpion_t.translation.x;
        let sy = scorpion_t.translation.y;

        for (entity, pos, _mushroom, mat_handle) in &mut mushroom_q {
            let mx = grid_to_world_x(pos.col);
            let my = grid_to_world_y(pos.row);
            if (sx - mx).abs() < CELL_SIZE * 0.6 && (sy - my).abs() < CELL_SIZE * 0.6 {
                commands.entity(entity).insert(Poisoned);
                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.color = mushroom_color(0, true);
                }
            }
        }
    }
}

fn despawn_offscreen_enemies(
    mut commands: Commands,
    flea_q: Query<(Entity, &Transform), With<Flea>>,
    scorpion_q: Query<(Entity, &Transform), With<Scorpion>>,
) {
    let bottom = -WINDOW_HEIGHT / 2.0 - CELL_SIZE;
    let left = -WINDOW_WIDTH / 2.0 - CELL_SIZE * 2.0;
    let right = WINDOW_WIDTH / 2.0 + CELL_SIZE * 2.0;

    for (entity, t) in &flea_q {
        if t.translation.y < bottom {
            commands.entity(entity).despawn();
        }
    }
    for (entity, t) in &scorpion_q {
        if t.translation.x < left || t.translation.x > right {
            commands.entity(entity).despawn();
        }
    }
}
