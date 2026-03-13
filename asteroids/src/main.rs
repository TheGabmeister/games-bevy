use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

// ── Window ────────────────────────────────────────────────────────────────────
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const HALF_W: f32 = WINDOW_WIDTH / 2.0;
const HALF_H: f32 = WINDOW_HEIGHT / 2.0;

// ── Ship ──────────────────────────────────────────────────────────────────────
const SHIP_ROTATION_SPEED: f32 = 3.0; // radians/sec
const SHIP_THRUST: f32 = 250.0; // pixels/sec²
const SHIP_DRAG: f32 = 0.97; // velocity multiplier per frame
const SHIP_MAX_SPEED: f32 = 400.0; // pixels/sec
const SHIP_RADIUS: f32 = 12.0; // collision radius

// ── Bullet ────────────────────────────────────────────────────────────────────
const BULLET_SPEED: f32 = 500.0;
const BULLET_LIFETIME: f32 = 1.2; // seconds
const BULLET_RADIUS: f32 = 3.0;
const SHOOT_COOLDOWN: f32 = 0.25; // seconds between shots

// ── Asteroids ─────────────────────────────────────────────────────────────────
const ASTEROID_LARGE_RADIUS: f32 = 40.0;
const ASTEROID_MEDIUM_RADIUS: f32 = 22.0;
const ASTEROID_SMALL_RADIUS: f32 = 12.0;
const ASTEROID_LARGE_SPEED: f32 = 60.0;
const ASTEROID_MEDIUM_SPEED: f32 = 100.0;
const ASTEROID_SMALL_SPEED: f32 = 150.0;

// ── Gameplay ──────────────────────────────────────────────────────────────────
const STARTING_LIVES: u32 = 3;
const INITIAL_ASTEROIDS: u32 = 4; // large asteroids per wave (increases each wave)
const SCORE_LARGE: u32 = 20;
const SCORE_MEDIUM: u32 = 50;
const SCORE_SMALL: u32 = 100;
const INVINCIBILITY_DURATION: f32 = 2.0; // seconds after respawn

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Playing,
    GameOver,
}

// ── Components ────────────────────────────────────────────────────────────────

/// Marks the player ship entity.
#[derive(Component)]
struct Ship;

/// Velocity shared by ship, bullets, and asteroids.
#[derive(Component)]
struct Velocity(Vec2);

/// Size of an asteroid, used for splitting and scoring.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

/// Marks an asteroid entity.
#[derive(Component)]
struct Asteroid {
    size: AsteroidSize,
}

/// Marks a bullet entity.
#[derive(Component)]
struct Bullet;

/// Seconds until a bullet is despawned.
#[derive(Component)]
struct Lifetime(f32);

/// Post-respawn invincibility; removed when it reaches zero.
#[derive(Component)]
struct Invincible(f32);

// ── UI marker components ───────────────────────────────────────────────────────

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct LivesText;

#[derive(Component)]
struct GameOverText;

// ── Resources ─────────────────────────────────────────────────────────────────

/// All mutable game state in one place.
#[derive(Resource)]
struct GameData {
    score: u32,
    lives: u32,
    wave: u32,
    shoot_timer: f32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            lives: STARTING_LIVES,
            wave: 1,
            shoot_timer: 0.0,
        }
    }
}

/// Pre-built mesh and material handles reused by dynamic spawning.
#[derive(Resource)]
struct GameAssets {
    ship_mesh: Handle<Mesh>,
    ship_material: Handle<ColorMaterial>,
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<ColorMaterial>,
    asteroid_large_mesh: Handle<Mesh>,
    asteroid_medium_mesh: Handle<Mesh>,
    asteroid_small_mesh: Handle<Mesh>,
    asteroid_material: Handle<ColorMaterial>,
}

// ── Helper functions ──────────────────────────────────────────────────────────

fn asteroid_radius(size: AsteroidSize) -> f32 {
    match size {
        AsteroidSize::Large => ASTEROID_LARGE_RADIUS,
        AsteroidSize::Medium => ASTEROID_MEDIUM_RADIUS,
        AsteroidSize::Small => ASTEROID_SMALL_RADIUS,
    }
}

fn asteroid_score(size: AsteroidSize) -> u32 {
    match size {
        AsteroidSize::Large => SCORE_LARGE,
        AsteroidSize::Medium => SCORE_MEDIUM,
        AsteroidSize::Small => SCORE_SMALL,
    }
}

fn asteroid_spin(size: AsteroidSize) -> f32 {
    match size {
        AsteroidSize::Large => 0.5,
        AsteroidSize::Medium => 0.8,
        AsteroidSize::Small => 1.2,
    }
}

// ── Spawn helpers ─────────────────────────────────────────────────────────────

fn spawn_ship(commands: &mut Commands, assets: &GameAssets) {
    commands.spawn((
        Ship,
        Velocity(Vec2::ZERO),
        Invincible(INVINCIBILITY_DURATION),
        Mesh2d(assets.ship_mesh.clone()),
        MeshMaterial2d(assets.ship_material.clone()),
        // z = 1 so the ship renders on top of asteroids
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));
}

fn spawn_asteroid(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    velocity: Vec2,
    size: AsteroidSize,
) {
    let mesh = match size {
        AsteroidSize::Large => assets.asteroid_large_mesh.clone(),
        AsteroidSize::Medium => assets.asteroid_medium_mesh.clone(),
        AsteroidSize::Small => assets.asteroid_small_mesh.clone(),
    };
    commands.spawn((
        Asteroid { size },
        Velocity(velocity),
        Mesh2d(mesh),
        MeshMaterial2d(assets.asteroid_material.clone()),
        Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
    ));
}

/// Deterministic spawn position: radius 250 from center, distributed by index.
/// The wave offset ensures asteroids don't always appear in the same spots.
fn asteroid_start_position(index: u32, count: u32, wave: u32) -> Vec2 {
    let angle = (index as f32 / count as f32) * TAU + wave as f32 * 0.7;
    Vec2::new(angle.cos() * 250.0, angle.sin() * 250.0)
}

/// Initial velocity roughly perpendicular to spawn direction so asteroids
/// travel across the screen rather than toward/away from the center.
fn asteroid_start_velocity(pos: Vec2, size: AsteroidSize) -> Vec2 {
    let speed = match size {
        AsteroidSize::Large => ASTEROID_LARGE_SPEED,
        AsteroidSize::Medium => ASTEROID_MEDIUM_SPEED,
        AsteroidSize::Small => ASTEROID_SMALL_SPEED,
    };
    let angle = pos.y.atan2(pos.x) + PI / 2.0;
    Vec2::new(angle.cos() * speed, angle.sin() * speed)
}

fn spawn_wave(commands: &mut Commands, assets: &GameAssets, wave: u32) {
    // One extra asteroid per wave, capped at 8
    let count = (INITIAL_ASTEROIDS + wave - 1).min(8);
    for i in 0..count {
        let pos = asteroid_start_position(i, count, wave);
        let vel = asteroid_start_velocity(pos, AsteroidSize::Large);
        spawn_asteroid(commands, assets, pos.extend(0.0), vel, AsteroidSize::Large);
    }
}

/// Split a destroyed asteroid into two smaller ones at ±30° from its velocity.
fn spawn_fragments(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    parent_vel: Vec2,
    parent_size: AsteroidSize,
) {
    let child_size = match parent_size {
        AsteroidSize::Large => AsteroidSize::Medium,
        AsteroidSize::Medium => AsteroidSize::Small,
        AsteroidSize::Small => return, // smallest size: no fragments
    };
    let speed = match child_size {
        AsteroidSize::Medium => ASTEROID_MEDIUM_SPEED,
        AsteroidSize::Small => ASTEROID_SMALL_SPEED,
        AsteroidSize::Large => unreachable!(),
    };
    // Use parent velocity direction; fall back to rightward if nearly zero
    let base_angle = if parent_vel.length_squared() > 0.01 {
        parent_vel.y.atan2(parent_vel.x)
    } else {
        0.0
    };
    for offset in [PI / 6.0, -PI / 6.0] {
        let angle = base_angle + offset;
        let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        spawn_asteroid(commands, assets, position, vel, child_size);
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // 2D camera — no configuration needed
    commands.spawn(Camera2d);

    // Build all mesh and material handles once; store them as a resource so
    // systems that spawn entities dynamically can clone the handles cheaply.
    let assets = GameAssets {
        // Triangle pointing up: nose at +Y aligns with transform.up()
        ship_mesh: meshes.add(Triangle2d::new(
            Vec2::new(0.0, 18.0),    // nose
            Vec2::new(-12.0, -12.0), // bottom-left
            Vec2::new(12.0, -12.0),  // bottom-right
        )),
        ship_material: materials.add(Color::WHITE),

        bullet_mesh: meshes.add(Circle::new(4.0)),
        bullet_material: materials.add(Color::srgb(1.0, 1.0, 0.0)), // yellow

        // Octagons (8 sides); circumradius matches the collision radius constant
        asteroid_large_mesh: meshes.add(RegularPolygon::new(ASTEROID_LARGE_RADIUS, 8)),
        asteroid_medium_mesh: meshes.add(RegularPolygon::new(ASTEROID_MEDIUM_RADIUS, 8)),
        asteroid_small_mesh: meshes.add(RegularPolygon::new(ASTEROID_SMALL_RADIUS, 8)),
        asteroid_material: materials.add(Color::srgb(0.6, 0.6, 0.6)), // gray
    };

    spawn_ship(&mut commands, &assets);
    spawn_wave(&mut commands, &assets, 1);

    // ── HUD ───────────────────────────────────────────────────────────────────

    // Score — top-left
    commands.spawn((
        ScoreText,
        Text::new("Score: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Lives — top-right
    commands.spawn((
        LivesText,
        Text::new(format!("Lives: {}", STARTING_LIVES)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));

    // Game-over overlay — hidden until the GameOver state is entered
    commands.spawn((
        GameOverText,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(25.0),
            ..default()
        },
        TextFont {
            font_size: 32.0,
            ..default()
        },
        Visibility::Hidden,
    ));

    commands.insert_resource(assets);
}

// ── Playing systems ───────────────────────────────────────────────────────────

/// Left/Right arrows rotate the ship.
fn ship_rotation_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Ship>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    if input.pressed(KeyCode::ArrowLeft) {
        transform.rotate_z(SHIP_ROTATION_SPEED * dt);
    }
    if input.pressed(KeyCode::ArrowRight) {
        transform.rotate_z(-SHIP_ROTATION_SPEED * dt);
    }
}

/// Up arrow applies thrust in the direction the ship faces; drag is always applied.
fn ship_thrust_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Velocity), With<Ship>>,
) {
    let Ok((transform, mut velocity)) = query.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    if input.pressed(KeyCode::ArrowUp) {
        // transform.up() returns the local +Y direction in world space (Dir3).
        // .truncate() converts Vec3 → Vec2 via Deref<Target = Vec3>.
        let forward = transform.up().truncate();
        velocity.0 += forward * SHIP_THRUST * dt;
    }
    velocity.0 *= SHIP_DRAG;
    velocity.0 = velocity.0.clamp_length_max(SHIP_MAX_SPEED);
}

/// Space fires a bullet from the ship nose, respecting a shoot cooldown.
fn ship_shoot_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    assets: Res<GameAssets>,
    query: Query<&Transform, With<Ship>>,
) {
    game_data.shoot_timer -= time.delta_secs();

    let Ok(transform) = query.single() else {
        return;
    };
    if input.just_pressed(KeyCode::Space) && game_data.shoot_timer <= 0.0 {
        game_data.shoot_timer = SHOOT_COOLDOWN;
        let dir = transform.up().truncate();
        // Spawn just ahead of the ship nose so it doesn't immediately self-collide
        let spawn_pos = transform.translation + dir.extend(0.0) * (SHIP_RADIUS + 8.0);
        commands.spawn((
            Bullet,
            Velocity(dir * BULLET_SPEED),
            Lifetime(BULLET_LIFETIME),
            Mesh2d(assets.bullet_mesh.clone()),
            MeshMaterial2d(assets.bullet_material.clone()),
            Transform::from_translation(Vec3::new(spawn_pos.x, spawn_pos.y, 0.5)),
        ));
    }
}

/// Counts down the invincibility timer; removes the component when it expires.
fn invincibility_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Invincible)>,
) {
    for (entity, mut inv) in &mut query {
        inv.0 -= time.delta_secs();
        if inv.0 <= 0.0 {
            commands.entity(entity).remove::<Invincible>();
        }
    }
}

/// Moves every entity that has a Velocity component.
fn movement_system(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let dt = time.delta_secs();
    for (vel, mut transform) in &mut query {
        transform.translation.x += vel.0.x * dt;
        transform.translation.y += vel.0.y * dt;
    }
}

/// Wraps any moving entity that drifts off-screen to the opposite edge.
fn screen_wrap_system(mut query: Query<&mut Transform, With<Velocity>>) {
    for mut transform in &mut query {
        let pos = &mut transform.translation;
        if pos.x > HALF_W {
            pos.x -= WINDOW_WIDTH;
        }
        if pos.x < -HALF_W {
            pos.x += WINDOW_WIDTH;
        }
        if pos.y > HALF_H {
            pos.y -= WINDOW_HEIGHT;
        }
        if pos.y < -HALF_H {
            pos.y += WINDOW_HEIGHT;
        }
    }
}

/// Slowly spins asteroids for visual interest (cosmetic only).
fn asteroid_rotation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Asteroid)>,
) {
    let dt = time.delta_secs();
    for (mut transform, asteroid) in &mut query {
        transform.rotate_z(asteroid_spin(asteroid.size) * dt);
    }
}

/// Despawns bullets whose lifetime has expired.
fn bullet_lifetime_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifetime), With<Bullet>>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut query {
        lifetime.0 -= dt;
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Circle-circle collision between bullets and asteroids.
/// Collisions are collected first, then processed, to avoid borrow conflicts.
fn bullet_asteroid_collision_system(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    assets: Res<GameAssets>,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    asteroids: Query<(Entity, &Transform, &Asteroid, &Velocity)>,
) {
    // (bullet_entity, asteroid_entity, asteroid_world_pos, asteroid_vel, asteroid_size)
    let mut hits: Vec<(Entity, Entity, Vec3, Vec2, AsteroidSize)> = Vec::new();

    for (bullet_entity, bullet_tf) in &bullets {
        for (asteroid_entity, asteroid_tf, asteroid, asteroid_vel) in &asteroids {
            let dist = bullet_tf.translation.distance(asteroid_tf.translation);
            if dist < BULLET_RADIUS + asteroid_radius(asteroid.size) {
                hits.push((
                    bullet_entity,
                    asteroid_entity,
                    asteroid_tf.translation,
                    asteroid_vel.0,
                    asteroid.size,
                ));
            }
        }
    }

    // Guard against the same bullet or asteroid being processed twice
    // (e.g. one bullet hitting two asteroids at once)
    let mut used_bullets = std::collections::HashSet::new();
    let mut used_asteroids = std::collections::HashSet::new();

    for (bullet_entity, asteroid_entity, pos, vel, size) in hits {
        if used_bullets.contains(&bullet_entity) || used_asteroids.contains(&asteroid_entity) {
            continue;
        }
        used_bullets.insert(bullet_entity);
        used_asteroids.insert(asteroid_entity);

        commands.entity(bullet_entity).despawn();
        commands.entity(asteroid_entity).despawn();
        game_data.score += asteroid_score(size);
        spawn_fragments(&mut commands, &assets, pos, vel, size);
    }
}

/// Circle-circle collision between the ship and asteroids.
/// Only runs when the ship does NOT have the Invincible component.
fn ship_asteroid_collision_system(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<AppState>>,
    // Without<Invincible> means invincible ships are never matched
    ship_query: Query<(Entity, &Transform), (With<Ship>, Without<Invincible>)>,
    asteroid_query: Query<(&Transform, &Asteroid)>,
) {
    let Ok((ship_entity, ship_tf)) = ship_query.single() else {
        return;
    };
    for (asteroid_tf, asteroid) in &asteroid_query {
        let dist = ship_tf.translation.distance(asteroid_tf.translation);
        if dist < SHIP_RADIUS + asteroid_radius(asteroid.size) {
            commands.entity(ship_entity).despawn();
            game_data.lives = game_data.lives.saturating_sub(1);
            if game_data.lives == 0 {
                next_state.set(AppState::GameOver);
            } else {
                // Respawn at center with temporary invincibility
                spawn_ship(&mut commands, &assets);
            }
            return; // handle only one collision per frame
        }
    }
}

/// When all asteroids are cleared, spawn the next wave with one extra asteroid.
fn wave_clear_system(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    assets: Res<GameAssets>,
    asteroids: Query<(), With<Asteroid>>,
) {
    if asteroids.iter().count() == 0 {
        game_data.wave += 1;
        spawn_wave(&mut commands, &assets, game_data.wave);
    }
}

/// Refreshes the score and lives HUD text whenever GameData changes.
fn hud_update_system(
    game_data: Res<GameData>,
    // The Without<LivesText> / Without<ScoreText> guards prove to Bevy that
    // these two queries with &mut Text are mutually exclusive (no overlap).
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<LivesText>)>,
    mut lives_q: Query<&mut Text, (With<LivesText>, Without<ScoreText>)>,
) {
    if !game_data.is_changed() {
        return;
    }
    if let Ok(mut text) = score_q.single_mut() {
        *text = Text::new(format!("Score: {}", game_data.score));
    }
    if let Ok(mut text) = lives_q.single_mut() {
        *text = Text::new(format!("Lives: {}", game_data.lives));
    }
}

// ── GameOver systems ──────────────────────────────────────────────────────────

/// Called once when entering the GameOver state.
/// Cleans up any remaining bullets/asteroids and shows the overlay.
fn game_over_setup_system(
    game_data: Res<GameData>,
    mut overlay_q: Query<(&mut Visibility, &mut Text), With<GameOverText>>,
    to_despawn: Query<Entity, Or<(With<Bullet>, With<Asteroid>)>>,
    mut commands: Commands,
) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
    if let Ok((mut vis, mut text)) = overlay_q.single_mut() {
        *vis = Visibility::Visible;
        *text = Text::new(format!(
            "GAME OVER\n\nFinal Score: {}\n\nPress R to restart",
            game_data.score
        ));
    }
}

/// Waits for the player to press R, then resets everything and returns to Playing.
fn game_over_input_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<AppState>>,
    mut overlay_q: Query<&mut Visibility, With<GameOverText>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        *game_data = GameData::default();

        if let Ok(mut vis) = overlay_q.single_mut() {
            *vis = Visibility::Hidden;
        }

        spawn_ship(&mut commands, &assets);
        spawn_wave(&mut commands, &assets, 1);

        next_state.set(AppState::Playing);
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Asteroids".into(),
                resolution: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).into(),
                ..default()
            }),
            ..default()
        }))
        // Register the state machine; default state is AppState::Playing
        .init_state::<AppState>()
        .insert_resource(GameData::default())
        .add_systems(Startup, setup)
        // Run game-over setup once when the state is entered
        .add_systems(OnEnter(AppState::GameOver), game_over_setup_system)
        // All gameplay systems run only while Playing.
        // .chain() enforces sequential execution so that, for example,
        // movement always runs before collision detection.
        .add_systems(
            Update,
            (
                ship_rotation_system,
                ship_thrust_system,
                ship_shoot_system,
                invincibility_system,
                movement_system,
                screen_wrap_system,
                asteroid_rotation_system,
                bullet_asteroid_collision_system,
                ship_asteroid_collision_system,
                wave_clear_system,
                bullet_lifetime_system,
                hud_update_system,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            game_over_input_system.run_if(in_state(AppState::GameOver)),
        )
        .run();
}
