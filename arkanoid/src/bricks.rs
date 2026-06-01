use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{Brick, BrickColor};
use crate::constants::*;
use crate::resources::{Round, Score};
use crate::states::{AppState, PlayState};

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
            // Each round's bricks spawn as its "ROUND n READY" intro begins.
            .add_systems(OnEnter(PlayState::Ready), spawn_current_round)
            .add_systems(
                Update,
                (
                    apply_score.run_if(in_state(AppState::Playing)),
                    check_round_clear.run_if(in_state(PlayState::Running)),
                ),
            );
    }
}

/// Hand-built round layouts. Each string is one row of 9 cells; characters map to
/// brick colors via [`BrickColor::from_code`] and `.` is an empty cell. Rounds beyond
/// the list wrap around (Phase 7 replaces these with data-driven RON layouts).
const LAYOUTS: &[&[&str]] = &[
    // Round 1 — a designed diamond-and-pyramid stage.
    &[
        "....Y....",
        "...YPY...",
        "..YPBPY..",
        ".YPBRBPY.",
        "YPBRGRBPY",
        ".CCCCCCC.",
        "..GGGGG..",
        "...OOO...",
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

/// When the board is empty, advance the round counter and drop back into `Ready`, which
/// shows the next "ROUND n READY" intro, spawns the new layout, and re-serves the ball.
/// Works for any removal — gameplay or the debug "destroy all" key — since it just
/// checks whether bricks remain.
fn check_round_clear(
    bricks: Query<(), With<Brick>>,
    mut round: ResMut<Round>,
    mut next: ResMut<NextState<PlayState>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    if !bricks.is_empty() {
        return;
    }
    round.0 += 1;
    commands.spawn((
        AudioPlayer(assets.music.round_clear.clone()),
        PlaybackSettings::DESPAWN,
    ));
    next.set(PlayState::Ready);
}
