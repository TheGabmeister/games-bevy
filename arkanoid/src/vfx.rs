use bevy::prelude::*;

use crate::components::VfxAnim;
use crate::constants::*;
use crate::states::AppState;

/// Reusable one-shot animated VFX: spawn a flipbook of frames with [`spawn_vfx`] and this
/// plugin advances it frame-by-frame, despawning the entity after the last frame.
pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_vfx);
    }
}

/// Spawns an animated VFX flipbook at `position`, starting on its first frame. Auto-despawns
/// when it leaves `Playing` or finishes, whichever comes first.
pub fn spawn_vfx(commands: &mut Commands, frames: &[Handle<Image>], position: Vec2) {
    let Some(first) = frames.first() else {
        return;
    };
    commands.spawn((
        VfxAnim {
            frames: frames.to_vec(),
            index: 0,
            timer: Timer::from_seconds(VFX_FRAME_TIME, TimerMode::Repeating),
        },
        Sprite::from_image(first.clone()),
        Transform::from_xyz(position.x, position.y, Z_VFX),
        DespawnOnExit(AppState::Playing),
    ));
}

fn animate_vfx(
    mut commands: Commands,
    time: Res<Time>,
    mut anims: Query<(Entity, &mut VfxAnim, &mut Sprite)>,
) {
    for (entity, mut anim, mut sprite) in &mut anims {
        if !anim.timer.tick(time.delta()).just_finished() {
            continue;
        }
        anim.index += 1;
        if anim.index >= anim.frames.len() {
            commands.entity(entity).despawn();
        } else {
            sprite.image = anim.frames[anim.index].clone();
        }
    }
}
