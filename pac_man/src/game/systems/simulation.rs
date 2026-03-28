use bevy::prelude::*;

use crate::game::{
    components::{Direction, Ghost, GhostPersonality, GridMover, Pellet, PelletKind, Player},
    constants::{
        GHOST_COLLISION_RADIUS, GHOST_EATEN_SPEED, GHOST_FRIGHTENED_SPEED, GHOST_SPEED,
        PLAYER_SCORE_GHOST, PLAYER_SCORE_PELLET, PLAYER_SCORE_POWER, TURN_TOLERANCE, TILE_SIZE,
    },
    level::LevelLayout,
    logic::{chase_target, choose_direction_away, choose_direction_toward},
    resources::{GameSession, RoundState},
};

#[allow(clippy::type_complexity)]
pub fn plan_ghost_turns(
    layout: Res<LevelLayout>,
    session: Res<GameSession>,
    player: Single<(&Transform, &GridMover), (With<Player>, Without<Ghost>)>,
    mut ghosts: ParamSet<(
        Query<(&Ghost, &Transform), Without<Player>>,
        Query<(&mut Ghost, &Transform, &mut GridMover), Without<Player>>,
    )>,
) {
    let player_tile = layout.world_to_tile(player.0.translation.truncate());
    let player_direction = player.1.current.unwrap_or(Direction::Left);

    let mut blinky_tile = player_tile;
    for (ghost, ghost_transform) in &ghosts.p0() {
        if ghost.personality == GhostPersonality::Blinky {
            blinky_tile = layout.world_to_tile(ghost_transform.translation.truncate());
            break;
        }
    }

    for (mut ghost, ghost_transform, mut mover) in &mut ghosts.p1() {
        mover.speed = if ghost.returning_home {
            GHOST_EATEN_SPEED
        } else if session.frightened_active() {
            GHOST_FRIGHTENED_SPEED
        } else {
            GHOST_SPEED
        };

        let position = ghost_transform.translation.truncate();
        let tile = layout.world_to_tile(position);
        let centered = is_centered(position, tile, &layout);

        if ghost.returning_home && centered && tile == ghost.home_tile {
            ghost.returning_home = false;
            mover.current = Some(Direction::Up);
            mover.desired = Some(Direction::Up);
        }

        if !centered {
            continue;
        }

        let mut options: Vec<_> = Direction::ALL
            .into_iter()
            .filter(|direction| layout.can_move(tile, *direction))
            .collect();

        if options.len() > 1 && let Some(current) = mover.current {
            options.retain(|direction| *direction != current.opposite());
        }

        if options.is_empty() {
            continue;
        }

        let next_direction = if ghost.returning_home {
            choose_direction_toward(tile, ghost.home_tile, &options)
        } else if session.frightened_active() {
            choose_direction_away(tile, player_tile, &options)
        } else {
            let target = if session.scatter_mode {
                ghost.scatter_target
            } else {
                chase_target(
                    &ghost,
                    tile,
                    player_tile,
                    player_direction,
                    blinky_tile,
                    &layout,
                )
            };
            choose_direction_toward(tile, target, &options)
        };

        mover.current = Some(next_direction);
        mover.desired = Some(next_direction);
    }
}

pub fn move_player(
    time: Res<Time<Fixed>>,
    layout: Res<LevelLayout>,
    player: Single<(&mut Transform, &mut GridMover), With<Player>>,
) {
    let (mut transform, mut mover) = player.into_inner();
    step_mover(&mut transform, &mut mover, &layout, time.delta_secs());
}

pub fn move_ghosts(
    time: Res<Time<Fixed>>,
    layout: Res<LevelLayout>,
    mut ghosts: Query<(&mut Transform, &mut GridMover), With<Ghost>>,
) {
    for (mut transform, mut mover) in &mut ghosts {
        step_mover(&mut transform, &mut mover, &layout, time.delta_secs());
    }
}

pub fn collect_pellets(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<RoundState>>,
    player: Single<&Transform, With<Player>>,
    pellets: Query<(Entity, &Pellet, &Transform)>,
    mut ghosts: Query<(&Ghost, &mut GridMover)>,
) {
    let player_position = player.translation.truncate();
    let mut ate_power_pellet = false;

    for (entity, pellet, pellet_transform) in &pellets {
        if player_position.distance_squared(pellet_transform.translation.truncate())
            > (TILE_SIZE * 0.35).powi(2)
        {
            continue;
        }

        commands.entity(entity).despawn();
        session.pellets_remaining = session.pellets_remaining.saturating_sub(1);

        match pellet.kind {
            PelletKind::Dot => session.score += PLAYER_SCORE_PELLET,
            PelletKind::Power => {
                session.score += PLAYER_SCORE_POWER;
                session.set_frightened();
                ate_power_pellet = true;
            }
        }

        if session.pellets_remaining == 0 {
            next_state.set(RoundState::Won);
        }
    }

    if ate_power_pellet {
        for (ghost, mut mover) in &mut ghosts {
            if ghost.returning_home {
                continue;
            }

            if let Some(current) = mover.current {
                let reversed = current.opposite();
                mover.current = Some(reversed);
                mover.desired = Some(reversed);
            }
        }
    }
}

pub fn resolve_ghost_collisions(
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<RoundState>>,
    mut player: Single<(&mut Transform, &mut GridMover), With<Player>>,
    mut ghosts: Query<(&mut Ghost, &mut Transform, &mut GridMover)>,
) {
    let player_position = player.0.translation.truncate();
    let collision_radius_sq = GHOST_COLLISION_RADIUS * GHOST_COLLISION_RADIUS;
    let mut player_hit = false;

    for (mut ghost, ghost_transform, mut mover) in &mut ghosts {
        if player_position.distance_squared(ghost_transform.translation.truncate())
            > collision_radius_sq
        {
            continue;
        }

        if ghost.returning_home {
            continue;
        }

        if session.frightened_active() {
            ghost.returning_home = true;
            let combo_multiplier = 1_u32 << u32::from(session.ghost_combo.min(3));
            session.score += PLAYER_SCORE_GHOST * combo_multiplier;
            session.ghost_combo = session.ghost_combo.saturating_add(1);
            mover.speed = GHOST_EATEN_SPEED;
            continue;
        }

        player_hit = true;
        break;
    }

    if !player_hit {
        return;
    }

    if session.lives > 1 {
        session.lives -= 1;
        player.1.current = None;
        player.1.desired = player.1.spawn_direction;
        next_state.set(RoundState::Ready);
    } else {
        session.lives = 0;
        session.clear_round_effects();
        next_state.set(RoundState::GameOver);
    }
}

fn step_mover(transform: &mut Transform, mover: &mut GridMover, layout: &LevelLayout, delta: f32) {
    let tile = layout.world_to_tile(transform.translation.truncate());
    let center = layout.tile_to_world(tile);
    let centered = is_centered(transform.translation.truncate(), tile, layout);

    if centered {
        if let Some(desired) = mover.desired && layout.can_move(tile, desired) {
            mover.current = Some(desired);
        }

        if let Some(current) = mover.current && !layout.can_move(tile, current) {
            mover.current = None;
        }

        match mover.current {
            Some(current) if current.is_horizontal() => {
                transform.translation.y = center.y;
            }
            Some(_) => {
                transform.translation.x = center.x;
            }
            None => {
                transform.translation.x = center.x;
                transform.translation.y = center.y;
            }
        }
    }

    let Some(current) = mover.current else {
        return;
    };

    if current.is_horizontal() {
        transform.translation.y = transform.translation.y.lerp(center.y, 0.45);
    } else {
        transform.translation.x = transform.translation.x.lerp(center.x, 0.45);
    }

    transform.translation += (current.vec2() * mover.speed * delta).extend(0.0);
    transform.rotation = current.rotation();

    if current.is_horizontal() {
        layout.wrap_translation(&mut transform.translation, tile.y);
    }
}

fn is_centered(position: Vec2, tile: IVec2, layout: &LevelLayout) -> bool {
    position.distance(layout.tile_to_world(tile)) <= TURN_TOLERANCE
}
