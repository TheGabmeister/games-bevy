use bevy::audio::PlaybackSettings;
use bevy::prelude::*;
use std::path::Path;

use crate::components::Music;
use crate::constants::GAMEPLAY_MUSIC_ASSET_PATH;
use crate::states::AppState;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), (stop_music, play_music).chain())
            .add_systems(OnExit(AppState::Playing), stop_music);
    }
}

fn stop_music(mut commands: Commands, query: Query<Entity, With<Music>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn play_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    let music_file_path = Path::new("assets").join(GAMEPLAY_MUSIC_ASSET_PATH);
    if !music_file_path.exists() {
        bevy::log::warn!(
            "Skipping gameplay music because the asset is missing: {}",
            music_file_path.display()
        );
        return;
    }

    commands.spawn((
        Music,
        AudioPlayer::new(asset_server.load(GAMEPLAY_MUSIC_ASSET_PATH)),
        PlaybackSettings::LOOP,
    ));
}
