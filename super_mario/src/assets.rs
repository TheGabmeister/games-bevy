use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::states::AppState;

// ── Sub-structs ──

pub struct TileAssets {
    pub mesh: Handle<Mesh>,
    pub pipe_top_mesh: Handle<Mesh>,
    pub ground_mat: Handle<ColorMaterial>,
    pub brick_mat: Handle<ColorMaterial>,
    pub question_mat: Handle<ColorMaterial>,
    pub empty_block_mat: Handle<ColorMaterial>,
    pub solid_mat: Handle<ColorMaterial>,
    pub pipe_mat: Handle<ColorMaterial>,
}

impl TileAssets {
    /// Spawn a tile entity. Returns the entity id.
    /// `tile_pos` should be `Some((col, row))` for hittable blocks (B, ?, M).
    pub fn spawn(
        &self,
        commands: &mut Commands,
        tile_type: TileType,
        wx: f32,
        wy: f32,
        tile_pos: Option<(i32, i32)>,
    ) -> Entity {
        let (mesh, mat, z) = match tile_type {
            TileType::Ground => (self.mesh.clone(), self.ground_mat.clone(), Z_TILE),
            TileType::Brick => (self.mesh.clone(), self.brick_mat.clone(), Z_TILE),
            TileType::QuestionBlock => (self.mesh.clone(), self.question_mat.clone(), Z_TILE),
            TileType::Solid => (self.mesh.clone(), self.solid_mat.clone(), Z_TILE),
            TileType::PipeTopLeft | TileType::PipeTopRight => {
                (self.pipe_top_mesh.clone(), self.pipe_mat.clone(), Z_PIPE)
            }
            TileType::PipeBodyLeft | TileType::PipeBodyRight => {
                (self.mesh.clone(), self.pipe_mat.clone(), Z_PIPE)
            }
            TileType::Empty => (self.mesh.clone(), self.empty_block_mat.clone(), Z_TILE),
        };

        let mut entity = commands.spawn((
            Tile,
            tile_type,
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(wx, wy, z),
            DespawnOnExit(AppState::Playing),
        ));

        if let Some((col, row)) = tile_pos {
            entity.insert(TilePos { col, row });
        }

        entity.id()
    }
}

pub struct PlayerAssets {
    pub small_mesh: Handle<Mesh>,
    pub big_mesh: Handle<Mesh>,
    pub normal_mat: Handle<ColorMaterial>,
    pub fire_mat: Handle<ColorMaterial>,
}

impl PlayerAssets {
    pub fn spawn(&self, commands: &mut Commands, x: f32, y: f32) -> Entity {
        commands
            .spawn((
                Player,
                PlayerSize::default(),
                CollisionSize {
                    width: PLAYER_WIDTH,
                    height: PLAYER_SMALL_HEIGHT,
                },
                Velocity::default(),
                FacingDirection::default(),
                Grounded::default(),
                Mesh2d(self.small_mesh.clone()),
                MeshMaterial2d(self.normal_mat.clone()),
                Transform::from_xyz(x, y, Z_PLAYER),
                DespawnOnExit(AppState::Playing),
            ))
            .id()
    }
}

pub struct GoombaAssets {
    pub body_mesh: Handle<Mesh>,
    pub body_mat: Handle<ColorMaterial>,
    pub feet_mesh: Handle<Mesh>,
    pub feet_mat: Handle<ColorMaterial>,
}

impl GoombaAssets {
    pub fn spawn(&self, commands: &mut Commands, wx: f32, wy: f32) -> Entity {
        commands
            .spawn((
                Goomba,
                EnemyWalker {
                    speed: GOOMBA_SPEED,
                    direction: -1.0,
                },
                CollisionSize {
                    width: GOOMBA_WIDTH,
                    height: GOOMBA_HEIGHT,
                },
                Velocity::default(),
                Grounded::default(),
                Mesh2d(self.body_mesh.clone()),
                MeshMaterial2d(self.body_mat.clone()),
                Transform::from_xyz(wx, wy, Z_ENEMY),
                DespawnOnExit(AppState::Playing),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh2d(self.feet_mesh.clone()),
                    MeshMaterial2d(self.feet_mat.clone()),
                    Transform::from_xyz(0.0, -5.0, 0.0),
                ));
            })
            .id()
    }
}

pub struct KoopaAssets {
    pub body_mesh: Handle<Mesh>,
    pub body_mat: Handle<ColorMaterial>,
    pub head_mesh: Handle<Mesh>,
    pub head_mat: Handle<ColorMaterial>,
}

impl KoopaAssets {
    pub fn spawn(&self, commands: &mut Commands, wx: f32, wy: f32) -> Entity {
        commands
            .spawn((
                KoopaTroopa,
                EnemyWalker {
                    speed: KOOPA_SPEED,
                    direction: -1.0,
                },
                CollisionSize {
                    width: KOOPA_WIDTH,
                    height: KOOPA_HEIGHT,
                },
                Velocity::default(),
                Grounded::default(),
                Mesh2d(self.body_mesh.clone()),
                MeshMaterial2d(self.body_mat.clone()),
                Transform::from_xyz(wx, wy, Z_ENEMY),
                DespawnOnExit(AppState::Playing),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh2d(self.head_mesh.clone()),
                    MeshMaterial2d(self.head_mat.clone()),
                    Transform::from_xyz(0.0, 11.0, 0.0),
                ));
            })
            .id()
    }
}

pub struct ShellAssets {
    pub mesh: Handle<Mesh>,
    pub mat: Handle<ColorMaterial>,
}

impl ShellAssets {
    pub fn spawn(&self, commands: &mut Commands, x: f32, y: f32) -> Entity {
        commands
            .spawn((
                Shell {
                    state: ShellState::Stationary,
                    chain_kills: 0,
                },
                EnemyWalker {
                    speed: 0.0,
                    direction: 1.0,
                },
                CollisionSize {
                    width: SHELL_WIDTH,
                    height: SHELL_HEIGHT,
                },
                Velocity::default(),
                Grounded(true),
                EnemyActive,
                Mesh2d(self.mesh.clone()),
                MeshMaterial2d(self.mat.clone()),
                Transform::from_xyz(x, y, Z_ENEMY),
                DespawnOnExit(AppState::Playing),
            ))
            .id()
    }
}

pub struct FloatingCoinAssets {
    pub mesh: Handle<Mesh>,
    pub mat: Handle<ColorMaterial>,
}

impl FloatingCoinAssets {
    pub fn spawn(&self, commands: &mut Commands, wx: f32, wy: f32) -> Entity {
        commands
            .spawn((
                FloatingCoin,
                Mesh2d(self.mesh.clone()),
                MeshMaterial2d(self.mat.clone()),
                Transform::from_xyz(wx, wy, Z_ITEM),
                DespawnOnExit(AppState::Playing),
            ))
            .id()
    }
}

pub struct FlagpoleAssets {
    pub pole_mesh: Handle<Mesh>,
    pub pole_mat: Handle<ColorMaterial>,
    pub flag_mesh: Handle<Mesh>,
    pub flag_mat: Handle<ColorMaterial>,
    pub ball_mesh: Handle<Mesh>,
    pub ball_mat: Handle<ColorMaterial>,
}

impl FlagpoleAssets {
    /// Spawn a single pole segment (tile).
    pub fn spawn_pole(&self, commands: &mut Commands, wx: f32, wy: f32) -> Entity {
        commands
            .spawn((
                Tile,
                Mesh2d(self.pole_mesh.clone()),
                MeshMaterial2d(self.pole_mat.clone()),
                Transform::from_xyz(wx, wy, Z_TILE),
                DespawnOnExit(AppState::Playing),
            ))
            .id()
    }

    /// Spawn the flag and ball at the top of the flagpole.
    pub fn spawn_top(&self, commands: &mut Commands, wx: f32, wy: f32) {
        commands.spawn((
            FlagpoleFlag,
            Mesh2d(self.flag_mesh.clone()),
            MeshMaterial2d(self.flag_mat.clone()),
            Transform::from_xyz(
                wx - FLAGPOLE_FLAG_SIZE / 2.0 - FLAGPOLE_POLE_WIDTH / 2.0,
                wy,
                Z_TILE + 0.1,
            ),
            DespawnOnExit(AppState::Playing),
        ));

        commands.spawn((
            Mesh2d(self.ball_mesh.clone()),
            MeshMaterial2d(self.ball_mat.clone()),
            Transform::from_xyz(wx, wy + TILE_SIZE / 2.0 + 3.0, Z_TILE + 0.1),
            DespawnOnExit(AppState::Playing),
        ));
    }
}

pub struct CastleAssets {
    pub body_mesh: Handle<Mesh>,
    pub body_mat: Handle<ColorMaterial>,
    pub roof_mesh: Handle<Mesh>,
    pub roof_mat: Handle<ColorMaterial>,
    pub door_mesh: Handle<Mesh>,
    pub door_mat: Handle<ColorMaterial>,
}

impl CastleAssets {
    /// Spawn the full castle (body + roof + door). Returns the body entity (has `Castle` marker).
    pub fn spawn(&self, commands: &mut Commands, castle_x: f32, ground_top_y: f32) -> Entity {
        let body = commands
            .spawn((
                Castle,
                Mesh2d(self.body_mesh.clone()),
                MeshMaterial2d(self.body_mat.clone()),
                Transform::from_xyz(castle_x, ground_top_y + 24.0, Z_DECORATION),
                DespawnOnExit(AppState::Playing),
            ))
            .id();

        commands.spawn((
            Mesh2d(self.roof_mesh.clone()),
            MeshMaterial2d(self.roof_mat.clone()),
            Transform::from_xyz(castle_x, ground_top_y + 58.0, Z_DECORATION),
            DespawnOnExit(AppState::Playing),
        ));

        commands.spawn((
            Mesh2d(self.door_mesh.clone()),
            MeshMaterial2d(self.door_mat.clone()),
            Transform::from_xyz(castle_x, ground_top_y + 8.0, Z_DECORATION + 0.1),
            DespawnOnExit(AppState::Playing),
        ));

        body
    }
}

pub struct MushroomAssets {
    pub cap_mesh: Handle<Mesh>,
    pub cap_mat: Handle<ColorMaterial>,
    pub stem_mesh: Handle<Mesh>,
    pub stem_mat: Handle<ColorMaterial>,
}

impl MushroomAssets {
    pub fn spawn(&self, commands: &mut Commands, block_pos: Vec3) -> Entity {
        commands
            .spawn((
                Mushroom,
                MushroomEmerging { remaining: TILE_SIZE },
                CollisionSize {
                    width: MUSHROOM_WIDTH,
                    height: MUSHROOM_HEIGHT,
                },
                Velocity::default(),
                Grounded::default(),
                Mesh2d(self.cap_mesh.clone()),
                MeshMaterial2d(self.cap_mat.clone()),
                Transform::from_xyz(block_pos.x, block_pos.y, Z_ITEM),
                DespawnOnExit(AppState::Playing),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh2d(self.stem_mesh.clone()),
                    MeshMaterial2d(self.stem_mat.clone()),
                    Transform::from_xyz(0.0, -5.0, 0.0),
                ));
            })
            .id()
    }
}

pub struct FireFlowerAssets {
    pub mesh: Handle<Mesh>,
    pub mat: Handle<ColorMaterial>,
    pub stem_mesh: Handle<Mesh>,
    pub stem_mat: Handle<ColorMaterial>,
}

impl FireFlowerAssets {
    pub fn spawn(&self, commands: &mut Commands, block_pos: Vec3) -> Entity {
        commands
            .spawn((
                FireFlower,
                FireFlowerEmerging { remaining: TILE_SIZE },
                CollisionSize {
                    width: MUSHROOM_WIDTH,
                    height: MUSHROOM_HEIGHT,
                },
                Mesh2d(self.mesh.clone()),
                MeshMaterial2d(self.mat.clone()),
                Transform::from_xyz(block_pos.x, block_pos.y, Z_ITEM),
                DespawnOnExit(AppState::Playing),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh2d(self.stem_mesh.clone()),
                    MeshMaterial2d(self.stem_mat.clone()),
                    Transform::from_xyz(0.0, -6.0, 0.0),
                ));
            })
            .id()
    }
}

pub struct CoinPopAssets {
    pub mesh: Handle<Mesh>,
    pub mat: Handle<ColorMaterial>,
}

impl CoinPopAssets {
    pub fn spawn(&self, commands: &mut Commands, block_pos: Vec3) -> Entity {
        commands
            .spawn((
                CoinPop {
                    vel_y: COIN_POP_IMPULSE,
                    timer: Timer::from_seconds(COIN_POP_DURATION, TimerMode::Once),
                },
                Mesh2d(self.mesh.clone()),
                MeshMaterial2d(self.mat.clone()),
                Transform::from_xyz(block_pos.x, block_pos.y + TILE_SIZE, Z_ITEM),
                DespawnOnExit(AppState::Playing),
            ))
            .id()
    }
}

pub struct BrickParticleAssets {
    pub mesh: Handle<Mesh>,
    pub mat: Handle<ColorMaterial>,
}

impl BrickParticleAssets {
    pub fn spawn(&self, commands: &mut Commands, pos: Vec3, vx: f32, vy: f32) -> Entity {
        commands
            .spawn((
                BrickParticle {
                    vel_x: vx,
                    vel_y: vy,
                    timer: Timer::from_seconds(BRICK_PARTICLE_DURATION, TimerMode::Once),
                },
                Mesh2d(self.mesh.clone()),
                MeshMaterial2d(self.mat.clone()),
                Transform::from_xyz(pos.x, pos.y, Z_ITEM),
                DespawnOnExit(AppState::Playing),
            ))
            .id()
    }
}

pub struct FireballAssets {
    pub mesh: Handle<Mesh>,
    pub mat: Handle<ColorMaterial>,
}

impl FireballAssets {
    pub fn spawn(&self, commands: &mut Commands, x: f32, y: f32, direction: f32) -> Entity {
        commands
            .spawn((
                Fireball { direction },
                Velocity {
                    x: FIREBALL_SPEED * direction,
                    y: 0.0,
                },
                CollisionSize {
                    width: FIREBALL_RADIUS * 2.0,
                    height: FIREBALL_RADIUS * 2.0,
                },
                Mesh2d(self.mesh.clone()),
                MeshMaterial2d(self.mat.clone()),
                Transform::from_xyz(x, y, Z_ITEM),
                DespawnOnExit(AppState::Playing),
            ))
            .id()
    }
}

// ── Main Resource ──

/// Shared mesh and material handles for all entity rendering.
/// Initialized once on startup so every level and gameplay system
/// can clone cheap handles instead of recreating assets.
#[derive(Resource)]
pub struct GameAssets {
    pub tile: TileAssets,
    pub player: PlayerAssets,
    pub goomba: GoombaAssets,
    pub koopa: KoopaAssets,
    pub shell: ShellAssets,
    pub floating_coin: FloatingCoinAssets,
    pub flagpole: FlagpoleAssets,
    pub castle: CastleAssets,
    pub mushroom: MushroomAssets,
    pub fire_flower: FireFlowerAssets,
    pub coin_pop: CoinPopAssets,
    pub brick_particle: BrickParticleAssets,
    pub fireball: FireballAssets,
}

pub fn init_game_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(GameAssets {
        tile: TileAssets {
            mesh: meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE)),
            pipe_top_mesh: meshes.add(Rectangle::new(TILE_SIZE + PIPE_LIP_OVERHANG, TILE_SIZE)),
            ground_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.55, 0.27, 0.07))),
            brick_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.72, 0.40, 0.10))),
            question_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.90, 0.75, 0.10))),
            empty_block_mat: materials
                .add(ColorMaterial::from_color(Color::srgb(0.35, 0.25, 0.15))),
            solid_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.45, 0.30, 0.15))),
            pipe_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.0, 0.65, 0.15))),
        },

        player: PlayerAssets {
            small_mesh: meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_SMALL_HEIGHT)),
            big_mesh: meshes.add(Rectangle::new(PLAYER_WIDTH, PLAYER_BIG_HEIGHT)),
            normal_mat: materials
                .add(ColorMaterial::from_color(Color::srgb(0.8, 0.1, 0.1))),
            fire_mat: materials
                .add(ColorMaterial::from_color(Color::srgb(0.95, 0.95, 0.95))),
        },

        goomba: GoombaAssets {
            body_mesh: meshes.add(Ellipse::new(6.0, 5.0)),
            body_mat: materials
                .add(ColorMaterial::from_color(Color::srgb(0.55, 0.30, 0.10))),
            feet_mesh: meshes.add(Rectangle::new(12.0, 4.0)),
            feet_mat: materials
                .add(ColorMaterial::from_color(Color::srgb(0.35, 0.18, 0.05))),
        },

        koopa: KoopaAssets {
            body_mesh: meshes.add(Rectangle::new(KOOPA_WIDTH, 16.0)),
            body_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.7, 0.2))),
            head_mesh: meshes.add(Circle::new(5.0)),
            head_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.3, 0.8, 0.3))),
        },

        shell: ShellAssets {
            mesh: meshes.add(Rectangle::new(SHELL_WIDTH, SHELL_HEIGHT)),
            mat: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.65, 0.2))),
        },

        floating_coin: FloatingCoinAssets {
            mesh: meshes.add(Circle::new(FLOATING_COIN_SIZE / 2.0)),
            mat: materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.85, 0.0))),
        },

        flagpole: FlagpoleAssets {
            pole_mesh: meshes.add(Rectangle::new(FLAGPOLE_POLE_WIDTH, TILE_SIZE)),
            pole_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.5, 0.5, 0.5))),
            flag_mesh: meshes.add(Rectangle::new(FLAGPOLE_FLAG_SIZE, FLAGPOLE_FLAG_SIZE)),
            flag_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.8, 0.2))),
            ball_mesh: meshes.add(Circle::new(3.0)),
            ball_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.9, 0.9, 0.0))),
        },

        castle: CastleAssets {
            body_mesh: meshes.add(Rectangle::new(48.0, 48.0)),
            body_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.5, 0.35, 0.2))),
            roof_mesh: meshes.add(RegularPolygon::new(20.0, 3)),
            roof_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.6, 0.15, 0.15))),
            door_mesh: meshes.add(Rectangle::new(12.0, 16.0)),
            door_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.1, 0.1, 0.1))),
        },

        mushroom: MushroomAssets {
            cap_mesh: meshes.add(Ellipse::new(7.0, 5.0)),
            cap_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.8, 0.1, 0.1))),
            stem_mesh: meshes.add(Rectangle::new(8.0, 6.0)),
            stem_mat: materials
                .add(ColorMaterial::from_color(Color::srgb(0.95, 0.85, 0.7))),
        },

        fire_flower: FireFlowerAssets {
            mesh: meshes.add(Circle::new(5.0)),
            mat: materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.4, 0.1))),
            stem_mesh: meshes.add(Rectangle::new(4.0, 8.0)),
            stem_mat: materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.7, 0.2))),
        },

        coin_pop: CoinPopAssets {
            mesh: meshes.add(Circle::new(4.0)),
            mat: materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.85, 0.0))),
        },

        brick_particle: BrickParticleAssets {
            mesh: meshes.add(Rectangle::new(BRICK_PARTICLE_SIZE, BRICK_PARTICLE_SIZE)),
            mat: materials.add(ColorMaterial::from_color(Color::srgb(0.72, 0.40, 0.10))),
        },

        fireball: FireballAssets {
            mesh: meshes.add(Circle::new(FIREBALL_RADIUS)),
            mat: materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.5, 0.0))),
        },
    });
}
