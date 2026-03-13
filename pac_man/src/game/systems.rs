use bevy::prelude::*;

use crate::game::{
    components::{
        Direction, GameplayEntity, Ghost, GhostPersonality, GridMover, LivesText, MessageText,
        PacmanMouth, Pellet, PelletKind, Player, ScoreText,
    },
    constants::{
        ACTOR_Z, DETAIL_Z, GHOST_COLLISION_RADIUS, GHOST_EATEN_SPEED, GHOST_FRIGHTENED_SPEED,
        GHOST_RADIUS, GHOST_SPEED, HUD_COLOR, MESSAGE_COLOR, PACMAN_RADIUS, PELLET_Z,
        PLAYER_SCORE_GHOST, PLAYER_SCORE_PELLET, PLAYER_SCORE_POWER, PLAYER_SPEED, TILE_SIZE,
        TURN_TOLERANCE, WALL_Z,
    },
    level::LevelLayout,
    resources::{GameMaterials, GameMeshes, GameSession, RoundPhase},
};

pub fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    layout: Res<LevelLayout>,
) {
    let mesh_handles = GameMeshes::new(&mut meshes);
    let material_handles = GameMaterials::new(&mut materials);
    let session = GameSession::new(layout.pellets_total());

    commands.spawn(Camera2d);
    spawn_hud(&mut commands);
    spawn_gameplay(&mut commands, &layout, &mesh_handles, &material_handles);

    commands.insert_resource(mesh_handles);
    commands.insert_resource(material_handles);
    commands.insert_resource(session);
}

pub fn handle_restart_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    gameplay_roots: Query<Entity, (With<GameplayEntity>, Without<ChildOf>)>,
    layout: Res<LevelLayout>,
    meshes: Res<GameMeshes>,
    materials: Res<GameMaterials>,
    mut session: ResMut<GameSession>,
) {
    if !matches!(session.phase, RoundPhase::Won | RoundPhase::GameOver) {
        return;
    }

    if !(keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter)) {
        return;
    }

    cleanup_gameplay(&mut commands, &gameplay_roots);
    session.reset_for_new_game(layout.pellets_total());
    spawn_gameplay(&mut commands, &layout, &meshes, &materials);
}

pub fn tick_round_state(time: Res<Time>, mut session: ResMut<GameSession>) {
    match session.phase {
        RoundPhase::Ready => {
            session.phase_timer.tick(time.delta());
            if session.phase_timer.is_finished() {
                session.phase = RoundPhase::Playing;
            }
        }
        RoundPhase::Playing => {
            let was_frightened = session.frightened_active();
            session.frightened_seconds = (session.frightened_seconds - time.delta_secs()).max(0.0);
            if was_frightened && !session.frightened_active() {
                session.ghost_combo = 0;
            }
            session.advance_mode_cycle(time.delta_secs());
        }
        RoundPhase::Won | RoundPhase::GameOver => {}
    }
}

pub fn handle_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    session: Res<GameSession>,
    mut player: Query<&mut GridMover, With<Player>>,
) {
    if matches!(session.phase, RoundPhase::Won | RoundPhase::GameOver) {
        return;
    }

    let Some(direction) = latest_direction_input(&keyboard) else {
        return;
    };

    let Ok(mut mover) = player.single_mut() else {
        return;
    };

    mover.desired = Some(direction);
}

pub fn plan_ghost_turns(
    layout: Res<LevelLayout>,
    session: Res<GameSession>,
    player: Query<(&Transform, &GridMover), (With<Player>, Without<Ghost>)>,
    mut ghosts: ParamSet<(
        Query<(&Ghost, &Transform), Without<Player>>,
        Query<(&mut Ghost, &Transform, &mut GridMover), Without<Player>>,
    )>,
) {
    if session.phase != RoundPhase::Playing {
        return;
    }

    let Ok((player_transform, player_mover)) = player.single() else {
        return;
    };

    let player_tile = layout.world_to_tile(player_transform.translation.truncate());
    let player_direction = player_mover.current.unwrap_or(Direction::Left);

    let mut blinky_tile = player_tile;
    for (ghost, ghost_transform) in &ghosts.p0() {
        if ghost.personality == GhostPersonality::Blinky {
            blinky_tile = layout.world_to_tile(ghost_transform.translation.truncate());
            break;
        }
    }

    for (mut ghost, ghost_transform, mut mover) in &mut ghosts.p1() {
        mover.speed = if ghost.returning_home {
            GHOST_EATEN_SPEED
        } else if session.frightened_active() {
            GHOST_FRIGHTENED_SPEED
        } else {
            GHOST_SPEED
        };

        let position = ghost_transform.translation.truncate();
        let tile = layout.world_to_tile(position);
        let centered = is_centered(position, tile, &layout);

        if ghost.returning_home && centered && tile == ghost.home_tile {
            ghost.returning_home = false;
            mover.current = Some(Direction::Up);
            mover.desired = Some(Direction::Up);
        }

        if !centered {
            continue;
        }

        let mut options: Vec<_> = Direction::ALL
            .into_iter()
            .filter(|direction| layout.can_move(tile, *direction))
            .collect();

        if options.len() > 1 {
            if let Some(current) = mover.current {
                options.retain(|direction| *direction != current.opposite());
            }
        }

        if options.is_empty() {
            continue;
        }

        let next_direction = if ghost.returning_home {
            choose_direction_toward(tile, ghost.home_tile, &options)
        } else if session.frightened_active() {
            choose_direction_away(tile, player_tile, &options)
        } else {
            let target = if session.scatter_mode {
                ghost.scatter_target
            } else {
                chase_target(
                    &ghost,
                    tile,
                    player_tile,
                    player_direction,
                    blinky_tile,
                    &layout,
                )
            };
            choose_direction_toward(tile, target, &options)
        };

        mover.current = Some(next_direction);
        mover.desired = Some(next_direction);
    }
}

pub fn move_player(
    time: Res<Time>,
    layout: Res<LevelLayout>,
    session: Res<GameSession>,
    mut player: Query<(&mut Transform, &mut GridMover), With<Player>>,
) {
    if session.phase != RoundPhase::Playing {
        return;
    }

    let Ok((mut transform, mut mover)) = player.single_mut() else {
        return;
    };

    step_mover(&mut transform, &mut mover, &layout, time.delta_secs());
}

pub fn move_ghosts(
    time: Res<Time>,
    layout: Res<LevelLayout>,
    session: Res<GameSession>,
    mut ghosts: Query<(&mut Transform, &mut GridMover), With<Ghost>>,
) {
    if session.phase != RoundPhase::Playing {
        return;
    }

    for (mut transform, mut mover) in &mut ghosts {
        step_mover(&mut transform, &mut mover, &layout, time.delta_secs());
    }
}

pub fn collect_pellets(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    player: Query<&Transform, With<Player>>,
    pellets: Query<(Entity, &Pellet, &Transform)>,
    mut ghosts: Query<(&Ghost, &mut GridMover)>,
) {
    if session.phase != RoundPhase::Playing {
        return;
    }

    let Ok(player_transform) = player.single() else {
        return;
    };

    let player_position = player_transform.translation.truncate();
    let mut ate_power_pellet = false;

    for (entity, pellet, pellet_transform) in &pellets {
        if player_position.distance_squared(pellet_transform.translation.truncate())
            > (TILE_SIZE * 0.35).powi(2)
        {
            continue;
        }

        commands.entity(entity).despawn();
        session.pellets_remaining = session.pellets_remaining.saturating_sub(1);

        match pellet.kind {
            PelletKind::Dot => session.score += PLAYER_SCORE_PELLET,
            PelletKind::Power => {
                session.score += PLAYER_SCORE_POWER;
                session.set_frightened();
                ate_power_pellet = true;
            }
        }

        if session.pellets_remaining == 0 {
            session.phase = RoundPhase::Won;
        }
    }

    if ate_power_pellet {
        for (ghost, mut mover) in &mut ghosts {
            if ghost.returning_home {
                continue;
            }

            if let Some(current) = mover.current {
                let reversed = current.opposite();
                mover.current = Some(reversed);
                mover.desired = Some(reversed);
            }
        }
    }
}

pub fn resolve_ghost_collisions(
    layout: Res<LevelLayout>,
    mut session: ResMut<GameSession>,
    mut queries: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<(&mut Ghost, &Transform, &mut GridMover)>,
        Query<(&mut Transform, &mut GridMover), With<Player>>,
        Query<(&mut Ghost, &mut Transform, &mut GridMover)>,
    )>,
) {
    if session.phase != RoundPhase::Playing {
        return;
    }

    let player_query = queries.p0();
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_position = player_transform.translation.truncate();
    let collision_radius_sq = GHOST_COLLISION_RADIUS * GHOST_COLLISION_RADIUS;
    let mut player_hit = false;

    for (mut ghost, ghost_transform, mut mover) in &mut queries.p1() {
        if player_position.distance_squared(ghost_transform.translation.truncate())
            > collision_radius_sq
        {
            continue;
        }

        if ghost.returning_home {
            continue;
        }

        if session.frightened_active() {
            ghost.returning_home = true;
            let combo_multiplier = 1_u32 << u32::from(session.ghost_combo.min(3));
            session.score += PLAYER_SCORE_GHOST * combo_multiplier;
            session.ghost_combo = session.ghost_combo.saturating_add(1);
            mover.speed = GHOST_EATEN_SPEED;
            continue;
        }

        player_hit = true;
        break;
    }

    if !player_hit {
        return;
    }

    if session.lives > 1 {
        session.lives -= 1;
        session.begin_ready_phase();

        if let Ok((mut player_transform, mut player_mover)) = queries.p2().single_mut() {
            reset_mover_position(&mut player_transform, &mut player_mover, &layout);
            player_mover.speed = PLAYER_SPEED;
        }

        for (mut ghost, mut transform, mut mover) in &mut queries.p3() {
            ghost.returning_home = false;
            reset_mover_position(&mut transform, &mut mover, &layout);
            mover.speed = GHOST_SPEED;
        }
    } else {
        session.lives = 0;
        session.phase = RoundPhase::GameOver;
        session.frightened_seconds = 0.0;
        session.ghost_combo = 0;
    }
}

pub fn sync_ghost_appearance(
    session: Res<GameSession>,
    materials: Res<GameMaterials>,
    mut ghosts: Query<(&Ghost, &mut MeshMaterial2d<ColorMaterial>)>,
) {
    for (ghost, mut material) in &mut ghosts {
        let next_material = if ghost.returning_home {
            materials.hidden_ghost.clone()
        } else if session.frightened_active() {
            materials.frightened_ghost.clone()
        } else {
            ghost.normal_material.clone()
        };

        *material = MeshMaterial2d(next_material);
    }
}

pub fn animate_pacman(
    time: Res<Time>,
    session: Res<GameSession>,
    player: Query<&GridMover, With<Player>>,
    mut mouth: Query<&mut Transform, With<PacmanMouth>>,
) {
    let Ok(player_mover) = player.single() else {
        return;
    };
    let Ok(mut mouth_transform) = mouth.single_mut() else {
        return;
    };

    let moving = session.phase == RoundPhase::Playing && player_mover.current.is_some();
    let openness = if moving {
        0.15 + time.elapsed_secs().sin().abs() * 0.95
    } else {
        0.12
    };

    mouth_transform.scale.y = openness;
}

pub fn animate_power_pellets(time: Res<Time>, mut pellets: Query<(&Pellet, &mut Transform)>) {
    let pulse = 0.85 + time.elapsed_secs().sin().abs() * 0.4;

    for (pellet, mut transform) in &mut pellets {
        if pellet.kind == PelletKind::Power {
            transform.scale = Vec3::splat(pulse);
        }
    }
}

pub fn update_hud(
    session: Res<GameSession>,
    mut hud_text: ParamSet<(
        Query<&mut Text, With<ScoreText>>,
        Query<&mut Text, With<LivesText>>,
        Query<&mut Text, With<MessageText>>,
    )>,
) {
    if let Ok(mut text) = hud_text.p0().single_mut() {
        text.0 = format!("SCORE {:05}", session.score);
    }

    if let Ok(mut text) = hud_text.p1().single_mut() {
        text.0 = format!("LIVES {}", session.lives);
    }

    if let Ok(mut text) = hud_text.p2().single_mut() {
        text.0 = match session.phase {
            RoundPhase::Ready => "READY!".into(),
            RoundPhase::Playing => String::new(),
            RoundPhase::Won => "YOU WIN  PRESS SPACE".into(),
            RoundPhase::GameOver => "GAME OVER  PRESS SPACE".into(),
        };
    }
}

fn spawn_hud(commands: &mut Commands) {
    commands.spawn((
        Text::new("SCORE 00000"),
        TextFont::from_font_size(28.0),
        TextColor(HUD_COLOR),
        Node {
            position_type: PositionType::Absolute,
            top: px(14),
            left: px(18),
            ..default()
        },
        ScoreText,
    ));

    commands.spawn((
        Text::new("LIVES 3"),
        TextFont::from_font_size(28.0),
        TextColor(HUD_COLOR),
        Node {
            position_type: PositionType::Absolute,
            top: px(14),
            right: px(18),
            ..default()
        },
        LivesText,
    ));

    commands.spawn((
        Text::new(""),
        TextFont::from_font_size(38.0),
        TextColor(MESSAGE_COLOR),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: percent(46),
            width: percent(100),
            ..default()
        },
        MessageText,
    ));
}

fn spawn_gameplay(
    commands: &mut Commands,
    layout: &LevelLayout,
    meshes: &GameMeshes,
    materials: &GameMaterials,
) {
    for y in 0..layout.height {
        for x in 0..layout.width {
            let tile = IVec2::new(x, y);
            if !layout.is_wall(tile) {
                continue;
            }

            commands.spawn((
                GameplayEntity,
                Mesh2d(meshes.wall.clone()),
                MeshMaterial2d(materials.wall.clone()),
                Transform::from_translation(layout.tile_to_world(tile).extend(WALL_Z)),
            ));
        }
    }

    for (tile, kind) in &layout.pellet_spawns {
        let mesh = match kind {
            PelletKind::Dot => meshes.pellet.clone(),
            PelletKind::Power => meshes.power_pellet.clone(),
        };

        commands.spawn((
            GameplayEntity,
            Pellet { kind: *kind },
            Mesh2d(mesh),
            MeshMaterial2d(materials.pellet.clone()),
            Transform::from_translation(layout.tile_to_world(*tile).extend(PELLET_Z)),
        ));
    }

    commands
        .spawn((
            GameplayEntity,
            Player,
            GridMover {
                current: Some(Direction::Left),
                desired: Some(Direction::Left),
                speed: PLAYER_SPEED,
                spawn_tile: layout.player_spawn,
                spawn_direction: Some(Direction::Left),
            },
            Mesh2d(meshes.actor.clone()),
            MeshMaterial2d(materials.pacman.clone()),
            Transform::from_translation(layout.tile_to_world(layout.player_spawn).extend(ACTOR_Z))
                .with_rotation(Direction::Left.rotation()),
        ))
        .with_children(|parent| {
            parent.spawn((
                PacmanMouth,
                Mesh2d(meshes.mouth.clone()),
                MeshMaterial2d(materials.mouth_cutout.clone()),
                Transform::from_xyz(0.0, 0.0, DETAIL_Z),
            ));
        });

    for (index, spawn_tile) in layout.ghost_spawns.iter().take(4).enumerate() {
        let personality = GhostPersonality::ORDER[index];
        let normal_material = materials.ghost_colors[personality.index()].clone();
        let scatter_target = scatter_corner(personality, layout);
        let initial_direction = if matches!(
            personality,
            GhostPersonality::Blinky | GhostPersonality::Inky
        ) {
            Direction::Left
        } else {
            Direction::Right
        };

        commands
            .spawn((
                GameplayEntity,
                Ghost {
                    personality,
                    home_tile: *spawn_tile,
                    scatter_target,
                    normal_material: normal_material.clone(),
                    returning_home: false,
                },
                GridMover {
                    current: Some(initial_direction),
                    desired: Some(initial_direction),
                    speed: GHOST_SPEED,
                    spawn_tile: *spawn_tile,
                    spawn_direction: Some(initial_direction),
                },
                Mesh2d(meshes.actor.clone()),
                MeshMaterial2d(normal_material),
                Transform::from_translation(layout.tile_to_world(*spawn_tile).extend(ACTOR_Z))
                    .with_scale(Vec3::splat(GHOST_RADIUS / PACMAN_RADIUS)),
            ))
            .with_children(|parent| {
                let eye_y = GHOST_RADIUS * 0.18;
                let eye_x = GHOST_RADIUS * 0.32;

                parent.spawn((
                    Mesh2d(meshes.eye.clone()),
                    MeshMaterial2d(materials.eye_white.clone()),
                    Transform::from_xyz(-eye_x, eye_y, DETAIL_Z),
                ));
                parent.spawn((
                    Mesh2d(meshes.eye.clone()),
                    MeshMaterial2d(materials.eye_white.clone()),
                    Transform::from_xyz(eye_x, eye_y, DETAIL_Z),
                ));
                parent.spawn((
                    Mesh2d(meshes.pupil.clone()),
                    MeshMaterial2d(materials.eye_pupil.clone()),
                    Transform::from_xyz(
                        -eye_x + GHOST_RADIUS * 0.05,
                        eye_y - GHOST_RADIUS * 0.02,
                        DETAIL_Z + 0.05,
                    ),
                ));
                parent.spawn((
                    Mesh2d(meshes.pupil.clone()),
                    MeshMaterial2d(materials.eye_pupil.clone()),
                    Transform::from_xyz(
                        eye_x + GHOST_RADIUS * 0.05,
                        eye_y - GHOST_RADIUS * 0.02,
                        DETAIL_Z + 0.05,
                    ),
                ));
            });
    }
}

fn cleanup_gameplay(
    commands: &mut Commands,
    gameplay_roots: &Query<Entity, (With<GameplayEntity>, Without<ChildOf>)>,
) {
    for entity in gameplay_roots.iter() {
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .despawn();
    }
}

fn latest_direction_input(keyboard: &ButtonInput<KeyCode>) -> Option<Direction> {
    for (key, direction) in [
        (KeyCode::ArrowUp, Direction::Up),
        (KeyCode::ArrowLeft, Direction::Left),
        (KeyCode::ArrowDown, Direction::Down),
        (KeyCode::ArrowRight, Direction::Right),
        (KeyCode::KeyW, Direction::Up),
        (KeyCode::KeyA, Direction::Left),
        (KeyCode::KeyS, Direction::Down),
        (KeyCode::KeyD, Direction::Right),
    ] {
        if keyboard.just_pressed(key) {
            return Some(direction);
        }
    }

    for (key, direction) in [
        (KeyCode::ArrowUp, Direction::Up),
        (KeyCode::ArrowLeft, Direction::Left),
        (KeyCode::ArrowDown, Direction::Down),
        (KeyCode::ArrowRight, Direction::Right),
        (KeyCode::KeyW, Direction::Up),
        (KeyCode::KeyA, Direction::Left),
        (KeyCode::KeyS, Direction::Down),
        (KeyCode::KeyD, Direction::Right),
    ] {
        if keyboard.pressed(key) {
            return Some(direction);
        }
    }

    None
}

fn step_mover(transform: &mut Transform, mover: &mut GridMover, layout: &LevelLayout, delta: f32) {
    let tile = layout.world_to_tile(transform.translation.truncate());
    let center = layout.tile_to_world(tile);
    let centered = is_centered(transform.translation.truncate(), tile, layout);

    if centered {
        transform.translation.x = center.x;
        transform.translation.y = center.y;

        if let Some(desired) = mover.desired {
            if layout.can_move(tile, desired) {
                mover.current = Some(desired);
            }
        }

        if let Some(current) = mover.current {
            if !layout.can_move(tile, current) {
                mover.current = None;
            }
        }
    }

    let Some(current) = mover.current else {
        return;
    };

    if current.is_horizontal() {
        transform.translation.y = transform.translation.y.lerp(center.y, 0.45);
    } else {
        transform.translation.x = transform.translation.x.lerp(center.x, 0.45);
    }

    transform.translation += (current.vec2() * mover.speed * delta).extend(0.0);
    transform.rotation = current.rotation();

    if current.is_horizontal() {
        layout.wrap_translation(&mut transform.translation, tile.y);
    }
}

fn reset_mover_position(transform: &mut Transform, mover: &mut GridMover, layout: &LevelLayout) {
    transform.translation = layout.tile_to_world(mover.spawn_tile).extend(ACTOR_Z);
    if let Some(direction) = mover.spawn_direction {
        transform.rotation = direction.rotation();
    }
    mover.current = mover.spawn_direction;
    mover.desired = mover.spawn_direction;
}

fn is_centered(position: Vec2, tile: IVec2, layout: &LevelLayout) -> bool {
    position.distance(layout.tile_to_world(tile)) <= TURN_TOLERANCE
}

fn choose_direction_toward(tile: IVec2, target: IVec2, options: &[Direction]) -> Direction {
    let mut best_direction = options[0];
    let mut best_distance = f32::MAX;

    for direction in options {
        let next_tile = tile + direction.ivec2();
        let distance = target.as_vec2().distance_squared(next_tile.as_vec2());
        if distance < best_distance {
            best_distance = distance;
            best_direction = *direction;
        }
    }

    best_direction
}

fn choose_direction_away(tile: IVec2, target: IVec2, options: &[Direction]) -> Direction {
    let mut best_direction = options[0];
    let mut best_distance = f32::MIN;

    for direction in options {
        let next_tile = tile + direction.ivec2();
        let distance = target.as_vec2().distance_squared(next_tile.as_vec2());
        if distance > best_distance {
            best_distance = distance;
            best_direction = *direction;
        }
    }

    best_direction
}

fn chase_target(
    ghost: &Ghost,
    ghost_tile: IVec2,
    player_tile: IVec2,
    player_direction: Direction,
    blinky_tile: IVec2,
    layout: &LevelLayout,
) -> IVec2 {
    let ahead = player_direction.ivec2();

    let raw_target = match ghost.personality {
        GhostPersonality::Blinky => player_tile,
        GhostPersonality::Pinky => player_tile + ahead * 4,
        GhostPersonality::Inky => player_tile + ahead * 2 + (player_tile - blinky_tile),
        GhostPersonality::Clyde => {
            if ghost_tile.as_vec2().distance(player_tile.as_vec2()) > 7.0 {
                player_tile
            } else {
                ghost.scatter_target
            }
        }
    };

    layout.clamp_target(raw_target)
}

fn scatter_corner(personality: GhostPersonality, layout: &LevelLayout) -> IVec2 {
    match personality {
        GhostPersonality::Blinky => IVec2::new(layout.width - 2, 1),
        GhostPersonality::Pinky => IVec2::new(1, 1),
        GhostPersonality::Inky => IVec2::new(layout.width - 2, layout.height - 2),
        GhostPersonality::Clyde => IVec2::new(1, layout.height - 2),
    }
}
