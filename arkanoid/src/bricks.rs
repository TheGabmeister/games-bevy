use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{Brick, BrickKind, Indestructible, Silver};
use crate::constants::*;
use crate::resources::{Round, Score};
use crate::states::{AppState, PlayState};

/// A brick was destroyed, worth `points`, at world `position`. Consumed by scoring, the
/// ball-speed ramp, and the capsule-drop director.
#[derive(Message)]
pub struct BrickDestroyed {
    pub points: u32,
    pub position: Vec2,
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
                    update_silver_damage.run_if(in_state(AppState::Playing)),
                    check_round_clear.run_if(in_state(PlayState::Running)),
                ),
            );
    }
}

/// Hand-built round layouts. Each string is one row of 9 cells; characters map to brick
/// kinds via [`BrickKind::from_code`] (color codes, `S` = silver, `X` = gold) and `.` is
/// an empty cell. Rounds beyond the list wrap around (Phase 7 replaces these with
/// data-driven RON layouts).
const LAYOUTS: &[&[&str]] = &[
    // Round 1 — a diamond framed in silver, with two gold pillars guarding the base.
    &[
        "....S....",
        "...SPS...",
        "..SPBPS..",
        ".SPBRBPS.",
        "SPBRGRBPS",
        ".CCCCCCC.",
        "X.GGGGG.X",
        "...OOO...",
    ],
    // Round 2 — staggered checkerboard with a gold-capped silver wall over a solid base.
    &[
        "XR.R.R.RX",
        "O.O.O.O.O",
        "Y.Y.Y.Y.Y",
        "G.G.G.G.G",
        "SSSSSSSSS",
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

/// Required hits for a silver brick in `round`: a base value rising by one every
/// [`SILVER_HITS_ROUND_STEP`] rounds (classic-style scaling difficulty).
fn silver_hits(round: u32) -> u32 {
    SILVER_BASE_HITS + (round - 1) / SILVER_HITS_ROUND_STEP
}

/// Spawns every brick of the layout for `round` (1-based), wrapping past the list end.
fn spawn_round(commands: &mut Commands, assets: &GameAssets, round: u32) {
    let layout = LAYOUTS[(round as usize - 1) % LAYOUTS.len()];
    let bricks = &assets.sprites.bricks;
    for (row, line) in layout.iter().enumerate() {
        for (col, code) in line.chars().enumerate() {
            let Some(kind) = BrickKind::from_code(code) else {
                continue;
            };
            let pos = brick_position(row, col);
            let transform = Transform::from_xyz(pos.x, pos.y, Z_BRICK);
            let mut entity = commands.spawn((transform, DespawnOnExit(AppState::Playing)));
            match kind {
                BrickKind::Colored(color) => {
                    entity.insert((
                        Brick {
                            points: color.points(),
                            hits_remaining: 1,
                            max_hits: 1,
                        },
                        Sprite::from_image(bricks.handle(color)),
                    ));
                }
                BrickKind::Silver => {
                    let hits = silver_hits(round);
                    entity.insert((
                        Brick {
                            points: SILVER_POINTS_PER_ROUND * round,
                            hits_remaining: hits,
                            max_hits: hits,
                        },
                        Silver,
                        Sprite::from_image(bricks.silver_frame(0)),
                    ));
                }
                BrickKind::Gold => {
                    entity.insert((
                        Brick {
                            points: 0,
                            hits_remaining: u32::MAX,
                            max_hits: u32::MAX,
                        },
                        Indestructible,
                        Sprite::from_image(bricks.gold.clone()),
                    ));
                }
            }
        }
    }
}

/// Spawns the current round's bricks. Runs `OnEnter(Ready)`, which fires only at round
/// boundaries — so it first clears any bricks left over from the previous round
/// (indestructible gold survives a round-clear and would otherwise stack up).
fn spawn_current_round(
    mut commands: Commands,
    assets: Res<GameAssets>,
    round: Res<Round>,
    existing: Query<Entity, With<Brick>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }
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

/// Swaps a silver brick to its cracked sprite as it takes damage. Runs only on the frame a
/// brick's `hits_remaining` changes thanks to `Changed<Brick>` (also fires on spawn, which
/// just (re)sets the pristine frame).
#[allow(clippy::type_complexity)]
fn update_silver_damage(
    assets: Res<GameAssets>,
    mut silver: Query<(&Brick, &mut Sprite), (With<Silver>, Changed<Brick>)>,
) {
    for (brick, mut sprite) in &mut silver {
        let damage = brick.max_hits - brick.hits_remaining;
        sprite.image = assets.sprites.bricks.silver_frame(damage);
    }
}

/// When no destructible bricks remain, advance the round counter and drop back into
/// `Ready`, which shows the next "ROUND n READY" intro, spawns the new layout, and
/// re-serves the ball. Indestructible (gold) bricks are excluded from the count via a
/// query filter — a board of only gold still counts as cleared. Works for any removal —
/// gameplay or the debug "destroy all" key — since it just checks whether bricks remain.
fn check_round_clear(
    bricks: Query<(), (With<Brick>, Without<Indestructible>)>,
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
