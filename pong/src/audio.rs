use bevy::prelude::*;

use crate::{components::PaddleHitEvent, state::Phase};

pub struct PongAudioPlugin;

impl Plugin for PongAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicPlayback>()
            .add_systems(Startup, load_audio_assets)
            .add_systems(OnEnter(Phase::Playing), start_music_once)
            .add_observer(play_paddle_hit_sfx);
    }
}

#[derive(Resource)]
pub struct AudioAssets {
    pub music: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
}

#[derive(Resource, Default)]
struct MusicPlayback {
    started: bool,
}

fn load_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AudioAssets {
        music: asset_server.load("music_spaceshooter.ogg"),
        hit: asset_server.load("sfx_hit.ogg"),
    });
}

fn start_music_once(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    mut playback: ResMut<MusicPlayback>,
) {
    if playback.started {
        return;
    }

    commands.spawn((
        AudioPlayer::new(audio_assets.music.clone()),
        PlaybackSettings::LOOP,
    ));
    playback.started = true;
}

fn play_paddle_hit_sfx(
    _event: On<PaddleHitEvent>,
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
) {
    commands.spawn((
        AudioPlayer::new(audio_assets.hit.clone()),
        PlaybackSettings::DESPAWN,
    ));
}
