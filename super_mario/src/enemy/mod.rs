mod goomba;
mod koopa;

use bevy::prelude::*;

use crate::collision::{self, WallAction};
use crate::components::*;
use crate::constants::*;
use crate::level::LevelGrid;
use crate::states::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            enemy_activation.in_set(GameplaySet::Input),
        )
        .add_systems(
            Update,
            (enemy_walk, enemy_gravity, enemy_apply_velocity, enemy_tile_collision)
                .chain()
                .in_set(GameplaySet::Physics),
        )
        .add_systems(
            Update,
            (
                goomba::mario_goomba_collision,
                koopa::mario_koopa_collision,
                koopa::mario_shell_collision,
                koopa::shell_enemy_collision,
                enemy_despawn_out_of_bounds,
            )
                .in_set(GameplaySet::Late),
        )
        .add_systems(
            Update,
            (squish_timer, score_popup_system)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

// ── Shared helper ──

pub fn mario_take_damage(
    commands: &mut Commands,
    player_entity: Entity,
    player_size: PlayerSize,
    next_play_state: &mut ResMut<NextState<PlayState>>,
) {
    match player_size {
        PlayerSize::Big | PlayerSize::Fire => {
            // Shrink to Small (Fire skips Big, goes straight to Small)
            commands.entity(player_entity).insert(GrowthAnimation {
                timer: Timer::from_seconds(GROWTH_DURATION, TimerMode::Once),
                flash_timer: Timer::from_seconds(GROWTH_FLASH_INTERVAL, TimerMode::Repeating),
                growing: false,
            });
            next_play_state.set(PlayState::Growing);
        }
        PlayerSize::Small => {
            next_play_state.set(PlayState::Dying);
        }
    }
}

// ── Activation ──

fn enemy_activation(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera2d>>,
    query: Query<(Entity, &Transform), (With<EnemyWalker>, Without<EnemyActive>)>,
) {
    let Ok(camera_tf) = camera_query.single() else { return };
    let activate_x = camera_tf.translation.x + CAMERA_VISIBLE_WIDTH / 2.0 + TILE_SIZE;

    for (entity, transform) in &query {
        if transform.translation.x <= activate_x {
            commands.entity(entity).insert(EnemyActive);
        }
    }
}

// ── Shared Physics ──

fn enemy_walk(
    mut query: Query<
        (&mut Velocity, &EnemyWalker),
        (With<EnemyActive>, Without<Squished>),
    >,
) {
    for (mut vel, walker) in &mut query {
        vel.x = walker.speed * walker.direction;
    }
}

fn enemy_gravity(
    time: Res<Time>,
    mut query: Query<
        (&mut Velocity, &Grounded),
        (With<EnemyActive>, Without<Squished>, Without<Player>),
    >,
) {
    for (mut vel, grounded) in &mut query {
        if grounded.0 {
            continue;
        }
        vel.y -= GRAVITY_DESCENDING * time.delta_secs();
        vel.y = vel.y.max(-TERMINAL_VELOCITY);
    }
}

fn enemy_apply_velocity(
    time: Res<Time>,
    mut query: Query<
        (&Velocity, &mut Transform),
        (With<EnemyActive>, Without<Squished>, Without<Player>),
    >,
) {
    for (vel, mut transform) in &mut query {
        transform.translation.x += vel.x * time.delta_secs();
        transform.translation.y += vel.y * time.delta_secs();
    }
}

fn enemy_tile_collision(
    level: Res<LevelGrid>,
    mut query: Query<
        (&mut Velocity, &mut Transform, &mut Grounded, &mut EnemyWalker, &CollisionSize),
        (With<EnemyActive>, Without<Squished>),
    >,
) {
    for (mut vel, mut transform, mut grounded, mut walker, coll_size) in &mut query {
        let result = collision::resolve_tile_collisions(
            &level,
            &mut transform.translation,
            &mut vel,
            coll_size.width / 2.0,
            coll_size.height / 2.0,
            WallAction::Reverse,
            &mut walker.direction,
        );
        grounded.0 = result.grounded;
    }
}

// ── Shared Cleanup ──

fn enemy_despawn_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<EnemyActive>, Without<Player>)>,
) {
    for (entity, transform) in &query {
        if transform.translation.y < DEATH_Y {
            commands.entity(entity).despawn();
        }
    }
}

fn squish_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Squished)>,
) {
    for (entity, mut squished) in &mut query {
        squished.0.tick(time.delta());
        if squished.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn score_popup_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScorePopup, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut popup, mut transform, mut color) in &mut query {
        popup.0.tick(time.delta());
        transform.translation.y += SCORE_POPUP_RISE_SPEED * time.delta_secs();

        let alpha = 1.0 - popup.0.fraction();
        color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);

        if popup.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
