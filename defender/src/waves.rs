use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::spawning::*;
use crate::states::AppState;
use crate::terrain::get_terrain_y_at;

pub fn wave_intro_setup(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut wave_state: ResMut<WaveState>,
) {
    game_state.current_wave += 1;
    let wave = game_state.current_wave;

    // Configure wave enemies
    let landers = (4 + wave * 2).min(20);
    let bombers = if wave >= 2 { (wave - 1).min(8) } else { 0 };
    let pods = if wave >= 3 { (wave - 2).min(5) } else { 0 };

    wave_state.landers_to_spawn = landers;
    wave_state.bombers_to_spawn = bombers;
    wave_state.pods_to_spawn = pods;
    wave_state.spawn_timer.reset();
    wave_state.baiter_timer.reset();
    wave_state.baiter_interval.reset();
    wave_state.baiters_active = false;
    wave_state.wave_active = true;

    // Spawn wave banner
    commands.insert_resource(WaveIntroTimer(Timer::from_seconds(2.0, TimerMode::Once)));

    commands.spawn((
        WaveBanner,
        Text::new(format!("WAVE {}", wave)),
        TextFont {
            font_size: 60.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            left: Val::Percent(38.0),
            ..default()
        },
    ));
}

pub fn wave_intro_timer(
    time: Res<Time>,
    mut timer: ResMut<WaveIntroTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next_state.set(AppState::Playing);
    }
}

pub fn wave_intro_cleanup(
    mut commands: Commands,
    banners: Query<Entity, With<WaveBanner>>,
) {
    for entity in &banners {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_wave_enemies(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut wave_state: ResMut<WaveState>,
    player_q: Query<&WorldPosition, With<Player>>,
) {
    if !wave_state.wave_active {
        return;
    }

    let player_x = player_q
        .single()
        .map(|wp| wp.0)
        .unwrap_or(WORLD_WIDTH / 2.0);

    wave_state.spawn_timer.tick(time.delta());
    if wave_state.spawn_timer.just_finished() {
        if wave_state.landers_to_spawn > 0 {
            wave_state.landers_to_spawn -= 1;
            let x = rand_world_x_far_from(player_x);
            spawn_lander(&mut commands, &mut meshes, &mut materials, x);
        } else if wave_state.bombers_to_spawn > 0 {
            wave_state.bombers_to_spawn -= 1;
            let x = rand_world_x_far_from(player_x);
            spawn_bomber(&mut commands, &mut meshes, &mut materials, x);
        } else if wave_state.pods_to_spawn > 0 {
            wave_state.pods_to_spawn -= 1;
            let x = rand_world_x_far_from(player_x);
            spawn_pod(&mut commands, &mut meshes, &mut materials, x);
        }
    }

    // Baiter timer
    wave_state.baiter_timer.tick(time.delta());
    if wave_state.baiter_timer.is_finished() {
        wave_state.baiters_active = true;
    }

    if wave_state.baiters_active {
        wave_state.baiter_interval.tick(time.delta());
        if wave_state.baiter_interval.just_finished() {
            let x = rand_world_x_far_from(player_x);
            spawn_baiter(&mut commands, &mut meshes, &mut materials, x);
        }
    }
}

pub fn wave_check(
    enemies: Query<Entity, With<Enemy>>,
    wave_state: Res<WaveState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !wave_state.wave_active {
        return;
    }

    let total_to_spawn =
        wave_state.landers_to_spawn + wave_state.bombers_to_spawn + wave_state.pods_to_spawn;

    if total_to_spawn == 0 && enemies.iter().count() == 0 {
        next_state.set(AppState::WaveIntro);
    }
}

pub fn spawn_initial_humans_if_needed(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    terrain: Res<TerrainData>,
    mut game_state: ResMut<GameState>,
    existing_humans: Query<Entity, With<Human>>,
) {
    // Only spawn humans if none exist (wave 1 or all died)
    if existing_humans.iter().count() > 0 {
        return;
    }
    game_state.humans_alive = HUMANS_PER_WAVE;
    for i in 0..HUMANS_PER_WAVE {
        let x = (i as f32 / HUMANS_PER_WAVE as f32) * WORLD_WIDTH + 100.0;
        let terrain_y = get_terrain_y_at(&terrain, x);
        spawn_human(&mut commands, &mut meshes, &mut materials, x, terrain_y);
    }
}

pub fn check_all_humans_dead(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_state: Res<GameState>,
    landers: Query<(Entity, &WorldPosition, &Transform), With<Lander>>,
) {
    if game_state.humans_alive == 0 {
        // All humans dead - convert all landers to mutants
        for (entity, wp, tf) in &landers {
            let wx = wp.0;
            let y = tf.translation.y;
            commands.entity(entity).despawn();
            spawn_mutant(&mut commands, &mut meshes, &mut materials, wx, y);
        }
    }
}
