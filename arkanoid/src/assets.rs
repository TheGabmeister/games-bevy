use bevy::prelude::*;

use crate::components::BrickColor;

/// Central registry of preloaded asset handles, grouped by category so call sites
/// read like `assets.sfx.wall_bounce`. Handles are cheap to clone and loading is
/// shared, so cloning from here avoids redundant `asset_server.load(...)` calls.
///
/// Grows per phase — add a field (or a nested category struct, e.g.
/// `sprites.bricks.red`) when a phase introduces new assets.
#[derive(Resource)]
pub struct GameAssets {
    pub sprites: Sprites,
    pub sfx: Sfx,
    pub music: Music,
}

pub struct Sprites {
    pub vaus: Handle<Image>,
    pub vaus_life_icon: Handle<Image>,
    pub ball: Handle<Image>,
    pub border_frame: Handle<Image>,
    pub title_screen: Handle<Image>,
    pub bricks: Bricks,
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
                vaus_life_icon: server.load("sprites/vaus/vaus-life-icon.png"),
                ball: server.load("sprites/ball/ball.png"),
                border_frame: server.load("sprites/playfield/border-frame.png"),
                title_screen: server.load("ui/title-screen.png"),
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
            sfx: Sfx {
                wall_bounce: server.load("audio/sfx/wall-bounce.ogg"),
                paddle_bounce: server.load("audio/sfx/paddle-bounce.ogg"),
                brick_break: server.load("audio/sfx/brick-break.ogg"),
                ball_lost: server.load("audio/sfx/ball-lost.ogg"),
                hard_brick: server.load("audio/sfx/hard-brick-clink.ogg"),
                ball_speedup: server.load("audio/sfx/ball-speedup.ogg"),
            },
            music: Music {
                round_clear: server.load("audio/music/round-clear.ogg"),
                game_over: server.load("audio/music/game-over.ogg"),
            },
        }
    }
}
