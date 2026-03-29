mod arena;
mod combat;
mod components;
mod constants;
mod effects;
mod enemy;
mod human;
mod player;
mod resources;
mod states;
mod ui;
mod waves;

use bevy::{
    camera::ScalingMode,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    window::WindowResolution,
};

use constants::*;
use resources::*;
use states::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                title: "Robotron 2084".to_string(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_sub_state::<PlayState>()
        .insert_resource(GameState {
            score: 0,
            high_score: load_high_score(),
            lives: STARTING_LIVES,
            current_wave: 1,
            rescue_count_this_wave: 0,
            next_extra_life_score: EXTRA_LIFE_EVERY,
        })
        .init_resource::<ScreenShake>()
        // System set ordering: Input -> Movement -> Confinement -> Combat -> Resolution
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Movement,
                GameSet::Confinement,
                GameSet::Combat,
                GameSet::Resolution,
            )
                .chain()
                .run_if(in_state(PlayState::WaveActive)),
        )
        .add_systems(Startup, (setup_camera, setup_assets))
        .add_plugins((
            arena::ArenaPlugin,
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            human::HumanPlugin,
            combat::CombatPlugin,
            waves::WavePlugin,
            effects::EffectsPlugin,
            ui::UiPlugin,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: WINDOW_HEIGHT as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        DebandDither::Enabled,
    ));
}
