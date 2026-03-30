use bevy::prelude::*;

use crate::components::*;
use crate::resources::GameAssets;
use crate::spawn::spawn_bullet;
use crate::state::AppState;
use crate::{
    BULLET_SPEED, GameSet, SHIP_DRAG, SHIP_MAX_SPEED, SHIP_RADIUS, SHIP_ROTATION_SPEED,
    SHIP_THRUST, SHOOT_COOLDOWN,
};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        // Shoot uses just_pressed — must stay in Update to avoid missed/double inputs
        app.add_systems(
            Update,
            ship_shoot_system
                .in_set(GameSet::Input)
                .run_if(in_state(AppState::Playing)),
        )
        // Rotation and thrust modify physics state — run in FixedUpdate
        .add_systems(
            FixedUpdate,
            (ship_rotation_system, ship_thrust_system)
                .chain()
                .in_set(GameSet::Movement)
                .run_if(in_state(AppState::Playing)),
        )
        // Timer-driven cleanup runs in FixedUpdate for consistent tick rates
        .add_systems(
            FixedUpdate,
            (bullet_lifetime_system, invincibility_system)
                .in_set(GameSet::Cleanup)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Left/Right arrows rotate the ship.
fn ship_rotation_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Ship>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    if input.pressed(KeyCode::ArrowLeft) {
        transform.rotate_z(SHIP_ROTATION_SPEED * dt);
    }
    if input.pressed(KeyCode::ArrowRight) {
        transform.rotate_z(-SHIP_ROTATION_SPEED * dt);
    }
}

/// Up arrow applies thrust in the direction the ship faces; drag is always applied.
fn ship_thrust_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Velocity), With<Ship>>,
) {
    let Ok((transform, mut velocity)) = query.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    if input.pressed(KeyCode::ArrowUp) {
        // transform.up() returns the local +Y direction in world space (Dir3).
        // .truncate() converts Vec3 → Vec2 via Deref<Target = Vec3>.
        let forward = transform.up().truncate();
        velocity.0 += forward * SHIP_THRUST * dt;
    }
    // Frame-rate independent drag: 0.97 was tuned as per-frame at 60fps
    velocity.0 *= SHIP_DRAG.powf(60.0 * dt);
    velocity.0 = velocity.0.clamp_length_max(SHIP_MAX_SPEED);
}

/// Space fires a bullet from the ship nose, respecting a shoot cooldown.
fn ship_shoot_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut query: Query<(&Transform, &mut ShootCooldown), With<Ship>>,
) {
    let Ok((transform, mut cooldown)) = query.single_mut() else {
        return;
    };
    cooldown.0.tick(time.delta());
    if input.just_pressed(KeyCode::Space) && cooldown.0.is_finished() {
        cooldown.0 = Timer::from_seconds(SHOOT_COOLDOWN, TimerMode::Once);
        let dir = transform.up().truncate();
        // Spawn just ahead of the ship nose so it doesn't immediately self-collide
        let spawn_pos = transform.translation + dir.extend(0.0) * (SHIP_RADIUS + 8.0);
        spawn_bullet(
            &mut commands,
            &assets,
            Vec3::new(spawn_pos.x, spawn_pos.y, 0.5),
            dir,
            BULLET_SPEED,
        );
    }
}

/// Counts down the invincibility timer; removes the component when it expires.
fn invincibility_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Invincible)>,
) {
    for (entity, mut inv) in &mut query {
        inv.0.tick(time.delta());
        if inv.0.is_finished() {
            commands.entity(entity).remove::<Invincible>();
        }
    }
}

/// Despawns bullets whose lifetime has expired.
fn bullet_lifetime_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifetime), With<Bullet>>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0.tick(time.delta());
        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
