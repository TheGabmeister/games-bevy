mod constants;
mod states;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use constants::*;
use states::AppState;

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
        .add_systems(Startup, (setup_camera, setup_render_assets))
        .add_systems(OnEnter(AppState::StartScreen), spawn_start_screen)
        .add_systems(
            Update,
            start_screen_input.run_if(in_state(AppState::StartScreen)),
        )
        .add_systems(OnEnter(AppState::Playing), spawn_level)
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
    gold_mesh: Handle<Mesh>,
    gold_material: Handle<ColorMaterial>,
    player_mesh: Handle<Mesh>,
    player_material: Handle<ColorMaterial>,
    guard_mesh: Handle<Mesh>,
    guard_material: Handle<ColorMaterial>,
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
        gold_mesh: meshes.add(Circle::new(CELL_SIZE * 0.25)),
        gold_material: materials.add(ColorMaterial::from_color(COLOR_GOLD)),
        player_mesh: meshes.add(Rectangle::new(CELL_SIZE * 0.7, CELL_SIZE * 0.9)),
        player_material: materials.add(ColorMaterial::from_color(COLOR_PLAYER)),
        guard_mesh: meshes.add(Rectangle::new(CELL_SIZE * 0.7, CELL_SIZE * 0.9)),
        guard_material: materials.add(ColorMaterial::from_color(COLOR_GUARD)),
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

// --- Tile and Level Data ---

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tile {
    Empty,
    Brick,
    Concrete,
    Ladder,
    Bar,
    HiddenLadder,
}

#[derive(Resource)]
struct LevelGrid {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl LevelGrid {
    fn get(&self, x: usize, y: usize) -> Tile {
        self.tiles[y * self.width + x]
    }
}

struct ParsedLevel {
    grid: LevelGrid,
    player_spawn: IVec2,
    guard_spawns: Vec<IVec2>,
    gold_positions: Vec<IVec2>,
}

fn parse_level(source: &str) -> ParsedLevel {
    let lines: Vec<&str> = source.lines().collect();
    let height = lines.len();
    let width = lines[0].len();

    for (i, line) in lines.iter().enumerate() {
        assert_eq!(
            line.len(),
            width,
            "Row {i} has width {}, expected {width}",
            line.len()
        );
    }

    let mut tiles = vec![Tile::Empty; width * height];
    let mut player_spawn = IVec2::ZERO;
    let mut guard_spawns = Vec::new();
    let mut gold_positions = Vec::new();

    for (row_from_top, line) in lines.iter().enumerate() {
        let y = height - 1 - row_from_top;
        for (x, ch) in line.chars().enumerate() {
            let tile = match ch {
                '.' => Tile::Empty,
                '#' => Tile::Brick,
                '=' => Tile::Concrete,
                'H' => Tile::Ladder,
                '-' => Tile::Bar,
                '^' => Tile::HiddenLadder,
                '$' => {
                    gold_positions.push(IVec2::new(x as i32, y as i32));
                    Tile::Empty
                }
                'S' => {
                    gold_positions.push(IVec2::new(x as i32, y as i32));
                    Tile::Ladder
                }
                'P' => {
                    player_spawn = IVec2::new(x as i32, y as i32);
                    Tile::Empty
                }
                'G' => {
                    guard_spawns.push(IVec2::new(x as i32, y as i32));
                    Tile::Empty
                }
                _ => Tile::Empty,
            };
            tiles[y * width + x] = tile;
        }
    }

    ParsedLevel {
        grid: LevelGrid {
            width,
            height,
            tiles,
        },
        player_spawn,
        guard_spawns,
        gold_positions,
    }
}

fn grid_to_world(pos: IVec2, width: usize, height: usize, cell_size: f32) -> Vec2 {
    let offset_x = -(width as f32 * cell_size) * 0.5 + cell_size * 0.5;
    let offset_y = -(height as f32 * cell_size) * 0.5 + cell_size * 0.5;
    Vec2::new(
        offset_x + pos.x as f32 * cell_size,
        offset_y + pos.y as f32 * cell_size,
    )
}

// --- Hardcoded Level ---

//                             0123456789012345678901234567
const LEVEL_1: &str = "\
............................
............................
...H..$..............$..H...
...H.########..########.H...
...H....................H...
...H..$..............$..H...
...H.########..########.H...
...H..........--........H...
...H....................H...
...H.########..########.H...
...H..$..............$..H...
...H.########..########.H...
...H..P..............G..H...
...H.########..########.H...
...H....................H...
============================";

// --- Level Rendering ---

fn spawn_level(mut commands: Commands, render_assets: Res<RenderAssets>) {
    let parsed = parse_level(LEVEL_1);
    let width = parsed.grid.width;
    let height = parsed.grid.height;

    // Spawn tile entities
    for y in 0..height {
        for x in 0..width {
            let tile = parsed.grid.get(x, y);
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

    // Gold markers
    for &gp in &parsed.gold_positions {
        let pos = grid_to_world(gp, width, height, CELL_SIZE);
        commands.spawn((
            Mesh2d(render_assets.gold_mesh.clone()),
            MeshMaterial2d(render_assets.gold_material.clone()),
            Transform::from_xyz(pos.x, pos.y, 2.0),
            DespawnOnExit(AppState::Playing),
        ));
    }

    // Player marker
    let pos = grid_to_world(parsed.player_spawn, width, height, CELL_SIZE);
    commands.spawn((
        Mesh2d(render_assets.player_mesh.clone()),
        MeshMaterial2d(render_assets.player_material.clone()),
        Transform::from_xyz(pos.x, pos.y, 3.0),
        DespawnOnExit(AppState::Playing),
    ));

    // Guard markers
    for &gp in &parsed.guard_spawns {
        let pos = grid_to_world(gp, width, height, CELL_SIZE);
        commands.spawn((
            Mesh2d(render_assets.guard_mesh.clone()),
            MeshMaterial2d(render_assets.guard_material.clone()),
            Transform::from_xyz(pos.x, pos.y, 3.0),
            DespawnOnExit(AppState::Playing),
        ));
    }

    commands.insert_resource(parsed.grid);
}
