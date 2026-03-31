use bevy::prelude::*;

use crate::collision::entities_overlap;
use crate::components::*;
use crate::constants::*;
use crate::level::SpawnerRegistry;
use crate::resources::ScoreEvent;
use crate::states::*;
use crate::ui;

use super::mario_take_damage;

// ── Plugin ──

pub struct GoombaPlugin;

impl Plugin for GoombaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_goomba)
            .add_systems(
                Update,
                mario_goomba_collision.in_set(GameplaySet::Late),
            );
    }
}

/// Create goomba mesh/material handles and register the 'G' level spawner.
fn init_goomba(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut registry: ResMut<SpawnerRegistry>,
) {
    let body_mesh = meshes.add(Ellipse::new(6.0, 5.0));
    let body_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.55, 0.30, 0.10)));
    let feet_mesh = meshes.add(Rectangle::new(12.0, 4.0));
    let feet_mat = materials.add(ColorMaterial::from_color(Color::srgb(0.35, 0.18, 0.05)));

    registry.register(
        'G',
        Box::new(move |commands, wx, wy, _, _| {
            commands
                .spawn((
                    Goomba,
                    EnemyWalker {
                        speed: GOOMBA_SPEED,
                        direction: -1.0,
                    },
                    CollisionSize {
                        width: GOOMBA_WIDTH,
                        height: GOOMBA_HEIGHT,
                    },
                    Velocity::default(),
                    Grounded::default(),
                    Mesh2d(body_mesh.clone()),
                    MeshMaterial2d(body_mat.clone()),
                    Transform::from_xyz(wx, wy, Z_ENEMY),
                    DespawnOnExit(AppState::Playing),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh2d(feet_mesh.clone()),
                        MeshMaterial2d(feet_mat.clone()),
                        Transform::from_xyz(0.0, -5.0, 0.0),
                    ));
                });
        }),
    );
}

// ── Collision ──

pub fn mario_goomba_collision(
    mut commands: Commands,
    mut player_query: Query<
        (
            Entity,
            &mut Velocity,
            &Transform,
            &CollisionSize,
            &PlayerSize,
            Has<Invincible>,
        ),
        With<Player>,
    >,
    mut goomba_query: Query<
        (Entity, &mut Transform, &mut Velocity, &CollisionSize),
        (
            With<Goomba>,
            With<EnemyActive>,
            Without<Squished>,
            Without<Player>,
            Without<KoopaTroopa>,
            Without<Shell>,
        ),
    >,
    mut score_events: MessageWriter<ScoreEvent>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    let Ok((player_entity, mut player_vel, player_tf, player_coll, &player_size, is_invincible)) =
        player_query.single_mut()
    else {
        return;
    };

    if is_invincible {
        return;
    }

    for (entity, mut enemy_tf, mut enemy_vel, enemy_coll) in &mut goomba_query {
        if !entities_overlap(player_tf, player_coll, &enemy_tf, &enemy_coll) {
            continue;
        }

        if player_tf.translation.y > enemy_tf.translation.y && player_vel.y <= 0.0 {
            // Stomp
            player_vel.y = STOMP_BOUNCE_IMPULSE;
            score_events.write(ScoreEvent { points: STOMP_SCORE });

            enemy_tf.scale.y = 0.3;
            enemy_vel.x = 0.0;
            enemy_vel.y = 0.0;
            commands
                .entity(entity)
                .insert(Squished(Timer::from_seconds(
                    SQUISH_DURATION,
                    TimerMode::Once,
                )))
                .remove::<EnemyWalker>();

            ui::spawn_score_popup(
                &mut commands,
                STOMP_SCORE,
                enemy_tf.translation.x,
                enemy_tf.translation.y + GOOMBA_HEIGHT,
            );

            return;
        } else {
            mario_take_damage(
                &mut commands,
                player_entity,
                player_size,
                &mut next_play_state,
            );
            return;
        }
    }
}
