use crate::constants::*;

/// Convert grid (col, row) to world-space center of that tile.
/// Row 0 = top, Row 14 = bottom. Y increases upward.
pub fn tile_to_world(col: usize, row: usize) -> (f32, f32) {
    let x = LEVEL_ORIGIN_X + col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let y = LEVEL_ORIGIN_Y + (LEVEL_HEIGHT - 1 - row) as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    (x, y)
}

/// Level 1-1 tile grid (211 columns × 15 rows).
///
/// Tile legend:
/// `.` empty   `#` ground   `B` brick   `?` question (coin)   `M` question (mushroom)
/// `[` pipe-top-left   `]` pipe-top-right   `{` pipe-body-left   `}` pipe-body-right
/// `X` solid (staircase)   `S` spawn   `G` Goomba   `K` Koopa   `F` flagpole
///
/// Row 0 = top of screen, Row 14 = bottom. Ground is 2 tiles thick (rows 13–14).

pub const LEVEL_WIDTH: usize = 211;
pub const LEVEL_HEIGHT: usize = 15;

pub fn level_1_1() -> [[char; LEVEL_WIDTH]; LEVEL_HEIGHT] {
    let mut g = [['.'; LEVEL_WIDTH]; LEVEL_HEIGHT];

    // ── Ground (rows 13–14) with pits ──
    for col in 0..LEVEL_WIDTH {
        let is_pit = matches!(col, 69..=70 | 86..=88 | 153..=155);
        if !is_pit {
            g[13][col] = '#';
            g[14][col] = '#';
        }
    }

    // ── Mario spawn ──
    g[12][3] = 'S';

    // ── Flagpole (col 198, rows 3–12) ──
    for row in 3..=12 {
        g[row][198] = 'F';
    }

    // ── Question blocks — coins ──
    set_tiles(&mut g, '?', &[
        (16, 9), (21, 5), (22, 9), (23, 9),
        (94, 9), (106, 5), (109, 5), (109, 9), (112, 5),
        (130, 9), (170, 9),
    ]);

    // ── Question blocks — mushroom / power-up ──
    set_tiles(&mut g, 'M', &[(20, 9), (78, 9), (129, 9), (171, 9)]);

    // ── Brick blocks ──
    set_tiles(&mut g, 'B', &[
        (20, 5), (22, 5), (24, 9),
        (77, 9), (79, 9), (80, 5), (81, 5),
        (91, 9), (92, 9), (93, 9), (94, 5), (100, 5), (101, 9),
        (106, 9), (118, 9),
        (121, 5), (122, 5), (123, 5),
        (128, 9), (129, 5), (130, 5), (131, 9),
        (168, 5), (169, 5), (168, 9), (169, 9),
    ]);

    // ── Pipes — (left_col, top_row) ──
    // Body fills from top_row+1 down to row 13 (ground surface).
    pipe(&mut g, 28, 12);   // height 2
    pipe(&mut g, 38, 11);   // height 3
    pipe(&mut g, 46, 10);   // height 4
    pipe(&mut g, 57, 10);   // height 4
    pipe(&mut g, 163, 12);  // height 2
    pipe(&mut g, 179, 12);  // height 2

    // ── Staircases ──
    staircase(&mut g, 134, 4, true);   // ascending right
    staircase(&mut g, 140, 4, false);  // descending right
    staircase(&mut g, 148, 4, true);   // ascending right + flat top:
    for row in 9..=12 {
        g[row][152] = 'X';
    }
    staircase(&mut g, 190, 8, true); // final grand staircase

    // ── Goombas ──
    set_tiles(&mut g, 'G', &[
        (22, 12), (40, 12), (51, 12), (52, 12),
        (82, 12), (83, 12), (97, 12), (98, 12),
        (114, 12), (115, 12), (124, 12), (125, 12),
        (128, 12), (129, 12), (174, 12), (175, 12),
    ]);

    // ── Koopa Troopa ──
    g[12][107] = 'K';

    g
}

fn set_tiles(g: &mut [[char; LEVEL_WIDTH]; LEVEL_HEIGHT], ch: char, positions: &[(usize, usize)]) {
    for &(col, row) in positions {
        g[row][col] = ch;
    }
}

fn pipe(g: &mut [[char; LEVEL_WIDTH]; LEVEL_HEIGHT], col: usize, top_row: usize) {
    g[top_row][col] = '[';
    g[top_row][col + 1] = ']';
    for row in (top_row + 1)..=13 {
        g[row][col] = '{';
        g[row][col + 1] = '}';
    }
}

fn staircase(
    g: &mut [[char; LEVEL_WIDTH]; LEVEL_HEIGHT],
    start_col: usize,
    height: usize,
    ascending: bool,
) {
    for i in 0..height {
        let col = start_col + i;
        let blocks = if ascending { i + 1 } else { height - i };
        for b in 0..blocks {
            g[12 - b][col] = 'X';
        }
    }
}
