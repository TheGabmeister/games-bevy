use bevy::{
    camera::ScalingMode,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    window::WindowResolution,
};
use rand::Rng;
use std::fs;
use std::path::PathBuf;

// --- Constants ---

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 720;

const ARENA_HALF_WIDTH: f32 = 440.0;
const ARENA_HALF_HEIGHT: f32 = 320.0;
const ARENA_BORDER_THICKNESS: f32 = 3.0;

const PLAYER_SPEED: f32 = 300.0;
const PLAYER_RADIUS: f32 = 8.0;
const PLAYER_FIRE_COOLDOWN: f32 = 0.08;
const PLAYER_INVINCIBILITY_DURATION: f32 = 2.0;
const PLAYER_BLINK_INTERVAL: f32 = 0.1;
const STARTING_LIVES: u32 = 3;
const EXTRA_LIFE_EVERY: u32 = 25_000;

const BULLET_SPEED: f32 = 800.0;
const BULLET_RADIUS: f32 = 3.0;
const MAX_PLAYER_BULLETS: u32 = 15;

const GRUNT_BASE_SPEED: f32 = 120.0;
const GRUNT_RADIUS: f32 = 10.0;

const HULK_SPEED: f32 = 60.0;
const HULK_RADIUS: f32 = 16.0;
const HULK_KNOCKBACK_STRENGTH: f32 = 200.0;
const HULK_KNOCKBACK_DECAY: f32 = 5.0;
const HULK_WANDER_INTERVAL: f32 = 2.0;

const BRAIN_SPEED: f32 = 100.0;
const BRAIN_RADIUS: f32 = 10.0;
const BRAIN_MISSILE_COOLDOWN: f32 = 3.0;
const MISSILE_SPEED: f32 = 150.0;
const MISSILE_TURN_RATE: f32 = 2.5;
const MISSILE_RADIUS: f32 = 4.0;
const MISSILE_LIFETIME: f32 = 6.0;

const PROG_SPEED: f32 = 160.0;
const PROG_RADIUS: f32 = 8.0;

const SPHEROID_SPEED: f32 = 80.0;
const SPHEROID_RADIUS: f32 = 12.0;
const SPHEROID_SPAWN_COOLDOWN: f32 = 4.0;
const SPHEROID_MAX_CHILDREN: u32 = 3;

const ENFORCER_SPEED: f32 = 90.0;
const ENFORCER_RADIUS: f32 = 8.0;
const ENFORCER_FIRE_COOLDOWN: f32 = 2.5;
const SPARK_SPEED: f32 = 250.0;
const SPARK_RADIUS: f32 = 3.0;
const SPARK_LIFETIME: f32 = 3.0;

const QUARK_SPEED: f32 = 70.0;
const QUARK_RADIUS: f32 = 12.0;
const QUARK_SPAWN_COOLDOWN: f32 = 5.0;
const QUARK_MAX_CHILDREN: u32 = 2;

const TANK_SPEED: f32 = 50.0;
const TANK_RADIUS: f32 = 10.0;
const TANK_FIRE_COOLDOWN: f32 = 3.0;
const SHELL_SPEED: f32 = 200.0;
const SHELL_RADIUS: f32 = 4.0;
const SHELL_MAX_BOUNCES: u32 = 3;
const SHELL_LIFETIME: f32 = 5.0;

const ELECTRODE_RADIUS: f32 = 6.0;

const HUMAN_SPEED: f32 = 40.0;
const HUMAN_RADIUS: f32 = 6.0;
const HUMAN_WANDER_INTERVAL: f32 = 2.0;

const SPAWN_EXCLUSION_RADIUS: f32 = 100.0;
const SPAWN_MIN_SEPARATION: f32 = 20.0;
const MAX_TOTAL_ENEMIES: u32 = 150;

const WAVE_INTRO_DURATION: f32 = 1.5;
const DEATH_PAUSE_DURATION: f32 = 1.5;
const GAME_OVER_INPUT_DELAY: f32 = 2.0;

const MAX_PARTICLES: u32 = 200;
const EXPLOSION_PARTICLE_COUNT: u32 = 16;
const RESCUE_PARTICLE_COUNT: u32 = 10;
const DEATH_PARTICLE_COUNT: u32 = 30;
const PARTICLE_LIFETIME: f32 = 0.5;
const PARTICLE_SPEED: f32 = 300.0;
const PARTICLE_RADIUS: f32 = 2.0;
const PARTICLE_DRAG: f32 = 3.0;

const SCREEN_SHAKE_MAX_OFFSET: f32 = 8.0;
const SCREEN_SHAKE_DECAY: f32 = 3.0;

const SCORE_POPUP_LIFETIME: f32 = 0.8;
const SCORE_POPUP_RISE_SPEED: f32 = 60.0;

const HIGH_SCORE_FILE: &str = "highscore.txt";

fn high_score_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join(HIGH_SCORE_FILE);
        }
    }
    PathBuf::from(HIGH_SCORE_FILE)
}

fn load_high_score() -> u32 {
    fs::read_to_string(high_score_path())
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_high_score(score: u32) {
    let _ = fs::write(high_score_path(), score.to_string());
}

// --- States ---

#[derive(States, Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
enum AppState {
    #[default]
    StartScreen,
    Playing,
    GameOver,
}

#[derive(SubStates, Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
enum PlayState {
    #[default]
    WaveIntro,
    WaveActive,
    WaveClear,
    PlayerDeath,
    Paused,
}

// --- Components ---

#[derive(Component)] struct Player;
#[derive(Component)] struct Enemy;
#[derive(Component)] struct Grunt;
#[derive(Component)] struct Hulk;
#[derive(Component)] struct Brain;
#[derive(Component)] struct Prog;
#[derive(Component)] struct Spheroid;
#[derive(Component)] struct Enforcer;
#[derive(Component)] struct Quark;
#[derive(Component)] struct Tank;
#[derive(Component)] struct Human;
#[derive(Component)] struct Electrode;

#[derive(Component)] struct Killable;
#[derive(Component)] struct DamagesPlayer;
#[derive(Component)] struct PlayerBullet;
#[derive(Component)] struct EnemyProjectile;
#[derive(Component)] struct WaveEntity;
#[derive(Component)] struct Confined;

#[derive(Component)] struct Velocity(Vec2);
#[derive(Component)] struct Facing(Vec2);
#[derive(Component)] struct CollisionRadius(f32);
#[derive(Component)] struct PointValue(u32);
#[derive(Component)] struct FireCooldown(Timer);
#[derive(Component)] struct Invincible(Timer);
#[derive(Component)] struct Lifetime(Timer);
#[derive(Component)] struct GruntSteerOffset(f32);
#[derive(Component)] struct Knockback(Vec2);
#[derive(Component)] struct WanderTarget(Vec2);
#[derive(Component)] struct WanderTimer(Timer);
#[derive(Component)] struct HomingMissile { turn_rate: f32 }
#[derive(Component)] struct BouncesRemaining(u32);

#[derive(Component)]
struct SpawnerState {
    children_spawned: u32,
    max_children: u32,
    cooldown: Timer,
}

#[derive(Component)] struct Particle;
#[derive(Component)] struct ScorePopup;

#[derive(Component)] struct ScoreText;
#[derive(Component)] struct LivesText;
#[derive(Component)] struct WaveText;
#[derive(Component)] struct HighScoreText;
#[derive(Component)] struct PauseOverlay;

// --- Resources ---

#[derive(Resource)]
struct GameAssets {
    player_mesh: Handle<Mesh>,
    player_material: Handle<ColorMaterial>,
    grunt_mesh: Handle<Mesh>,
    grunt_material: Handle<ColorMaterial>,
    hulk_mesh: Handle<Mesh>,
    hulk_material: Handle<ColorMaterial>,
    brain_mesh: Handle<Mesh>,
    brain_material: Handle<ColorMaterial>,
    prog_mesh: Handle<Mesh>,
    prog_material: Handle<ColorMaterial>,
    spheroid_mesh: Handle<Mesh>,
    spheroid_material: Handle<ColorMaterial>,
    enforcer_mesh: Handle<Mesh>,
    enforcer_material: Handle<ColorMaterial>,
    quark_mesh: Handle<Mesh>,
    quark_material: Handle<ColorMaterial>,
    tank_mesh: Handle<Mesh>,
    tank_material: Handle<ColorMaterial>,
    human_mesh: Handle<Mesh>,
    human_materials: [Handle<ColorMaterial>; 3],
    electrode_mesh: Handle<Mesh>,
    electrode_material: Handle<ColorMaterial>,
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<ColorMaterial>,
    missile_mesh: Handle<Mesh>,
    missile_material: Handle<ColorMaterial>,
    spark_mesh: Handle<Mesh>,
    spark_material: Handle<ColorMaterial>,
    shell_mesh: Handle<Mesh>,
    shell_material: Handle<ColorMaterial>,
    particle_mesh: Handle<Mesh>,
    particle_material: Handle<ColorMaterial>,
    border_mesh_h: Handle<Mesh>,
    border_mesh_v: Handle<Mesh>,
    border_material: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct GameState {
    score: u32,
    high_score: u32,
    lives: u32,
    current_wave: u32,
    rescue_count_this_wave: u32,
    next_extra_life_score: u32,
}

#[derive(Resource)]
struct WaveState {
    intro_timer: Timer,
    death_timer: Timer,
}

#[derive(Resource, Default)]
struct ScreenShake {
    trauma: f32,
}

#[derive(Resource)]
struct GameOverTimer(Timer);

// --- Wave Definition ---

struct WaveDefinition {
    grunts: u32,
    hulks: u32,
    brains: u32,
    spheroids: u32,
    quarks: u32,
    electrodes: u32,
    humans: u32,
    speed_mult: f32,
}

fn wave_definition(wave: u32) -> WaveDefinition {
    WaveDefinition {
        grunts: (15 + (wave - 1) * 5).min(60),
        hulks: (wave / 3).min(8),
        brains: if wave >= 3 { ((wave - 2) / 2).min(5) } else { 0 },
        spheroids: if wave >= 4 { ((wave - 3) / 3).min(3) } else { 0 },
        quarks: if wave >= 6 { ((wave - 5) / 3).min(2) } else { 0 },
        electrodes: (5 + wave * 2).min(25),
        humans: 5.min(3 + wave),
        speed_mult: 1.0 + (wave - 1) as f32 * 0.05,
    }
}

// --- App ---

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                title: "Robotron 2084".to_string(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_sub_state::<PlayState>()
        .insert_resource(GameState {
            score: 0,
            high_score: load_high_score(),
            lives: STARTING_LIVES,
            current_wave: 1,
            rescue_count_this_wave: 0,
            next_extra_life_score: EXTRA_LIFE_EVERY,
        })
        .insert_resource(WaveState {
            intro_timer: Timer::from_seconds(WAVE_INTRO_DURATION, TimerMode::Once),
            death_timer: Timer::from_seconds(DEATH_PAUSE_DURATION, TimerMode::Once),
        })
        .init_resource::<ScreenShake>()
        .insert_resource(GameOverTimer(Timer::from_seconds(
            GAME_OVER_INPUT_DELAY,
            TimerMode::Once,
        )))
        // Startup
        .add_systems(Startup, (setup_camera, setup_assets))
        // Start screen
        .add_systems(OnEnter(AppState::StartScreen), spawn_start_screen)
        .add_systems(
            Update,
            start_screen_input.run_if(in_state(AppState::StartScreen)),
        )
        // Playing - setup
        .add_systems(
            OnEnter(AppState::Playing),
            (reset_game_state, spawn_arena, spawn_player, spawn_hud),
        )
        // Wave intro
        .add_systems(
            OnEnter(PlayState::WaveIntro),
            (despawn_wave_entities, spawn_wave, spawn_wave_overlay).chain(),
        )
        .add_systems(
            Update,
            tick_wave_intro.run_if(in_state(PlayState::WaveIntro)),
        )
        // Wave active
        .add_systems(OnEnter(PlayState::WaveActive), respawn_player_if_needed)
        .add_systems(
            Update,
            (
                (player_movement, player_aim),
                (grunt_ai, hulk_ai, brain_ai, prog_ai, spawner_wander, enforcer_ai, tank_ai, human_wander),
                (player_fire, spawner_spawn, enforcer_fire, tank_fire, apply_velocity, homing_missile_steer, bounce_shell_reflect),
                confine_entities,
                player_vs_human,
                (bullet_collisions, despawn_oob_bullets),
                (hazard_vs_player, proj_vs_player, electrode_vs_killable, hulk_vs_human, brain_vs_human),
                check_wave_clear,
            )
                .chain()
                .run_if(in_state(PlayState::WaveActive)),
        )
        // Pause
        .add_systems(OnEnter(PlayState::Paused), spawn_pause_overlay)
        .add_systems(OnExit(PlayState::Paused), despawn_pause_overlay)
        .add_systems(
            Update,
            unpause_input.run_if(in_state(PlayState::Paused)),
        )
        // Pause toggle (during WaveActive)
        .add_systems(
            Update,
            pause_input.run_if(in_state(PlayState::WaveActive)),
        )
        // Player death
        .add_systems(OnEnter(PlayState::PlayerDeath), start_death_timer)
        .add_systems(
            Update,
            tick_player_death.run_if(in_state(PlayState::PlayerDeath)),
        )
        // Always-on during Playing
        .add_systems(
            Update,
            (tick_invincibility, tick_lifetimes, tick_particles, tick_score_popups, apply_screen_shake, update_hud)
                .run_if(in_state(AppState::Playing)),
        )
        // Game over
        .add_systems(OnEnter(AppState::GameOver), (spawn_game_over, start_game_over_timer))
        .add_systems(
            Update,
            game_over_input.run_if(in_state(AppState::GameOver)),
        )
        .run();
}

// --- Startup Systems ---

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: WINDOW_HEIGHT as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        DebandDither::Enabled,
    ));
}

fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player_mesh = meshes.add(RegularPolygon::new(PLAYER_RADIUS, 4));
    let grunt_mesh = meshes.add(RegularPolygon::new(GRUNT_RADIUS, 4));
    let hulk_mesh = meshes.add(RegularPolygon::new(HULK_RADIUS, 4));
    let brain_mesh = meshes.add(Circle::new(BRAIN_RADIUS));
    let prog_mesh = meshes.add(Capsule2d::new(PROG_RADIUS / 2.0, PROG_RADIUS));
    let spheroid_mesh = meshes.add(Circle::new(SPHEROID_RADIUS));
    let enforcer_mesh = meshes.add(RegularPolygon::new(ENFORCER_RADIUS, 3));
    let quark_mesh = meshes.add(RegularPolygon::new(QUARK_RADIUS, 6));
    let tank_mesh = meshes.add(RegularPolygon::new(TANK_RADIUS, 5));
    let human_mesh = meshes.add(Capsule2d::new(HUMAN_RADIUS / 2.0, HUMAN_RADIUS));
    let electrode_mesh = meshes.add(Circle::new(ELECTRODE_RADIUS));
    let bullet_mesh = meshes.add(Circle::new(BULLET_RADIUS));
    let missile_mesh = meshes.add(Circle::new(MISSILE_RADIUS));
    let spark_mesh = meshes.add(Circle::new(SPARK_RADIUS));
    let shell_mesh = meshes.add(Circle::new(SHELL_RADIUS));
    let particle_mesh = meshes.add(Circle::new(PARTICLE_RADIUS));
    let border_mesh_h = meshes.add(Capsule2d::new(
        ARENA_BORDER_THICKNESS / 2.0,
        ARENA_HALF_WIDTH * 2.0,
    ));
    let border_mesh_v = meshes.add(Capsule2d::new(
        ARENA_BORDER_THICKNESS / 2.0,
        ARENA_HALF_HEIGHT * 2.0,
    ));

    // Bright colors for bloom (values > 1.0)
    let player_material = materials.add(ColorMaterial::from_color(Color::srgb(0.2, 1.0, 5.0)));
    let grunt_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 0.2, 0.2)));
    let hulk_material = materials.add(ColorMaterial::from_color(Color::srgb(0.2, 5.0, 0.2)));
    let brain_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 0.5, 5.0)));
    let prog_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 0.2, 3.0)));
    let spheroid_material = materials.add(ColorMaterial::from_color(Color::srgb(3.0, 3.0, 5.0)));
    let enforcer_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 2.0, 0.2)));
    let quark_material = materials.add(ColorMaterial::from_color(Color::srgb(2.0, 5.0, 3.0)));
    let tank_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 3.0, 1.0)));
    let human_materials = [
        materials.add(ColorMaterial::from_color(Color::srgb(5.0, 1.0, 2.0))),
        materials.add(ColorMaterial::from_color(Color::srgb(5.0, 4.0, 0.5))),
        materials.add(ColorMaterial::from_color(Color::srgb(1.0, 2.0, 5.0))),
    ];
    let electrode_material =
        materials.add(ColorMaterial::from_color(Color::srgb(4.0, 4.0, 5.0)));
    let bullet_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 5.0, 0.5)));
    let missile_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 0.5, 1.0)));
    let spark_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 3.0, 0.2)));
    let shell_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 4.0, 2.0)));
    let particle_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 5.0, 5.0)));
    let border_material = materials.add(ColorMaterial::from_color(Color::srgb(0.5, 0.5, 5.0)));

    commands.insert_resource(GameAssets {
        player_mesh, player_material,
        grunt_mesh, grunt_material,
        hulk_mesh, hulk_material,
        brain_mesh, brain_material,
        prog_mesh, prog_material,
        spheroid_mesh, spheroid_material,
        enforcer_mesh, enforcer_material,
        quark_mesh, quark_material,
        tank_mesh, tank_material,
        human_mesh, human_materials,
        electrode_mesh, electrode_material,
        bullet_mesh, bullet_material,
        missile_mesh, missile_material,
        spark_mesh, spark_material,
        shell_mesh, shell_material,
        particle_mesh, particle_material,
        border_mesh_h, border_mesh_v, border_material,
    });
}

// --- Start Screen ---

fn spawn_start_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            DespawnOnExit(AppState::StartScreen),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("ROBOTRON 2084"),
                TextFont { font_size: 60.0, ..default() },
                TextColor(Color::srgb(5.0, 1.0, 0.2)),
            ));
            parent.spawn((
                Text::new("WASD to move  |  Arrow keys to aim & fire"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("ESC to pause"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
            parent.spawn((
                Text::new("Press SPACE to start"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

fn start_screen_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

// --- Playing Setup ---

fn reset_game_state(mut game: ResMut<GameState>, mut shake: ResMut<ScreenShake>) {
    game.score = 0;
    game.lives = STARTING_LIVES;
    game.current_wave = 1;
    game.rescue_count_this_wave = 0;
    game.next_extra_life_score = EXTRA_LIFE_EVERY;
    shake.trauma = 0.0;
}

fn spawn_arena(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Mesh2d(assets.border_mesh_h.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(0.0, ARENA_HALF_HEIGHT, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        Mesh2d(assets.border_mesh_h.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(0.0, -ARENA_HALF_HEIGHT, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        Mesh2d(assets.border_mesh_v.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(-ARENA_HALF_WIDTH, 0.0, 0.0),
        DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        Mesh2d(assets.border_mesh_v.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(ARENA_HALF_WIDTH, 0.0, 0.0),
        DespawnOnExit(AppState::Playing),
    ));
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    spawn_player_entity(&mut commands, &assets);
}

fn spawn_player_entity(commands: &mut Commands, assets: &GameAssets) {
    commands.spawn((
        Player, Confined,
        Mesh2d(assets.player_mesh.clone()),
        MeshMaterial2d(assets.player_material.clone()),
        Transform::from_xyz(0.0, 0.0, 1.0),
        Velocity(Vec2::ZERO),
        Facing(Vec2::Y),
        CollisionRadius(PLAYER_RADIUS),
        FireCooldown(Timer::from_seconds(PLAYER_FIRE_COOLDOWN, TimerMode::Once)),
        Invincible(Timer::from_seconds(PLAYER_INVINCIBILITY_DURATION, TimerMode::Once)),
        DespawnOnExit(AppState::Playing),
    ));
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Text::new("SCORE: 0"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(10.0), ..default() },
        ScoreText, DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        Text::new("WAVE: 1"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(5.0, 5.0, 0.5)),
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Percent(50.0), ..default() },
        WaveText, DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        Text::new(format!("LIVES: {}", STARTING_LIVES)),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Px(10.0), right: Val::Px(10.0), ..default() },
        LivesText, DespawnOnExit(AppState::Playing),
    ));
    commands.spawn((
        Text::new("HI: 0"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(10.0), left: Val::Px(10.0), ..default() },
        HighScoreText, DespawnOnExit(AppState::Playing),
    ));
}

// --- Wave Management ---

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
                0 => (rng.random_range(-ARENA_HALF_WIDTH + radius..ARENA_HALF_WIDTH - radius), ARENA_HALF_HEIGHT - radius),
                1 => (rng.random_range(-ARENA_HALF_WIDTH + radius..ARENA_HALF_WIDTH - radius), -ARENA_HALF_HEIGHT + radius),
                2 => (-ARENA_HALF_WIDTH + radius, rng.random_range(-ARENA_HALF_HEIGHT + radius..ARENA_HALF_HEIGHT - radius)),
                _ => (ARENA_HALF_WIDTH - radius, rng.random_range(-ARENA_HALF_HEIGHT + radius..ARENA_HALF_HEIGHT - radius)),
            };
            if x * x + y * y > SPAWN_EXCLUSION_RADIUS * SPAWN_EXCLUSION_RADIUS {
                return (x, y);
            }
        }
    };

    let spawn_inside = |rng: &mut rand::rngs::ThreadRng, radius: f32, occupied: &[Vec2]| -> (f32, f32) {
        loop {
            let x = rng.random_range(-ARENA_HALF_WIDTH + radius..ARENA_HALF_WIDTH - radius);
            let y = rng.random_range(-ARENA_HALF_HEIGHT + radius..ARENA_HALF_HEIGHT - radius);
            if x * x + y * y < SPAWN_EXCLUSION_RADIUS * SPAWN_EXCLUSION_RADIUS { continue; }
            let pos = Vec2::new(x, y);
            if occupied.iter().any(|o| o.distance(pos) < SPAWN_MIN_SEPARATION) { continue; }
            return (x, y);
        }
    };

    // Grunts
    for _ in 0..def.grunts {
        let (x, y) = spawn_on_edge(&mut rng, GRUNT_RADIUS);
        let steer_offset: f32 = rng.random_range(-0.5..0.5);
        commands.spawn((
            Enemy, Grunt, Killable, DamagesPlayer, Confined, WaveEntity,
            Mesh2d(assets.grunt_mesh.clone()),
            MeshMaterial2d(assets.grunt_material.clone()),
            Transform::from_xyz(x, y, 1.0).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            Velocity(Vec2::ZERO), CollisionRadius(GRUNT_RADIUS), PointValue(100),
            GruntSteerOffset(steer_offset),
            DespawnOnExit(AppState::Playing),
        ));
    }

    // Hulks
    for _ in 0..def.hulks {
        let (x, y) = spawn_on_edge(&mut rng, HULK_RADIUS);
        commands.spawn((
            Enemy, Hulk, DamagesPlayer, Confined, WaveEntity,
            Mesh2d(assets.hulk_mesh.clone()),
            MeshMaterial2d(assets.hulk_material.clone()),
            Transform::from_xyz(x, y, 1.0).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            Velocity(Vec2::ZERO), CollisionRadius(HULK_RADIUS), PointValue(0),
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
            Enemy, Brain, Killable, DamagesPlayer, Confined, WaveEntity,
            Mesh2d(assets.brain_mesh.clone()),
            MeshMaterial2d(assets.brain_material.clone()),
            Transform::from_xyz(x, y, 1.0),
            Velocity(Vec2::ZERO), CollisionRadius(BRAIN_RADIUS), PointValue(500),
            FireCooldown(Timer::from_seconds(BRAIN_MISSILE_COOLDOWN, TimerMode::Repeating)),
            DespawnOnExit(AppState::Playing),
        ));
    }

    // Spheroids
    for _ in 0..def.spheroids {
        let (x, y) = spawn_on_edge(&mut rng, SPHEROID_RADIUS);
        commands.spawn((
            Enemy, Spheroid, Killable, DamagesPlayer, Confined, WaveEntity,
            Mesh2d(assets.spheroid_mesh.clone()),
            MeshMaterial2d(assets.spheroid_material.clone()),
            Transform::from_xyz(x, y, 1.0),
            Velocity(Vec2::ZERO), CollisionRadius(SPHEROID_RADIUS), PointValue(1000),
            DespawnOnExit(AppState::Playing),
        )).insert((
            SpawnerState { children_spawned: 0, max_children: SPHEROID_MAX_CHILDREN, cooldown: Timer::from_seconds(SPHEROID_SPAWN_COOLDOWN, TimerMode::Repeating) },
            WanderTarget(Vec2::ZERO),
            WanderTimer(Timer::from_seconds(2.0, TimerMode::Repeating)),
        ));
    }

    // Quarks
    for _ in 0..def.quarks {
        let (x, y) = spawn_on_edge(&mut rng, QUARK_RADIUS);
        commands.spawn((
            Enemy, Quark, Killable, DamagesPlayer, Confined, WaveEntity,
            Mesh2d(assets.quark_mesh.clone()),
            MeshMaterial2d(assets.quark_material.clone()),
            Transform::from_xyz(x, y, 1.0),
            Velocity(Vec2::ZERO), CollisionRadius(QUARK_RADIUS), PointValue(1000),
            DespawnOnExit(AppState::Playing),
        )).insert((
            SpawnerState { children_spawned: 0, max_children: QUARK_MAX_CHILDREN, cooldown: Timer::from_seconds(QUARK_SPAWN_COOLDOWN, TimerMode::Repeating) },
            WanderTarget(Vec2::ZERO),
            WanderTimer(Timer::from_seconds(2.5, TimerMode::Repeating)),
        ));
    }

    // Electrodes
    for _ in 0..def.electrodes {
        let (x, y) = spawn_inside(&mut rng, ELECTRODE_RADIUS, &occupied);
        occupied.push(Vec2::new(x, y));
        commands.spawn((
            Electrode, DamagesPlayer, WaveEntity,
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
            Human, Confined, WaveEntity,
            Mesh2d(assets.human_mesh.clone()),
            MeshMaterial2d(assets.human_materials[variant].clone()),
            Transform::from_xyz(x, y, 0.8),
            Velocity(Vec2::ZERO), CollisionRadius(HUMAN_RADIUS),
            WanderTarget(Vec2::ZERO),
            WanderTimer(Timer::from_seconds(HUMAN_WANDER_INTERVAL, TimerMode::Repeating)),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

fn spawn_wave_overlay(mut commands: Commands, game: Res<GameState>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()
            },
            DespawnOnExit(PlayState::WaveIntro),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("WAVE {}", game.current_wave)),
                TextFont { font_size: 48.0, ..default() },
                TextColor(Color::srgb(5.0, 5.0, 0.5)),
            ));
        });
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

fn respawn_player_if_needed(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_q: Query<(), With<Player>>,
) {
    if player_q.is_empty() {
        spawn_player_entity(&mut commands, &assets);
    }
}

// --- Player Systems ---

fn player_movement(input: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    let Ok(mut vel) = query.single_mut() else { return };
    let mut dir = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) { dir.y += 1.0; }
    if input.pressed(KeyCode::KeyS) { dir.y -= 1.0; }
    if input.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
    if input.pressed(KeyCode::KeyD) { dir.x += 1.0; }
    vel.0 = dir.normalize_or_zero() * PLAYER_SPEED;
}

fn player_aim(input: Res<ButtonInput<KeyCode>>, mut query: Query<(&mut Facing, &mut Transform), With<Player>>) {
    let Ok((mut facing, mut tf)) = query.single_mut() else { return };
    let mut dir = Vec2::ZERO;
    if input.pressed(KeyCode::ArrowUp) { dir.y += 1.0; }
    if input.pressed(KeyCode::ArrowDown) { dir.y -= 1.0; }
    if input.pressed(KeyCode::ArrowLeft) { dir.x -= 1.0; }
    if input.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }
    if dir != Vec2::ZERO {
        let dir = dir.normalize();
        facing.0 = dir;
        tf.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
    }
}

fn player_fire(
    mut commands: Commands, time: Res<Time>, input: Res<ButtonInput<KeyCode>>,
    assets: Res<GameAssets>,
    mut player_q: Query<(&Transform, &Facing, &mut FireCooldown), With<Player>>,
    bullet_q: Query<(), With<PlayerBullet>>,
) {
    let Ok((tf, facing, mut cooldown)) = player_q.single_mut() else { return };
    cooldown.0.tick(time.delta());
    let aiming = input.pressed(KeyCode::ArrowUp) || input.pressed(KeyCode::ArrowDown)
        || input.pressed(KeyCode::ArrowLeft) || input.pressed(KeyCode::ArrowRight);
    if !aiming || !cooldown.0.is_finished() { return; }
    if bullet_q.iter().count() >= MAX_PLAYER_BULLETS as usize { return; }
    cooldown.0.reset();
    let pos = tf.translation.truncate();
    commands.spawn((
        PlayerBullet,
        Mesh2d(assets.bullet_mesh.clone()),
        MeshMaterial2d(assets.bullet_material.clone()),
        Transform::from_xyz(pos.x, pos.y, 0.5),
        Velocity(facing.0 * BULLET_SPEED), CollisionRadius(BULLET_RADIUS),
        DespawnOnExit(AppState::Playing),
    ));
}

// --- Enemy AI Systems ---

fn grunt_ai(
    player_q: Query<&Transform, With<Player>>,
    mut grunt_q: Query<(&Transform, &mut Velocity, &GruntSteerOffset), With<Grunt>>,
    game: Res<GameState>,
) {
    let Ok(player_tf) = player_q.single() else { return };
    let player_pos = player_tf.translation.truncate();
    let speed_mult = wave_definition(game.current_wave).speed_mult;
    for (tf, mut vel, offset) in &mut grunt_q {
        let pos = tf.translation.truncate();
        let dir = (player_pos - pos).normalize_or_zero();
        let angle = dir.y.atan2(dir.x) + offset.0;
        vel.0 = Vec2::new(angle.cos(), angle.sin()) * GRUNT_BASE_SPEED * speed_mult;
    }
}

fn hulk_ai(
    time: Res<Time>, player_q: Query<&Transform, With<Player>>,
    mut hulk_q: Query<(&Transform, &mut Velocity, &mut WanderTimer, &mut WanderTarget, &mut Knockback), With<Hulk>>,
) {
    let player_pos = player_q.single().map(|tf| tf.translation.truncate()).unwrap_or(Vec2::ZERO);
    let mut rng = rand::rng();
    let dt = time.delta_secs();
    for (tf, mut vel, mut timer, mut target, mut kb) in &mut hulk_q {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let pos = tf.translation.truncate();
            let bias = (player_pos - pos).normalize_or_zero() * 0.3;
            let random_dir = Vec2::new(rng.random_range(-1.0f32..1.0), rng.random_range(-1.0f32..1.0)).normalize_or_zero();
            target.0 = (random_dir * 0.7 + bias).normalize_or_zero();
        }
        kb.0 *= (-HULK_KNOCKBACK_DECAY * dt).exp();
        if kb.0.length() < 1.0 { kb.0 = Vec2::ZERO; }
        vel.0 = target.0 * HULK_SPEED + kb.0;
    }
}

fn brain_ai(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_q: Query<&Transform, With<Player>>,
    human_q: Query<&Transform, With<Human>>,
    mut brain_q: Query<(&Transform, &mut Velocity, &mut FireCooldown), With<Brain>>,
    enemy_count: Query<(), With<Enemy>>,
) {
    let player_pos = player_q.single().map(|tf| tf.translation.truncate()).unwrap_or(Vec2::ZERO);
    for (tf, mut vel, mut cooldown) in &mut brain_q {
        let pos = tf.translation.truncate();
        // Seek nearest human, or player if none
        let target = human_q.iter()
            .map(|h| h.translation.truncate())
            .min_by(|a, b| a.distance_squared(pos).partial_cmp(&b.distance_squared(pos)).unwrap())
            .unwrap_or(player_pos);
        let dir = (target - pos).normalize_or_zero();
        vel.0 = dir * BRAIN_SPEED;

        // Fire homing missiles
        cooldown.0.tick(time.delta());
        if cooldown.0.just_finished() && enemy_count.iter().count() < MAX_TOTAL_ENEMIES as usize {
            let missile_dir = (player_pos - pos).normalize_or_zero();
            commands.spawn((
                EnemyProjectile, DamagesPlayer, WaveEntity,
                Mesh2d(assets.missile_mesh.clone()),
                MeshMaterial2d(assets.missile_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 0.5),
                Velocity(missile_dir * MISSILE_SPEED),
                CollisionRadius(MISSILE_RADIUS),
                HomingMissile { turn_rate: MISSILE_TURN_RATE },
                Lifetime(Timer::from_seconds(MISSILE_LIFETIME, TimerMode::Once)),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

fn prog_ai(
    player_q: Query<&Transform, With<Player>>,
    mut prog_q: Query<(&Transform, &mut Velocity), With<Prog>>,
    game: Res<GameState>,
) {
    let Ok(player_tf) = player_q.single() else { return };
    let player_pos = player_tf.translation.truncate();
    let speed_mult = wave_definition(game.current_wave).speed_mult;
    for (tf, mut vel) in &mut prog_q {
        let dir = (player_pos - tf.translation.truncate()).normalize_or_zero();
        vel.0 = dir * PROG_SPEED * speed_mult;
    }
}

fn spawner_wander(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut WanderTimer, &mut WanderTarget), Or<(With<Spheroid>, With<Quark>)>>,
) {
    let mut rng = rand::rng();
    for (mut vel, mut timer, mut target) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let tx = rng.random_range(-ARENA_HALF_WIDTH * 0.8..ARENA_HALF_WIDTH * 0.8);
            let ty = rng.random_range(-ARENA_HALF_HEIGHT * 0.8..ARENA_HALF_HEIGHT * 0.8);
            target.0 = Vec2::new(tx, ty);
        }
        // Move toward target point
        let speed = if vel.0.length() > 0.0 { vel.0.length() } else { SPHEROID_SPEED };
        vel.0 = target.0.normalize_or_zero() * speed;
    }
}

fn spawner_spawn(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut spheroid_q: Query<(&Transform, &mut SpawnerState), With<Spheroid>>,
    mut quark_q: Query<(&Transform, &mut SpawnerState), (With<Quark>, Without<Spheroid>)>,
    enemy_count: Query<(), With<Enemy>>,
) {
    let total = enemy_count.iter().count();
    // Spheroids spawn Enforcers
    for (tf, mut state) in &mut spheroid_q {
        state.cooldown.tick(time.delta());
        if state.cooldown.just_finished() && state.children_spawned < state.max_children && total < MAX_TOTAL_ENEMIES as usize {
            state.children_spawned += 1;
            let pos = tf.translation.truncate();
            commands.spawn((
                Enemy, Enforcer, Killable, DamagesPlayer, Confined, WaveEntity,
                Mesh2d(assets.enforcer_mesh.clone()),
                MeshMaterial2d(assets.enforcer_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 1.0),
                Velocity(Vec2::ZERO), CollisionRadius(ENFORCER_RADIUS), PointValue(150),
                DespawnOnExit(AppState::Playing),
            )).insert((
                FireCooldown(Timer::from_seconds(ENFORCER_FIRE_COOLDOWN, TimerMode::Repeating)),
                WanderTarget(Vec2::ZERO),
                WanderTimer(Timer::from_seconds(2.0, TimerMode::Repeating)),
            ));
        }
    }
    // Quarks spawn Tanks
    for (tf, mut state) in &mut quark_q {
        state.cooldown.tick(time.delta());
        if state.cooldown.just_finished() && state.children_spawned < state.max_children && total < MAX_TOTAL_ENEMIES as usize {
            state.children_spawned += 1;
            let pos = tf.translation.truncate();
            commands.spawn((
                Enemy, Tank, Killable, DamagesPlayer, Confined, WaveEntity,
                Mesh2d(assets.tank_mesh.clone()),
                MeshMaterial2d(assets.tank_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 1.0),
                Velocity(Vec2::ZERO), CollisionRadius(TANK_RADIUS), PointValue(200),
                DespawnOnExit(AppState::Playing),
            )).insert((
                FireCooldown(Timer::from_seconds(TANK_FIRE_COOLDOWN, TimerMode::Repeating)),
                WanderTarget(Vec2::ZERO),
                WanderTimer(Timer::from_seconds(3.0, TimerMode::Repeating)),
            ));
        }
    }
}

fn enforcer_ai(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &mut Velocity, &mut WanderTimer, &mut WanderTarget), With<Enforcer>>,
) {
    let player_pos = player_q.single().map(|tf| tf.translation.truncate()).unwrap_or(Vec2::ZERO);
    let mut rng = rand::rng();
    for (tf, mut vel, mut timer, mut target) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let pos = tf.translation.truncate();
            let bias = (player_pos - pos).normalize_or_zero() * 0.3;
            let rand_dir = Vec2::new(rng.random_range(-1.0f32..1.0), rng.random_range(-1.0f32..1.0)).normalize_or_zero();
            target.0 = (rand_dir * 0.7 + bias).normalize_or_zero();
        }
        vel.0 = target.0 * ENFORCER_SPEED;
    }
}

fn enforcer_fire(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_q: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &mut FireCooldown), With<Enforcer>>,
) {
    let Ok(player_tf) = player_q.single() else { return };
    let player_pos = player_tf.translation.truncate();
    for (tf, mut cooldown) in &mut query {
        cooldown.0.tick(time.delta());
        if cooldown.0.just_finished() {
            let pos = tf.translation.truncate();
            let dir = (player_pos - pos).normalize_or_zero();
            commands.spawn((
                EnemyProjectile, DamagesPlayer, WaveEntity,
                Mesh2d(assets.spark_mesh.clone()),
                MeshMaterial2d(assets.spark_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 0.5),
                Velocity(dir * SPARK_SPEED), CollisionRadius(SPARK_RADIUS),
                Lifetime(Timer::from_seconds(SPARK_LIFETIME, TimerMode::Once)),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

fn tank_ai(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &mut Velocity, &mut WanderTimer, &mut WanderTarget), With<Tank>>,
) {
    let player_pos = player_q.single().map(|tf| tf.translation.truncate()).unwrap_or(Vec2::ZERO);
    let mut rng = rand::rng();
    for (tf, mut vel, mut timer, mut target) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let pos = tf.translation.truncate();
            let bias = (player_pos - pos).normalize_or_zero() * 0.2;
            let rand_dir = Vec2::new(rng.random_range(-1.0f32..1.0), rng.random_range(-1.0f32..1.0)).normalize_or_zero();
            target.0 = (rand_dir * 0.8 + bias).normalize_or_zero();
        }
        vel.0 = target.0 * TANK_SPEED;
    }
}

fn tank_fire(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_q: Query<&Transform, With<Player>>,
    mut query: Query<(&Transform, &mut FireCooldown), With<Tank>>,
) {
    let Ok(player_tf) = player_q.single() else { return };
    let player_pos = player_tf.translation.truncate();
    for (tf, mut cooldown) in &mut query {
        cooldown.0.tick(time.delta());
        if cooldown.0.just_finished() {
            let pos = tf.translation.truncate();
            let dir = (player_pos - pos).normalize_or_zero();
            commands.spawn((
                EnemyProjectile, DamagesPlayer, Confined, WaveEntity,
                Mesh2d(assets.shell_mesh.clone()),
                MeshMaterial2d(assets.shell_material.clone()),
                Transform::from_xyz(pos.x, pos.y, 0.5),
                Velocity(dir * SHELL_SPEED), CollisionRadius(SHELL_RADIUS),
                BouncesRemaining(SHELL_MAX_BOUNCES),
                Lifetime(Timer::from_seconds(SHELL_LIFETIME, TimerMode::Once)),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

fn human_wander(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut WanderTimer, &mut WanderTarget), With<Human>>,
) {
    let mut rng = rand::rng();
    for (mut vel, mut timer, mut target) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
            target.0 = Vec2::new(angle.cos(), angle.sin());
        }
        vel.0 = target.0 * HUMAN_SPEED;
    }
}

// --- Projectile Systems ---

fn homing_missile_steer(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut missile_q: Query<(&Transform, &mut Velocity, &HomingMissile)>,
) {
    let Ok(player_tf) = player_q.single() else { return };
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_secs();
    for (tf, mut vel, homing) in &mut missile_q {
        let pos = tf.translation.truncate();
        let desired = (player_pos - pos).normalize_or_zero();
        let current = vel.0.normalize_or_zero();
        let cross = current.x * desired.y - current.y * desired.x;
        let turn = cross.clamp(-1.0, 1.0) * homing.turn_rate * dt;
        let speed = vel.0.length();
        let angle = current.y.atan2(current.x) + turn;
        vel.0 = Vec2::new(angle.cos(), angle.sin()) * speed;
    }
}

fn bounce_shell_reflect(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Velocity, &mut BouncesRemaining)>,
) {
    for (entity, tf, mut vel, mut bounces) in &mut query {
        let pos = tf.translation;
        let mut bounced = false;
        if pos.x <= -ARENA_HALF_WIDTH + SHELL_RADIUS || pos.x >= ARENA_HALF_WIDTH - SHELL_RADIUS {
            vel.0.x = -vel.0.x;
            bounced = true;
        }
        if pos.y <= -ARENA_HALF_HEIGHT + SHELL_RADIUS || pos.y >= ARENA_HALF_HEIGHT - SHELL_RADIUS {
            vel.0.y = -vel.0.y;
            bounced = true;
        }
        if bounced {
            if bounces.0 == 0 {
                commands.entity(entity).despawn();
            } else {
                bounces.0 -= 1;
            }
        }
    }
}

// --- Movement Systems ---

fn apply_velocity(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let dt = time.delta_secs();
    for (vel, mut tf) in &mut query {
        tf.translation.x += vel.0.x * dt;
        tf.translation.y += vel.0.y * dt;
    }
}

fn confine_entities(mut query: Query<(&mut Transform, &CollisionRadius), With<Confined>>) {
    for (mut tf, radius) in &mut query {
        tf.translation.x = tf.translation.x.clamp(-ARENA_HALF_WIDTH + radius.0, ARENA_HALF_WIDTH - radius.0);
        tf.translation.y = tf.translation.y.clamp(-ARENA_HALF_HEIGHT + radius.0, ARENA_HALF_HEIGHT - radius.0);
    }
}

fn despawn_oob_bullets(mut commands: Commands, query: Query<(Entity, &Transform), With<PlayerBullet>>) {
    for (entity, tf) in &query {
        let pos = tf.translation;
        if pos.x < -ARENA_HALF_WIDTH - 50.0 || pos.x > ARENA_HALF_WIDTH + 50.0
            || pos.y < -ARENA_HALF_HEIGHT - 50.0 || pos.y > ARENA_HALF_HEIGHT + 50.0
        {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_lifetimes(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Lifetime)>) {
    for (entity, mut lt) in &mut query {
        lt.0.tick(time.delta());
        if lt.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// --- Combat Systems ---

fn player_vs_human(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    assets: Res<GameAssets>,
    player_q: Query<(&Transform, &CollisionRadius), With<Player>>,
    human_q: Query<(Entity, &Transform, &CollisionRadius), With<Human>>,
) {
    let Ok((p_tf, p_radius)) = player_q.single() else { return };
    let p_pos = p_tf.translation.truncate();
    for (h_entity, h_tf, h_radius) in &human_q {
        let h_pos = h_tf.translation.truncate();
        let radii = p_radius.0 + h_radius.0;
        if p_pos.distance_squared(h_pos) < radii * radii {
            commands.entity(h_entity).despawn();
            game.rescue_count_this_wave += 1;
            let bonus = match game.rescue_count_this_wave {
                1 => 1000, 2 => 2000, 3 => 3000, 4 => 4000, _ => 5000,
            };
            award_score(&mut game, bonus);
            spawn_particles(&mut commands, &assets, h_pos, RESCUE_PARTICLE_COUNT, Color::srgb(1.0, 2.0, 5.0));
            spawn_score_popup(&mut commands, h_pos, bonus);
        }
    }
}

fn bullet_collisions(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    mut shake: ResMut<ScreenShake>,
    assets: Res<GameAssets>,
    bullet_q: Query<(Entity, &Transform, &Velocity, &CollisionRadius), With<PlayerBullet>>,
    killable_q: Query<(Entity, &Transform, &CollisionRadius, &PointValue), With<Killable>>,
    mut hulk_q: Query<(&Transform, &CollisionRadius, &mut Knockback), With<Hulk>>,
    electrode_q: Query<(Entity, &Transform, &CollisionRadius), With<Electrode>>,
) {
    for (b_entity, b_tf, b_vel, b_radius) in &bullet_q {
        let b_pos = b_tf.translation.truncate();
        let mut hit = false;

        for (e_entity, e_tf, e_radius, points) in &killable_q {
            let e_pos = e_tf.translation.truncate();
            let radii = b_radius.0 + e_radius.0;
            if b_pos.distance_squared(e_pos) < radii * radii {
                commands.entity(b_entity).despawn();
                commands.entity(e_entity).despawn();
                award_score(&mut game, points.0);
                spawn_particles(&mut commands, &assets, e_pos, EXPLOSION_PARTICLE_COUNT, Color::srgb(5.0, 1.0, 0.2));
                spawn_score_popup(&mut commands, e_pos, points.0);
                hit = true;
                break;
            }
        }

        if !hit {
            for (h_tf, h_radius, mut kb) in &mut hulk_q {
                let h_pos = h_tf.translation.truncate();
                let radii = b_radius.0 + h_radius.0;
                if b_pos.distance_squared(h_pos) < radii * radii {
                    commands.entity(b_entity).despawn();
                    kb.0 += b_vel.0.normalize_or_zero() * HULK_KNOCKBACK_STRENGTH;
                    shake.trauma = (shake.trauma + 0.1).min(1.0);
                    hit = true;
                    break;
                }
            }
        }

        if !hit {
            for (el_entity, el_tf, el_radius) in &electrode_q {
                let el_pos = el_tf.translation.truncate();
                let radii = b_radius.0 + el_radius.0;
                if b_pos.distance_squared(el_pos) < radii * radii {
                    commands.entity(b_entity).despawn();
                    commands.entity(el_entity).despawn();
                    award_score(&mut game, 25);
                    spawn_particles(&mut commands, &assets, el_pos, 8, Color::srgb(4.0, 4.0, 5.0));
                    hit = true;
                    break;
                }
            }
        }

        let _ = hit;
    }
}

fn hazard_vs_player(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    mut next_play: ResMut<NextState<PlayState>>,
    mut shake: ResMut<ScreenShake>,
    assets: Res<GameAssets>,
    player_q: Query<(Entity, &Transform, &CollisionRadius, Option<&Invincible>), With<Player>>,
    hazard_q: Query<(&Transform, &CollisionRadius), (With<DamagesPlayer>, Without<EnemyProjectile>)>,
) {
    let Ok((p_entity, p_tf, p_radius, invincible)) = player_q.single() else { return };
    if invincible.is_some() { return; }
    let p_pos = p_tf.translation.truncate();
    for (h_tf, h_radius) in &hazard_q {
        let h_pos = h_tf.translation.truncate();
        let radii = p_radius.0 + h_radius.0;
        if p_pos.distance_squared(h_pos) < radii * radii {
            commands.entity(p_entity).despawn();
            game.lives = game.lives.saturating_sub(1);
            shake.trauma = 1.0;
            spawn_particles(&mut commands, &assets, p_pos, DEATH_PARTICLE_COUNT, Color::srgb(0.2, 1.0, 5.0));
            next_play.set(PlayState::PlayerDeath);
            return;
        }
    }
}

fn proj_vs_player(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    mut next_play: ResMut<NextState<PlayState>>,
    mut shake: ResMut<ScreenShake>,
    assets: Res<GameAssets>,
    player_q: Query<(Entity, &Transform, &CollisionRadius, Option<&Invincible>), With<Player>>,
    proj_q: Query<(Entity, &Transform, &CollisionRadius), With<EnemyProjectile>>,
) {
    let Ok((p_entity, p_tf, p_radius, invincible)) = player_q.single() else { return };
    if invincible.is_some() { return; }
    let p_pos = p_tf.translation.truncate();
    for (proj_entity, proj_tf, proj_r) in &proj_q {
        let proj_pos = proj_tf.translation.truncate();
        let radii = p_radius.0 + proj_r.0;
        if p_pos.distance_squared(proj_pos) < radii * radii {
            commands.entity(p_entity).despawn();
            commands.entity(proj_entity).despawn();
            game.lives = game.lives.saturating_sub(1);
            shake.trauma = 1.0;
            spawn_particles(&mut commands, &assets, p_pos, DEATH_PARTICLE_COUNT, Color::srgb(0.2, 1.0, 5.0));
            next_play.set(PlayState::PlayerDeath);
            return;
        }
    }
}

fn electrode_vs_killable(
    mut commands: Commands,
    electrode_q: Query<(&Transform, &CollisionRadius), With<Electrode>>,
    killable_q: Query<(Entity, &Transform, &CollisionRadius), With<Killable>>,
) {
    for (el_tf, el_radius) in &electrode_q {
        let el_pos = el_tf.translation.truncate();
        for (k_entity, k_tf, k_radius) in &killable_q {
            let k_pos = k_tf.translation.truncate();
            let radii = el_radius.0 + k_radius.0;
            if el_pos.distance_squared(k_pos) < radii * radii {
                commands.entity(k_entity).despawn();
            }
        }
    }
}

fn hulk_vs_human(
    mut commands: Commands,
    hulk_q: Query<(&Transform, &CollisionRadius), With<Hulk>>,
    human_q: Query<(Entity, &Transform, &CollisionRadius), With<Human>>,
) {
    for (hulk_tf, hulk_r) in &hulk_q {
        let hulk_pos = hulk_tf.translation.truncate();
        for (h_entity, h_tf, h_r) in &human_q {
            let h_pos = h_tf.translation.truncate();
            let radii = hulk_r.0 + h_r.0;
            if hulk_pos.distance_squared(h_pos) < radii * radii {
                commands.entity(h_entity).despawn();
            }
        }
    }
}

fn brain_vs_human(
    mut commands: Commands,
    assets: Res<GameAssets>,
    brain_q: Query<(&Transform, &CollisionRadius), With<Brain>>,
    human_q: Query<(Entity, &Transform, &CollisionRadius), With<Human>>,
    game: Res<GameState>,
) {
    let speed_mult = wave_definition(game.current_wave).speed_mult;
    for (br_tf, br_r) in &brain_q {
        let br_pos = br_tf.translation.truncate();
        for (h_entity, h_tf, h_r) in &human_q {
            let h_pos = h_tf.translation.truncate();
            let radii = br_r.0 + h_r.0;
            if br_pos.distance_squared(h_pos) < radii * radii {
                commands.entity(h_entity).despawn();
                // Convert to Prog
                commands.spawn((
                    Enemy, Prog, Killable, DamagesPlayer, Confined, WaveEntity,
                    Mesh2d(assets.prog_mesh.clone()),
                    MeshMaterial2d(assets.prog_material.clone()),
                    Transform::from_xyz(h_pos.x, h_pos.y, 1.0),
                    Velocity(Vec2::ZERO),
                    CollisionRadius(PROG_RADIUS),
                    PointValue(100),
                    DespawnOnExit(AppState::Playing),
                ));
                let _ = speed_mult; // will be used by prog_ai when it runs
            }
        }
    }
}

fn check_wave_clear(
    mut game: ResMut<GameState>,
    mut next_state: ResMut<NextState<PlayState>>,
    mut shake: ResMut<ScreenShake>,
    killable_q: Query<(), With<Killable>>,
) {
    if killable_q.is_empty() {
        game.current_wave += 1;
        shake.trauma = (shake.trauma + 0.3).min(1.0);
        next_state.set(PlayState::WaveIntro);
    }
}

// --- Player Death ---

fn start_death_timer(mut wave_state: ResMut<WaveState>) {
    wave_state.death_timer.reset();
}

fn tick_player_death(
    time: Res<Time>,
    mut wave_state: ResMut<WaveState>,
    mut game: ResMut<GameState>,
    mut next_app: ResMut<NextState<AppState>>,
    mut next_play: ResMut<NextState<PlayState>>,
    killable_q: Query<(), With<Killable>>,
) {
    wave_state.death_timer.tick(time.delta());
    if !wave_state.death_timer.is_finished() { return; }
    if game.lives == 0 {
        next_app.set(AppState::GameOver);
    } else if killable_q.is_empty() {
        game.current_wave += 1;
        next_play.set(PlayState::WaveIntro);
    } else {
        next_play.set(PlayState::WaveActive);
    }
}

// --- Pause ---

fn pause_input(input: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<PlayState>>) {
    if input.just_pressed(KeyCode::Escape) {
        next_state.set(PlayState::Paused);
    }
}

fn unpause_input(input: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<PlayState>>) {
    if input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::Space) {
        next_state.set(PlayState::WaveActive);
    }
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            PauseOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont { font_size: 48.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

fn despawn_pause_overlay(mut commands: Commands, query: Query<Entity, With<PauseOverlay>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// --- Invincibility ---

fn tick_invincibility(
    mut commands: Commands, time: Res<Time>,
    mut query: Query<(Entity, &mut Invincible, &mut Visibility)>,
) {
    for (entity, mut inv, mut vis) in &mut query {
        inv.0.tick(time.delta());
        let elapsed = inv.0.elapsed_secs();
        let blink = (elapsed / PLAYER_BLINK_INTERVAL) as u32 % 2 == 0;
        *vis = if blink { Visibility::Inherited } else { Visibility::Hidden };
        if inv.0.is_finished() {
            commands.entity(entity).remove::<Invincible>();
            *vis = Visibility::Inherited;
        }
    }
}

// --- Particles ---

fn spawn_particles(commands: &mut Commands, assets: &GameAssets, pos: Vec2, count: u32, color: Color) {
    let mut rng = rand::rng();
    let particle_count = commands.spawn_batch(Vec::<()>::new()); // no-op to avoid unused
    let _ = particle_count;
    for _ in 0..count {
        let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(PARTICLE_SPEED * 0.3..PARTICLE_SPEED);
        let _ = color; // color used below
        commands.spawn((
            Particle, WaveEntity,
            Mesh2d(assets.particle_mesh.clone()),
            MeshMaterial2d(assets.particle_material.clone()),
            Transform::from_xyz(pos.x, pos.y, 2.0),
            Velocity(Vec2::new(angle.cos(), angle.sin()) * speed),
            Lifetime(Timer::from_seconds(PARTICLE_LIFETIME, TimerMode::Once)),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

fn tick_particles(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform, &Lifetime), With<Particle>>,
) {
    let dt = time.delta_secs();
    for (mut vel, mut tf, lifetime) in &mut query {
        vel.0 *= (-PARTICLE_DRAG * dt).exp();
        let frac = lifetime.0.fraction_remaining();
        tf.scale = Vec3::splat(frac);
    }
}

// --- Score Popups ---

fn spawn_score_popup(commands: &mut Commands, pos: Vec2, points: u32) {
    commands.spawn((
        ScorePopup,
        Text2d::new(format!("+{}", points)),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(5.0, 5.0, 0.5)),
        Transform::from_xyz(pos.x, pos.y + 10.0, 5.0),
        Lifetime(Timer::from_seconds(SCORE_POPUP_LIFETIME, TimerMode::Once)),
        DespawnOnExit(AppState::Playing),
    ));
}

fn tick_score_popups(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Lifetime), With<ScorePopup>>,
) {
    let dt = time.delta_secs();
    for (mut tf, lifetime) in &mut query {
        tf.translation.y += SCORE_POPUP_RISE_SPEED * dt;
        let alpha = lifetime.0.fraction_remaining();
        tf.scale = Vec3::splat(alpha);
    }
}

// --- Screen Shake ---

fn apply_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
) {
    let dt = time.delta_secs();
    shake.trauma = (shake.trauma - SCREEN_SHAKE_DECAY * dt).max(0.0);
    let Ok(mut cam_tf) = camera_q.single_mut() else { return };
    if shake.trauma > 0.0 {
        let mut rng = rand::rng();
        let intensity = shake.trauma * shake.trauma;
        cam_tf.translation.x = rng.random_range(-1.0f32..1.0) * SCREEN_SHAKE_MAX_OFFSET * intensity;
        cam_tf.translation.y = rng.random_range(-1.0f32..1.0) * SCREEN_SHAKE_MAX_OFFSET * intensity;
    } else {
        cam_tf.translation.x = 0.0;
        cam_tf.translation.y = 0.0;
    }
}

// --- HUD ---

fn update_hud(
    mut game: ResMut<GameState>,
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<LivesText>, Without<WaveText>, Without<HighScoreText>)>,
    mut lives_q: Query<&mut Text, (With<LivesText>, Without<ScoreText>, Without<WaveText>, Without<HighScoreText>)>,
    mut wave_q: Query<&mut Text, (With<WaveText>, Without<ScoreText>, Without<LivesText>, Without<HighScoreText>)>,
    mut hi_q: Query<&mut Text, (With<HighScoreText>, Without<ScoreText>, Without<LivesText>, Without<WaveText>)>,
) {
    if game.score > game.high_score {
        game.high_score = game.score;
        save_high_score(game.high_score);
    }
    if let Ok(mut text) = score_q.single_mut() { *text = Text::new(format!("SCORE: {}", game.score)); }
    if let Ok(mut text) = lives_q.single_mut() { *text = Text::new(format!("LIVES: {}", game.lives)); }
    if let Ok(mut text) = wave_q.single_mut() { *text = Text::new(format!("WAVE: {}", game.current_wave)); }
    if let Ok(mut text) = hi_q.single_mut() { *text = Text::new(format!("HI: {}", game.high_score)); }
}

// --- Score Helper ---

fn award_score(game: &mut GameState, points: u32) {
    game.score += points;
    if game.score >= game.next_extra_life_score {
        game.lives += 1;
        game.next_extra_life_score += EXTRA_LIFE_EVERY;
    }
}

// --- Game Over ---

fn start_game_over_timer(mut timer: ResMut<GameOverTimer>) {
    timer.0.reset();
}

fn spawn_game_over(mut commands: Commands, game: Res<GameState>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0), height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                row_gap: Val::Px(20.0), ..default()
            },
            DespawnOnExit(AppState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont { font_size: 60.0, ..default() },
                TextColor(Color::srgb(5.0, 0.2, 0.2)),
            ));
            parent.spawn((
                Text::new(format!("Score: {}", game.score)),
                TextFont { font_size: 30.0, ..default() },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(format!("High Score: {}", game.high_score)),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(5.0, 5.0, 0.5)),
            ));
            parent.spawn((
                Text::new(format!("Wave: {}", game.current_wave)),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("Press SPACE to restart"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

fn game_over_input(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut timer: ResMut<GameOverTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.is_finished() { return; }
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}
