use bevy::prelude::*;

use crate::assets::GameAssets;

/// Requests a one-shot bounce sound. Emitted by collision, played by [`AudioPlugin`].
#[derive(Message, Clone, Copy)]
pub enum BounceSound {
    Wall,
    Paddle,
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BounceSound>()
            .add_systems(Update, play_bounce_sounds);
    }
}

fn play_bounce_sounds(
    mut commands: Commands,
    mut bounce: MessageReader<BounceSound>,
    assets: Res<GameAssets>,
) {
    for event in bounce.read() {
        let source = match event {
            BounceSound::Wall => assets.sfx.wall_bounce.clone(),
            BounceSound::Paddle => assets.sfx.paddle_bounce.clone(),
        };
        commands.spawn((AudioPlayer(source), PlaybackSettings::DESPAWN));
    }
}
