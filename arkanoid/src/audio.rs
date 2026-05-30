use bevy::prelude::*;

/// Requests a one-shot bounce sound. Emitted by collision, played by [`AudioPlugin`].
#[derive(Message, Clone, Copy)]
pub enum BounceSound {
    Wall,
    Paddle,
}

/// Cached handles for the bounce SFX so they aren't reloaded on every hit.
#[derive(Resource)]
struct BounceSfx {
    wall: Handle<AudioSource>,
    paddle: Handle<AudioSource>,
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BounceSound>()
            .add_systems(Startup, load_sfx)
            .add_systems(Update, play_bounce_sounds);
    }
}

fn load_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BounceSfx {
        wall: asset_server.load("audio/sfx/wall-bounce.ogg"),
        paddle: asset_server.load("audio/sfx/paddle-bounce.ogg"),
    });
}

fn play_bounce_sounds(
    mut commands: Commands,
    mut bounce: MessageReader<BounceSound>,
    sfx: Res<BounceSfx>,
) {
    for event in bounce.read() {
        let source = match event {
            BounceSound::Wall => sfx.wall.clone(),
            BounceSound::Paddle => sfx.paddle.clone(),
        };
        commands.spawn((AudioPlayer(source), PlaybackSettings::DESPAWN));
    }
}
