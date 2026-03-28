use bevy::prelude::*;

use crate::game::{
    components::{
        Direction, Ghost, GhostAppearance, GhostPersonality, GridMover, LevelEntity, LivesText,
        MessageText, PacmanMouth, Pellet, PelletKind, Player, RoundEntity, ScoreText,
    },
    constants::{
        ACTOR_Z, DETAIL_Z, GHOST_RADIUS, GHOST_SPEED, HUD_COLOR, MESSAGE_COLOR, PACMAN_RADIUS,
        PELLET_Z, PLAYER_SPEED, WALL_Z,
    },
    level::LevelLayout,
    logic::scatter_corner,
    resources::{GameMaterials, GameMeshes},
};

pub fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(GameMeshes::new(&mut meshes));
    commands.insert_resource(GameMaterials::new(&mut materials));
}

pub fn setup_scene(
    mut commands: Commands,
    layout: Res<LevelLayout>,
    meshes: Res<GameMeshes>,
    materials: Res<GameMaterials>,
) {
    commands.spawn((Name::new("Camera"), Camera2d));
    spawn_hud(&mut commands);
    spawn_level_geometry(&mut commands, &layout, &meshes, &materials);
}

pub fn spawn_round(
    mut commands: Commands,
    layout: Res<LevelLayout>,
    meshes: Res<GameMeshes>,
    materials: Res<GameMaterials>,
) {
    spawn_round_entities(&mut commands, &layout, &meshes, &materials);
}

pub(crate) fn spawn_round_entities(
    commands: &mut Commands,
    layout: &LevelLayout,
    meshes: &GameMeshes,
    materials: &GameMaterials,
) {
    spawn_pellets(commands, layout, meshes, materials);
    spawn_actors(commands, layout, meshes, materials);
}

pub(crate) fn cleanup_round_entities(
    commands: &mut Commands,
    round_roots: &Query<Entity, (With<RoundEntity>, Without<ChildOf>)>,
) {
    for entity in round_roots.iter() {
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .despawn();
    }
}

fn spawn_hud(commands: &mut Commands) {
    commands.spawn((
        Name::new("HUD Score"),
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
        Name::new("HUD Lives"),
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
        Name::new("HUD Message"),
        Text::new("READY!"),
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

fn spawn_level_geometry(
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
                Name::new(format!("Wall ({x}, {y})")),
                LevelEntity,
                Mesh2d(meshes.wall.clone()),
                MeshMaterial2d(materials.wall.clone()),
                Transform::from_translation(layout.tile_to_world(tile).extend(WALL_Z)),
            ));
        }
    }
}

fn spawn_pellets(
    commands: &mut Commands,
    layout: &LevelLayout,
    meshes: &GameMeshes,
    materials: &GameMaterials,
) {
    for (tile, kind) in &layout.pellet_spawns {
        let mesh = match kind {
            PelletKind::Dot => meshes.pellet.clone(),
            PelletKind::Power => meshes.power_pellet.clone(),
        };

        commands.spawn((
            Name::new(format!("Pellet ({}, {})", tile.x, tile.y)),
            RoundEntity,
            Pellet { kind: *kind },
            Mesh2d(mesh),
            MeshMaterial2d(materials.pellet.clone()),
            Transform::from_translation(layout.tile_to_world(*tile).extend(PELLET_Z)),
        ));
    }
}

fn spawn_actors(
    commands: &mut Commands,
    layout: &LevelLayout,
    meshes: &GameMeshes,
    materials: &GameMaterials,
) {
    commands
        .spawn((
            Name::new("Pac-Man"),
            RoundEntity,
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
                Name::new("Pac-Man Mouth"),
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
                Name::new(ghost_name(personality)),
                RoundEntity,
                Ghost {
                    personality,
                    home_tile: *spawn_tile,
                    scatter_target,
                    returning_home: false,
                },
                GhostAppearance {
                    normal_material: normal_material.clone(),
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
                    Name::new("Left Eye"),
                    Mesh2d(meshes.eye.clone()),
                    MeshMaterial2d(materials.eye_white.clone()),
                    Transform::from_xyz(-eye_x, eye_y, DETAIL_Z),
                ));
                parent.spawn((
                    Name::new("Right Eye"),
                    Mesh2d(meshes.eye.clone()),
                    MeshMaterial2d(materials.eye_white.clone()),
                    Transform::from_xyz(eye_x, eye_y, DETAIL_Z),
                ));
                parent.spawn((
                    Name::new("Left Pupil"),
                    Mesh2d(meshes.pupil.clone()),
                    MeshMaterial2d(materials.eye_pupil.clone()),
                    Transform::from_xyz(
                        -eye_x + GHOST_RADIUS * 0.05,
                        eye_y - GHOST_RADIUS * 0.02,
                        DETAIL_Z + 0.05,
                    ),
                ));
                parent.spawn((
                    Name::new("Right Pupil"),
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

fn ghost_name(personality: GhostPersonality) -> &'static str {
    match personality {
        GhostPersonality::Blinky => "Blinky",
        GhostPersonality::Pinky => "Pinky",
        GhostPersonality::Inky => "Inky",
        GhostPersonality::Clyde => "Clyde",
    }
}
