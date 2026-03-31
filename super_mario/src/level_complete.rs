use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::level::{tile_to_world, world_to_col, world_to_row, LevelGrid};
use crate::resources::*;
use crate::states::*;
use crate::ui;

pub struct LevelCompletePlugin;

impl Plugin for LevelCompletePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            flagpole_collision.in_set(GameplaySet::Late),
        )
        .add_systems(
            Update,
            level_complete_system.run_if(in_state(PlayState::LevelComplete)),
        );
    }
}

fn flagpole_collision(
    mut commands: Commands,
    level: Res<LevelGrid>,
    player_query: Query<(Entity, &Transform, &CollisionSize), With<Player>>,
    mut score_events: MessageWriter<ScoreEvent>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    let Ok((player_entity, player_tf, player_coll)) = player_query.single() else {
        return;
    };

    let half_w = player_coll.width / 2.0;
    let half_h = player_coll.height / 2.0;

    let col_min = world_to_col(player_tf.translation.x - half_w);
    let col_max = world_to_col(player_tf.translation.x + half_w);
    let row_min = world_to_row(player_tf.translation.y + half_h);
    let row_max = world_to_row(player_tf.translation.y - half_h);

    for row in row_min..=row_max {
        for col in col_min..=col_max {
            if level.get_char(col, row) != 'F' {
                continue;
            }

            // Mario touched the flagpole!
            let contact_row = world_to_row(player_tf.translation.y)
                .clamp(FLAGPOLE_TOP_ROW as i32, FLAGPOLE_BOTTOM_ROW as i32)
                as usize;

            let score_range = FLAGPOLE_TOP_SCORE - FLAGPOLE_BOTTOM_SCORE;
            let row_range = (FLAGPOLE_BOTTOM_ROW - FLAGPOLE_TOP_ROW) as u32;
            let rows_from_bottom = (FLAGPOLE_BOTTOM_ROW - contact_row) as u32;
            let flagpole_score =
                FLAGPOLE_BOTTOM_SCORE + score_range * rows_from_bottom / row_range;

            score_events.write(ScoreEvent { points: flagpole_score });

            // Score popup
            ui::spawn_score_popup(
                &mut commands,
                flagpole_score,
                player_tf.translation.x + 15.0,
                player_tf.translation.y,
            );

            // Compute animation data
            let pole_col = col;
            let (pole_x, _) = tile_to_world(pole_col as usize, FLAGPOLE_TOP_ROW);
            let (_, ground_y) = tile_to_world(pole_col as usize, 13);
            let pole_base_y = ground_y + TILE_SIZE / 2.0 + player_coll.height / 2.0;

            let castle_col = pole_col as usize + CASTLE_OFFSET_TILES;
            let (castle_x, _) = tile_to_world(castle_col, 12);

            commands.insert_resource(LevelCompleteAnimation {
                phase: LevelCompletePhase::SlideDown,
                pole_x,
                pole_base_y,
                castle_x,
                done_timer: Timer::from_seconds(LEVEL_COMPLETE_DONE_DELAY, TimerMode::Once),
                flagpole_score,
            });

            // Clear invincibility so the player is fully visible
            commands.entity(player_entity).remove::<Invincible>();
            commands.entity(player_entity).insert(Visibility::Inherited);

            next_play_state.set(PlayState::LevelComplete);
            return;
        }
    }
}

fn level_complete_system(
    time: Res<Time>,
    mut commands: Commands,
    mut anim: ResMut<LevelCompleteAnimation>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    mut flag_query: Query<&mut Transform, (With<FlagpoleFlag>, Without<Player>)>,
    mut game_timer: ResMut<GameTimer>,
    mut score_events: MessageWriter<ScoreEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut level_list: ResMut<LevelList>,
) {
    let Ok((mut player_tf, mut vel)) = player_query.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    vel.x = 0.0;
    vel.y = 0.0;

    match anim.phase {
        LevelCompletePhase::SlideDown => {
            player_tf.translation.x = anim.pole_x;
            player_tf.translation.y -= FLAGPOLE_SLIDE_SPEED * dt;

            // Slide the flag down too
            if let Ok(mut flag_tf) = flag_query.single_mut() {
                flag_tf.translation.y = player_tf.translation.y;
            }

            if player_tf.translation.y <= anim.pole_base_y {
                player_tf.translation.y = anim.pole_base_y;
                anim.phase = LevelCompletePhase::WalkToCastle;
            }
        }
        LevelCompletePhase::WalkToCastle => {
            player_tf.translation.x += FLAGPOLE_WALK_SPEED * dt;

            if player_tf.translation.x >= anim.castle_x {
                player_tf.translation.x = anim.castle_x;
                anim.phase = LevelCompletePhase::TimeTally;
            }
        }
        LevelCompletePhase::TimeTally => {
            if game_timer.time > 0.0 {
                let ticks = (TIME_TALLY_RATE * dt).ceil().min(game_timer.time) as u32;
                game_timer.time -= ticks as f32;
                score_events.write(ScoreEvent {
                    points: ticks * TIME_BONUS_PER_TICK,
                });
            } else {
                game_timer.time = 0.0;
                anim.phase = LevelCompletePhase::Done;
            }
        }
        LevelCompletePhase::Done => {
            anim.done_timer.tick(time.delta());
            if anim.done_timer.is_finished() {
                commands.remove_resource::<LevelCompleteAnimation>();
                level_list.advance();
                next_app_state.set(AppState::LevelTransition);
            }
        }
    }
}
