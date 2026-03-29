use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;

use crate::constants::*;

// --- Game Assets ---

#[derive(Resource)]
pub struct GameAssets {
    pub player_mesh: Handle<Mesh>,
    pub player_material: Handle<ColorMaterial>,
    pub grunt_mesh: Handle<Mesh>,
    pub grunt_material: Handle<ColorMaterial>,
    pub hulk_mesh: Handle<Mesh>,
    pub hulk_material: Handle<ColorMaterial>,
    pub brain_mesh: Handle<Mesh>,
    pub brain_material: Handle<ColorMaterial>,
    pub prog_mesh: Handle<Mesh>,
    pub prog_material: Handle<ColorMaterial>,
    pub spheroid_mesh: Handle<Mesh>,
    pub spheroid_material: Handle<ColorMaterial>,
    pub enforcer_mesh: Handle<Mesh>,
    pub enforcer_material: Handle<ColorMaterial>,
    pub quark_mesh: Handle<Mesh>,
    pub quark_material: Handle<ColorMaterial>,
    pub tank_mesh: Handle<Mesh>,
    pub tank_material: Handle<ColorMaterial>,
    pub human_mesh: Handle<Mesh>,
    pub human_materials: [Handle<ColorMaterial>; 3],
    pub electrode_mesh: Handle<Mesh>,
    pub electrode_material: Handle<ColorMaterial>,
    pub bullet_mesh: Handle<Mesh>,
    pub bullet_material: Handle<ColorMaterial>,
    pub missile_mesh: Handle<Mesh>,
    pub missile_material: Handle<ColorMaterial>,
    pub spark_mesh: Handle<Mesh>,
    pub spark_material: Handle<ColorMaterial>,
    pub shell_mesh: Handle<Mesh>,
    pub shell_material: Handle<ColorMaterial>,
    pub particle_mesh: Handle<Mesh>,
    pub particle_material_explosion: Handle<ColorMaterial>,
    pub particle_material_rescue: Handle<ColorMaterial>,
    pub particle_material_death: Handle<ColorMaterial>,
    pub particle_material_electrode: Handle<ColorMaterial>,
    pub border_mesh_h: Handle<Mesh>,
    pub border_mesh_v: Handle<Mesh>,
    pub border_material: Handle<ColorMaterial>,
}

// --- Game State ---

#[derive(Resource)]
pub struct GameState {
    pub score: u32,
    pub high_score: u32,
    pub lives: u32,
    pub current_wave: u32,
    pub rescue_count_this_wave: u32,
    pub next_extra_life_score: u32,
}

impl GameState {
    pub fn award_score(&mut self, points: u32) {
        self.score += points;
        if self.score >= self.next_extra_life_score {
            self.lives += 1;
            self.next_extra_life_score += EXTRA_LIFE_EVERY;
        }
    }

    pub fn reset(&mut self) {
        self.score = 0;
        self.lives = STARTING_LIVES;
        self.current_wave = 1;
        self.rescue_count_this_wave = 0;
        self.next_extra_life_score = EXTRA_LIFE_EVERY;
    }
}

// --- Wave State ---

#[derive(Resource)]
pub struct WaveState {
    pub intro_timer: Timer,
    pub clear_timer: Timer,
    pub death_timer: Timer,
}

// --- Screen Shake ---

#[derive(Resource, Default)]
pub struct ScreenShake {
    pub trauma: f32,
}

// --- Game Over Timer ---

#[derive(Resource)]
pub struct GameOverTimer(pub Timer);

// --- High Score Persistence ---

pub fn high_score_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        return dir.join(HIGH_SCORE_FILE);
    }
    PathBuf::from(HIGH_SCORE_FILE)
}

pub fn load_high_score() -> u32 {
    fs::read_to_string(high_score_path())
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

pub fn save_high_score(score: u32) {
    let _ = fs::write(high_score_path(), score.to_string());
}

// --- Asset Setup ---

pub fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player_mesh = meshes.add(RegularPolygon::new(PLAYER_RADIUS, 4));
    let grunt_mesh = meshes.add(RegularPolygon::new(GRUNT_RADIUS, 4));
    let hulk_mesh = meshes.add(RegularPolygon::new(HULK_RADIUS, 4));
    let brain_mesh = meshes.add(Circle::new(BRAIN_RADIUS));
    let prog_mesh = meshes.add(Capsule2d::new(PROG_RADIUS / 2.0, PROG_RADIUS));
    let spheroid_mesh = meshes.add(Circle::new(SPHEROID_RADIUS));
    let enforcer_mesh = meshes.add(RegularPolygon::new(ENFORCER_RADIUS, 3));
    let quark_mesh = meshes.add(RegularPolygon::new(QUARK_RADIUS, 6));
    let tank_mesh = meshes.add(RegularPolygon::new(TANK_RADIUS, 5));
    let human_mesh = meshes.add(Capsule2d::new(HUMAN_RADIUS / 2.0, HUMAN_RADIUS));
    let electrode_mesh = meshes.add(Circle::new(ELECTRODE_RADIUS));
    let bullet_mesh = meshes.add(Circle::new(BULLET_RADIUS));
    let missile_mesh = meshes.add(Circle::new(MISSILE_RADIUS));
    let spark_mesh = meshes.add(Circle::new(SPARK_RADIUS));
    let shell_mesh = meshes.add(Circle::new(SHELL_RADIUS));
    let particle_mesh = meshes.add(Circle::new(PARTICLE_RADIUS));
    let border_mesh_h = meshes.add(Capsule2d::new(
        ARENA_BORDER_THICKNESS / 2.0,
        ARENA_HALF_WIDTH * 2.0,
    ));
    let border_mesh_v = meshes.add(Capsule2d::new(
        ARENA_BORDER_THICKNESS / 2.0,
        ARENA_HALF_HEIGHT * 2.0,
    ));

    // HDR colors (values > 1.0 for bloom glow)
    let player_material = materials.add(ColorMaterial::from_color(Color::srgb(0.2, 1.0, 5.0)));
    let grunt_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 0.2, 0.2)));
    let hulk_material = materials.add(ColorMaterial::from_color(Color::srgb(0.2, 5.0, 0.2)));
    let brain_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 0.5, 5.0)));
    let prog_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 0.2, 3.0)));
    let spheroid_material =
        materials.add(ColorMaterial::from_color(Color::srgb(3.0, 3.0, 5.0)));
    let enforcer_material =
        materials.add(ColorMaterial::from_color(Color::srgb(5.0, 2.0, 0.2)));
    let quark_material = materials.add(ColorMaterial::from_color(Color::srgb(2.0, 5.0, 3.0)));
    let tank_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 3.0, 1.0)));
    let human_materials = [
        materials.add(ColorMaterial::from_color(Color::srgb(5.0, 1.0, 2.0))),
        materials.add(ColorMaterial::from_color(Color::srgb(5.0, 4.0, 0.5))),
        materials.add(ColorMaterial::from_color(Color::srgb(1.0, 2.0, 5.0))),
    ];
    let electrode_material =
        materials.add(ColorMaterial::from_color(Color::srgb(4.0, 4.0, 5.0)));
    let bullet_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 5.0, 0.5)));
    let missile_material =
        materials.add(ColorMaterial::from_color(Color::srgb(5.0, 0.5, 1.0)));
    let spark_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 3.0, 0.2)));
    let shell_material = materials.add(ColorMaterial::from_color(Color::srgb(5.0, 4.0, 2.0)));
    let border_material = materials.add(ColorMaterial::from_color(Color::srgb(0.5, 0.5, 5.0)));

    // Particle materials (colored per effect type)
    let particle_material_explosion =
        materials.add(ColorMaterial::from_color(Color::srgb(5.0, 1.0, 0.2)));
    let particle_material_rescue =
        materials.add(ColorMaterial::from_color(Color::srgb(1.0, 2.0, 5.0)));
    let particle_material_death =
        materials.add(ColorMaterial::from_color(Color::srgb(0.2, 1.0, 5.0)));
    let particle_material_electrode =
        materials.add(ColorMaterial::from_color(Color::srgb(4.0, 4.0, 5.0)));

    commands.insert_resource(GameAssets {
        player_mesh,
        player_material,
        grunt_mesh,
        grunt_material,
        hulk_mesh,
        hulk_material,
        brain_mesh,
        brain_material,
        prog_mesh,
        prog_material,
        spheroid_mesh,
        spheroid_material,
        enforcer_mesh,
        enforcer_material,
        quark_mesh,
        quark_material,
        tank_mesh,
        tank_material,
        human_mesh,
        human_materials,
        electrode_mesh,
        electrode_material,
        bullet_mesh,
        bullet_material,
        missile_mesh,
        missile_material,
        spark_mesh,
        spark_material,
        shell_mesh,
        shell_material,
        particle_mesh,
        particle_material_explosion,
        particle_material_rescue,
        particle_material_death,
        particle_material_electrode,
        border_mesh_h,
        border_mesh_v,
        border_material,
    });
}
