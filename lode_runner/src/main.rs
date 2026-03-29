mod components;
mod constants;
mod grid;
mod states;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use components::*;
use constants::*;
use grid::*;
use states::AppState;

// --- System Sets ---

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet {
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
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .init_state::<AppState>()
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Simulate.after(GameSet::Input),
                GameSet::Resolve.after(GameSet::Simulate),
                GameSet::Presentation.after(GameSet::Resolve),
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Startup, (setup_camera, setup_render_assets))
        // Start screen
        .add_systems(OnEnter(AppState::StartScreen), spawn_start_screen)
        .add_systems(
            Update,
            start_screen_input.run_if(in_state(AppState::StartScreen)),
        )
        // Playing
        .add_systems(OnEnter(AppState::Playing), spawn_level)
        .add_systems(Update, player_input.in_set(GameSet::Input))
        .add_systems(
            Update,
            (advance_movement, tick_holes, apply_gravity)
                .chain()
                .in_set(GameSet::Simulate),
        )
        .add_systems(
            Update,
            (collect_gold, check_exit).chain().in_set(GameSet::Resolve),
        )
        .add_systems(Update, sync_transforms.in_set(GameSet::Presentation))
        // Level complete
        .add_systems(OnEnter(AppState::LevelComplete), spawn_level_complete_screen)
        .add_systems(
            Update,
            level_complete_input.run_if(in_state(AppState::LevelComplete)),
        )
        // Game over (all levels beaten)
        .add_systems(OnEnter(AppState::GameOver), spawn_game_over_screen)
        .add_systems(
            Update,
            game_over_input.run_if(in_state(AppState::GameOver)),
        )
        .run();
}

// --- Camera ---

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// --- Render Assets ---

#[derive(Resource)]
struct RenderAssets {
    tile_mesh: Handle<Mesh>,
    bar_mesh: Handle<Mesh>,
    brick_material: Handle<ColorMaterial>,
    concrete_material: Handle<ColorMaterial>,
    ladder_material: Handle<ColorMaterial>,
    bar_material: Handle<ColorMaterial>,
    hidden_ladder_material: Handle<ColorMaterial>,
    gold_mesh: Handle<Mesh>,
    gold_material: Handle<ColorMaterial>,
    player_mesh: Handle<Mesh>,
    player_material: Handle<ColorMaterial>,
    guard_mesh: Handle<Mesh>,
    guard_material: Handle<ColorMaterial>,
    hole_mesh: Handle<Mesh>,
    hole_material: Handle<ColorMaterial>,
}

fn setup_render_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tile_size = CELL_SIZE - 2.0;
    commands.insert_resource(RenderAssets {
        tile_mesh: meshes.add(Rectangle::new(tile_size, tile_size)),
        bar_mesh: meshes.add(Rectangle::new(tile_size, 4.0)),
        brick_material: materials.add(ColorMaterial::from_color(COLOR_BRICK)),
        concrete_material: materials.add(ColorMaterial::from_color(COLOR_CONCRETE)),
        ladder_material: materials.add(ColorMaterial::from_color(COLOR_LADDER)),
        bar_material: materials.add(ColorMaterial::from_color(COLOR_BAR)),
        hidden_ladder_material: materials.add(ColorMaterial::from_color(COLOR_HIDDEN_LADDER)),
        gold_mesh: meshes.add(Circle::new(CELL_SIZE * 0.25)),
        gold_material: materials.add(ColorMaterial::from_color(COLOR_GOLD)),
        player_mesh: meshes.add(Rectangle::new(CELL_SIZE * 0.7, CELL_SIZE * 0.9)),
        player_material: materials.add(ColorMaterial::from_color(COLOR_PLAYER)),
        guard_mesh: meshes.add(Rectangle::new(CELL_SIZE * 0.7, CELL_SIZE * 0.9)),
        guard_material: materials.add(ColorMaterial::from_color(COLOR_GUARD)),
        hole_mesh: meshes.add(Rectangle::new(tile_size, tile_size)),
        hole_material: materials.add(ColorMaterial::from_color(COLOR_BACKGROUND)),
    });
}

// --- Game State ---

#[derive(Resource)]
struct GameState {
    score: u32,
    lives: u32,
    current_level: usize,
    total_gold: u32,
    exit_unlocked: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: STARTING_LIVES,
            current_level: 0,
            total_gold: 0,
            exit_unlocked: false,
        }
    }
}

// --- Level Catalog ---

const LEVELS: &[&str] = &[LEVEL_1, LEVEL_2];

// 28 columns x 16 rows
//                             0         1         2
//                             0123456789012345678901234567
const LEVEL_1: &str = "\
............................
..^....................^....
..^..$...............$..^...
..H.##########..########.H..
..H......................H..
..H..$...............$...H..
..H.##########..#########H..
..H..........--..........H..
..H......................H..
..H.##########..#########H..
..H..$...............$...H..
..H.##########..#########H..
..H..P................G..H..
..H.######################..
..H......................H..
============================";

const LEVEL_2: &str = "\
............................
..^........................^
..^..$..................$..^
..H.########.H..########.H.
..H..........H...........H.
..H..$.......H........$..H.
..H.####.H.###..#########H.
..H......H...............H.
..H......H...............H.
..H.####.H.###..#########H.
..H..$.......H........$..H.
..H.####.H.###..#########H.
..H..P.......H........G..H.
..H.######################..
..H......................H..
============================";

// --- Start Screen ---

fn spawn_start_screen(mut commands: Commands) {
    commands.insert_resource(GameState::default());
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            DespawnOnExit(AppState::StartScreen),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("LODE RUNNER"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(COLOR_GOLD),
            ));
            parent.spawn((
                Text::new("Press SPACE to play"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn start_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);
    }
}

// --- Level Spawning ---

fn spawn_level(
    mut commands: Commands,
    render_assets: Res<RenderAssets>,
    mut game_state: ResMut<GameState>,
) {
    let level_idx = game_state.current_level;
    let level_source = LEVELS[level_idx];
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
                Tile::Empty => continue,
                Tile::HiddenLadder => continue, // invisible until revealed
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

    // Hidden ladder tile entities (spawned invisible, revealed later)
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
        SpawnPoint(sp),
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
            SpawnPoint(gp),
            MovementState::Idle,
            Mesh2d(render_assets.guard_mesh.clone()),
            MeshMaterial2d(render_assets.guard_material.clone()),
            Transform::from_xyz(pos.x, pos.y, 3.0),
            DespawnOnExit(AppState::Playing),
        ));
    }

    commands.insert_resource(parsed.grid);
    commands.insert_resource(HoleMap::default());
}

// --- Input ---

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    grid: Res<LevelGrid>,
    hole_map: Res<HoleMap>,
    render_assets: Res<RenderAssets>,
    mut commands: Commands,
    mut query: Query<(&GridPosition, &mut MovementState), With<Player>>,
) {
    let Ok((grid_pos, mut movement)) = query.single_mut() else {
        return;
    };

    if !matches!(*movement, MovementState::Idle) {
        return;
    }

    let pos = grid_pos.0;

    let dig_left = keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::KeyZ);
    let dig_right = keyboard.just_pressed(KeyCode::KeyE) || keyboard.just_pressed(KeyCode::KeyX);

    if dig_left && grid.can_dig(pos, -1, &hole_map) {
        start_dig(
            &mut commands,
            &mut movement,
            &grid,
            &render_assets,
            pos,
            HorizontalDir::Left,
        );
        return;
    }
    if dig_right && grid.can_dig(pos, 1, &hole_map) {
        start_dig(
            &mut commands,
            &mut movement,
            &grid,
            &render_assets,
            pos,
            HorizontalDir::Right,
        );
        return;
    }

    let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);
    let up = keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp);
    let down = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);

    if up && grid.can_climb_up(pos.x, pos.y, &hole_map) {
        *movement = MovementState::Climbing {
            from: pos,
            to: pos + IVec2::Y,
            progress: 0.0,
        };
    } else if down && grid.can_climb_down(pos.x, pos.y, &hole_map) {
        let target = pos - IVec2::Y;
        let here = grid.effective_tile(pos.x, pos.y, &hole_map);
        if here == Tile::Ladder
            || grid.effective_tile(target.x, target.y, &hole_map) == Tile::Ladder
        {
            *movement = MovementState::Climbing {
                from: pos,
                to: target,
                progress: 0.0,
            };
        } else {
            *movement = MovementState::Falling {
                from: pos,
                to: target,
                progress: 0.0,
            };
        }
    } else if left && grid.can_enter(pos.x - 1, pos.y, &hole_map) {
        *movement = MovementState::Moving {
            from: pos,
            to: pos - IVec2::X,
            progress: 0.0,
        };
    } else if right && grid.can_enter(pos.x + 1, pos.y, &hole_map) {
        *movement = MovementState::Moving {
            from: pos,
            to: pos + IVec2::X,
            progress: 0.0,
        };
    }
}

fn start_dig(
    commands: &mut Commands,
    movement: &mut MovementState,
    grid: &LevelGrid,
    render_assets: &RenderAssets,
    pos: IVec2,
    side: HorizontalDir,
) {
    let dx = match side {
        HorizontalDir::Left => -1,
        HorizontalDir::Right => 1,
    };
    let target = IVec2::new(pos.x + dx, pos.y - 1);

    *movement = MovementState::Digging {
        side,
        timer: Timer::from_seconds(DIG_DURATION, TimerMode::Once),
    };

    let world_pos = grid_to_world(target, grid.width, grid.height, CELL_SIZE);
    commands.spawn((
        Hole {
            cell: target,
            phase: HolePhase::Open,
            timer: Timer::from_seconds(HOLE_OPEN_DURATION, TimerMode::Once),
        },
        Mesh2d(render_assets.hole_mesh.clone()),
        MeshMaterial2d(render_assets.hole_material.clone()),
        Transform::from_xyz(world_pos.x, world_pos.y, 1.5),
        DespawnOnExit(AppState::Playing),
    ));
}

// --- Movement Simulation ---

fn advance_movement(
    time: Res<Time>,
    mut query: Query<(&mut GridPosition, &mut MovementState)>,
    mut hole_map: ResMut<HoleMap>,
) {
    let dt = time.delta_secs();

    for (mut grid_pos, mut movement) in &mut query {
        match &mut *movement {
            MovementState::Idle => continue,
            MovementState::Digging { timer, side } => {
                timer.tick(time.delta());
                if timer.is_finished() {
                    let dx = match side {
                        HorizontalDir::Left => -1,
                        HorizontalDir::Right => 1,
                    };
                    let target = IVec2::new(grid_pos.0.x + dx, grid_pos.0.y - 1);
                    hole_map.insert(target.x, target.y, HolePhase::Open);
                    *movement = MovementState::Idle;
                }
                continue;
            }
            _ => {}
        }

        let speed = match *movement {
            MovementState::Moving { .. } => PLAYER_MOVE_SPEED,
            MovementState::Climbing { .. } => PLAYER_CLIMB_SPEED,
            MovementState::Falling { .. } => PLAYER_FALL_SPEED,
            _ => continue,
        };

        let (_from, to, progress) = match &mut *movement {
            MovementState::Moving {
                from,
                to,
                progress,
            }
            | MovementState::Climbing {
                from,
                to,
                progress,
            }
            | MovementState::Falling {
                from,
                to,
                progress,
            } => (*from, *to, progress),
            _ => unreachable!(),
        };

        *progress += speed * dt;

        if *progress >= 1.0 {
            grid_pos.0 = to;
            *movement = MovementState::Idle;
        }
    }
}

// --- Hole Lifecycle ---

fn tick_holes(
    mut commands: Commands,
    time: Res<Time>,
    mut hole_map: ResMut<HoleMap>,
    mut query: Query<(Entity, &mut Hole)>,
) {
    for (entity, mut hole) in &mut query {
        hole.timer.tick(time.delta());
        if !hole.timer.is_finished() {
            continue;
        }

        match hole.phase {
            HolePhase::Open => {
                hole.phase = HolePhase::Closing;
                hole.timer = Timer::from_seconds(HOLE_CLOSE_DURATION, TimerMode::Once);
                hole_map.insert(hole.cell.x, hole.cell.y, HolePhase::Closing);
            }
            HolePhase::Closing => {
                hole_map.remove(hole.cell.x, hole.cell.y);
                commands.entity(entity).despawn();
            }
        }
    }
}

// --- Gravity ---

fn apply_gravity(
    grid: Res<LevelGrid>,
    hole_map: Res<HoleMap>,
    mut query: Query<(&GridPosition, &mut MovementState)>,
) {
    for (grid_pos, mut movement) in &mut query {
        if !matches!(*movement, MovementState::Idle) {
            continue;
        }

        let pos = grid_pos.0;
        if !grid.is_supported(pos.x, pos.y, &hole_map) {
            *movement = MovementState::Falling {
                from: pos,
                to: pos - IVec2::Y,
                progress: 0.0,
            };
        }
    }
}

// --- Gold Collection ---

fn collect_gold(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut grid: ResMut<LevelGrid>,
    player_query: Query<&GridPosition, With<Player>>,
    gold_query: Query<(Entity, &GridPosition), With<Gold>>,
    mut hidden_query: Query<&mut Visibility, With<HiddenLadderTile>>,
) {
    let Ok(player_pos) = player_query.single() else {
        return;
    };

    for (entity, gold_pos) in &gold_query {
        if gold_pos.0 == player_pos.0 {
            commands.entity(entity).despawn();
            game_state.score += GOLD_SCORE;
            game_state.total_gold = game_state.total_gold.saturating_sub(1);

            // All gold collected — reveal hidden ladders
            if game_state.total_gold == 0 && !game_state.exit_unlocked {
                game_state.exit_unlocked = true;
                grid.reveal_hidden_ladders();
                for mut vis in &mut hidden_query {
                    *vis = Visibility::Visible;
                }
            }
        }
    }
}

// --- Exit Check ---

fn check_exit(
    game_state: Res<GameState>,
    player_query: Query<&GridPosition, With<Player>>,
    grid: Res<LevelGrid>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !game_state.exit_unlocked {
        return;
    }

    let Ok(player_pos) = player_query.single() else {
        return;
    };

    // Exit = reaching the top row (y == height - 1)
    if player_pos.0.y >= grid.height as i32 - 1 {
        next_state.set(AppState::LevelComplete);
    }
}

// --- Level Complete Screen ---

fn spawn_level_complete_screen(mut commands: Commands, mut game_state: ResMut<GameState>) {
    game_state.score += LEVEL_COMPLETE_SCORE;

    let has_next = game_state.current_level + 1 < LEVELS.len();
    let message = if has_next {
        "Press SPACE for next level"
    } else {
        "Press SPACE to continue"
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            DespawnOnExit(AppState::LevelComplete),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("LEVEL COMPLETE!"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(COLOR_GOLD),
            ));
            parent.spawn((
                Text::new(format!("Score: {}", game_state.score)),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(message),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn level_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        game_state.current_level += 1;
        if game_state.current_level < LEVELS.len() {
            next_state.set(AppState::Playing);
        } else {
            next_state.set(AppState::GameOver);
        }
    }
}

// --- Game Over (Victory) Screen ---

fn spawn_game_over_screen(mut commands: Commands, game_state: Res<GameState>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            DespawnOnExit(AppState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("YOU WIN!"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(COLOR_GOLD),
            ));
            parent.spawn((
                Text::new(format!("Final Score: {}", game_state.score)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press SPACE to play again"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::StartScreen);
    }
}

// --- Presentation ---

fn sync_transforms(
    grid: Res<LevelGrid>,
    mut query: Query<(&GridPosition, &MovementState, &mut Transform)>,
) {
    for (grid_pos, movement, mut transform) in &mut query {
        let world_pos = match movement {
            MovementState::Idle | MovementState::Digging { .. } => {
                grid_to_world(grid_pos.0, grid.width, grid.height, CELL_SIZE)
            }
            MovementState::Moving {
                from, to, progress, ..
            }
            | MovementState::Climbing {
                from, to, progress, ..
            }
            | MovementState::Falling {
                from, to, progress, ..
            } => {
                let a = grid_to_world(*from, grid.width, grid.height, CELL_SIZE);
                let b = grid_to_world(*to, grid.width, grid.height, CELL_SIZE);
                a.lerp(b, progress.clamp(0.0, 1.0))
            }
        };

        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}
