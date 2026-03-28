use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::constants::*;

#[derive(Resource)]
pub struct GameState {
    pub score: u32,
    pub lives: u32,
    pub smart_bombs: u32,
    pub current_wave: u32,
    pub next_extra_life_score: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: STARTING_LIVES,
            smart_bombs: STARTING_SMART_BOMBS,
            current_wave: 0,
            next_extra_life_score: EXTRA_LIFE_INTERVAL,
        }
    }
}

#[derive(Resource)]
pub struct WaveState {
    pub landers_to_spawn: u32,
    pub bombers_to_spawn: u32,
    pub pods_to_spawn: u32,
    pub spawn_timer: Timer,
    pub baiter_timer: Timer,
    pub baiter_interval: Timer,
    pub baiters_active: bool,
    pub wave_active: bool,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            landers_to_spawn: 0,
            bombers_to_spawn: 0,
            pods_to_spawn: 0,
            spawn_timer: Timer::from_seconds(ENEMY_SPAWN_INTERVAL, TimerMode::Repeating),
            baiter_timer: Timer::from_seconds(BAITER_SPAWN_DELAY, TimerMode::Once),
            baiter_interval: Timer::from_seconds(BAITER_SPAWN_INTERVAL, TimerMode::Repeating),
            baiters_active: false,
            wave_active: false,
        }
    }
}

#[derive(Resource)]
pub struct TerrainData {
    pub points: Vec<Vec2>,
}

#[derive(Resource)]
pub struct CameraWorldPos(pub f32);

impl Default for CameraWorldPos {
    fn default() -> Self {
        Self(WORLD_WIDTH / 2.0)
    }
}

#[derive(Resource)]
pub struct WaveIntroTimer(pub Timer);

#[derive(Resource)]
pub struct DeathTimer(pub Timer);

#[derive(Resource)]
pub struct GameRng {
    seed: u64,
    rng: SmallRng,
}

impl Default for GameRng {
    fn default() -> Self {
        Self::from_seed(0xDEF3_4D3E_2026)
    }
}

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            seed,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn reset(&mut self) {
        self.rng = SmallRng::seed_from_u64(self.seed);
    }

    pub fn f32(&mut self) -> f32 {
        self.rng.random::<f32>()
    }

    pub fn sign(&mut self) -> f32 {
        if self.rng.random::<bool>() { 1.0 } else { -1.0 }
    }

    pub fn y_range(&mut self) -> f32 {
        GROUND_Y + 50.0 + self.f32() * (CEILING_Y - GROUND_Y - 100.0)
    }

    pub fn world_x(&mut self) -> f32 {
        self.f32() * WORLD_WIDTH
    }

    pub fn world_x_far_from(&mut self, player_x: f32) -> f32 {
        let min_dist = WORLD_WIDTH * 0.2;
        loop {
            let x = self.world_x();
            let dx = (x - player_x).abs().min(WORLD_WIDTH - (x - player_x).abs());
            if dx > min_dist {
                return x;
            }
        }
    }
}

#[derive(Resource)]
pub struct GameplayAssets {
    pub player_mesh: Handle<Mesh>,
    pub player_material: Handle<ColorMaterial>,
    pub human_mesh: Handle<Mesh>,
    pub human_material: Handle<ColorMaterial>,
    pub lander_mesh: Handle<Mesh>,
    pub lander_material: Handle<ColorMaterial>,
    pub mutant_mesh: Handle<Mesh>,
    pub mutant_material: Handle<ColorMaterial>,
    pub bomber_mesh: Handle<Mesh>,
    pub bomber_material: Handle<ColorMaterial>,
    pub pod_mesh: Handle<Mesh>,
    pub pod_material: Handle<ColorMaterial>,
    pub swarmer_mesh: Handle<Mesh>,
    pub swarmer_material: Handle<ColorMaterial>,
    pub baiter_mesh: Handle<Mesh>,
    pub baiter_material: Handle<ColorMaterial>,
    pub player_projectile_mesh: Handle<Mesh>,
    pub player_projectile_material: Handle<ColorMaterial>,
    pub enemy_projectile_mesh: Handle<Mesh>,
    pub enemy_projectile_material: Handle<ColorMaterial>,
    pub mine_mesh: Handle<Mesh>,
    pub mine_material: Handle<ColorMaterial>,
    pub explosion_mesh: Handle<Mesh>,
}

impl FromWorld for GameplayAssets {
    fn from_world(world: &mut World) -> Self {
        let (
            player_mesh,
            human_mesh,
            lander_mesh,
            mutant_mesh,
            bomber_mesh,
            pod_mesh,
            swarmer_mesh,
            baiter_mesh,
            player_projectile_mesh,
            enemy_projectile_mesh,
            mine_mesh,
            explosion_mesh,
        ) = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            (
                meshes.add(Triangle2d::new(
                    Vec2::new(20.0, 0.0),
                    Vec2::new(-10.0, -8.0),
                    Vec2::new(-10.0, 8.0),
                )),
                meshes.add(Rectangle::new(4.0, 10.0)),
                meshes.add(Rectangle::new(14.0, 14.0)),
                meshes.add(RegularPolygon::new(12.0, 5)),
                meshes.add(Rectangle::new(18.0, 10.0)),
                meshes.add(Circle::new(10.0)),
                meshes.add(Triangle2d::new(
                    Vec2::new(8.0, 0.0),
                    Vec2::new(-5.0, -4.0),
                    Vec2::new(-5.0, 4.0),
                )),
                meshes.add(RegularPolygon::new(14.0, 4)),
                meshes.add(Rectangle::new(14.0, 2.0)),
                meshes.add(Rectangle::new(8.0, 3.0)),
                meshes.add(Circle::new(4.0)),
                meshes.add(Circle::new(5.0)),
            )
        };

        let (
            player_material,
            human_material,
            lander_material,
            mutant_material,
            bomber_material,
            pod_material,
            swarmer_material,
            baiter_material,
            player_projectile_material,
            enemy_projectile_material,
            mine_material,
        ) = {
            let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
            (
                materials.add(ColorMaterial::from_color(COLOR_PLAYER)),
                materials.add(ColorMaterial::from_color(COLOR_HUMAN)),
                materials.add(ColorMaterial::from_color(COLOR_LANDER)),
                materials.add(ColorMaterial::from_color(COLOR_MUTANT)),
                materials.add(ColorMaterial::from_color(COLOR_BOMBER)),
                materials.add(ColorMaterial::from_color(COLOR_POD)),
                materials.add(ColorMaterial::from_color(COLOR_SWARMER)),
                materials.add(ColorMaterial::from_color(COLOR_BAITER)),
                materials.add(ColorMaterial::from_color(COLOR_PLAYER_PROJECTILE)),
                materials.add(ColorMaterial::from_color(COLOR_ENEMY_PROJECTILE)),
                materials.add(ColorMaterial::from_color(COLOR_MINE)),
            )
        };

        Self {
            player_mesh,
            player_material,
            human_mesh,
            human_material,
            lander_mesh,
            lander_material,
            mutant_mesh,
            mutant_material,
            bomber_mesh,
            bomber_material,
            pod_mesh,
            pod_material,
            swarmer_mesh,
            swarmer_material,
            baiter_mesh,
            baiter_material,
            player_projectile_mesh,
            player_projectile_material,
            enemy_projectile_mesh,
            enemy_projectile_material,
            mine_mesh,
            mine_material,
            explosion_mesh,
        }
    }
}
