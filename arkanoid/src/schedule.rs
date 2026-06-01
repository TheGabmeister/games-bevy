use bevy::prelude::*;

/// Ordered phases of the fixed-timestep gameplay pipeline. Each gameplay plugin tags its
/// `FixedUpdate` systems into one of these sets, and [`SchedulePlugin`] chains the sets in
/// the order below — so the cross-plugin physics ordering lives in one place instead of
/// being scattered across `.after(other_plugin::system)` calls.
///
/// Ordering is configured at the set level; individual systems keep their own `run_if`
/// state gating (e.g. paddle control runs through all of `Playing`, while movement and
/// collision run only while `Running`). The few intra-set dependencies that remain (e.g.
/// `ball_brick_collision` after `ball_collision`) are expressed with `.after()` on the
/// systems themselves.
#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Physics {
    /// The player moves the paddle.
    PaddleInput,
    /// A stuck ball tracks the paddle so it launches from the current Vaus position.
    BallFollow,
    /// Everything integrates its position from velocity (ball, capsules, lasers, enemies).
    Movement,
    /// Contacts are resolved against the freshly-moved positions.
    Collision,
}

/// Configures the canonical [`Physics`] set order for `FixedUpdate`.
pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            (
                Physics::PaddleInput,
                Physics::BallFollow,
                Physics::Movement,
                Physics::Collision,
            )
                .chain(),
        );
    }
}
