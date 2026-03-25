use bevy::prelude::*;
use bevy::audio::PlaybackSettings;

use crate::components::Music;
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
    commands.spawn((
        Music,
        AudioPlayer::new(asset_server.load("music_spaceshooter.ogg")),
        PlaybackSettings::LOOP,
    ));
}
