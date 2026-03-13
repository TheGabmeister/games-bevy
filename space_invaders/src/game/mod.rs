mod gameplay;
mod presentation;

use bevy::prelude::*;

pub use gameplay::SpaceInvadersGameplayPlugin;
pub use presentation::SpaceInvadersPresentationPlugin;

pub const WINDOW_WIDTH: f32 = 600.0;
pub const WINDOW_HEIGHT: f32 = 800.0;

pub fn background_color() -> Color {
    Color::srgb(0.02, 0.03, 0.08)
}

pub struct SpaceInvadersPlugin;

impl Plugin for SpaceInvadersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SpaceInvadersGameplayPlugin, SpaceInvadersPresentationPlugin));
    }
}
