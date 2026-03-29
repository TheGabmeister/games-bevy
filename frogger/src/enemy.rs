use bevy::prelude::*;

use crate::components::Enemy;
use crate::constants::{ENEMY_COUNT, ENEMY_Y, WINDOW_WIDTH};
use crate::states::AppState;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_enemies)
            .add_systems(OnExit(AppState::Playing), cleanup_enemies);
    }
}

fn spawn_enemies(mut commands: Commands, asset_server: Res<AssetServer>) {
    let section_width = WINDOW_WIDTH / ENEMY_COUNT as f32;

    for i in 0..ENEMY_COUNT {
        let x = -WINDOW_WIDTH / 2.0 + section_width * (i as f32 + 0.5);
        commands.spawn((
            Enemy,
            Sprite::from_image(asset_server.load("enemy_ufo_green.png")),
            Transform::from_translation(Vec3::new(x, ENEMY_Y, 0.0)),
        ));
    }
}

fn cleanup_enemies(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
