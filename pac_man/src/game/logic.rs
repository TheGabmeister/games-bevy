use bevy::prelude::*;

use crate::game::{
    components::{Direction, Ghost, GhostPersonality},
    level::LevelLayout,
};

pub fn choose_direction_toward(tile: IVec2, target: IVec2, options: &[Direction]) -> Direction {
    let mut best_direction = options[0];
    let mut best_distance = f32::MAX;

    for direction in options {
        let next_tile = tile + direction.ivec2();
        let distance = target.as_vec2().distance_squared(next_tile.as_vec2());
        if distance < best_distance {
            best_distance = distance;
            best_direction = *direction;
        }
    }

    best_direction
}

pub fn choose_direction_away(tile: IVec2, target: IVec2, options: &[Direction]) -> Direction {
    let mut best_direction = options[0];
    let mut best_distance = f32::MIN;

    for direction in options {
        let next_tile = tile + direction.ivec2();
        let distance = target.as_vec2().distance_squared(next_tile.as_vec2());
        if distance > best_distance {
            best_distance = distance;
            best_direction = *direction;
        }
    }

    best_direction
}

pub fn chase_target(
    ghost: &Ghost,
    ghost_tile: IVec2,
    player_tile: IVec2,
    player_direction: Direction,
    blinky_tile: IVec2,
    layout: &LevelLayout,
) -> IVec2 {
    let ahead = player_direction.ivec2();

    let raw_target = match ghost.personality {
        GhostPersonality::Blinky => player_tile,
        GhostPersonality::Pinky => player_tile + ahead * 4,
        GhostPersonality::Inky => player_tile + ahead * 2 + (player_tile - blinky_tile),
        GhostPersonality::Clyde => {
            if ghost_tile.as_vec2().distance(player_tile.as_vec2()) > 7.0 {
                player_tile
            } else {
                ghost.scatter_target
            }
        }
    };

    layout.clamp_target(raw_target)
}

pub fn scatter_corner(personality: GhostPersonality, layout: &LevelLayout) -> IVec2 {
    match personality {
        GhostPersonality::Blinky => IVec2::new(layout.width - 2, 1),
        GhostPersonality::Pinky => IVec2::new(1, 1),
        GhostPersonality::Inky => IVec2::new(layout.width - 2, layout.height - 2),
        GhostPersonality::Clyde => IVec2::new(1, layout.height - 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_layout() -> LevelLayout {
        LevelLayout::from_ascii(&[
            "#####",
            "#P..#",
            "#...#",
            "#..G#",
            "#####",
        ])
    }

    fn sample_ghost(personality: GhostPersonality, scatter_target: IVec2) -> Ghost {
        Ghost {
            personality,
            home_tile: IVec2::new(3, 3),
            scatter_target,
            returning_home: false,
        }
    }

    #[test]
    fn chooses_closest_direction_toward_target() {
        let options = [Direction::Up, Direction::Left, Direction::Right];
        let direction = choose_direction_toward(IVec2::new(5, 5), IVec2::new(8, 5), &options);
        assert_eq!(direction, Direction::Right);
    }

    #[test]
    fn chooses_farthest_direction_away_from_target() {
        let options = [Direction::Up, Direction::Left, Direction::Right];
        let direction = choose_direction_away(IVec2::new(5, 5), IVec2::new(8, 5), &options);
        assert_eq!(direction, Direction::Left);
    }

    #[test]
    fn clyde_switches_to_scatter_when_close() {
        let layout = sample_layout();
        let ghost = sample_ghost(GhostPersonality::Clyde, IVec2::new(1, 3));
        let target = chase_target(
            &ghost,
            IVec2::new(4, 4),
            IVec2::new(3, 3),
            Direction::Left,
            IVec2::new(0, 0),
            &layout,
        );

        assert_eq!(target, IVec2::new(1, 3));
    }

    #[test]
    fn scatter_corner_uses_expected_quadrants() {
        let layout = sample_layout();
        assert_eq!(
            scatter_corner(GhostPersonality::Blinky, &layout),
            IVec2::new(3, 1)
        );
        assert_eq!(
            scatter_corner(GhostPersonality::Clyde, &layout),
            IVec2::new(1, 3)
        );
    }
}
