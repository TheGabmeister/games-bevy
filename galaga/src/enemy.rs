use bevy::prelude::*;

use crate::components::{DivingEnemy, Enemy, EnemyBullet, FormationSlot};
use crate::constants::*;
use crate::resources::{
    DiveSelectionCursor, DiveTimer, FormationSway, GameData, StageClearTimer, WavePhase,
};
use crate::states::AppState;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), init_enemies)
            .add_systems(OnExit(AppState::Playing), cleanup_all)
            .add_systems(
                Update,
                (
                    formation_sway,
                    select_divers.after(formation_sway),
                    update_divers.after(formation_sway),
                    diving_enemy_shoot.after(update_divers),
                    move_enemy_bullets.after(diving_enemy_shoot),
                    spawning_to_combat,
                    check_spawn_new_wave,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

pub fn score_for_row(row: usize) -> u32 {
    match row {
        0 => ENEMY_SCORE_ROW0,
        1 => ENEMY_SCORE_ROW1,
        2 => ENEMY_SCORE_ROW2,
        _ => ENEMY_SCORE_ROW3,
    }
}

fn enemy_color_for_row(row: usize) -> Color {
    match row {
        0 => Color::srgb(1.0, 0.2, 0.2),
        1 => Color::srgb(1.0, 0.8, 0.0),
        2 => Color::srgb(0.0, 0.8, 0.2),
        _ => Color::srgb(0.3, 0.7, 1.0),
    }
}

fn formation_base_position(col: usize, row: usize) -> Vec2 {
    let total_width = (FORMATION_COLS - 1) as f32 * FORMATION_SPACING_X;
    let x = -(total_width / 2.0) + col as f32 * FORMATION_SPACING_X;
    let y = FORMATION_TOP_Y - row as f32 * FORMATION_SPACING_Y;
    Vec2::new(x, y)
}

fn formation_dimensions_for_wave(wave: u32) -> (usize, usize) {
    let cols = (4 + wave as usize).min(FORMATION_COLS);
    let rows = (2 + (wave as usize / 2)).min(FORMATION_ROWS);
    (cols, rows)
}

fn dive_interval_for_wave(wave: u32) -> f32 {
    (DIVE_INTERVAL_BASE - (wave as f32 - 1.0) * DIVE_INTERVAL_REDUCTION).max(DIVE_INTERVAL_MIN)
}

fn spawn_formation_entities(commands: &mut Commands, wave: u32) {
    let (cols, rows) = formation_dimensions_for_wave(wave);
    let col_start = (FORMATION_COLS - cols) / 2;

    for row in 0..rows {
        for col in col_start..(col_start + cols) {
            let pos = formation_base_position(col, row);
            commands.spawn((
                Enemy,
                FormationSlot { col, row },
                Sprite {
                    color: enemy_color_for_row(row),
                    custom_size: Some(Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0)),
            ));
        }
    }
}

fn init_enemies(mut commands: Commands, game_data: Res<GameData>) {
    spawn_formation_entities(&mut commands, game_data.wave);

    let dive_interval = dive_interval_for_wave(game_data.wave);
    commands.insert_resource(DiveTimer(Timer::from_seconds(
        dive_interval,
        TimerMode::Repeating,
    )));
    commands.insert_resource(FormationSway::default());
    commands.insert_resource(DiveSelectionCursor::default());
}

#[allow(clippy::type_complexity)]
fn formation_sway(
    time: Res<Time>,
    game_data: Res<GameData>,
    mut sway: ResMut<FormationSway>,
    mut query: Query<(&FormationSlot, &mut Transform), (With<Enemy>, Without<DivingEnemy>)>,
) {
    if game_data.phase != WavePhase::Combat && game_data.phase != WavePhase::Spawning {
        return;
    }

    sway.time += time.delta_secs();
    let offset = (sway.time * FORMATION_SWAY_SPEED).sin() * FORMATION_SWAY_AMOUNT;

    for (slot, mut transform) in &mut query {
        let base = formation_base_position(slot.col, slot.row);
        transform.translation.x = base.x + offset;
        transform.translation.y = base.y;
    }
}

#[allow(clippy::type_complexity)]
fn select_divers(
    mut commands: Commands,
    time: Res<Time>,
    mut dive_timer: ResMut<DiveTimer>,
    mut selection_cursor: ResMut<DiveSelectionCursor>,
    game_data: Res<GameData>,
    candidates: Query<
        (Entity, &Transform, &FormationSlot),
        (With<Enemy>, Without<DivingEnemy>),
    >,
    current_divers: Query<(), With<DivingEnemy>>,
) {
    if game_data.phase != WavePhase::Combat {
        return;
    }

    dive_timer.0.tick(time.delta());
    if !dive_timer.0.just_finished() {
        return;
    }

    let max_divers = MAX_DIVERS_BASE + (game_data.wave as usize / 3);
    if current_divers.iter().count() >= max_divers {
        return;
    }

    let mut candidates_vec: Vec<_> = candidates.iter().collect();
    if candidates_vec.is_empty() {
        return;
    }

    candidates_vec.sort_by_key(|(_, _, slot)| (slot.row, slot.col));
    let index = selection_cursor.next_index % candidates_vec.len();
    selection_cursor.next_index = selection_cursor.next_index.wrapping_add(1);

    let (entity, transform, _) = candidates_vec[index];
    let curve_dir = if transform.translation.x > 0.0 {
        -1.0
    } else {
        1.0
    };

    commands.entity(entity).insert(DivingEnemy {
        time: 0.0,
        start_x: transform.translation.x,
        curve_direction: curve_dir,
        returning: false,
    });
}

fn update_divers(
    mut commands: Commands,
    time: Res<Time>,
    sway: Res<FormationSway>,
    mut query: Query<(Entity, &mut DivingEnemy, &mut Transform, &FormationSlot), With<Enemy>>,
) {
    for (entity, mut diver, mut transform, slot) in &mut query {
        diver.time += time.delta_secs();

        if !diver.returning {
            // Dive down with sinusoidal curve
            transform.translation.y -= DIVE_SPEED * time.delta_secs();
            transform.translation.x = diver.start_x
                + diver.curve_direction
                    * DIVE_CURVE_AMPLITUDE
                    * (diver.time * DIVE_CURVE_FREQUENCY).sin();

            // Clamp X to screen
            let half_w = WINDOW_WIDTH / 2.0 - ENEMY_SIZE / 2.0;
            transform.translation.x = transform.translation.x.clamp(-half_w, half_w);

            // Went below screen - start returning
            if transform.translation.y < -WINDOW_HEIGHT / 2.0 - ENEMY_SIZE {
                diver.returning = true;
                transform.translation.y = WINDOW_HEIGHT / 2.0 + ENEMY_SIZE;
            }
        } else {
            // Return to formation slot
            let sway_offset = (sway.time * FORMATION_SWAY_SPEED).sin() * FORMATION_SWAY_AMOUNT;
            let target = formation_base_position(slot.col, slot.row);
            let target_pos = Vec3::new(target.x + sway_offset, target.y, 0.0);

            let direction = target_pos - transform.translation;
            let distance = direction.length();

            if distance < 5.0 {
                transform.translation = target_pos;
                commands.entity(entity).remove::<DivingEnemy>();
            } else {
                let move_speed = DIVE_SPEED * 1.2;
                transform.translation += direction.normalize() * move_speed * time.delta_secs();
            }
        }
    }
}

fn diving_enemy_shoot(
    mut commands: Commands,
    time: Res<Time>,
    game_data: Res<GameData>,
    query: Query<(&Transform, &DivingEnemy), With<Enemy>>,
) {
    if game_data.phase != WavePhase::Combat {
        return;
    }

    let dt = time.delta_secs();
    for (transform, diver) in &query {
        if diver.returning {
            continue;
        }
        // Fire once at 0.5s into dive
        let prev = diver.time - dt;
        if prev < 0.5 && diver.time >= 0.5 {
            commands.spawn((
                EnemyBullet,
                Sprite {
                    color: Color::srgb(1.0, 0.3, 0.1),
                    custom_size: Some(Vec2::new(ENEMY_BULLET_SIZE, ENEMY_BULLET_SIZE * 1.5)),
                    ..default()
                },
                Transform::from_translation(transform.translation),
            ));
        }
    }
}

fn move_enemy_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform), With<EnemyBullet>>,
) {
    let bottom = -WINDOW_HEIGHT / 2.0 - ENEMY_BULLET_SIZE;
    for (entity, mut transform) in &mut query {
        transform.translation.y -= ENEMY_BULLET_SPEED * time.delta_secs();
        if transform.translation.y < bottom {
            commands.entity(entity).despawn();
        }
    }
}

fn spawning_to_combat(
    mut game_data: ResMut<GameData>,
    enemy_query: Query<(), With<Enemy>>,
) {
    if game_data.phase == WavePhase::Spawning && !enemy_query.is_empty() {
        game_data.phase = WavePhase::Combat;
    }
}

fn check_spawn_new_wave(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    time: Res<Time>,
    timer: Option<ResMut<StageClearTimer>>,
) {
    if game_data.phase != WavePhase::StageClear {
        return;
    }

    let Some(mut timer) = timer else { return };

    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        game_data.wave += 1;
        game_data.phase = WavePhase::Spawning;
        commands.remove_resource::<StageClearTimer>();

        spawn_formation_entities(&mut commands, game_data.wave);

        let dive_interval = dive_interval_for_wave(game_data.wave);
        commands.insert_resource(DiveTimer(Timer::from_seconds(
            dive_interval,
            TimerMode::Repeating,
        )));
        commands.insert_resource(DiveSelectionCursor::default());
    }
}

fn cleanup_all(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    bullet_query: Query<Entity, With<EnemyBullet>>,
) {
    for entity in &enemy_query {
        commands.entity(entity).despawn();
    }
    for entity in &bullet_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DiveTimer>();
    commands.remove_resource::<DiveSelectionCursor>();
    commands.remove_resource::<FormationSway>();
    commands.remove_resource::<StageClearTimer>();
}
