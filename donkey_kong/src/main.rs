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

use constants::*;
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
        .insert_resource(DeathSequence {
            elapsed: 0.0,
            cause: DeathCause::Fall,
        })
        .insert_resource(level::StageData::new())
        .add_plugins((
            level::LevelPlugin,
            player::PlayerPlugin,
            hazards::HazardsPlugin,
            combat::CombatPlugin,
            ui::UiPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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

    commands.insert_resource(GameMeshes {
        player: meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
        barrel: meshes.add(Circle::new(BARREL_RADIUS)),
        fireball: meshes.add(Circle::new(FIREBALL_RADIUS)),
        hammer_pickup: meshes.add(Rectangle::new(HAMMER_PICKUP_SIZE, HAMMER_PICKUP_SIZE)),
        bonus_item: meshes.add(Rectangle::new(BONUS_ITEM_SIZE, BONUS_ITEM_SIZE)),
    });

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
