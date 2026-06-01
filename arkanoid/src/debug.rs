use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{Ball, Brick, Capsule, Paddle, PowerupKind};
use crate::constants::*;
use crate::states::AppState;

/// Developer-only tooling: a collider overlay and a "clear the board" shortcut.
///
/// Debug systems read the keyboard directly (rather than going through the
/// `InputActions` abstraction) on purpose — this is tooling, not gameplay, and
/// shouldn't pollute the gameplay input contract.
///
/// Keys (while in `AppState::Playing`):
/// - **F1** — toggle the collider overlay
/// - **F2** — destroy all bricks (clears the round, triggering the next layout)
/// - **F3** — drop a capsule above the paddle, cycling through the seven power-ups
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugSettings>().add_systems(
            Update,
            (toggle_debug, debug_destroy_bricks, debug_spawn_capsule, draw_colliders)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

#[derive(Resource, Default)]
struct DebugSettings {
    show_colliders: bool,
    next_capsule: usize,
}

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<DebugSettings>) {
    if keys.just_pressed(KeyCode::F1) {
        settings.show_colliders = !settings.show_colliders;
    }
}

fn debug_destroy_bricks(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    bricks: Query<Entity, With<Brick>>,
) {
    if keys.just_pressed(KeyCode::F2) {
        for entity in &bricks {
            commands.entity(entity).despawn();
        }
    }
}

/// Drops a capsule above the paddle on F3, cycling through every power-up so each can be
/// caught and tested without waiting for the drop schedule.
fn debug_spawn_capsule(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut settings: ResMut<DebugSettings>,
    assets: Res<GameAssets>,
    paddle: Query<&Transform, With<Paddle>>,
) {
    if !keys.just_pressed(KeyCode::F3) {
        return;
    }
    const KINDS: [PowerupKind; 7] = [
        PowerupKind::Catch,
        PowerupKind::Laser,
        PowerupKind::Expand,
        PowerupKind::Disruption,
        PowerupKind::Slow,
        PowerupKind::Break,
        PowerupKind::Player,
    ];
    let kind = KINDS[settings.next_capsule % KINDS.len()];
    settings.next_capsule += 1;
    let x = paddle.single().map(|t| t.translation.x).unwrap_or(0.0);
    commands.spawn((
        Capsule { kind },
        Sprite::from_image(assets.sprites.capsules.handle(kind)),
        Transform::from_xyz(x, BRICK_FIELD_TOP, Z_CAPSULE),
        DespawnOnExit(AppState::Playing),
    ));
}

/// Draws the collider outline for every collidable thing when the overlay is on:
/// the playfield walls, each brick, the paddle, and the ball.
fn draw_colliders(
    settings: Res<DebugSettings>,
    mut gizmos: Gizmos,
    bricks: Query<&Transform, With<Brick>>,
    paddle: Query<(&Transform, &Paddle)>,
    balls: Query<&Transform, With<Ball>>,
) {
    if !settings.show_colliders {
        return;
    }

    // Playfield walls (left/right verticals, top horizontal; bottom is open).
    let wall = Color::srgb(1.0, 0.2, 0.2);
    gizmos.line_2d(
        Vec2::new(PLAYFIELD_LEFT, PLAYFIELD_BOTTOM),
        Vec2::new(PLAYFIELD_LEFT, PLAYFIELD_TOP),
        wall,
    );
    gizmos.line_2d(
        Vec2::new(PLAYFIELD_RIGHT, PLAYFIELD_BOTTOM),
        Vec2::new(PLAYFIELD_RIGHT, PLAYFIELD_TOP),
        wall,
    );
    gizmos.line_2d(
        Vec2::new(PLAYFIELD_LEFT, PLAYFIELD_TOP),
        Vec2::new(PLAYFIELD_RIGHT, PLAYFIELD_TOP),
        wall,
    );

    let brick_size = Vec2::new(BRICK_WIDTH, BRICK_HEIGHT);
    for transform in &bricks {
        gizmos.rect_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            brick_size,
            Color::srgb(0.2, 1.0, 0.4),
        );
    }

    for (transform, paddle) in &paddle {
        gizmos.rect_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            Vec2::new(paddle.half_width * 2.0, PADDLE_HEIGHT),
            Color::srgb(0.2, 0.8, 1.0),
        );
    }

    for transform in &balls {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            BALL_RADIUS,
            Color::srgb(1.0, 1.0, 0.2),
        );
    }
}
