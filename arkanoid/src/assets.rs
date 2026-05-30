use bevy::prelude::*;

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
}

pub struct Sprites {
    pub vaus: Handle<Image>,
    pub ball: Handle<Image>,
    pub border_frame: Handle<Image>,
}

pub struct Sfx {
    pub wall_bounce: Handle<AudioSource>,
    pub paddle_bounce: Handle<AudioSource>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        Self {
            sprites: Sprites {
                vaus: server.load("sprites/vaus/vaus.png"),
                ball: server.load("sprites/ball/ball.png"),
                border_frame: server.load("sprites/playfield/border-frame.png"),
            },
            sfx: Sfx {
                wall_bounce: server.load("audio/sfx/wall-bounce.ogg"),
                paddle_bounce: server.load("audio/sfx/paddle-bounce.ogg"),
            },
        }
    }
}
