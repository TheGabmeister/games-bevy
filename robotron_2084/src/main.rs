use bevy::{
    camera::ScalingMode,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    window::WindowResolution,
};
use rand::Rng;

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

const INITIAL_GRUNT_COUNT: u32 = 15;
const GRUNT_INCREMENT_PER_WAVE: u32 = 5;
const MAX_GRUNTS_PER_WAVE: u32 = 60;

const SPAWN_EXCLUSION_RADIUS: f32 = 100.0;

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

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Grunt;

#[derive(Component)]
struct PlayerBullet;

#[derive(Component)]
struct WaveEntity;

#[derive(Component)]
struct Confined;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Facing(Vec2);

#[derive(Component)]
struct CollisionRadius(f32);

#[derive(Component)]
struct PointValue(u32);

#[derive(Component)]
struct FireCooldown(Timer);

#[derive(Component)]
struct Invincible(Timer);

#[derive(Component)]
struct GruntSteerOffset(f32);

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct LivesText;

// --- Resources ---

#[derive(Resource)]
struct GameAssets {
    player_mesh: Handle<Mesh>,
    player_material: Handle<ColorMaterial>,
    grunt_mesh: Handle<Mesh>,
    grunt_material: Handle<ColorMaterial>,
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<ColorMaterial>,
    border_mesh_h: Handle<Mesh>,
    border_mesh_v: Handle<Mesh>,
    border_material: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct GameState {
    score: u32,
    lives: u32,
    current_wave: u32,
    next_extra_life_score: u32,
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
            lives: STARTING_LIVES,
            current_wave: 1,
            next_extra_life_score: EXTRA_LIFE_EVERY,
        })
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
        .add_systems(
            OnEnter(PlayState::WaveIntro),
            (despawn_wave_entities, spawn_wave, enter_wave_active).chain(),
        )
        // Playing - gameplay (ordered groups in a chain)
        .add_systems(
            Update,
            (
                (player_movement, player_aim, grunt_ai),
                (player_fire, apply_velocity),
                confine_entities,
                (despawn_oob_bullets, bullet_vs_enemy, enemy_vs_player),
                check_wave_clear,
            )
                .chain()
                .run_if(in_state(PlayState::WaveActive)),
        )
        // Playing - always-on during Playing
        .add_systems(
            Update,
            (tick_invincibility, update_hud).run_if(in_state(AppState::Playing)),
        )
        // Game over
        .add_systems(OnEnter(AppState::GameOver), spawn_game_over)
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
    let bullet_mesh = meshes.add(Circle::new(BULLET_RADIUS));
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
    let bullet_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 5.0, 0.5)));
    let border_material = materials.add(ColorMaterial::from_color(Color::srgb(0.5, 0.5, 5.0)));

    commands.insert_resource(GameAssets {
        player_mesh,
        player_material,
        grunt_mesh,
        grunt_material,
        bullet_mesh,
        bullet_material,
        border_mesh_h,
        border_mesh_v,
        border_material,
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
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(5.0, 1.0, 0.2)),
            ));
            parent.spawn((
                Text::new("WASD to move  |  Arrow keys to aim & fire"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("Press SPACE to start"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
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

fn reset_game_state(mut game: ResMut<GameState>) {
    game.score = 0;
    game.lives = STARTING_LIVES;
    game.current_wave = 1;
    game.next_extra_life_score = EXTRA_LIFE_EVERY;
}

fn spawn_arena(mut commands: Commands, assets: Res<GameAssets>) {
    // Top border (horizontal — capsule rotated 90°)
    commands.spawn((
        Mesh2d(assets.border_mesh_h.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(0.0, ARENA_HALF_HEIGHT, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        DespawnOnExit(AppState::Playing),
    ));
    // Bottom border
    commands.spawn((
        Mesh2d(assets.border_mesh_h.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(0.0, -ARENA_HALF_HEIGHT, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        DespawnOnExit(AppState::Playing),
    ));
    // Left border (vertical — no rotation needed)
    commands.spawn((
        Mesh2d(assets.border_mesh_v.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(-ARENA_HALF_WIDTH, 0.0, 0.0),
        DespawnOnExit(AppState::Playing),
    ));
    // Right border
    commands.spawn((
        Mesh2d(assets.border_mesh_v.clone()),
        MeshMaterial2d(assets.border_material.clone()),
        Transform::from_xyz(ARENA_HALF_WIDTH, 0.0, 0.0),
        DespawnOnExit(AppState::Playing),
    ));
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Player,
        Confined,
        Mesh2d(assets.player_mesh.clone()),
        MeshMaterial2d(assets.player_material.clone()),
        Transform::from_xyz(0.0, 0.0, 1.0),
        Velocity(Vec2::ZERO),
        Facing(Vec2::Y),
        CollisionRadius(PLAYER_RADIUS),
        FireCooldown(Timer::from_seconds(PLAYER_FIRE_COOLDOWN, TimerMode::Once)),
        Invincible(Timer::from_seconds(
            PLAYER_INVINCIBILITY_DURATION,
            TimerMode::Once,
        )),
        DespawnOnExit(AppState::Playing),
    ));
}

fn spawn_hud(mut commands: Commands) {
    // Score — top left
    commands.spawn((
        Text::new("SCORE: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
        DespawnOnExit(AppState::Playing),
    ));
    // Lives — top right
    commands.spawn((
        Text::new(format!("LIVES: {}", STARTING_LIVES)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        LivesText,
        DespawnOnExit(AppState::Playing),
    ));
}

// --- Wave Management ---

fn despawn_wave_entities(mut commands: Commands, query: Query<Entity, With<WaveEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_wave(mut commands: Commands, assets: Res<GameAssets>, game: Res<GameState>) {
    let mut rng = rand::rng();
    let grunt_count = (INITIAL_GRUNT_COUNT + (game.current_wave - 1) * GRUNT_INCREMENT_PER_WAVE)
        .min(MAX_GRUNTS_PER_WAVE);

    for _ in 0..grunt_count {
        let (x, y) = loop {
            let edge: u32 = rng.random_range(0..4);
            let (x, y) = match edge {
                0 => (
                    rng.random_range(
                        -ARENA_HALF_WIDTH + GRUNT_RADIUS..ARENA_HALF_WIDTH - GRUNT_RADIUS,
                    ),
                    ARENA_HALF_HEIGHT - GRUNT_RADIUS,
                ),
                1 => (
                    rng.random_range(
                        -ARENA_HALF_WIDTH + GRUNT_RADIUS..ARENA_HALF_WIDTH - GRUNT_RADIUS,
                    ),
                    -ARENA_HALF_HEIGHT + GRUNT_RADIUS,
                ),
                2 => (
                    -ARENA_HALF_WIDTH + GRUNT_RADIUS,
                    rng.random_range(
                        -ARENA_HALF_HEIGHT + GRUNT_RADIUS..ARENA_HALF_HEIGHT - GRUNT_RADIUS,
                    ),
                ),
                _ => (
                    ARENA_HALF_WIDTH - GRUNT_RADIUS,
                    rng.random_range(
                        -ARENA_HALF_HEIGHT + GRUNT_RADIUS..ARENA_HALF_HEIGHT - GRUNT_RADIUS,
                    ),
                ),
            };
            if x * x + y * y > SPAWN_EXCLUSION_RADIUS * SPAWN_EXCLUSION_RADIUS {
                break (x, y);
            }
        };

        let steer_offset: f32 = rng.random_range(-0.5..0.5);

        commands.spawn((
            Enemy,
            Grunt,
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
}

fn enter_wave_active(mut next_state: ResMut<NextState<PlayState>>) {
    next_state.set(PlayState::WaveActive);
}

// --- Player Systems ---

fn player_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let Ok(mut vel) = query.single_mut() else {
        return;
    };
    let mut dir = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    vel.0 = dir.normalize_or_zero() * PLAYER_SPEED;
}

fn player_aim(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Facing, &mut Transform), With<Player>>,
) {
    let Ok((mut facing, mut tf)) = query.single_mut() else {
        return;
    };
    let mut dir = Vec2::ZERO;
    if input.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if input.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if input.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if dir != Vec2::ZERO {
        let dir = dir.normalize();
        facing.0 = dir;
        // Rotate player diamond to face aim direction
        tf.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
    }
}

fn player_fire(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    assets: Res<GameAssets>,
    mut player_q: Query<(&Transform, &Facing, &mut FireCooldown), With<Player>>,
    bullet_q: Query<(), With<PlayerBullet>>,
) {
    let Ok((tf, facing, mut cooldown)) = player_q.single_mut() else {
        return;
    };
    cooldown.0.tick(time.delta());

    let aiming = input.pressed(KeyCode::ArrowUp)
        || input.pressed(KeyCode::ArrowDown)
        || input.pressed(KeyCode::ArrowLeft)
        || input.pressed(KeyCode::ArrowRight);

    if !aiming || !cooldown.0.is_finished() {
        return;
    }
    if bullet_q.iter().count() >= MAX_PLAYER_BULLETS as usize {
        return;
    }

    cooldown.0.reset();

    let pos = tf.translation.truncate();
    let dir = facing.0;

    commands.spawn((
        PlayerBullet,
        Mesh2d(assets.bullet_mesh.clone()),
        MeshMaterial2d(assets.bullet_material.clone()),
        Transform::from_xyz(pos.x, pos.y, 0.5),
        Velocity(dir * BULLET_SPEED),
        CollisionRadius(BULLET_RADIUS),
        DespawnOnExit(AppState::Playing),
    ));
}

// --- Enemy Systems ---

fn grunt_ai(
    player_q: Query<&Transform, With<Player>>,
    mut grunt_q: Query<(&Transform, &mut Velocity, &GruntSteerOffset), With<Grunt>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (tf, mut vel, offset) in &mut grunt_q {
        let pos = tf.translation.truncate();
        let dir = (player_pos - pos).normalize_or_zero();
        let angle = dir.y.atan2(dir.x) + offset.0;
        let steered = Vec2::new(angle.cos(), angle.sin());
        vel.0 = steered * GRUNT_BASE_SPEED;
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
        tf.translation.x = tf.translation.x.clamp(
            -ARENA_HALF_WIDTH + radius.0,
            ARENA_HALF_WIDTH - radius.0,
        );
        tf.translation.y = tf.translation.y.clamp(
            -ARENA_HALF_HEIGHT + radius.0,
            ARENA_HALF_HEIGHT - radius.0,
        );
    }
}

fn despawn_oob_bullets(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<PlayerBullet>>,
) {
    for (entity, tf) in &query {
        let pos = tf.translation;
        if pos.x < -ARENA_HALF_WIDTH - 50.0
            || pos.x > ARENA_HALF_WIDTH + 50.0
            || pos.y < -ARENA_HALF_HEIGHT - 50.0
            || pos.y > ARENA_HALF_HEIGHT + 50.0
        {
            commands.entity(entity).despawn();
        }
    }
}

// --- Combat Systems ---

fn bullet_vs_enemy(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    bullet_q: Query<(Entity, &Transform, &CollisionRadius), With<PlayerBullet>>,
    enemy_q: Query<(Entity, &Transform, &CollisionRadius, &PointValue), With<Enemy>>,
) {
    for (b_entity, b_tf, b_radius) in &bullet_q {
        let b_pos = b_tf.translation.truncate();
        for (e_entity, e_tf, e_radius, points) in &enemy_q {
            let e_pos = e_tf.translation.truncate();
            let dist_sq = b_pos.distance_squared(e_pos);
            let radii = b_radius.0 + e_radius.0;
            if dist_sq < radii * radii {
                commands.entity(b_entity).despawn();
                commands.entity(e_entity).despawn();
                game.score += points.0;
                if game.score >= game.next_extra_life_score {
                    game.lives += 1;
                    game.next_extra_life_score += EXTRA_LIFE_EVERY;
                }
                break; // Bullet consumed
            }
        }
    }
}

fn enemy_vs_player(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    mut next_app_state: ResMut<NextState<AppState>>,
    assets: Res<GameAssets>,
    player_q: Query<(Entity, &Transform, &CollisionRadius, Option<&Invincible>), With<Player>>,
    enemy_q: Query<(&Transform, &CollisionRadius), With<Enemy>>,
) {
    let Ok((p_entity, p_tf, p_radius, invincible)) = player_q.single() else {
        return;
    };
    if invincible.is_some() {
        return;
    }

    let p_pos = p_tf.translation.truncate();

    for (e_tf, e_radius) in &enemy_q {
        let e_pos = e_tf.translation.truncate();
        let dist_sq = p_pos.distance_squared(e_pos);
        let radii = p_radius.0 + e_radius.0;
        if dist_sq < radii * radii {
            commands.entity(p_entity).despawn();
            game.lives = game.lives.saturating_sub(1);

            if game.lives == 0 {
                next_app_state.set(AppState::GameOver);
            } else {
                // Respawn player at center with invincibility
                commands.spawn((
                    Player,
                    Confined,
                    Mesh2d(assets.player_mesh.clone()),
                    MeshMaterial2d(assets.player_material.clone()),
                    Transform::from_xyz(0.0, 0.0, 1.0),
                    Velocity(Vec2::ZERO),
                    Facing(Vec2::Y),
                    CollisionRadius(PLAYER_RADIUS),
                    FireCooldown(Timer::from_seconds(PLAYER_FIRE_COOLDOWN, TimerMode::Once)),
                    Invincible(Timer::from_seconds(
                        PLAYER_INVINCIBILITY_DURATION,
                        TimerMode::Once,
                    )),
                    DespawnOnExit(AppState::Playing),
                ));
            }
            return; // Only one death per frame
        }
    }
}

fn check_wave_clear(mut game: ResMut<GameState>, mut next_state: ResMut<NextState<PlayState>>, enemy_q: Query<(), With<Grunt>>) {
    if enemy_q.is_empty() {
        game.current_wave += 1;
        next_state.set(PlayState::WaveIntro);
    }
}

// --- Invincibility ---

fn tick_invincibility(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invincible, &mut Visibility)>,
) {
    for (entity, mut inv, mut vis) in &mut query {
        inv.0.tick(time.delta());
        let elapsed = inv.0.elapsed_secs();
        let blink = (elapsed / PLAYER_BLINK_INTERVAL) as u32 % 2 == 0;
        *vis = if blink {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if inv.0.is_finished() {
            commands.entity(entity).remove::<Invincible>();
            *vis = Visibility::Inherited;
        }
    }
}

// --- HUD ---

fn update_hud(
    game: Res<GameState>,
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<LivesText>)>,
    mut lives_q: Query<&mut Text, (With<LivesText>, Without<ScoreText>)>,
) {
    if let Ok(mut text) = score_q.single_mut() {
        *text = Text::new(format!("SCORE: {}", game.score));
    }
    if let Ok(mut text) = lives_q.single_mut() {
        *text = Text::new(format!("LIVES: {}", game.lives));
    }
}

// --- Game Over ---

fn spawn_game_over(mut commands: Commands, game: Res<GameState>) {
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
            DespawnOnExit(AppState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(5.0, 0.2, 0.2)),
            ));
            parent.spawn((
                Text::new(format!("Score: {}", game.score)),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(format!("Wave: {}", game.current_wave)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("Press SPACE to restart"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn game_over_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}
