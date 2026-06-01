use bevy::prelude::*;

use crate::components::{BrickColor, PowerupKind};

/// Central registry of preloaded asset handles, grouped by category so call sites
/// read like `assets.sfx.wall_bounce`. Handles are cheap to clone and loading is
/// shared, so cloning from here avoids redundant `asset_server.load(...)` calls.
///
/// Grows per phase — add a field (or a nested category struct, e.g.
/// `sprites.bricks.red`) when a phase introduces new assets.
#[derive(Resource)]
pub struct GameAssets {
    pub sprites: Sprites,
    pub vfx: Vfx,
    pub sfx: Sfx,
    pub music: Music,
}

pub struct Sprites {
    pub vaus: Handle<Image>,
    pub vaus_expanded: Handle<Image>,
    pub vaus_life_icon: Handle<Image>,
    pub ball: Handle<Image>,
    pub border_frame: Handle<Image>,
    pub title_screen: Handle<Image>,
    pub bricks: Bricks,
    pub capsules: Capsules,
    pub laser_bolt: Handle<Image>,
    pub warp_gate: Handle<Image>,
}

/// The seven lettered power-up capsule sprites, indexable by [`PowerupKind`].
pub struct Capsules {
    pub catch: Handle<Image>,
    pub laser: Handle<Image>,
    pub expand: Handle<Image>,
    pub disruption: Handle<Image>,
    pub slow: Handle<Image>,
    pub brk: Handle<Image>,
    pub player: Handle<Image>,
}

impl Capsules {
    /// Returns the sprite handle for a given power-up kind.
    pub fn handle(&self, kind: PowerupKind) -> Handle<Image> {
        match kind {
            PowerupKind::Catch => self.catch.clone(),
            PowerupKind::Laser => self.laser.clone(),
            PowerupKind::Expand => self.expand.clone(),
            PowerupKind::Disruption => self.disruption.clone(),
            PowerupKind::Slow => self.slow.clone(),
            PowerupKind::Break => self.brk.clone(),
            PowerupKind::Player => self.player.clone(),
        }
    }
}

/// Animated VFX flipbook frames (see `vfx.rs`).
pub struct Vfx {
    pub capsule_catch: [Handle<Image>; 5],
    pub laser_impact: [Handle<Image>; 4],
}

/// The colored brick sprites (indexable by [`BrickColor`]) plus the silver damage frames
/// and the gold (indestructible) brick.
pub struct Bricks {
    pub white: Handle<Image>,
    pub orange: Handle<Image>,
    pub cyan: Handle<Image>,
    pub green: Handle<Image>,
    pub red: Handle<Image>,
    pub blue: Handle<Image>,
    pub pink: Handle<Image>,
    pub yellow: Handle<Image>,
    /// Silver brick frames ordered pristine → most cracked (damage state 0..=3).
    pub silver: [Handle<Image>; 4],
    pub gold: Handle<Image>,
}

impl Bricks {
    /// Returns the sprite handle for a given brick color.
    pub fn handle(&self, color: BrickColor) -> Handle<Image> {
        match color {
            BrickColor::White => self.white.clone(),
            BrickColor::Orange => self.orange.clone(),
            BrickColor::Cyan => self.cyan.clone(),
            BrickColor::Green => self.green.clone(),
            BrickColor::Red => self.red.clone(),
            BrickColor::Blue => self.blue.clone(),
            BrickColor::Pink => self.pink.clone(),
            BrickColor::Yellow => self.yellow.clone(),
        }
    }

    /// Silver frame for a given damage amount (hits taken), clamped to the last frame.
    pub fn silver_frame(&self, damage: u32) -> Handle<Image> {
        let index = (damage as usize).min(self.silver.len() - 1);
        self.silver[index].clone()
    }
}

pub struct Sfx {
    pub wall_bounce: Handle<AudioSource>,
    pub paddle_bounce: Handle<AudioSource>,
    pub brick_break: Handle<AudioSource>,
    pub ball_lost: Handle<AudioSource>,
    /// Clink when the ball strikes a hard brick (silver that survives, or gold).
    pub hard_brick: Handle<AudioSource>,
    /// Cue played when the ball ramps up to a higher speed.
    pub ball_speedup: Handle<AudioSource>,
    // Power-up cues (Phase 5).
    pub capsule_catch: Handle<AudioSource>,
    pub laser_fire: Handle<AudioSource>,
    pub expand: Handle<AudioSource>,
    pub multiball: Handle<AudioSource>,
    pub slow: Handle<AudioSource>,
    pub extra_life: Handle<AudioSource>,
    pub warp_gate_open: Handle<AudioSource>,
}

pub struct Music {
    pub round_clear: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        Self {
            sprites: Sprites {
                vaus: server.load("sprites/vaus/vaus.png"),
                vaus_expanded: server.load("sprites/vaus/vaus-expanded.png"),
                vaus_life_icon: server.load("sprites/vaus/vaus-life-icon.png"),
                ball: server.load("sprites/ball/ball.png"),
                border_frame: server.load("sprites/playfield/border-frame.png"),
                title_screen: server.load("ui/title-screen.png"),
                laser_bolt: server.load("sprites/weapons/laser-bolt.png"),
                warp_gate: server.load("sprites/playfield/warp-gate.png"),
                capsules: Capsules {
                    catch: server.load("sprites/capsules/capsule-c-catch.png"),
                    laser: server.load("sprites/capsules/capsule-l-laser.png"),
                    expand: server.load("sprites/capsules/capsule-e-expand.png"),
                    disruption: server.load("sprites/capsules/capsule-d-disruption.png"),
                    slow: server.load("sprites/capsules/capsule-s-slow.png"),
                    brk: server.load("sprites/capsules/capsule-b-break.png"),
                    player: server.load("sprites/capsules/capsule-p-player.png"),
                },
                bricks: Bricks {
                    white: server.load("sprites/bricks/brick-white.png"),
                    orange: server.load("sprites/bricks/brick-orange.png"),
                    cyan: server.load("sprites/bricks/brick-cyan.png"),
                    green: server.load("sprites/bricks/brick-green.png"),
                    red: server.load("sprites/bricks/brick-red.png"),
                    blue: server.load("sprites/bricks/brick-blue.png"),
                    pink: server.load("sprites/bricks/brick-pink.png"),
                    yellow: server.load("sprites/bricks/brick-yellow.png"),
                    silver: [
                        server.load("sprites/bricks/brick-silver-frame-01.png"),
                        server.load("sprites/bricks/brick-silver-frame-02.png"),
                        server.load("sprites/bricks/brick-silver-frame-03.png"),
                        server.load("sprites/bricks/brick-silver-frame-04.png"),
                    ],
                    gold: server.load("sprites/bricks/brick-gold.png"),
                },
            },
            vfx: Vfx {
                capsule_catch: [
                    server.load("vfx/capsule-catch-flash-frame-01.png"),
                    server.load("vfx/capsule-catch-flash-frame-02.png"),
                    server.load("vfx/capsule-catch-flash-frame-03.png"),
                    server.load("vfx/capsule-catch-flash-frame-04.png"),
                    server.load("vfx/capsule-catch-flash-frame-05.png"),
                ],
                laser_impact: [
                    server.load("vfx/laser-impact-frame-01.png"),
                    server.load("vfx/laser-impact-frame-02.png"),
                    server.load("vfx/laser-impact-frame-03.png"),
                    server.load("vfx/laser-impact-frame-04.png"),
                ],
            },
            sfx: Sfx {
                wall_bounce: server.load("audio/sfx/wall-bounce.ogg"),
                paddle_bounce: server.load("audio/sfx/paddle-bounce.ogg"),
                brick_break: server.load("audio/sfx/brick-break.ogg"),
                ball_lost: server.load("audio/sfx/ball-lost.ogg"),
                hard_brick: server.load("audio/sfx/hard-brick-clink.ogg"),
                ball_speedup: server.load("audio/sfx/ball-speedup.ogg"),
                capsule_catch: server.load("audio/sfx/capsule-catch.ogg"),
                laser_fire: server.load("audio/sfx/laser-fire.ogg"),
                expand: server.load("audio/sfx/expand.ogg"),
                multiball: server.load("audio/sfx/multiball.ogg"),
                slow: server.load("audio/sfx/slow.ogg"),
                extra_life: server.load("audio/sfx/extra-life.ogg"),
                warp_gate_open: server.load("audio/sfx/warp-gate-open.ogg"),
            },
            music: Music {
                round_clear: server.load("audio/music/round-clear.ogg"),
                game_over: server.load("audio/music/game-over.ogg"),
            },
        }
    }
}
