use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::{GameMaterials, GameMeshes};

// --- Level Data Types ---

#[derive(Clone, Copy)]
pub struct GirderDef {
    pub left: Vec2,
    pub right: Vec2,
    pub roll_direction: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LadderKind {
    Full,
    Broken,
}

#[derive(Clone, Copy)]
pub struct LadderDef {
    pub x: f32,
    pub bottom_girder: usize,
    pub top_girder: usize,
    pub kind: LadderKind,
}

// --- Stage Data Resource ---

#[derive(Resource)]
pub struct StageData {
    pub girders: Vec<GirderDef>,
    pub ladders: Vec<LadderDef>,
    pub player_spawn: Vec2,
    pub dk_position: Vec2,
    pub pauline_position: Vec2,
    pub oil_drum_position: Vec2,
    pub oil_drum_x: f32,
    pub hammer_positions: [Vec2; 2],
    pub bonus_item_position: Vec2,
    pub goal_zone_center: Vec2,
}

impl StageData {
    pub fn new() -> Self {
        let girders = vec![
            // P1 (0): flat bottom
            GirderDef { left: Vec2::new(-100.0, -108.0), right: Vec2::new(100.0, -108.0), roll_direction: -1.0 },
            // P2 (1): slopes down L→R (left higher)
            GirderDef { left: Vec2::new(-100.0, -68.0), right: Vec2::new(100.0, -76.0), roll_direction: 1.0 },
            // P3 (2): slopes down R→L (right higher)
            GirderDef { left: Vec2::new(-100.0, -40.0), right: Vec2::new(100.0, -32.0), roll_direction: -1.0 },
            // P4 (3): slopes down L→R (left higher)
            GirderDef { left: Vec2::new(-100.0, 4.0), right: Vec2::new(100.0, -4.0), roll_direction: 1.0 },
            // P5 (4): slopes down R→L (right higher)
            GirderDef { left: Vec2::new(-100.0, 32.0), right: Vec2::new(100.0, 40.0), roll_direction: -1.0 },
            // P6 (5): DK platform, shorter, flat
            GirderDef { left: Vec2::new(-100.0, 72.0), right: Vec2::new(-10.0, 72.0), roll_direction: 1.0 },
            // Perch (6): Pauline platform, tiny, flat
            GirderDef { left: Vec2::new(-60.0, 92.0), right: Vec2::new(-25.0, 92.0), roll_direction: 0.0 },
        ];

        let ladders = vec![
            // P1→P2
            LadderDef { x: -50.0, bottom_girder: 0, top_girder: 1, kind: LadderKind::Full },
            LadderDef { x: 70.0, bottom_girder: 0, top_girder: 1, kind: LadderKind::Full },
            // P2→P3
            LadderDef { x: -70.0, bottom_girder: 1, top_girder: 2, kind: LadderKind::Full },
            LadderDef { x: 30.0, bottom_girder: 1, top_girder: 2, kind: LadderKind::Broken },
            // P3→P4
            LadderDef { x: -40.0, bottom_girder: 2, top_girder: 3, kind: LadderKind::Broken },
            LadderDef { x: 80.0, bottom_girder: 2, top_girder: 3, kind: LadderKind::Full },
            // P4→P5
            LadderDef { x: -80.0, bottom_girder: 3, top_girder: 4, kind: LadderKind::Full },
            LadderDef { x: 40.0, bottom_girder: 3, top_girder: 4, kind: LadderKind::Full },
            // P5→P6
            LadderDef { x: -80.0, bottom_girder: 4, top_girder: 5, kind: LadderKind::Full },
        ];

        let player_spawn = Vec2::new(-65.0, girder_surface_y(&girders[0], -65.0) + PLAYER_HEIGHT / 2.0);
        let dk_pos = Vec2::new(-70.0, girder_surface_y(&girders[5], -70.0) + DK_HEIGHT / 2.0);
        let pauline_pos = Vec2::new(-42.0, girder_surface_y(&girders[6], -42.0) + PAULINE_HEIGHT / 2.0);
        let oil_drum_pos = Vec2::new(-90.0, girder_surface_y(&girders[0], -90.0) + OIL_DRUM_HEIGHT / 2.0);

        let ha_x = 0.0;
        let hammer_a = Vec2::new(ha_x, girder_surface_y(&girders[1], ha_x) + HAMMER_PICKUP_SIZE / 2.0);
        let hb_x = -20.0;
        let hammer_b = Vec2::new(hb_x, girder_surface_y(&girders[3], hb_x) + HAMMER_PICKUP_SIZE / 2.0);

        let bx = 20.0;
        let bonus_pos = Vec2::new(bx, girder_surface_y(&girders[2], bx) + BONUS_ITEM_SIZE / 2.0);

        Self {
            girders,
            ladders,
            player_spawn,
            dk_position: dk_pos,
            pauline_position: pauline_pos,
            oil_drum_position: oil_drum_pos,
            oil_drum_x: -90.0,
            hammer_positions: [hammer_a, hammer_b],
            bonus_item_position: bonus_pos,
            goal_zone_center: pauline_pos,
        }
    }
}

// --- Geometry Helpers ---

pub fn girder_y_at(girder: &GirderDef, x: f32) -> f32 {
    let range = girder.right.x - girder.left.x;
    if range.abs() < 0.001 {
        return girder.left.y;
    }
    let t = (x - girder.left.x) / range;
    girder.left.y + (girder.right.y - girder.left.y) * t
}

/// Y of the top surface of the girder at the given X.
pub fn girder_surface_y(girder: &GirderDef, x: f32) -> f32 {
    girder_y_at(girder, x) + GIRDER_THICKNESS / 2.0
}

/// Find the highest girder supporting an entity at (x, feet_y).
pub fn find_supporting_girder(stage: &StageData, x: f32, feet_y: f32) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, g) in stage.girders.iter().enumerate() {
        if x < g.left.x || x > g.right.x {
            continue;
        }
        let surface = girder_surface_y(g, x);
        let diff = feet_y - surface;
        if diff >= -GIRDER_SUPPORT_TOLERANCE && diff <= GIRDER_SUPPORT_TOLERANCE {
            match best {
                None => best = Some((i, surface)),
                Some((_, by)) if surface > by => best = Some((i, surface)),
                _ => {}
            }
        }
    }
    best.map(|(i, _)| i)
}

/// Find the highest girder whose surface is strictly below feet_y.
pub fn find_girder_below(stage: &StageData, x: f32, feet_y: f32) -> Option<(usize, f32)> {
    let mut best: Option<(usize, f32)> = None;
    for (i, g) in stage.girders.iter().enumerate() {
        if x < g.left.x || x > g.right.x {
            continue;
        }
        let surface = girder_surface_y(g, x);
        if surface < feet_y - GIRDER_SUPPORT_TOLERANCE {
            match best {
                None => best = Some((i, surface)),
                Some((_, by)) if surface > by => best = Some((i, surface)),
                _ => {}
            }
        }
    }
    best
}

// --- Spawning ---

pub fn spawn_stage(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    stage: &StageData,
) {
    let girder_mat = materials.add(GIRDER_COLOR);
    let ladder_mat = materials.add(LADDER_COLOR);
    let broken_mat = materials.add(LADDER_BROKEN_COLOR);
    let oil_mat = materials.add(OIL_DRUM_COLOR);
    let dk_mat = materials.add(DK_COLOR);
    let pauline_mat = materials.add(PAULINE_COLOR);

    // Girders
    for girder in &stage.girders {
        let dx = girder.right.x - girder.left.x;
        let dy = girder.right.y - girder.left.y;
        let length = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx);
        let cx = (girder.left.x + girder.right.x) / 2.0;
        let cy = (girder.left.y + girder.right.y) / 2.0;

        commands.spawn((
            StageEntity,
            GirderEntity,
            Mesh2d(meshes.add(Rectangle::new(length, GIRDER_THICKNESS))),
            MeshMaterial2d(girder_mat.clone()),
            Transform::from_xyz(cx, cy, 1.0).with_rotation(Quat::from_rotation_z(angle)),
        ));
    }

    // Ladders
    for (i, ladder) in stage.ladders.iter().enumerate() {
        let bot_y = girder_surface_y(&stage.girders[ladder.bottom_girder], ladder.x);
        let top_y = girder_surface_y(&stage.girders[ladder.top_girder], ladder.x);
        let height = top_y - bot_y;
        let cy = (top_y + bot_y) / 2.0;
        let mat = if ladder.kind == LadderKind::Full { &ladder_mat } else { &broken_mat };

        // Two rails
        let rail_w = 2.0;
        let spacing = LADDER_WIDTH / 2.0 - rail_w / 2.0;
        for dx in [-spacing, spacing] {
            commands.spawn((
                StageEntity,
                LadderEntity(i),
                Mesh2d(meshes.add(Rectangle::new(rail_w, height))),
                MeshMaterial2d(mat.clone()),
                Transform::from_xyz(ladder.x + dx, cy, 2.0),
            ));
        }
        // Rungs
        let count = ((height / 6.0) as i32).max(1);
        for r in 0..count {
            let ry = bot_y + (r as f32 + 0.5) * height / count as f32;
            commands.spawn((
                StageEntity,
                LadderEntity(i),
                Mesh2d(meshes.add(Rectangle::new(LADDER_WIDTH - 2.0, 1.5))),
                MeshMaterial2d(mat.clone()),
                Transform::from_xyz(ladder.x, ry, 2.0),
            ));
        }
    }

    // Oil drum
    commands.spawn((
        StageEntity,
        OilDrumEntity,
        Mesh2d(meshes.add(Rectangle::new(OIL_DRUM_WIDTH, OIL_DRUM_HEIGHT))),
        MeshMaterial2d(oil_mat),
        Transform::from_xyz(stage.oil_drum_position.x, stage.oil_drum_position.y, 3.0),
    ));

    // DK
    commands.spawn((
        StageEntity,
        DonkeyKong,
        DkState {
            anim: DkAnimState::Idle,
            timer: 0.0,
            throw_timer: 0.0,
            barrels_thrown: 0,
        },
        Mesh2d(meshes.add(Rectangle::new(DK_WIDTH, DK_HEIGHT))),
        MeshMaterial2d(dk_mat),
        Transform::from_xyz(stage.dk_position.x, stage.dk_position.y, 5.0),
    ));

    // Pauline
    commands.spawn((
        StageEntity,
        PaulineEntity,
        Mesh2d(meshes.add(Rectangle::new(PAULINE_WIDTH, PAULINE_HEIGHT))),
        MeshMaterial2d(pauline_mat),
        Transform::from_xyz(stage.pauline_position.x, stage.pauline_position.y, 5.0),
    ));

    // Goal zone (invisible)
    commands.spawn((
        StageEntity,
        GoalZoneEntity,
        Transform::from_xyz(stage.goal_zone_center.x, stage.goal_zone_center.y, 0.0),
        Visibility::Hidden,
    ));
}

pub fn spawn_hammers(
    commands: &mut Commands,
    game_meshes: &GameMeshes,
    game_mats: &GameMaterials,
    stage: &StageData,
) {
    for (i, pos) in stage.hammer_positions.iter().enumerate() {
        commands.spawn((
            StageEntity,
            HammerPickup(i),
            Mesh2d(game_meshes.hammer_pickup.clone()),
            MeshMaterial2d(game_mats.hammer_pickup.clone()),
            Transform::from_xyz(pos.x, pos.y, 4.0),
        ));
    }
}

pub fn spawn_player_entity(
    commands: &mut Commands,
    game_meshes: &GameMeshes,
    game_mats: &GameMaterials,
    stage: &StageData,
) {
    commands.spawn((
        StageEntity,
        Player,
        PlayerState::default(),
        Mesh2d(game_meshes.player.clone()),
        MeshMaterial2d(game_mats.player_normal.clone()),
        Transform::from_xyz(stage.player_spawn.x, stage.player_spawn.y, 7.0),
    ));
}
