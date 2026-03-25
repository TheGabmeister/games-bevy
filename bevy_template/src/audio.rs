use bevy::prelude::*;

use crate::states::AppState;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), play_music);
    }
}

fn play_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(AudioPlayer::new(
        asset_server.load("music_spaceshooter.ogg"),
    ));
}
