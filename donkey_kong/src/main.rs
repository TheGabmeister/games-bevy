#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod combat;
mod components;
mod constants;
mod hazards;
mod level;
mod player;
mod resources;
mod states;
mod ui;

use bevy::camera::ScalingMode;
use bevy::prelude::*;

use components::*;
use constants::*;
use level::*;
use resources::*;
use states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                resolution: bevy::window::WindowResolution::new(
                    WINDOW_WIDTH as u32,
                    WINDOW_HEIGHT as u32,
                ),
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BG_COLOR))
        .init_state::<AppState>()
        .init_resource::<SessionData>()
        .init_resource::<RunData>()
        .init_resource::<WaveRuntime>()
        .insert_resource(WaveConfig::from_wave(1))
        .insert_resource(DeathSequence { elapsed: 0.0, cause: DeathCause::Fall })
        .insert_resource(StageData::new())
        .add_plugins((
            player::PlayerPlugin,
            hazards::HazardsPlugin,
            combat::CombatPlugin,
            ui::UiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            OnEnter(AppState::Playing),
            (enter_playing_cleanup, ApplyDeferred, enter_playing_spawn, ui::spawn_hud).chain(),
        )
        .add_systems(OnEnter(AppState::Dying), enter_dying)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera with fixed scaling for logical playfield
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: PLAYFIELD_WIDTH,
                height: PLAYFIELD_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    // Pre-create shared mesh handles
    commands.insert_resource(GameMeshes {
        player: meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
        barrel: meshes.add(Circle::new(BARREL_RADIUS)),
        fireball: meshes.add(Circle::new(FIREBALL_RADIUS)),
        hammer_pickup: meshes.add(Rectangle::new(HAMMER_PICKUP_SIZE, HAMMER_PICKUP_SIZE)),
        bonus_item: meshes.add(Rectangle::new(BONUS_ITEM_SIZE, BONUS_ITEM_SIZE)),
    });

    // Pre-create shared material handles
    commands.insert_resource(GameMaterials {
        player_normal: materials.add(PLAYER_COLOR),
        player_hammer: materials.add(PLAYER_HAMMER_COLOR),
        barrel: materials.add(BARREL_COLOR),
        blue_barrel: materials.add(BLUE_BARREL_COLOR),
        fireball: materials.add(FIREBALL_COLOR),
        hammer_pickup: materials.add(HAMMER_PICKUP_COLOR),
        bonus_item: materials.add(BONUS_ITEM_COLOR),
    });
}

// --- Enter Playing State ---

fn enter_playing_cleanup(
    mut commands: Commands,
    run_data: Res<RunData>,
    stage_q: Query<Entity, With<StageEntity>>,
    barrel_q: Query<Entity, With<Barrel>>,
    fireball_q: Query<Entity, With<Fireball>>,
    hammer_q: Query<Entity, With<HammerPickup>>,
    bonus_q: Query<Entity, With<BonusItemEntity>>,
    hud_q: Query<Entity, With<GameHudUI>>,
    tally_q: Query<Entity, With<WaveTallyUI>>,
) {
    match run_data.next_entry {
        PlayingEntry::NewRun => {
            // Full cleanup
            for e in stage_q
                .iter()
                .chain(barrel_q.iter())
                .chain(fireball_q.iter())
                .chain(hammer_q.iter())
                .chain(bonus_q.iter())
                .chain(hud_q.iter())
                .chain(tally_q.iter())
            {
                commands.entity(e).despawn();
            }
        }
        PlayingEntry::RetryAfterDeath | PlayingEntry::NextWave => {
            // Partial cleanup — keep stage entities, player, DK, etc.
            for e in barrel_q
                .iter()
                .chain(fireball_q.iter())
                .chain(hammer_q.iter())
                .chain(bonus_q.iter())
                .chain(tally_q.iter())
            {
                commands.entity(e).despawn();
            }
        }
    }
}

fn enter_playing_spawn(
    mut commands: Commands,
    mut run_data: ResMut<RunData>,
    mut wave_rt: ResMut<WaveRuntime>,
    stage: Res<StageData>,
    game_meshes: Res<GameMeshes>,
    game_mats: Res<GameMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_q: Query<
        (
            &mut Transform,
            &mut PlayerState,
            &mut Visibility,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        With<Player>,
    >,
    mut dk_q: Query<&mut DkState>,
) {
    let entry = run_data.next_entry;

    match entry {
        PlayingEntry::NewRun => {
            *run_data = RunData::default();
            *wave_rt = WaveRuntime::default();
            commands.insert_resource(WaveConfig::from_wave(1));

            spawn_stage(&mut commands, &mut meshes, &mut materials, &stage);
            spawn_player_entity(&mut commands, &game_meshes, &game_mats, &stage);
            spawn_hammers(&mut commands, &game_meshes, &game_mats, &stage);
        }
        PlayingEntry::RetryAfterDeath => {
            if let Ok((mut tf, mut ps, mut vis, mut mat)) = player_q.single_mut() {
                tf.translation = stage.player_spawn.extend(7.0);
                *ps = PlayerState::default();
                *vis = Visibility::Visible;
                mat.0 = game_mats.player_normal.clone();
            }
            if let Ok(mut dk) = dk_q.single_mut() {
                dk.anim = DkAnimState::Idle;
                dk.timer = 0.0;
                dk.throw_timer = 0.0;
            }
            spawn_hammers(&mut commands, &game_meshes, &game_mats, &stage);
        }
        PlayingEntry::NextWave => {
            commands.insert_resource(WaveConfig::from_wave(run_data.wave));
            wave_rt.bonus_timer = BONUS_TIMER_START;
            wave_rt.bonus_tick = 0.0;
            wave_rt.elapsed_wave_time = 0.0;
            wave_rt.bonus_items = [BonusItemStatus::Pending; 3];

            if let Ok((mut tf, mut ps, mut vis, mut mat)) = player_q.single_mut() {
                tf.translation = stage.player_spawn.extend(7.0);
                *ps = PlayerState::default();
                *vis = Visibility::Visible;
                mat.0 = game_mats.player_normal.clone();
            }
            if let Ok(mut dk) = dk_q.single_mut() {
                *dk = DkState {
                    anim: DkAnimState::Idle,
                    timer: 0.0,
                    throw_timer: 0.0,
                    barrels_thrown: 0,
                };
            }
            spawn_hammers(&mut commands, &game_meshes, &game_mats, &stage);
        }
    }

    run_data.next_entry = PlayingEntry::NewRun;
}

// --- Enter Dying State ---

fn enter_dying(mut player_q: Query<&mut PlayerState, With<Player>>) {
    if let Ok(mut ps) = player_q.single_mut() {
        ps.locomotion = Locomotion::Dying;
    }
}
