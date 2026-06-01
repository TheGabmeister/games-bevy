use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{Ball, Brick, BrickColor, Velocity};
use crate::constants::*;
use crate::resources::{Round, Score};
use crate::states::AppState;

/// A brick was destroyed, worth `points`. Consumed by scoring (and later VFX/audio).
#[derive(Message)]
pub struct BrickDestroyed {
    pub points: u32,
}

/// The score changed; the HUD listens for this to refresh its readout.
#[derive(Message)]
pub struct ScoreChanged;

pub struct BrickPlugin;

impl Plugin for BrickPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BrickDestroyed>()
            .add_message::<ScoreChanged>()
            .add_systems(OnEnter(AppState::Playing), spawn_current_round)
            .add_systems(
                Update,
                (apply_score, check_round_clear).run_if(in_state(AppState::Playing)),
            );
    }
}

/// Hand-built round layouts. Each string is one row of 9 cells; characters map to
/// brick colors via [`BrickColor::from_code`] and `.` is an empty cell. Rounds beyond
/// the list wrap around (Phase 7 replaces these with data-driven RON layouts).
const LAYOUTS: &[&[&str]] = &[
    // Round 1 — full rainbow wall.
    &[
        "YYYYYYYYY",
        "PPPPPPPPP",
        "BBBBBBBBB",
        "RRRRRRRRR",
        "GGGGGGGGG",
        "CCCCCCCCC",
        "OOOOOOOOO",
        "WWWWWWWWW",
    ],
    // Round 2 — staggered checkerboard with a solid base row.
    &[
        "R.R.R.R.R",
        "O.O.O.O.O",
        "Y.Y.Y.Y.Y",
        "G.G.G.G.G",
        ".B.B.B.B.",
        ".C.C.C.C.",
        ".P.P.P.P.",
        "WWWWWWWWW",
    ],
];

/// World position of the center of brick cell `(row, col)`.
fn brick_position(row: usize, col: usize) -> Vec2 {
    let grid_left = -(BRICK_COLS as f32 * BRICK_WIDTH) / 2.0 + BRICK_WIDTH / 2.0;
    Vec2::new(
        grid_left + col as f32 * BRICK_WIDTH,
        BRICK_FIELD_TOP - row as f32 * BRICK_HEIGHT,
    )
}

/// Spawns every brick of the layout for `round` (1-based), wrapping past the list end.
fn spawn_round(commands: &mut Commands, assets: &GameAssets, round: u32) {
    let layout = LAYOUTS[(round as usize - 1) % LAYOUTS.len()];
    for (row, line) in layout.iter().enumerate() {
        for (col, code) in line.chars().enumerate() {
            let Some(color) = BrickColor::from_code(code) else {
                continue;
            };
            let pos = brick_position(row, col);
            commands.spawn((
                Brick {
                    points: color.points(),
                },
                Sprite::from_image(assets.sprites.bricks.handle(color)),
                Transform::from_xyz(pos.x, pos.y, Z_BRICK),
                DespawnOnExit(AppState::Playing),
            ));
        }
    }
}

fn spawn_current_round(mut commands: Commands, assets: Res<GameAssets>, round: Res<Round>) {
    spawn_round(&mut commands, &assets, round.0);
}

/// Adds the points from destroyed bricks to the score and signals the HUD.
fn apply_score(
    mut destroyed: MessageReader<BrickDestroyed>,
    mut score: ResMut<Score>,
    mut changed: MessageWriter<ScoreChanged>,
) {
    let gained: u32 = destroyed.read().map(|ev| ev.points).sum();
    if gained > 0 {
        score.add(gained);
        changed.write(ScoreChanged);
    }
}

/// When the playfield is cleared of bricks, advance to the next round's layout and
/// re-serve the ball onto the paddle.
///
/// `had_bricks` latches once a round has spawned, so an empty query only counts as a
/// clear *after* bricks have existed — this avoids a startup race before the first
/// round spawns, and works no matter how the bricks were removed (gameplay or the
/// debug "destroy all" key).
fn check_round_clear(
    mut had_bricks: Local<bool>,
    bricks: Query<(), With<Brick>>,
    mut round: ResMut<Round>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut balls: Query<(&mut Ball, &mut Velocity)>,
) {
    if !bricks.is_empty() {
        *had_bricks = true;
        return;
    }
    if !*had_bricks {
        return;
    }
    *had_bricks = false;
    round.0 += 1;
    spawn_round(&mut commands, &assets, round.0);
    for (mut ball, mut velocity) in &mut balls {
        ball.stuck = true;
        velocity.0 = Vec2::ZERO;
    }
}
