mod components;
mod constants;
mod gameplay;
mod grid;
mod guard;
mod levels;
mod movement;
mod player;
mod render;
mod resources;
mod states;
mod ui;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use components::*;
use constants::*;
use grid::*;
use render::RenderAssets;
use resources::*;
use states::{AppState, PlayState};

// --- System Sets ---

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Simulate,
    Resolve,
    Presentation,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lode Runner".into(),
                resolution: WindowResolution::new(
                    WINDOW_WIDTH as u32,
                    (WINDOW_HEIGHT + HUD_HEIGHT) as u32,
                ),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_sub_state::<PlayState>()
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Simulate.after(GameSet::Input),
                GameSet::Resolve.after(GameSet::Simulate),
                GameSet::Presentation.after(GameSet::Resolve),
            )
                .run_if(in_state(PlayState::Running)),
        )
        // Startup
        .add_systems(Startup, (setup_camera, render::setup_render_assets))
        // Start screen
        .add_systems(OnEnter(AppState::StartScreen), ui::spawn_start_screen)
        .add_systems(
            Update,
            ui::start_screen_input.run_if(in_state(AppState::StartScreen)),
        )
        // Playing
        .add_systems(OnEnter(AppState::Playing), (spawn_level, ui::spawn_hud))
        .add_systems(OnExit(AppState::Playing), cleanup_play_resources)
        .add_systems(Update, player::player_input.in_set(GameSet::Input))
        .add_systems(
            Update,
            (
                movement::advance_movement,
                movement::tick_holes,
                guard::guard_ai,
                movement::apply_gravity,
            )
                .chain()
                .in_set(GameSet::Simulate),
        )
        .add_systems(
            Update,
            (
                gameplay::collect_gold,
                gameplay::check_guard_trap,
                gameplay::check_player_death,
                gameplay::check_exit,
            )
                .chain()
                .in_set(GameSet::Resolve),
        )
        .add_systems(
            Update,
            (movement::sync_transforms, ui::update_hud).in_set(GameSet::Presentation),
        )
        // Pause
        .add_systems(Update, ui::pause_input.run_if(in_state(AppState::Playing)))
        .add_systems(OnEnter(PlayState::Paused), ui::spawn_pause_overlay)
        .add_systems(OnExit(PlayState::Paused), ui::despawn_pause_overlay)
        // Dying
        .add_systems(OnEnter(PlayState::Dying), gameplay::start_death_sequence)
        .add_systems(
            Update,
            gameplay::tick_death.run_if(in_state(PlayState::Dying)),
        )
        // Restarting (transient — immediately re-enters Playing)
        .add_systems(OnEnter(AppState::Restarting), gameplay::restart_level)
        // Level complete
        .add_systems(
            OnEnter(AppState::LevelComplete),
            ui::spawn_level_complete_screen,
        )
        .add_systems(
            Update,
            ui::level_complete_input.run_if(in_state(AppState::LevelComplete)),
        )
        // Game over
        .add_systems(OnEnter(AppState::GameOver), ui::spawn_game_over_screen)
        .add_systems(
            Update,
            ui::game_over_input.run_if(in_state(AppState::GameOver)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(COLOR_BACKGROUND),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.15,
            ..Bloom::NATURAL
        },
    ));
}

fn spawn_level(
    mut commands: Commands,
    render_assets: Res<RenderAssets>,
    mut game_state: ResMut<GameState>,
) {
    let level_idx = game_state.current_level;
    let level_source = levels::LEVELS[level_idx];
    let parsed = parse_level(level_source);
    let width = parsed.grid.width;
    let height = parsed.grid.height;

    game_state.total_gold = parsed.gold_positions.len() as u32;
    game_state.exit_unlocked = false;

    // Tiles
    for y in 0..height {
        for x in 0..width {
            let tile = parsed.grid.get(x as i32, y as i32);
            let (mesh, material) = match tile {
                Tile::Brick => (&render_assets.tile_mesh, &render_assets.brick_material),
                Tile::Concrete => (&render_assets.tile_mesh, &render_assets.concrete_material),
                Tile::Ladder => (&render_assets.tile_mesh, &render_assets.ladder_material),
                Tile::Bar => (&render_assets.bar_mesh, &render_assets.bar_material),
                Tile::Empty | Tile::HiddenLadder => continue,
            };

            let pos = grid_to_world(IVec2::new(x as i32, y as i32), width, height, CELL_SIZE);
            commands.spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(pos.x, pos.y, 1.0),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }

    // Hidden ladders
    for &hlp in &parsed.hidden_ladder_positions {
        let pos = grid_to_world(hlp, width, height, CELL_SIZE);
        commands.spawn((
            HiddenLadderTile,
            Mesh2d(render_assets.tile_mesh.clone()),
            MeshMaterial2d(render_assets.hidden_ladder_material.clone()),
            Transform::from_xyz(pos.x, pos.y, 1.0),
            Visibility::Hidden,
            DespawnOnExit(AppState::Playing),
        ));
    }

    // Gold
    for &gp in &parsed.gold_positions {
        let pos = grid_to_world(gp, width, height, CELL_SIZE);
        commands.spawn((
            Gold,
            GridPosition(gp),
            Mesh2d(render_assets.gold_mesh.clone()),
            MeshMaterial2d(render_assets.gold_material.clone()),
            Transform::from_xyz(pos.x, pos.y, 2.0),
            DespawnOnExit(AppState::Playing),
        ));
    }

    // Player
    let sp = parsed.player_spawn;
    let pos = grid_to_world(sp, width, height, CELL_SIZE);
    commands.spawn((
        Player,
        GridPosition(sp),
        MovementState::Idle,
        Mesh2d(render_assets.player_mesh.clone()),
        MeshMaterial2d(render_assets.player_material.clone()),
        Transform::from_xyz(pos.x, pos.y, 3.0),
        DespawnOnExit(AppState::Playing),
    ));

    // Guards
    for &gp in &parsed.guard_spawns {
        let pos = grid_to_world(gp, width, height, CELL_SIZE);
        commands.spawn((
            Guard,
            GridPosition(gp),
            MovementState::Idle,
            AiTimer(Timer::from_seconds(GUARD_AI_INTERVAL, TimerMode::Repeating)),
            Mesh2d(render_assets.guard_mesh.clone()),
            MeshMaterial2d(render_assets.guard_material.clone()),
            Transform::from_xyz(pos.x, pos.y, 3.0),
            DespawnOnExit(AppState::Playing),
        ));
    }

    commands.insert_resource(parsed.grid);
    commands.insert_resource(HoleMap::default());
}

fn cleanup_play_resources(mut commands: Commands) {
    commands.remove_resource::<DeathTimer>();
    commands.remove_resource::<HoleMap>();
    commands.remove_resource::<LevelGrid>();
}
