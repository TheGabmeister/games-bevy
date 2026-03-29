use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::AppState;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_pending_effects,
                animate_death_flashes,
                animate_score_popups,
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}

fn spawn_pending_effects(
    mut commands: Commands,
    mut pending: ResMut<PendingEffects>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for pos in pending.death_flashes.drain(..) {
        commands.spawn((
            GameplayEntity,
            DeathFlash(0.0),
            Mesh2d(meshes.add(Circle::new(FROG_BODY_RADIUS))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(COLOR_DEATH_FLASH))),
            Transform::from_translation(pos.extend(15.0)),
        ));
    }

    for (points, pos) in pending.score_popups.drain(..) {
        commands.spawn((
            GameplayEntity,
            ScorePopup(0.0),
            Text2d::new(format!("+{points}")),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(COLOR_HIGHLIGHT_TEXT),
            Transform::from_translation(pos.extend(20.0)),
        ));
    }
}

fn animate_death_flashes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut DeathFlash,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut flash, mut tf, mat_handle) in &mut query {
        flash.0 += time.delta_secs();
        if flash.0 >= DEATH_FLASH_DURATION {
            commands.entity(entity).despawn();
            continue;
        }
        let t = flash.0 / DEATH_FLASH_DURATION;
        tf.scale = Vec3::splat(1.0 + (DEATH_FLASH_SCALE - 1.0) * t);
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = Color::srgba(1.0, 0.3, 0.2, 1.0 - t);
        }
    }
}

fn animate_score_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScorePopup, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut popup, mut tf, mut color) in &mut query {
        popup.0 += time.delta_secs();
        if popup.0 >= SCORE_POPUP_DURATION {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation.y += SCORE_POPUP_SPEED * time.delta_secs();
        let alpha = 1.0 - popup.0 / SCORE_POPUP_DURATION;
        color.0 = Color::srgba(0.9, 0.9, 0.2, alpha);
    }
}
