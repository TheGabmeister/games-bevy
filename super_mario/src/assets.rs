use bevy::prelude::*;

use crate::constants::*;

/// Shared mesh and material handles for all entity rendering.
/// Initialized once on startup so every level and gameplay system
/// can clone cheap handles instead of recreating assets.
#[derive(Resource)]
pub struct GameAssets {
    // Tiles
    pub tile_mesh: Handle<Mesh>,
    pub pipe_top_mesh: Handle<Mesh>,
    pub ground_mat: Handle<ColorMaterial>,
    pub brick_mat: Handle<ColorMaterial>,
    pub question_mat: Handle<ColorMaterial>,
    pub empty_block_mat: Handle<ColorMaterial>,
    pub solid_mat: Handle<ColorMaterial>,
    pub pipe_mat: Handle<ColorMaterial>,

    // Player
    pub player_small_mesh: Handle<Mesh>,
    pub player_big_mesh: Handle<Mesh>,
    pub player_normal_mat: Handle<ColorMaterial>,
    pub player_fire_mat: Handle<ColorMaterial>,

    // Goomba
    pub goomba_body_mesh: Handle<Mesh>,
    pub goomba_feet_mesh: Handle<Mesh>,
    pub goomba_body_mat: Handle<ColorMaterial>,
    pub goomba_feet_mat: Handle<ColorMaterial>,

    // Koopa
    pub koopa_body_mesh: Handle<Mesh>,
    pub koopa_body_mat: Handle<ColorMaterial>,
    pub koopa_head_mesh: Handle<Mesh>,
    pub koopa_head_mat: Handle<ColorMaterial>,

    // Shell
    pub shell_mesh: Handle<Mesh>,
    pub shell_mat: Handle<ColorMaterial>,

    // Floating coin
    pub floating_coin_mesh: Handle<Mesh>,
    pub floating_coin_mat: Handle<ColorMaterial>,

    // Flagpole
    pub pole_mesh: Handle<Mesh>,
    pub pole_mat: Handle<ColorMaterial>,
    pub flag_mesh: Handle<Mesh>,
    pub flag_mat: Handle<ColorMaterial>,
    pub pole_ball_mesh: Handle<Mesh>,
    pub pole_ball_mat: Handle<ColorMaterial>,

    // Castle
    pub castle_body_mesh: Handle<Mesh>,
    pub castle_body_mat: Handle<ColorMaterial>,
    pub castle_roof_mesh: Handle<Mesh>,
    pub castle_roof_mat: Handle<ColorMaterial>,
    pub castle_door_mesh: Handle<Mesh>,
    pub castle_door_mat: Handle<ColorMaterial>,

    // Mushroom
    pub mushroom_cap_mesh: Handle<Mesh>,
    pub mushroom_cap_mat: Handle<ColorMaterial>,
    pub mushroom_stem_mesh: Handle<Mesh>,
    pub mushroom_stem_mat: Handle<ColorMaterial>,

    // Fire Flower
    pub fire_flower_mesh: Handle<Mesh>,
    pub fire_flower_mat: Handle<ColorMaterial>,
    pub fire_flower_stem_mesh: Handle<Mesh>,
    pub fire_flower_stem_mat: Handle<ColorMaterial>,

    // Coin pop
    pub coin_pop_mesh: Handle<Mesh>,
    pub coin_pop_mat: Handle<ColorMaterial>,

    // Brick particle
    pub brick_particle_mesh: Handle<Mesh>,
    pub brick_particle_mat: Handle<ColorMaterial>,

    // Fireball
    pub fireball_mesh: Handle<Mesh>,
    pub fireball_mat: Handle<ColorMaterial>,
}

pub fn init_game_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(GameAssets {
        // Tiles
        tile_mesh: meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE)),
        pipe_top_mesh: meshes.add(Rectangle::new(TILE_SIZE + PIPE_LIP_OVERHANG, TILE_SIZE)),
        ground_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.55, 0.27, 0.07))),
        brick_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.72, 0.40, 0.10))),
        question_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.90, 0.75, 0.10))),
        empty_block_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.35, 0.25, 0.15))),
        solid_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.45, 0.30, 0.15))),
        pipe_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.0, 0.65, 0.15))),

        // Player
        player_small_mesh: meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_SMALL_HEIGHT)),
        player_big_mesh: meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_BIG_HEIGHT)),
        player_normal_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.8, 0.1, 0.1))),
        player_fire_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.95, 0.95, 0.95))),

        // Goomba
        goomba_body_mesh: meshes.add(Ellipse::new(6.0, 5.0)),
        goomba_feet_mesh: meshes.add(Rectangle::new(12.0, 4.0)),
        goomba_body_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.55, 0.30, 0.10))),
        goomba_feet_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.35, 0.18, 0.05))),

        // Koopa
        koopa_body_mesh: meshes.add(Rectangle::new(KOOPA_WIDTH, 16.0)),
        koopa_body_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.7, 0.2))),
        koopa_head_mesh: meshes.add(Circle::new(5.0)),
        koopa_head_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.3, 0.8, 0.3))),

        // Shell
        shell_mesh: meshes.add(Rectangle::new(SHELL_WIDTH, SHELL_HEIGHT)),
        shell_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.65, 0.2))),

        // Floating coin
        floating_coin_mesh: meshes.add(Circle::new(FLOATING_COIN_SIZE / 2.0)),
        floating_coin_mat: materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.85, 0.0))),

        // Flagpole
        pole_mesh: meshes.add(Rectangle::new(FLAGPOLE_POLE_WIDTH, TILE_SIZE)),
        pole_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.5, 0.5, 0.5))),
        flag_mesh: meshes.add(Rectangle::new(FLAGPOLE_FLAG_SIZE, FLAGPOLE_FLAG_SIZE)),
        flag_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.8, 0.2))),
        pole_ball_mesh: meshes.add(Circle::new(3.0)),
        pole_ball_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.9, 0.9, 0.0))),

        // Castle
        castle_body_mesh: meshes.add(Rectangle::new(48.0, 48.0)),
        castle_body_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.5, 0.35, 0.2))),
        castle_roof_mesh: meshes.add(RegularPolygon::new(20.0, 3)),
        castle_roof_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.6, 0.15, 0.15))),
        castle_door_mesh: meshes.add(Rectangle::new(12.0, 16.0)),
        castle_door_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.1, 0.1, 0.1))),

        // Mushroom
        mushroom_cap_mesh: meshes.add(Ellipse::new(7.0, 5.0)),
        mushroom_cap_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.8, 0.1, 0.1))),
        mushroom_stem_mesh: meshes.add(Rectangle::new(8.0, 6.0)),
        mushroom_stem_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.95, 0.85, 0.7))),

        // Fire Flower
        fire_flower_mesh: meshes.add(Circle::new(5.0)),
        fire_flower_mat: materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.4, 0.1))),
        fire_flower_stem_mesh: meshes.add(Rectangle::new(4.0, 8.0)),
        fire_flower_stem_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.7, 0.2))),

        // Coin pop
        coin_pop_mesh: meshes.add(Circle::new(4.0)),
        coin_pop_mat: materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.85, 0.0))),

        // Brick particle
        brick_particle_mesh: meshes.add(Rectangle::new(BRICK_PARTICLE_SIZE, BRICK_PARTICLE_SIZE)),
        brick_particle_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.72, 0.40, 0.10))),

        // Fireball
        fireball_mesh: meshes.add(Circle::new(FIREBALL_RADIUS)),
        fireball_mat: materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.5, 0.0))),
    });
}
