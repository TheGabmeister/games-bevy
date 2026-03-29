use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::states::*;

pub struct HumanPlugin;

impl Plugin for HumanPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, human_wander.in_set(GameSet::Movement));
    }
}

fn human_wander(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut WanderTimer, &mut WanderTarget), With<Human>>,
) {
    let mut rng = rand::rng();
    for (mut vel, mut timer, mut target) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
            target.0 = Vec2::new(angle.cos(), angle.sin());
        }
        vel.0 = target.0 * HUMAN_SPEED;
    }
}
