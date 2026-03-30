# Tetris — Modernized 2D (Bevy)

A clean, modern Tetris built with Bevy using only primitive shapes (rectangles). No textures, no audio.

---

## Visual Style

- Each cell is a filled rounded-rectangle with a thin gap between cells (grid visible through gaps).
- Each tetromino type has a distinct **HDR color** that triggers bloom glow.
- Background is near-black; the playfield has a subtle dark-gray border.
- Ghost piece: same shape as active piece, drawn at its hard-drop destination with low alpha.
- Particle burst on line clear (small colored squares scatter outward and fade).
- Smooth animations: pieces lock with a brief flash, cleared rows collapse with a slide-down tween.

## Window

| Property | Value |
|---|---|
| Dimensions | 800 × 720 px |
| Title | "Tetris" |
| Clear color | `#0A0A0A` (near-black) |
| Resizable | No |

## Playfield

| Property | Value |
|---|---|
| Grid | 10 columns × 20 visible rows (+ 2 hidden buffer rows above, invisible) |
| Cell size | 30 px |
| Cell gap | 2 px (gap between adjacent cells) |
| Border | 2 px, color `#2A2A2A` |
| Origin | Bottom-left of playfield = row 0, col 0 |

Cells in buffer rows (20–21) are never rendered — they exist only for spawn/rotation logic.

## Tetrominoes

Standard 7-piece set (I, O, T, S, Z, J, L) defined as 4-cell patterns relative to a rotation center.

| Piece | Color (HDR sRGB) | Shape |
|---|---|---|
| I | Cyan `(0.0, 4.0, 4.0)` | `....` / `XXXX` (horizontal bar) |
| O | Yellow `(4.0, 4.0, 0.0)` | `XX` / `XX` (2×2 square) |
| T | Purple `(3.5, 0.5, 4.0)` | `.X.` / `XXX` |
| S | Green `(0.5, 4.0, 0.5)` | `.XX` / `XX.` |
| Z | Red `(4.0, 0.5, 0.5)` | `XX.` / `.XX` |
| J | Blue `(0.5, 0.5, 4.0)` | `X..` / `XXX` |
| L | Orange `(4.0, 2.0, 0.0)` | `..X` / `XXX` |

### Spawning

- Pieces spawn in **rotation state 0**, centered horizontally: columns 3–6 (0-indexed), in the buffer rows (rows 20–21).
- After spawn, gravity immediately pulls the piece into the visible area.
- If the spawned piece overlaps filled cells → **game over** (block out).

### Rotation

- **Super Rotation System (SRS)** — 4 rotation states (0 → R → 2 → L) with wall-kick tables.
- O-piece does not rotate.
- Wall kicks: try 5 offsets per rotation attempt; accept the first that fits.
- **I-piece uses a separate wall-kick table** from J/L/S/T/Z (per SRS spec).
- Reference: [SRS wall-kick data (tetris.wiki)](https://tetris.wiki/Super_Rotation_System)

## Core Mechanics

### Gravity & Drop

- Pieces fall one row per **gravity tick**. Tick interval decreases with level.
- **Soft drop**: hold Down to multiply gravity speed ×20.
- **Hard drop**: press Up/Space to instantly place piece at ghost position, lock immediately.

### Locking

- **Lock delay**: 0.5 s after a piece lands on a surface (i.e., cannot move down).
- Lock delay **pauses** if the piece is lifted off the surface (e.g., rotating sideways off a ledge).
- Lock delay **resets** on successful move or rotate (up to 15 resets max per piece) — prevents infinite stalling.
- Piece locks when delay expires or on hard drop.

### Piece Lifecycle

Lock → Clear lines → Spawn next piece (no spawn delay).

### Line Clears

- Any fully filled row is cleared.
- Rows above collapse downward.
- Simultaneous multi-line clears (1–4 lines) score progressively more.

### Scoring

| Action | Points |
|---|---|
| Single | 100 × level |
| Double | 300 × level |
| Triple | 500 × level |
| Tetris (4 lines) | 800 × level |
| Soft drop | 1 per row |
| Hard drop | 2 per row |
| T-Spin (bonus) | varies (stretch goal) |

### Leveling

- Level starts at 1.
- Level increases every 10 lines cleared.
- Gravity tick interval: `(0.8 − (level − 1) × 0.007) ^ (level − 1)` seconds (NES-inspired curve).
- Hard floor: gravity interval never goes below **0.05 s** regardless of level.

### Piece Randomizer

- **7-bag random generator**: shuffle all 7 pieces into a bag; draw sequentially. When bag is empty, generate a new bag. Guarantees no drought longer than 12 pieces.

### Hold

- Press hold key (C / Shift) to store the current piece and swap in the previously held piece (or draw next if hold is empty).
- The held piece always returns to **rotation state 0** when retrieved.
- A piece can only be held **once** per placement (flag resets on lock).

### Next Queue

- Show the next **5** upcoming pieces in a sidebar.

## UI Layout

```
┌──────────────────────────────────────────┐
│                                          │
│   ┌─────┐  ┌────────────────┐  ┌─────┐  │
│   │HOLD │  │                │  │NEXT │  │
│   │     │  │                │  │  1  │  │
│   └─────┘  │                │  │  2  │  │
│            │   PLAYFIELD    │  │  3  │  │
│   SCORE   │   10 × 20      │  │  4  │  │
│   LEVEL   │                │  │  5  │  │
│   LINES   │                │  └─────┘  │
│            │                │           │
│            └────────────────┘           │
│                                          │
└──────────────────────────────────────────┘
```

- HUD text rendered with `Text2d` + `TextFont`.
- Hold box and Next queue show pieces as mini grids of colored rectangles at **70% scale** (cell size × 0.7).
- Playfield is centered horizontally; sidebars are left/right of it with 20 px margins.

## Controls

| Action | Keys |
|---|---|
| Move left | Left Arrow / A |
| Move right | Right Arrow / D |
| Soft drop | Down Arrow / S |
| Hard drop | Up Arrow / Space |
| Rotate CW | X / E |
| Rotate CCW | Z / Q |
| Hold | C / Shift |
| Pause | Escape |
| Restart (game over) | Enter |

- **DAS (Delayed Auto Shift)**: initial delay 0.17 s, then auto-repeat rate (ARR) 0.05 s. Applies to left/right movement only.
- **Rotate, hold, and hard drop are single-fire** — one action per key press, no repeat.

## State Machine

```
StartScreen → Playing → GameOver
                 ↕
              Paused
```

- **StartScreen**: title + "Press Enter to start" prompt.
- **Playing**: active gameplay.
- **Paused** (sub-state of Playing): overlay text, systems frozen.
- **GameOver**: triggered by **block out** (spawned piece overlaps filled cells) or **lock out** (piece locks entirely above the visible area). Shows final score + "Press Enter to restart".
- **Restart** resets board, score, level, lines, hold, and bag to initial state.

## Non-Goals (explicitly excluded)

- Audio / music
- Textures / sprite sheets
- Online multiplayer
- Leaderboard persistence (score lives only for the session)
- T-Spin detection (stretch goal, not required for v1)

---

## Task List — Implementation Order

The tasks are ordered for incremental learning: each phase produces something visible and testable before the next begins.

### Phase 1: Window, Camera & Constants
- [x] Set up window size, title, and clear color
- [x] Spawn a 2D camera with bloom / HDR enabled
- [x] Create `constants.rs` with grid dimensions, cell size, colors, speeds

### Phase 2: Playfield Rendering
- [x] Create `board.rs` module with a `Board` resource (2D array of `Option<Color>`)
- [x] Render the empty playfield as a bordered rectangle
- [x] Render filled cells as colored rounded-rectangles with gaps

### Phase 3: Tetromino Data & Spawning
- [x] Define the 7 tetromino shapes and their rotation states in `tetromino.rs`
- [x] Implement the 7-bag randomizer
- [x] Spawn a piece at the top of the board and render it

### Phase 4: Movement & Input
- [x] Implement left/right movement with collision detection against walls and filled cells
- [x] Implement DAS (delayed auto-shift) for held keys
- [x] Implement soft drop (accelerated gravity)
- [x] Implement hard drop (instant placement)

### Phase 5: Rotation & Wall Kicks
- [x] Implement SRS rotation (CW and CCW)
- [x] Add wall-kick offset tables
- [x] Test rotation near walls and floor

### Phase 6: Gravity & Lock Delay
- [x] Implement gravity tick timer that moves piece down each interval
- [x] Implement lock delay (0.5 s) with reset-on-move (max 15 resets)
- [x] Lock piece into the board when delay expires

### Phase 7: Line Clears & Collapse
- [ ] Detect fully filled rows after piece locks
- [ ] Clear filled rows and shift rows above downward
- [ ] Add line-clear flash animation

### Phase 8: Scoring, Level & HUD
- [ ] Create `resources.rs` with Score, Level, Lines Cleared
- [ ] Implement scoring table (single/double/triple/tetris + drop bonuses)
- [ ] Implement leveling (every 10 lines) and gravity curve
- [ ] Render HUD: score, level, lines cleared as `Text2d`

### Phase 9: Ghost Piece, Hold & Next Queue
- [ ] Render ghost piece (translucent copy at hard-drop position)
- [ ] Implement hold mechanic (swap current piece with hold slot, once per lock)
- [ ] Render hold box and next-5 queue in sidebars

### Phase 10: State Machine & Menus
- [ ] Define `AppState` enum (StartScreen, Playing, GameOver) and `PlayState` sub-state (Running, Paused)
- [ ] Build start screen with title and prompt
- [ ] Build game-over screen with final score and restart
- [ ] Implement pause overlay (Escape toggles Running ↔ Paused)
- [ ] Gate all gameplay systems with `run_if(in_state(...))` and `DespawnOnExit`

### Phase 11: Juice & Polish
- [ ] Add particle burst effect on line clears
- [ ] Add brief lock-flash when a piece locks
- [ ] Smooth row-collapse animation (slide-down tween)
- [ ] Tune HDR colors and bloom intensity for visual pop
