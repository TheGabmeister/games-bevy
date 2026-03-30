use bevy::prelude::*;

use crate::constants::*;
use crate::resources::HoldPiece;
use crate::states::AppState;
use crate::tetromino::{PieceBag, RotationState, TetrominoKind};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct NextQueueCell {
    slot: usize,
    cell: usize,
}

#[derive(Component)]
struct HoldBoxCell(usize);

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

const SIDEBAR_X_RIGHT: f32 = PLAYFIELD_WIDTH / 2.0 + SIDEBAR_MARGIN + 45.0;
const SIDEBAR_X_LEFT: f32 = -(PLAYFIELD_WIDTH / 2.0 + SIDEBAR_MARGIN + 50.0);
const SIDEBAR_TOP_Y: f32 = PLAYFIELD_HEIGHT / 2.0 - 20.0;
const QUEUE_SLOT_SPACING: f32 = 55.0;
const QUEUE_FIRST_Y: f32 = SIDEBAR_TOP_Y - 50.0;
const HOLD_CENTER_Y: f32 = SIDEBAR_TOP_Y - 50.0;
const SIDEBAR_CELL_SIZE: f32 = CELL_SIZE * SIDEBAR_CELL_SCALE;
const SIDEBAR_CELL_INNER: f32 = SIDEBAR_CELL_SIZE - CELL_GAP;

const LABEL_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.4);
const LABEL_SIZE: f32 = 18.0;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SidebarPlugin;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_sidebar)
            .add_systems(
                Update,
                (sync_next_queue, sync_hold_box).run_if(in_state(AppState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

fn spawn_sidebar(mut commands: Commands) {
    // "NEXT" label
    commands.spawn((
        Text2d::new("NEXT"),
        TextFont {
            font_size: LABEL_SIZE,
            ..default()
        },
        TextColor(LABEL_COLOR),
        Transform::from_xyz(SIDEBAR_X_RIGHT, SIDEBAR_TOP_Y, Z_UI),
    ));

    // Next queue cells (5 slots x 4 cells)
    for slot in 0..NEXT_QUEUE_SIZE {
        for cell in 0..4 {
            commands.spawn((
                NextQueueCell { slot, cell },
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(SIDEBAR_CELL_INNER, SIDEBAR_CELL_INNER)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, Z_UI),
                Visibility::Hidden,
            ));
        }
    }

    // "HOLD" label
    commands.spawn((
        Text2d::new("HOLD"),
        TextFont {
            font_size: LABEL_SIZE,
            ..default()
        },
        TextColor(LABEL_COLOR),
        Transform::from_xyz(SIDEBAR_X_LEFT, SIDEBAR_TOP_Y, Z_UI),
    ));

    // Hold box cells (4 cells)
    for cell in 0..4 {
        commands.spawn((
            HoldBoxCell(cell),
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(SIDEBAR_CELL_INNER, SIDEBAR_CELL_INNER)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, Z_UI),
            Visibility::Hidden,
        ));
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Bounding-box center of a piece's R0 cells.
fn piece_center(kind: TetrominoKind) -> (f32, f32) {
    let cells = kind.cells(RotationState::R0);
    let min_r = cells.iter().map(|c| c.0 as f32).fold(f32::MAX, f32::min);
    let max_r = cells.iter().map(|c| c.0 as f32).fold(f32::MIN, f32::max);
    let min_c = cells.iter().map(|c| c.1 as f32).fold(f32::MAX, f32::min);
    let max_c = cells.iter().map(|c| c.1 as f32).fold(f32::MIN, f32::max);
    ((min_r + max_r + 1.0) / 2.0, (min_c + max_c + 1.0) / 2.0)
}

/// World position of a mini piece cell relative to a slot center.
fn mini_cell_pos(kind: TetrominoKind, cell_idx: usize, center: Vec2) -> Vec2 {
    let cells = kind.cells(RotationState::R0);
    let (mid_r, mid_c) = piece_center(kind);
    let (r, c) = cells[cell_idx];
    Vec2::new(
        center.x + (c as f32 + 0.5 - mid_c) * SIDEBAR_CELL_SIZE,
        center.y + (r as f32 + 0.5 - mid_r) * SIDEBAR_CELL_SIZE,
    )
}

// ---------------------------------------------------------------------------
// Sync systems
// ---------------------------------------------------------------------------

fn sync_next_queue(
    bag: Res<PieceBag>,
    mut query: Query<(&NextQueueCell, &mut Transform, &mut Sprite, &mut Visibility)>,
) {
    let upcoming = bag.peek(NEXT_QUEUE_SIZE);

    for (nq, mut transform, mut sprite, mut visibility) in &mut query {
        if nq.slot < upcoming.len() {
            let kind = upcoming[nq.slot];
            let center_y = QUEUE_FIRST_Y - nq.slot as f32 * QUEUE_SLOT_SPACING;
            let center = Vec2::new(SIDEBAR_X_RIGHT, center_y);
            let pos = mini_cell_pos(kind, nq.cell, center);
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
            sprite.color = kind.color();
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn sync_hold_box(
    hold: Res<HoldPiece>,
    mut query: Query<(&HoldBoxCell, &mut Transform, &mut Sprite, &mut Visibility)>,
) {
    let Some(kind) = hold.piece else {
        for (_, _, _, mut vis) in &mut query {
            *vis = Visibility::Hidden;
        }
        return;
    };

    let center = Vec2::new(SIDEBAR_X_LEFT, HOLD_CENTER_Y);
    let color = if hold.can_hold {
        kind.color()
    } else {
        kind.color().with_alpha(0.3)
    };

    for (hc, mut transform, mut sprite, mut visibility) in &mut query {
        let pos = mini_cell_pos(kind, hc.0, center);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
        sprite.color = color;
        *visibility = Visibility::Visible;
    }
}
