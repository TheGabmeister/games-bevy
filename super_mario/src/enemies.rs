use bevy::prelude::*;

use crate::{
    components::{
        Collider, Enemy, Goomba, Grounded, InvulnerabilityTimer, KoopaTroopa, Player, PowerState,
        Shell, Solid, Velocity,
    },
    constants::*,
    level::spawn_level,
    resources::{EnemyKind, GameData, LevelState},
    states::{AppState, PlayState},
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_enemies.after(spawn_level))
            .add_systems(
                Update,
                (
                    enemy_movement,
                    shell_movement,
                    enemy_and_shell_player_collision,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing))
                    .run_if(in_state(PlayState::Running)),
            )
            .add_systems(
                Update,
                (
                    animate_goomba_squish,
                    animate_defeat_particles,
                    animate_enemy_score_popups,
                    despawn_fallen_enemies,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

// Module-local components

#[derive(Component)]
struct GoombaSquish {
    timer: Timer,
}

#[derive(Component)]
struct ShellMoving {
    direction: f32,
}

#[derive(Component)]
struct DefeatParticle {
    velocity: Vec2,
    timer: Timer,
}

#[derive(Component)]
struct EnemyScorePopup {
    start_y: f32,
    timer: Timer,
}

// --- Spawning ---

fn spawn_enemies(
    mut commands: Commands,
    level_state: Res<LevelState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ground_top_y = -WINDOW_HEIGHT * 0.5 + GROUND_TILE_ROWS as f32 * TILE_SIZE;

    for spawn in &level_state.enemy_spawns {
        let x = spawn.tile_x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        match spawn.kind {
            EnemyKind::Goomba => {
                let y = ground_top_y + GOOMBA_HEIGHT * 0.5;
                spawn_goomba(&mut commands, &mut meshes, &mut materials, Vec2::new(x, y));
            }
            EnemyKind::Koopa => {
                let y = ground_top_y + KOOPA_HEIGHT * 0.5;
                spawn_koopa(&mut commands, &mut meshes, &mut materials, Vec2::new(x, y));
            }
        }
    }
}

fn spawn_goomba(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
) {
    let body_mesh = meshes.add(Circle::new(GOOMBA_WIDTH * 0.45));
    let foot_mesh = meshes.add(Rectangle::new(GOOMBA_WIDTH * 0.3, GOOMBA_HEIGHT * 0.2));
    let eye_mesh = meshes.add(Circle::new(2.5));

    let body_material = materials.add(COLOR_GOOMBA);
    let foot_material = materials.add(Color::srgb(0.35, 0.20, 0.08));
    let eye_material = materials.add(Color::WHITE);

    commands
        .spawn((
            Enemy,
            Goomba,
            DespawnOnExit(AppState::Playing),
            Velocity {
                x: -GOOMBA_SPEED,
                y: 0.0,
            },
            Grounded::default(),
            Collider {
                width: GOOMBA_WIDTH,
                height: GOOMBA_HEIGHT,
            },
            Transform::from_xyz(position.x, position.y, Z_ENEMIES),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(body_mesh),
                MeshMaterial2d(body_material),
                Transform::from_xyz(0.0, GOOMBA_HEIGHT * 0.1, 0.0),
            ));
            parent.spawn((
                Mesh2d(foot_mesh.clone()),
                MeshMaterial2d(foot_material.clone()),
                Transform::from_xyz(-GOOMBA_WIDTH * 0.2, -GOOMBA_HEIGHT * 0.35, 0.0),
            ));
            parent.spawn((
                Mesh2d(foot_mesh),
                MeshMaterial2d(foot_material),
                Transform::from_xyz(GOOMBA_WIDTH * 0.2, -GOOMBA_HEIGHT * 0.35, 0.0),
            ));
            parent.spawn((
                Mesh2d(eye_mesh.clone()),
                MeshMaterial2d(eye_material.clone()),
                Transform::from_xyz(-4.0, GOOMBA_HEIGHT * 0.15, 0.1),
            ));
            parent.spawn((
                Mesh2d(eye_mesh),
                MeshMaterial2d(eye_material),
                Transform::from_xyz(4.0, GOOMBA_HEIGHT * 0.15, 0.1),
            ));
        });
}

fn spawn_koopa(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
) {
    let shell_mesh = meshes.add(Circle::new(KOOPA_WIDTH * 0.45));
    let head_mesh = meshes.add(Circle::new(KOOPA_WIDTH * 0.25));
    let foot_mesh = meshes.add(Rectangle::new(KOOPA_WIDTH * 0.25, KOOPA_HEIGHT * 0.15));
    let eye_mesh = meshes.add(Circle::new(2.0));

    let shell_material = materials.add(COLOR_KOOPA_GREEN);
    let skin_material = materials.add(Color::srgb(0.95, 0.85, 0.55));
    let eye_material = materials.add(Color::WHITE);

    commands
        .spawn((
            Enemy,
            KoopaTroopa,
            DespawnOnExit(AppState::Playing),
            Velocity {
                x: -KOOPA_SPEED,
                y: 0.0,
            },
            Grounded::default(),
            Collider {
                width: KOOPA_WIDTH,
                height: KOOPA_HEIGHT,
            },
            Transform::from_xyz(position.x, position.y, Z_ENEMIES),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(shell_mesh),
                MeshMaterial2d(shell_material),
                Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.0, 0.8, 1.0)),
            ));
            parent.spawn((
                Mesh2d(head_mesh),
                MeshMaterial2d(skin_material.clone()),
                Transform::from_xyz(KOOPA_WIDTH * 0.2, KOOPA_HEIGHT * 0.25, 0.1),
            ));
            parent.spawn((
                Mesh2d(foot_mesh.clone()),
                MeshMaterial2d(skin_material.clone()),
                Transform::from_xyz(-KOOPA_WIDTH * 0.15, -KOOPA_HEIGHT * 0.38, 0.0),
            ));
            parent.spawn((
                Mesh2d(foot_mesh),
                MeshMaterial2d(skin_material),
                Transform::from_xyz(KOOPA_WIDTH * 0.15, -KOOPA_HEIGHT * 0.38, 0.0),
            ));
            parent.spawn((
                Mesh2d(eye_mesh),
                MeshMaterial2d(eye_material),
                Transform::from_xyz(KOOPA_WIDTH * 0.28, KOOPA_HEIGHT * 0.28, 0.2),
            ));
        });
}

fn spawn_shell_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
) {
    let shell_mesh = meshes.add(Circle::new(SHELL_WIDTH * 0.45));
    let shell_material = materials.add(COLOR_KOOPA_GREEN);

    commands
        .spawn((
            Shell,
            DespawnOnExit(AppState::Playing),
            Velocity { x: 0.0, y: 0.0 },
            Grounded::default(),
            Collider {
                width: SHELL_WIDTH,
                height: SHELL_HEIGHT,
            },
            Transform::from_xyz(position.x, position.y, Z_ENEMIES),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(shell_mesh),
                MeshMaterial2d(shell_material),
                Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.0, 0.65, 1.0)),
            ));
        });
}

fn spawn_goomba_squish(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec3,
) {
    let squish_mesh = meshes.add(Rectangle::new(GOOMBA_WIDTH * 1.2, GOOMBA_HEIGHT * 0.2));
    let squish_material = materials.add(COLOR_GOOMBA);

    commands.spawn((
        GoombaSquish {
            timer: Timer::from_seconds(GOOMBA_SQUISH_DURATION, TimerMode::Once),
        },
        DespawnOnExit(AppState::Playing),
        Mesh2d(squish_mesh),
        MeshMaterial2d(squish_material),
        Transform::from_xyz(position.x, position.y - GOOMBA_HEIGHT * 0.35, position.z),
    ));
}

fn spawn_defeat_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
    color: Color,
) {
    let particle_mesh = meshes.add(Circle::new(3.0));
    let particle_material = materials.add(color);

    let velocities = [
        Vec2::new(-120.0, 200.0),
        Vec2::new(-60.0, 280.0),
        Vec2::new(60.0, 280.0),
        Vec2::new(120.0, 200.0),
    ];

    for vel in velocities {
        commands.spawn((
            DefeatParticle {
                velocity: vel,
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            },
            DespawnOnExit(AppState::Playing),
            Mesh2d(particle_mesh.clone()),
            MeshMaterial2d(particle_material.clone()),
            Transform::from_xyz(position.x, position.y, Z_PARTICLES),
        ));
    }
}

fn spawn_enemy_score_popup(commands: &mut Commands, position: Vec3, score: u32) {
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        EnemyScorePopup {
            start_y: position.y + 16.0,
            timer: Timer::from_seconds(SCORE_POP_DURATION, TimerMode::Once),
        },
        Text2d::new(format!("+{score}")),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.98, 0.85, 1.0)),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(position.x, position.y + 16.0, Z_PARTICLES),
    ));
}

// --- Movement systems ---

fn enemy_movement(
    time: Res<Time>,
    mut enemy_query: Query<
        (&mut Transform, &mut Velocity, &mut Grounded, &Collider),
        (With<Enemy>, Without<Solid>, Without<GoombaSquish>),
    >,
    solid_query: Query<(&Transform, &Collider), (With<Solid>, Without<Enemy>)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut velocity, mut grounded, collider) in &mut enemy_query {
        velocity.y = (velocity.y + GRAVITY * dt).max(MAX_FALL_SPEED);

        let half_w = collider.width * 0.5;
        let half_h = collider.height * 0.5;

        // Horizontal movement + wall reversal
        let next_x = transform.translation.x + velocity.x * dt;
        let mut blocked_x = false;
        for (solid_tf, solid_col) in &solid_query {
            if aabb_overlap(
                next_x,
                transform.translation.y,
                half_w,
                half_h,
                solid_tf.translation.x,
                solid_tf.translation.y,
                solid_col.width * 0.5,
                solid_col.height * 0.5,
            ) {
                blocked_x = true;
                break;
            }
        }
        if blocked_x {
            velocity.x = -velocity.x;
        } else {
            transform.translation.x = next_x;
        }

        // Vertical movement + landing
        grounded.0 = false;
        let next_y = transform.translation.y + velocity.y * dt;
        let mut landed = false;
        let mut land_y = next_y;
        for (solid_tf, solid_col) in &solid_query {
            if aabb_overlap(
                transform.translation.x,
                next_y,
                half_w,
                half_h,
                solid_tf.translation.x,
                solid_tf.translation.y,
                solid_col.width * 0.5,
                solid_col.height * 0.5,
            ) {
                if velocity.y < 0.0 {
                    land_y = solid_tf.translation.y + solid_col.height * 0.5 + half_h;
                    landed = true;
                }
                break;
            }
        }
        if landed {
            transform.translation.y = land_y;
            velocity.y = 0.0;
            grounded.0 = true;
        } else {
            transform.translation.y = next_y;
        }
    }
}

fn shell_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut shell_query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &Collider,
            Option<&mut ShellMoving>,
        ),
        (With<Shell>, Without<Solid>, Without<Enemy>),
    >,
    solid_query: Query<(&Transform, &Collider), (With<Solid>, Without<Shell>)>,
    enemy_query: Query<(Entity, &Transform, &Collider), (With<Enemy>, Without<Shell>)>,
    mut game_data: ResMut<GameData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut velocity, collider, shell_moving) in &mut shell_query {
        // Gravity for all shells
        velocity.y = (velocity.y + GRAVITY * dt).max(MAX_FALL_SPEED);

        let half_w = collider.width * 0.5;
        let half_h = collider.height * 0.5;

        // Horizontal movement only for moving shells
        if let Some(mut moving) = shell_moving {
            let next_x = transform.translation.x + moving.direction * SHELL_SPEED * dt;
            let mut blocked = false;
            for (solid_tf, solid_col) in &solid_query {
                if aabb_overlap(
                    next_x,
                    transform.translation.y,
                    half_w,
                    half_h,
                    solid_tf.translation.x,
                    solid_tf.translation.y,
                    solid_col.width * 0.5,
                    solid_col.height * 0.5,
                ) {
                    blocked = true;
                    break;
                }
            }
            if blocked {
                moving.direction = -moving.direction;
            } else {
                transform.translation.x = next_x;
            }

            // Kill enemies on contact
            for (enemy_entity, enemy_tf, enemy_col) in &enemy_query {
                if aabb_overlap(
                    transform.translation.x,
                    transform.translation.y,
                    half_w,
                    half_h,
                    enemy_tf.translation.x,
                    enemy_tf.translation.y,
                    enemy_col.width * 0.5,
                    enemy_col.height * 0.5,
                ) {
                    game_data.score += SCORE_GOOMBA;
                    spawn_defeat_particles(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        enemy_tf.translation.truncate(),
                        COLOR_GOOMBA,
                    );
                    spawn_enemy_score_popup(&mut commands, enemy_tf.translation, SCORE_GOOMBA);
                    commands.entity(enemy_entity).despawn();
                }
            }
        }

        // Vertical movement
        let next_y = transform.translation.y + velocity.y * dt;
        let mut landed = false;
        let mut land_y = next_y;
        for (solid_tf, solid_col) in &solid_query {
            if aabb_overlap(
                transform.translation.x,
                next_y,
                half_w,
                half_h,
                solid_tf.translation.x,
                solid_tf.translation.y,
                solid_col.width * 0.5,
                solid_col.height * 0.5,
            ) {
                if velocity.y < 0.0 {
                    land_y = solid_tf.translation.y + solid_col.height * 0.5 + half_h;
                    landed = true;
                }
                break;
            }
        }
        if landed {
            transform.translation.y = land_y;
            velocity.y = 0.0;
        } else {
            transform.translation.y = next_y;
        }
    }
}

// --- Player collision ---

fn enemy_and_shell_player_collision(
    mut commands: Commands,
    mut player_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut Collider,
            &mut PowerState,
            Option<&InvulnerabilityTimer>,
        ),
        (With<Player>, Without<Enemy>, Without<Shell>),
    >,
    enemy_query: Query<
        (
            Entity,
            &Transform,
            &Collider,
            Option<&Goomba>,
            Option<&KoopaTroopa>,
        ),
        (With<Enemy>, Without<Player>, Without<GoombaSquish>),
    >,
    shell_query: Query<
        (Entity, &Transform, &Collider, Option<&ShellMoving>),
        (With<Shell>, Without<Player>, Without<Enemy>),
    >,
    mut game_data: ResMut<GameData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok((player_entity, mut player_tf, mut player_vel, mut player_col, mut power_state, invul)) =
        player_query.single_mut()
    else {
        return;
    };

    let p_hw = player_col.width * 0.5;
    let p_hh = player_col.height * 0.5;

    // Check enemies
    for (enemy_entity, enemy_tf, enemy_col, is_goomba, is_koopa) in &enemy_query {
        let e_hw = enemy_col.width * 0.5;
        let e_hh = enemy_col.height * 0.5;

        if !aabb_overlap(
            player_tf.translation.x,
            player_tf.translation.y,
            p_hw,
            p_hh,
            enemy_tf.translation.x,
            enemy_tf.translation.y,
            e_hw,
            e_hh,
        ) {
            continue;
        }

        let player_bottom = player_tf.translation.y - p_hh;
        let is_stomp = player_vel.y < 0.0 && player_bottom > enemy_tf.translation.y;

        if is_stomp {
            player_vel.y = STOMP_BOUNCE_FORCE;
            if is_goomba.is_some() {
                game_data.score += SCORE_GOOMBA;
                spawn_goomba_squish(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    enemy_tf.translation,
                );
                spawn_enemy_score_popup(&mut commands, enemy_tf.translation, SCORE_GOOMBA);
            } else if is_koopa.is_some() {
                game_data.score += SCORE_KOOPA;
                spawn_shell_entity(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    enemy_tf.translation.truncate(),
                );
                spawn_enemy_score_popup(&mut commands, enemy_tf.translation, SCORE_KOOPA);
            }
            commands.entity(enemy_entity).despawn();
            return;
        } else if invul.is_none() {
            damage_player(
                &mut commands,
                player_entity,
                &mut player_tf,
                &mut player_col,
                &mut power_state,
            );
            return;
        }
    }

    // Check shells
    for (shell_entity, shell_tf, shell_col, shell_moving) in &shell_query {
        let s_hw = shell_col.width * 0.5;
        let s_hh = shell_col.height * 0.5;

        if !aabb_overlap(
            player_tf.translation.x,
            player_tf.translation.y,
            p_hw,
            p_hh,
            shell_tf.translation.x,
            shell_tf.translation.y,
            s_hw,
            s_hh,
        ) {
            continue;
        }

        if shell_moving.is_none() {
            // Kick stationary shell
            let kick_dir = if player_tf.translation.x < shell_tf.translation.x {
                1.0
            } else {
                -1.0
            };
            commands
                .entity(shell_entity)
                .insert(ShellMoving { direction: kick_dir });
            player_tf.translation.x -= kick_dir * 4.0;
            return;
        }

        // Moving shell interaction
        let player_bottom = player_tf.translation.y - p_hh;
        let is_stomp = player_vel.y < 0.0 && player_bottom > shell_tf.translation.y;

        if is_stomp {
            commands.entity(shell_entity).remove::<ShellMoving>();
            player_vel.y = STOMP_BOUNCE_FORCE;
            return;
        } else if invul.is_none() {
            damage_player(
                &mut commands,
                player_entity,
                &mut player_tf,
                &mut player_col,
                &mut power_state,
            );
            return;
        }
    }
}

fn damage_player(
    commands: &mut Commands,
    player_entity: Entity,
    player_tf: &mut Transform,
    player_col: &mut Collider,
    power_state: &mut PowerState,
) {
    if *power_state == PowerState::Big {
        *power_state = PowerState::Small;
        player_col.width = PLAYER_WIDTH;
        player_col.height = PLAYER_HEIGHT;
        player_tf.translation.y -= (PLAYER_BIG_HEIGHT - PLAYER_HEIGHT) * 0.5;
        player_tf.scale.y = 1.0;
    }
    commands
        .entity(player_entity)
        .insert(InvulnerabilityTimer {
            timer: Timer::from_seconds(INVULNERABILITY_DURATION, TimerMode::Once),
        });
}

// --- Animations ---

fn animate_goomba_squish(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut GoombaSquish)>,
) {
    for (entity, mut squish) in &mut query {
        squish.timer.tick(time.delta());
        if squish.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_defeat_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut DefeatParticle)>,
) {
    for (entity, mut transform, mut particle) in &mut query {
        particle.timer.tick(time.delta());
        particle.velocity.y += GRAVITY * 0.5 * time.delta_secs();
        transform.translation += particle.velocity.extend(0.0) * time.delta_secs();
        if particle.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_enemy_score_popups(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut EnemyScorePopup)>,
) {
    for (entity, mut transform, mut text_color, mut popup) in &mut query {
        popup.timer.tick(time.delta());
        let progress = popup.timer.fraction();
        transform.translation.y = popup.start_y + progress * SCORE_POP_RISE;
        text_color.0.set_alpha(1.0 - progress);
        if popup.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_fallen_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Transform), Or<(With<Enemy>, With<Shell>)>>,
) {
    for (entity, transform) in &query {
        if transform.translation.y < DEATH_Y {
            commands.entity(entity).despawn();
        }
    }
}

// --- Utility ---

fn aabb_overlap(
    ax: f32,
    ay: f32,
    ahw: f32,
    ahh: f32,
    bx: f32,
    by: f32,
    bhw: f32,
    bhh: f32,
) -> bool {
    (ax - bx).abs() < ahw + bhw && (ay - by).abs() < ahh + bhh
}
