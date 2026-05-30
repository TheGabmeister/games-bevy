use bevy::prelude::*;

/// Linear velocity in pixels per second.
#[derive(Component)]
pub struct Velocity(pub Vec2);

/// The player-controlled Vaus paddle.
#[derive(Component)]
pub struct Paddle;

/// The energy ball. While `stuck`, it rides on top of the paddle and waits to be
/// launched; once launched it moves under its own `Velocity`.
#[derive(Component)]
pub struct Ball {
    pub stuck: bool,
}
