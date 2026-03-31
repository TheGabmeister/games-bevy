use bevy::prelude::*;

use crate::collision::aabb_overlap;
use crate::components::*;
use crate::constants::*;
use crate::resources::GameData;
use crate::states::*;

use super::mario_take_damage;

pub fn mario_goomba_collision(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &mut Velocity, &Transform, &CollisionSize, &PlayerSize, Has<Invincible>),
        With<Player>,
    >,
    mut goomba_query: Query<
        (Entity, &mut Transform, &mut Velocity, &CollisionSize),
        (With<Goomba>, With<EnemyActive>, Without<Squished>, Without<Player>, Without<KoopaTroopa>, Without<Shell>),
    >,
    mut game_data: ResMut<GameData>,
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

    let px = player_tf.translation.x;
    let py = player_tf.translation.y;
    let pvy = player_vel.y;

    for (entity, mut enemy_tf, mut enemy_vel, enemy_coll) in &mut goomba_query {
        if aabb_overlap(
            px, py, player_coll.width / 2.0, player_coll.height / 2.0,
            enemy_tf.translation.x, enemy_tf.translation.y,
            enemy_coll.width / 2.0, enemy_coll.height / 2.0,
        ).is_none() {
            continue;
        }

        if py > enemy_tf.translation.y && pvy <= 0.0 {
            // Stomp
            player_vel.y = STOMP_BOUNCE_IMPULSE;
            game_data.score += STOMP_SCORE;

            enemy_tf.scale.y = 0.3;
            enemy_vel.x = 0.0;
            enemy_vel.y = 0.0;
            commands
                .entity(entity)
                .insert(Squished(Timer::from_seconds(SQUISH_DURATION, TimerMode::Once)))
                .remove::<EnemyWalker>();

            commands.spawn((
                ScorePopup(Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once)),
                Text2d::new(format!("+{STOMP_SCORE}")),
                TextFont { font_size: 8.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(
                    enemy_tf.translation.x,
                    enemy_tf.translation.y + GOOMBA_HEIGHT,
                    Z_PLAYER + 1.0,
                ),
                DespawnOnExit(AppState::Playing),
            ));

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
