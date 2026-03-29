use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::*;

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaveState {
            intro_timer: Timer::from_seconds(WAVE_INTRO_DURATION, TimerMode::Once),
            clear_timer: Timer::from_seconds(WAVE_CLEAR_DURATION, TimerMode::Once),
            death_timer: Timer::from_seconds(DEATH_PAUSE_DURATION, TimerMode::Once),
        })
        .insert_resource(GameOverTimer(Timer::from_seconds(
            GAME_OVER_INPUT_DELAY,
            TimerMode::Once,
        )))
        .add_systems(OnEnter(AppState::Playing), reset_game_state)
        .add_systems(
            OnEnter(PlayState::WaveIntro),
            (despawn_wave_entities, spawn_wave).chain(),
        )
        .add_systems(
            Update,
            tick_wave_intro.run_if(in_state(PlayState::WaveIntro)),
        )
        .add_systems(OnEnter(PlayState::WaveClear), start_clear_timer)
        .add_systems(
            Update,
            tick_wave_clear.run_if(in_state(PlayState::WaveClear)),
        )
        .add_systems(OnEnter(PlayState::PlayerDeath), start_death_timer)
        .add_systems(
            Update,
            tick_player_death.run_if(in_state(PlayState::PlayerDeath)),
        );
    }
}

// --- Wave Definition ---

pub struct WaveDefinition {
    pub grunts: u32,
    pub hulks: u32,
    pub brains: u32,
    pub spheroids: u32,
    pub quarks: u32,
    pub electrodes: u32,
    pub humans: u32,
    pub speed_mult: f32,
}

pub fn wave_definition(wave: u32) -> WaveDefinition {
    WaveDefinition {
        grunts: (15 + (wave - 1) * 5).min(60),
        hulks: (wave / 3).min(8),
        brains: if wave >= 3 {
            ((wave - 2) / 2).min(5)
        } else {
            0
        },
        spheroids: if wave >= 4 {
            ((wave - 3) / 3).min(3)
        } else {
            0
        },
        quarks: if wave >= 6 {
            ((wave - 5) / 3).min(2)
        } else {
            0
        },
        electrodes: (5 + wave * 2).min(25),
        humans: 5.min(3 + wave),
        speed_mult: 1.0 + (wave - 1) as f32 * 0.05,
    }
}

// --- Systems ---

fn reset_game_state(mut game: ResMut<GameState>, mut shake: ResMut<ScreenShake>) {
    game.reset();
    shake.trauma = 0.0;
}

fn despawn_wave_entities(mut commands: Commands, query: Query<Entity, With<WaveEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_wave(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut game: ResMut<GameState>,
    mut wave_state: ResMut<WaveState>,
) {
    game.rescue_count_this_wave = 0;
    wave_state.intro_timer.reset();

    let def = wave_definition(game.current_wave);
    let mut rng = rand::rng();
    let mut occupied: Vec<Vec2> = Vec::new();

    let spawn_on_edge = |rng: &mut rand::rngs::ThreadRng, radius: f32| -> (f32, f32) {
        loop {
            let edge: u32 = rng.random_range(0..4);
            let (x, y) = match edge {
                0 => (
                    rng.random_range(-ARENA_HALF_WIDTH + radius..ARENA_HALF_WIDTH - radius),
                    ARENA_HALF_HEIGHT - radius,
                ),
                1 => (
                    rng.random_range(-ARENA_HALF_WIDTH + radius..ARENA_HALF_WIDTH - radius),
                    -ARENA_HALF_HEIGHT + radius,
                ),
                2 => (
                    -ARENA_HALF_WIDTH + radius,
                    rng.random_range(-ARENA_HALF_HEIGHT + radius..ARENA_HALF_HEIGHT - radius),
                ),
                _ => (
                    ARENA_HALF_WIDTH - radius,
                    rng.random_range(-ARENA_HALF_HEIGHT + radius..ARENA_HALF_HEIGHT - radius),
                ),
            };
            if x * x + y * y > SPAWN_EXCLUSION_RADIUS * SPAWN_EXCLUSION_RADIUS {
                return (x, y);
            }
        }
    };

    let spawn_inside =
        |rng: &mut rand::rngs::ThreadRng, radius: f32, occupied: &[Vec2]| -> (f32, f32) {
            loop {
                let x = rng.random_range(-ARENA_HALF_WIDTH + radius..ARENA_HALF_WIDTH - radius);
                let y = rng.random_range(-ARENA_HALF_HEIGHT + radius..ARENA_HALF_HEIGHT - radius);
                if x * x + y * y < SPAWN_EXCLUSION_RADIUS * SPAWN_EXCLUSION_RADIUS {
                    continue;
                }
                let pos = Vec2::new(x, y);
                if occupied
                    .iter()
                    .any(|o| o.distance(pos) < SPAWN_MIN_SEPARATION)
                {
                    continue;
                }
                return (x, y);
            }
        };

    // Grunts
    for _ in 0..def.grunts {
        let (x, y) = spawn_on_edge(&mut rng, GRUNT_RADIUS);
        let steer_offset: f32 = rng.random_range(-0.5..0.5);
        commands.spawn((
            Enemy,
            Grunt,
            Killable,
            DamagesPlayer,
            Confined,
            WaveEntity,
            Mesh2d(assets.grunt_mesh.clone()),
            MeshMaterial2d(assets.grunt_material.clone()),
            Transform::from_xyz(x, y, 1.0)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            Velocity(Vec2::ZERO),
            CollisionRadius(GRUNT_RADIUS),
            PointValue(100),
            GruntSteerOffset(steer_offset),
            DespawnOnExit(AppState::Playing),
        ));
    }

    // Hulks
    for _ in 0..def.hulks {
        let (x, y) = spawn_on_edge(&mut rng, HULK_RADIUS);
        commands.spawn((
            Enemy,
            Hulk,
            DamagesPlayer,
            Confined,
            WaveEntity,
            Mesh2d(assets.hulk_mesh.clone()),
            MeshMaterial2d(assets.hulk_material.clone()),
            Transform::from_xyz(x, y, 1.0)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            Velocity(Vec2::ZERO),
            CollisionRadius(HULK_RADIUS),
            PointValue(0),
            Knockback(Vec2::ZERO),
            WanderTarget(Vec2::ZERO),
            WanderTimer(Timer::from_seconds(HULK_WANDER_INTERVAL, TimerMode::Repeating)),
            DespawnOnExit(AppState::Playing),
        ));
    }

    // Brains
    for _ in 0..def.brains {
        let (x, y) = spawn_on_edge(&mut rng, BRAIN_RADIUS);
        commands.spawn((
            Enemy,
            Brain,
            Killable,
            DamagesPlayer,
            Confined,
            WaveEntity,
            Mesh2d(assets.brain_mesh.clone()),
            MeshMaterial2d(assets.brain_material.clone()),
            Transform::from_xyz(x, y, 1.0),
            Velocity(Vec2::ZERO),
            CollisionRadius(BRAIN_RADIUS),
            PointValue(500),
            FireCooldown(Timer::from_seconds(
                BRAIN_MISSILE_COOLDOWN,
                TimerMode::Repeating,
            )),
            DespawnOnExit(AppState::Playing),
        ));
    }

    // Spheroids
    for _ in 0..def.spheroids {
        let (x, y) = spawn_on_edge(&mut rng, SPHEROID_RADIUS);
        commands
            .spawn((
                Enemy,
                Spheroid,
                Killable,
                DamagesPlayer,
                Confined,
                WaveEntity,
                Mesh2d(assets.spheroid_mesh.clone()),
                MeshMaterial2d(assets.spheroid_material.clone()),
                Transform::from_xyz(x, y, 1.0),
                Velocity(Vec2::ZERO),
                CollisionRadius(SPHEROID_RADIUS),
                PointValue(1000),
                DespawnOnExit(AppState::Playing),
            ))
            .insert((
                SpawnerState {
                    children_spawned: 0,
                    max_children: SPHEROID_MAX_CHILDREN,
                    cooldown: Timer::from_seconds(SPHEROID_SPAWN_COOLDOWN, TimerMode::Repeating),
                },
                WanderTarget(Vec2::ZERO),
                WanderTimer(Timer::from_seconds(2.0, TimerMode::Repeating)),
            ));
    }

    // Quarks
    for _ in 0..def.quarks {
        let (x, y) = spawn_on_edge(&mut rng, QUARK_RADIUS);
        commands
            .spawn((
                Enemy,
                Quark,
                Killable,
                DamagesPlayer,
                Confined,
                WaveEntity,
                Mesh2d(assets.quark_mesh.clone()),
                MeshMaterial2d(assets.quark_material.clone()),
                Transform::from_xyz(x, y, 1.0),
                Velocity(Vec2::ZERO),
                CollisionRadius(QUARK_RADIUS),
                PointValue(1000),
                DespawnOnExit(AppState::Playing),
            ))
            .insert((
                SpawnerState {
                    children_spawned: 0,
                    max_children: QUARK_MAX_CHILDREN,
                    cooldown: Timer::from_seconds(QUARK_SPAWN_COOLDOWN, TimerMode::Repeating),
                },
                WanderTarget(Vec2::ZERO),
                WanderTimer(Timer::from_seconds(2.5, TimerMode::Repeating)),
            ));
    }

    // Electrodes
    for _ in 0..def.electrodes {
        let (x, y) = spawn_inside(&mut rng, ELECTRODE_RADIUS, &occupied);
        occupied.push(Vec2::new(x, y));
        commands.spawn((
            Electrode,
            DamagesPlayer,
            WaveEntity,
            Mesh2d(assets.electrode_mesh.clone()),
            MeshMaterial2d(assets.electrode_material.clone()),
            Transform::from_xyz(x, y, 0.5),
            CollisionRadius(ELECTRODE_RADIUS),
            DespawnOnExit(AppState::Playing),
        ));
    }

    // Humans
    for _ in 0..def.humans {
        let (x, y) = spawn_inside(&mut rng, HUMAN_RADIUS, &occupied);
        occupied.push(Vec2::new(x, y));
        let variant = rng.random_range(0usize..3);
        commands.spawn((
            Human,
            Confined,
            WaveEntity,
            Mesh2d(assets.human_mesh.clone()),
            MeshMaterial2d(assets.human_materials[variant].clone()),
            Transform::from_xyz(x, y, 0.8),
            Velocity(Vec2::ZERO),
            CollisionRadius(HUMAN_RADIUS),
            WanderTarget(Vec2::ZERO),
            WanderTimer(Timer::from_seconds(
                HUMAN_WANDER_INTERVAL,
                TimerMode::Repeating,
            )),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

fn tick_wave_intro(
    time: Res<Time>,
    mut wave_state: ResMut<WaveState>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    wave_state.intro_timer.tick(time.delta());
    if wave_state.intro_timer.is_finished() {
        next_state.set(PlayState::WaveActive);
    }
}

// NEW: WaveClear state — brief pause before advancing to next wave
fn start_clear_timer(mut wave_state: ResMut<WaveState>) {
    wave_state.clear_timer.reset();
}

fn tick_wave_clear(
    time: Res<Time>,
    mut wave_state: ResMut<WaveState>,
    mut game: ResMut<GameState>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    wave_state.clear_timer.tick(time.delta());
    if wave_state.clear_timer.is_finished() {
        game.current_wave += 1;
        next_state.set(PlayState::WaveIntro);
    }
}

fn start_death_timer(mut wave_state: ResMut<WaveState>) {
    wave_state.death_timer.reset();
}

fn tick_player_death(
    time: Res<Time>,
    mut wave_state: ResMut<WaveState>,
    game: Res<GameState>,
    mut next_app: ResMut<NextState<AppState>>,
    mut next_play: ResMut<NextState<PlayState>>,
    killable_q: Query<(), With<Killable>>,
) {
    wave_state.death_timer.tick(time.delta());
    if !wave_state.death_timer.is_finished() {
        return;
    }
    if game.lives == 0 {
        next_app.set(AppState::GameOver);
    } else if killable_q.is_empty() {
        // Wave cleared during death — advance via WaveClear
        next_play.set(PlayState::WaveClear);
    } else {
        next_play.set(PlayState::WaveActive);
    }
}
