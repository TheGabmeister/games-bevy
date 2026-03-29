use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::AppState;
use crate::ui;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Playing),
            (
                prepare_playing_world,
                ApplyDeferred,
                spawn_playing_world,
                ui::spawn_hud,
            )
                .chain(),
        )
        .add_systems(OnEnter(AppState::Dying), enter_dying);
    }
}

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
            GirderDef {
                left: Vec2::new(-100.0, -108.0),
                right: Vec2::new(100.0, -108.0),
                roll_direction: -1.0,
            },
            // P2 (1): slopes down L->R (left higher)
            GirderDef {
                left: Vec2::new(-100.0, -68.0),
                right: Vec2::new(100.0, -76.0),
                roll_direction: 1.0,
            },
            // P3 (2): slopes down R->L (right higher)
            GirderDef {
                left: Vec2::new(-100.0, -40.0),
                right: Vec2::new(100.0, -32.0),
                roll_direction: -1.0,
            },
            // P4 (3): slopes down L->R (left higher)
            GirderDef {
                left: Vec2::new(-100.0, 4.0),
                right: Vec2::new(100.0, -4.0),
                roll_direction: 1.0,
            },
            // P5 (4): slopes down R->L (right higher)
            GirderDef {
                left: Vec2::new(-100.0, 32.0),
                right: Vec2::new(100.0, 40.0),
                roll_direction: -1.0,
            },
            // P6 (5): DK platform, shorter, flat
            GirderDef {
                left: Vec2::new(-100.0, 72.0),
                right: Vec2::new(-10.0, 72.0),
                roll_direction: 1.0,
            },
            // Perch (6): Pauline platform, tiny, flat
            GirderDef {
                left: Vec2::new(-60.0, 92.0),
                right: Vec2::new(-25.0, 92.0),
                roll_direction: 0.0,
            },
        ];

        let ladders = vec![
            // P1->P2
            LadderDef {
                x: -50.0,
                bottom_girder: 0,
                top_girder: 1,
                kind: LadderKind::Full,
            },
            LadderDef {
                x: 70.0,
                bottom_girder: 0,
                top_girder: 1,
                kind: LadderKind::Full,
            },
            // P2->P3
            LadderDef {
                x: -70.0,
                bottom_girder: 1,
                top_girder: 2,
                kind: LadderKind::Full,
            },
            LadderDef {
                x: 30.0,
                bottom_girder: 1,
                top_girder: 2,
                kind: LadderKind::Broken,
            },
            // P3->P4
            LadderDef {
                x: -40.0,
                bottom_girder: 2,
                top_girder: 3,
                kind: LadderKind::Broken,
            },
            LadderDef {
                x: 80.0,
                bottom_girder: 2,
                top_girder: 3,
                kind: LadderKind::Full,
            },
            // P4->P5
            LadderDef {
                x: -80.0,
                bottom_girder: 3,
                top_girder: 4,
                kind: LadderKind::Full,
            },
            LadderDef {
                x: 40.0,
                bottom_girder: 3,
                top_girder: 4,
                kind: LadderKind::Full,
            },
            // P5->P6
            LadderDef {
                x: -80.0,
                bottom_girder: 4,
                top_girder: 5,
                kind: LadderKind::Full,
            },
        ];

        let player_spawn = Vec2::new(
            -65.0,
            girder_surface_y(&girders[0], -65.0) + PLAYER_HEIGHT / 2.0,
        );
        let dk_position = Vec2::new(
            -70.0,
            girder_surface_y(&girders[5], -70.0) + DK_HEIGHT / 2.0,
        );
        let pauline_position = Vec2::new(
            -42.0,
            girder_surface_y(&girders[6], -42.0) + PAULINE_HEIGHT / 2.0,
        );
        let oil_drum_position = Vec2::new(
            -90.0,
            girder_surface_y(&girders[0], -90.0) + OIL_DRUM_HEIGHT / 2.0,
        );

        let hammer_a_x = 0.0;
        let hammer_a = Vec2::new(
            hammer_a_x,
            girder_surface_y(&girders[1], hammer_a_x) + HAMMER_PICKUP_SIZE / 2.0,
        );
        let hammer_b_x = -20.0;
        let hammer_b = Vec2::new(
            hammer_b_x,
            girder_surface_y(&girders[3], hammer_b_x) + HAMMER_PICKUP_SIZE / 2.0,
        );

        let bonus_x = 20.0;
        let bonus_item_position = Vec2::new(
            bonus_x,
            girder_surface_y(&girders[2], bonus_x) + BONUS_ITEM_SIZE / 2.0,
        );

        Self {
            girders,
            ladders,
            player_spawn,
            dk_position,
            pauline_position,
            oil_drum_position,
            oil_drum_x: -90.0,
            hammer_positions: [hammer_a, hammer_b],
            bonus_item_position,
            goal_zone_center: pauline_position,
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
    for (index, girder) in stage.girders.iter().enumerate() {
        if x < girder.left.x || x > girder.right.x {
            continue;
        }

        let surface = girder_surface_y(girder, x);
        let diff = feet_y - surface;
        if (-GIRDER_SUPPORT_TOLERANCE..=GIRDER_SUPPORT_TOLERANCE).contains(&diff) {
            match best {
                None => best = Some((index, surface)),
                Some((_, best_y)) if surface > best_y => best = Some((index, surface)),
                _ => {}
            }
        }
    }
    best.map(|(index, _)| index)
}

// --- Spawning ---

pub fn spawn_stage(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    stage: &StageData,
) {
    let girder_material = materials.add(GIRDER_COLOR);
    let ladder_material = materials.add(LADDER_COLOR);
    let broken_ladder_material = materials.add(LADDER_BROKEN_COLOR);
    let oil_material = materials.add(OIL_DRUM_COLOR);
    let dk_material = materials.add(DK_COLOR);
    let pauline_material = materials.add(PAULINE_COLOR);

    for girder in &stage.girders {
        let dx = girder.right.x - girder.left.x;
        let dy = girder.right.y - girder.left.y;
        let length = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx);
        let center_x = (girder.left.x + girder.right.x) / 2.0;
        let center_y = (girder.left.y + girder.right.y) / 2.0;

        commands.spawn((
            StageEntity,
            GirderEntity,
            Mesh2d(meshes.add(Rectangle::new(length, GIRDER_THICKNESS))),
            MeshMaterial2d(girder_material.clone()),
            Transform::from_xyz(center_x, center_y, 1.0)
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }

    for ladder in &stage.ladders {
        let bottom_y = girder_surface_y(&stage.girders[ladder.bottom_girder], ladder.x);
        let top_y = girder_surface_y(&stage.girders[ladder.top_girder], ladder.x);
        let height = top_y - bottom_y;
        let center_y = (top_y + bottom_y) / 2.0;
        let material = if ladder.kind == LadderKind::Full {
            &ladder_material
        } else {
            &broken_ladder_material
        };

        let rail_width = 2.0;
        let rail_spacing = LADDER_WIDTH / 2.0 - rail_width / 2.0;
        for offset_x in [-rail_spacing, rail_spacing] {
            commands.spawn((
                StageEntity,
                LadderEntity,
                Mesh2d(meshes.add(Rectangle::new(rail_width, height))),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(ladder.x + offset_x, center_y, 2.0),
            ));
        }

        let rung_count = ((height / 6.0) as i32).max(1);
        for rung_index in 0..rung_count {
            let rung_y = bottom_y + (rung_index as f32 + 0.5) * height / rung_count as f32;
            commands.spawn((
                StageEntity,
                LadderEntity,
                Mesh2d(meshes.add(Rectangle::new(LADDER_WIDTH - 2.0, 1.5))),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(ladder.x, rung_y, 2.0),
            ));
        }
    }

    commands.spawn((
        StageEntity,
        OilDrumEntity,
        Mesh2d(meshes.add(Rectangle::new(OIL_DRUM_WIDTH, OIL_DRUM_HEIGHT))),
        MeshMaterial2d(oil_material),
        Transform::from_xyz(stage.oil_drum_position.x, stage.oil_drum_position.y, 3.0),
    ));

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
        MeshMaterial2d(dk_material),
        Transform::from_xyz(stage.dk_position.x, stage.dk_position.y, 5.0),
    ));

    commands.spawn((
        StageEntity,
        PaulineEntity,
        Mesh2d(meshes.add(Rectangle::new(PAULINE_WIDTH, PAULINE_HEIGHT))),
        MeshMaterial2d(pauline_material),
        Transform::from_xyz(stage.pauline_position.x, stage.pauline_position.y, 5.0),
    ));

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
    game_materials: &GameMaterials,
    stage: &StageData,
) {
    for position in stage.hammer_positions {
        commands.spawn((
            AttemptEntity,
            HammerPickup,
            Mesh2d(game_meshes.hammer_pickup.clone()),
            MeshMaterial2d(game_materials.hammer_pickup.clone()),
            Transform::from_xyz(position.x, position.y, 4.0),
        ));
    }
}

pub fn spawn_bonus_item(
    commands: &mut Commands,
    game_meshes: &GameMeshes,
    game_materials: &GameMaterials,
    stage: &StageData,
    bonus_index: usize,
) {
    commands.spawn((
        AttemptEntity,
        BonusItemEntity(bonus_index),
        Mesh2d(game_meshes.bonus_item.clone()),
        MeshMaterial2d(game_materials.bonus_item.clone()),
        Transform::from_xyz(
            stage.bonus_item_position.x,
            stage.bonus_item_position.y,
            4.0,
        ),
    ));
}

pub fn spawn_player_entity(
    commands: &mut Commands,
    game_meshes: &GameMeshes,
    game_materials: &GameMaterials,
    stage: &StageData,
) {
    commands.spawn((
        StageEntity,
        Player,
        PlayerState::default(),
        Mesh2d(game_meshes.player.clone()),
        MeshMaterial2d(game_materials.player_normal.clone()),
        Transform::from_xyz(stage.player_spawn.x, stage.player_spawn.y, 7.0),
    ));
}

fn prepare_playing_world(
    mut commands: Commands,
    run_data: Res<RunData>,
    stage_entities: Query<Entity, With<StageEntity>>,
    attempt_entities: Query<Entity, With<AttemptEntity>>,
    hud_entities: Query<Entity, With<GameHudUI>>,
    tally_entities: Query<Entity, With<WaveTallyUI>>,
) {
    match run_data.next_entry {
        PlayingEntry::NewRun => {
            for entity in stage_entities
                .iter()
                .chain(attempt_entities.iter())
                .chain(hud_entities.iter())
                .chain(tally_entities.iter())
            {
                commands.entity(entity).despawn();
            }
        }
        PlayingEntry::RetryAfterDeath | PlayingEntry::NextWave => {
            for entity in attempt_entities.iter().chain(tally_entities.iter()) {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn spawn_playing_world(
    mut commands: Commands,
    mut run_data: ResMut<RunData>,
    mut wave_runtime: ResMut<WaveRuntime>,
    stage: Res<StageData>,
    game_meshes: Res<GameMeshes>,
    game_materials: Res<GameMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut PlayerState,
            &mut Visibility,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        With<Player>,
    >,
    mut dk_query: Query<&mut DkState, With<DonkeyKong>>,
) {
    match run_data.next_entry {
        PlayingEntry::NewRun => {
            *run_data = RunData::default();
            *wave_runtime = WaveRuntime::default();
            commands.insert_resource(WaveConfig::from_wave(1));

            spawn_stage(&mut commands, &mut meshes, &mut materials, &stage);
            spawn_player_entity(&mut commands, &game_meshes, &game_materials, &stage);
            spawn_hammers(&mut commands, &game_meshes, &game_materials, &stage);
        }
        PlayingEntry::RetryAfterDeath => {
            respawn_player_for_attempt(
                &stage,
                &game_materials,
                &mut player_query,
                &mut dk_query,
            );
            spawn_hammers(&mut commands, &game_meshes, &game_materials, &stage);
            respawn_active_bonus_items(
                &mut commands,
                &wave_runtime,
                &game_meshes,
                &game_materials,
                &stage,
            );
        }
        PlayingEntry::NextWave => {
            commands.insert_resource(WaveConfig::from_wave(run_data.wave));
            wave_runtime.bonus_timer = BONUS_TIMER_START;
            wave_runtime.bonus_tick = 0.0;
            wave_runtime.elapsed_wave_time = 0.0;
            wave_runtime.bonus_items = [BonusItemStatus::Pending; 3];

            respawn_player_for_attempt(
                &stage,
                &game_materials,
                &mut player_query,
                &mut dk_query,
            );
            spawn_hammers(&mut commands, &game_meshes, &game_materials, &stage);
        }
    }

    run_data.next_entry = PlayingEntry::NewRun;
}

fn respawn_player_for_attempt(
    stage: &StageData,
    game_materials: &GameMaterials,
    player_query: &mut Query<
        (
            &mut Transform,
            &mut PlayerState,
            &mut Visibility,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        With<Player>,
    >,
    dk_query: &mut Query<&mut DkState, With<DonkeyKong>>,
) {
    if let Ok((mut transform, mut player_state, mut visibility, mut material)) =
        player_query.single_mut()
    {
        transform.translation = stage.player_spawn.extend(7.0);
        *player_state = PlayerState::default();
        *visibility = Visibility::Visible;
        material.0 = game_materials.player_normal.clone();
    }

    if let Ok(mut dk_state) = dk_query.single_mut() {
        *dk_state = DkState {
            anim: DkAnimState::Idle,
            timer: 0.0,
            throw_timer: 0.0,
            barrels_thrown: 0,
        };
    }
}

fn respawn_active_bonus_items(
    commands: &mut Commands,
    wave_runtime: &WaveRuntime,
    game_meshes: &GameMeshes,
    game_materials: &GameMaterials,
    stage: &StageData,
) {
    for (index, status) in wave_runtime.bonus_items.iter().enumerate() {
        if *status == BonusItemStatus::Active {
            spawn_bonus_item(commands, game_meshes, game_materials, stage, index);
        }
    }
}

fn enter_dying(mut player_query: Query<&mut PlayerState, With<Player>>) {
    if let Ok(mut player_state) = player_query.single_mut() {
        player_state.locomotion = Locomotion::Dying;
    }
}
