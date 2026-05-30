# Arkanoid — Implementation Plan

A phased plan to recreate the classic **Arkanoid** (Taito, 1986) in Bevy as a learning exercise. Each phase builds only on systems from earlier phases. Phases are feature- and asset-focused; a short **Bevy patterns introduced** note in each phase ties the work to the engine concepts being learned.

> No `SPEC.md` exists yet — this plan is derived from the canonical 1986 arcade design (Vaus paddle, energy ball, 33 rounds, DOH boss). Canonical numbers are noted inline where they define a system (3 lives, 33 rounds, extra life every 20,000 points, silver bricks worth 50 × round number).

---

## Asset Folder Structure

The tree below is a hand-off manifest for the asset creator. Every named asset from the phases maps to a concrete file path. The `(Pn)` tag on each entry is the phase it's needed in — lower numbers are higher priority. The `W×H` figure is the target pixel size. Paths live under the project's `assets/` directory (Bevy loads everything from there by relative path).

**Canvas & layout — the dimensions below are derived from this:**
- **Window:** 800 × 600 px (landscape).
- **Wall frame:** 20 px thick on the top, left, and right edges; the bottom is open.
- **Playfield interior:** 760 px wide (x 20 → 780), top wall at y 20, open at the bottom.
- **Brick grid:** 13 columns of 56 × 28 px bricks (728 px wide), centered with ~16 px clearance to each side wall. All brick variants (colored, silver, gold) share the 56 × 28 cell.
- **Vaus:** default paddle ≈ 1.7 bricks wide; rests near the bottom of the playfield.

**Conventions for the asset maker:**
- **Sprites:** PNG with transparency. Author at the exact `W×H` listed (1:1 pixel scale — no pre-scaling), with a consistent palette across the set. Pixel-art crispness preferred; avoid anti-aliased soft edges.
- **Animations / multi-state art:** deliver as a horizontal sprite sheet (frames left-to-right) *and* the individual frames, so we can use whichever Bevy texture-atlas path is cleaner. These are flagged `[sheet]` below.
- **Audio:** `.ogg` (Bevy's preferred format). SFX short and punchy; jingles/themes are noted under `music/`.
- **Fonts:** a single arcade-style bitmap/TTF font for all HUD text.
- `levels/` holds `.ron` data files authored by the developers, **not** art — listed only so the layout is complete.

```
assets/
├── sprites/
│   ├── vaus/
│   │   ├── vaus.png                  (P1)  96×24   default paddle
│   │   ├── vaus-expanded.png         (P5) 160×24   Expand power-up paddle
│   │   └── vaus-life-icon.png        (P3)  48×12   small HUD life icon
│   ├── ball/
│   │   └── ball.png                  (P1)  16×16   energy ball
│   ├── playfield/
│   │   ├── border-frame.png          (P1) 800×600  left/right/top wall frame (20px thick, transparent center)
│   │   ├── warp-gate.png             (P5)  32×80   Break power-up right-edge exit
│   │   └── spawn-gate.png   [sheet]  (P6)  48×24   top enemy gate (closed→open), per frame
│   ├── bricks/
│   │   ├── brick-white.png           (P2)  56×28
│   │   ├── brick-orange.png          (P2)  56×28
│   │   ├── brick-cyan.png            (P2)  56×28
│   │   ├── brick-green.png           (P2)  56×28
│   │   ├── brick-red.png             (P2)  56×28
│   │   ├── brick-blue.png            (P2)  56×28
│   │   ├── brick-pink.png            (P2)  56×28
│   │   ├── brick-yellow.png          (P2)  56×28
│   │   ├── brick-silver.png [sheet]  (P4)  56×28   multi-hit, damage states per frame
│   │   └── brick-gold.png            (P4)  56×28   indestructible
│   ├── capsules/
│   │   ├── capsule-c-catch.png       (P5)  32×16
│   │   ├── capsule-l-laser.png       (P5)  32×16
│   │   ├── capsule-e-expand.png      (P5)  32×16
│   │   ├── capsule-d-disruption.png  (P5)  32×16
│   │   ├── capsule-s-slow.png        (P5)  32×16
│   │   ├── capsule-b-break.png       (P5)  32×16
│   │   └── capsule-p-player.png      (P5)  32×16
│   ├── weapons/
│   │   └── laser-bolt.png            (P5)   8×24   Laser power-up projectile
│   ├── enemies/
│   │   ├── enemy-pyramid.png [sheet] (P6)  32×32   per frame
│   │   ├── enemy-molecule.png[sheet] (P6)  32×32   per frame
│   │   └── enemy-cube.png    [sheet] (P6)  32×32   per frame
│   └── boss/
│       └── doh.png          [sheet]  (P7) 128×144  DOH face + animation frames, per frame
├── vfx/
│   ├── capsule-catch-flash.png [sheet] (P5)  64×64   per frame
│   ├── laser-impact.png        [sheet] (P5)  32×32   per frame
│   ├── enemy-destroy-burst.png [sheet] (P6)  48×48   per frame
│   ├── ball-trail.png                  (P7)  16×16   trail particle
│   └── doh-defeat-explosion.png[sheet] (P7) 160×160  per frame
├── audio/
│   ├── sfx/
│   │   ├── wall-bounce.ogg           (P1)
│   │   ├── paddle-bounce.ogg         (P1)
│   │   ├── brick-break.ogg           (P2)
│   │   ├── ball-lost.ogg             (P3)
│   │   ├── hard-brick-clink.ogg      (P4) silver/gold hit
│   │   ├── ball-speedup.ogg          (P4)
│   │   ├── capsule-catch.ogg         (P5)
│   │   ├── laser-fire.ogg            (P5)
│   │   ├── expand.ogg                (P5)
│   │   ├── multiball.ogg             (P5) Disruption
│   │   ├── slow.ogg                  (P5)
│   │   ├── extra-life.ogg            (P5) Player capsule
│   │   ├── warp-gate-open.ogg        (P5) Break
│   │   ├── enemy-spawn.ogg           (P6)
│   │   └── enemy-destroyed.ogg       (P6)
│   └── music/
│       ├── round-clear.ogg           (P3) jingle
│       ├── game-over.ogg             (P3) jingle
│       ├── title-theme.ogg           (P7)
│       ├── intro-story.ogg           (P7) story-screen cue
│       └── ending-theme.ogg          (P7) victory theme
├── ui/
│   ├── title-screen.png              (P3) 800×600  title / attract art (full screen)
│   ├── round-ready-banner.png        (P3) 320×64   "ROUND n READY"
│   ├── intro-story-screen.png        (P7) 800×600  opening story art (full screen)
│   ├── victory-screen.png            (P7) 800×600  ending / victory art (full screen)
│   ├── high-score-table.png          (P7) 480×400  score-table frame
│   └── fonts/
│       └── arcade.ttf                (P2)          HUD / score font (vector, no fixed size)
└── levels/                           (data, authored by devs — not art)
    ├── round-01.level.ron            (P3)
    ├── round-02.level.ron … round-32 (P7)
    └── round-33-doh.level.ron        (P7) DOH boss round
```

---

## Phase 1 — Vaus, Ball, and Walls

Deliver the raw kinetics: a paddle you can move and a ball that bounces around the playfield. This is the feel of the game before any objective exists.

- Bordered playfield: left, right, and top walls; open bottom.
- The Vaus paddle moves horizontally along the bottom, clamped inside the walls (keyboard left/right and mouse/pointer control).
- The ball rests on the paddle, then launches on input.
- Ball bounces off the side and top walls and off the paddle.
- Paddle-reflection model: the horizontal position where the ball strikes the paddle determines the rebound angle (center = straight up, edges = sharp angle).
- Ball that exits the bottom is simply re-served on the paddle (no lives yet).
- Bounce SFX on wall and paddle contact.

**Bevy patterns introduced:** plugin-per-domain (`PaddlePlugin`, `BallPlugin`), marker + data components (`Paddle`, `Ball`, `Velocity`), the `InputActions` resource driven by an input plugin, fixed-timestep movement, and `With`/`Without` query filters.

### Assets

**2D Sprites**
- Vaus paddle (default size)
- Energy ball
- Playfield border / wall frame

**Audio**
- Wall bounce SFX
- Paddle bounce SFX

---

## Phase 2 — Bricks and Round Clear

Add the objective. Now the ball destroys bricks, you earn points, and clearing the field ends the round.

- Grid of single-hit colored bricks (white, orange, cyan, green, red, blue, pink, yellow), each with its own point value.
- Ball–brick collision destroys the brick, deflects the ball, and adds score.
- Round-clear detection: when all destructible bricks are gone, advance to the next round layout.
- On-screen score display.
- Round transition reloads the playfield with the next layout (a second hand-built layout is enough here).

**Bevy patterns introduced:** the `AppState` machine (`StartScreen → Playing`), `OnEnter`/`OnExit` for spawn/cleanup symmetry, `DespawnOnExit` for auto-cleanup of round entities, buffered messages (`BrickDestroyed`, `ScoreChanged`), and the `Score` resource.

### Assets

**2D Sprites**
- Colored brick set (8 colors)

**Audio**
- Brick break SFX

**UI**
- Score readout (1UP / HIGH SCORE)

---

## Phase 3 — Lives, Round Flow, and Game Over

Wrap the loop into a real game: limited lives, a serve sequence, round intros, and a start/lose flow.

- Three Vaus lives; losing the ball off the bottom costs a life and re-serves.
- Game Over when lives are exhausted, with a path back to start/restart.
- "ROUND n READY" intro before each round; current round number on screen.
- Title / start screen that begins a run.
- One fully designed Round 1 layout as the canonical first stage.

**Bevy patterns introduced:** sub-states for the serve/play/ready flow (e.g. a `PlayState` sub-state under `AppState::Playing`), state-scoped entities for HUD and ball, a `Lives` resource, and observers reacting to the ball-lost event.

### Assets

**2D Sprites**
- Vaus life icon (HUD)

**Audio**
- Ball-lost SFX
- Round-clear jingle
- Game Over jingle

**UI**
- Title screen
- "ROUND n READY" banner
- Lives indicator

---

**Vertical slice checkpoint — A complete, playable round: start the game, move the Vaus, serve the ball, clear a designed brick layout for points, lose lives when the ball drops, and hit Game Over / restart.**

---

## Phase 4 — Brick Variety and Difficulty Ramp

Deepen the core mechanic with bricks that resist or block the ball, plus rising ball speed.

- Silver bricks: multi-hit, with required hits scaling by round; worth 50 × round number.
- Gold bricks: indestructible obstacles that shape the playfield and never count toward round clear.
- Ball speed progression: the ball accelerates over time within a round and at gameplay milestones, then resets on serve.
- Visual hit-feedback on silver bricks as they take damage.

**Bevy patterns introduced:** richer data components (durability / hit-count), `Changed<T>` queries to drive hit-state visuals only when a brick is struck, and excluding indestructible bricks from clear-checks via query filters.

### Assets

**2D Sprites**
- Silver brick (with damage states)
- Gold (indestructible) brick

**Audio**
- Hard-brick clink SFX (indestructible / silver hit)
- Ball speed-up cue

---

## Phase 5 — Power-up Capsules

Introduce the signature Arkanoid power-ups that drop from bricks and transform the Vaus.

- Special bricks release a falling capsule when destroyed; the Vaus catches it by touch.
- Power-up set:
  - **C — Catch:** the ball sticks to the paddle and re-launches on input.
  - **L — Laser:** the Vaus can fire lasers to destroy bricks.
  - **E — Expand:** the paddle grows wider.
  - **D — Disruption:** splits the ball into multiple balls.
  - **S — Slow:** reduces ball speed.
  - **B — Break:** opens a warp exit on the right edge to skip to the next round.
  - **P — Player:** awards an extra life.
- Only one stateful power-up is active at a time (multi-ball coexists); effects reset on losing a life.
- Laser fire and the Break warp-exit gate behavior.

**Bevy patterns introduced:** messages for capsule spawn/catch events, observers reacting to a caught capsule, `Timer`-driven power-up durations, and marker components representing the active power-up state.

### Assets

**2D Sprites**
- Seven lettered capsule sprites (C, L, E, D, S, B, P)
- Expanded Vaus paddle
- Laser bolt
- Warp-exit gate

**VFX**
- Capsule-catch flash
- Laser impact

**Audio**
- Capsule catch SFX
- Laser fire SFX
- Expand SFX
- Multi-ball (Disruption) SFX
- Slow SFX
- Extra-life SFX
- Warp-gate open SFX

---

## Phase 6 — Enemy Aliens

Add the descending enemies that intrude into the playfield and disrupt the ball.

- Enemies emerge from top gates, drift down along wandering paths, and bounce the ball off-course.
- A small representative set of enemy types (e.g. Pyramid/Cone, Molecule, Cube) testing different movement.
- Enemies are destroyed by ball or laser contact for points; enemies reaching the bottom simply exit.
- Timed enemy spawning from the gates.

**Bevy patterns introduced:** `Timer` resources/components for spawn cadence, dedicated movement/AI systems in an `EnemyPlugin`, and observers on enemy spawn/death.

### Assets

**2D Sprites**
- Enemy: Pyramid / Cone
- Enemy: Molecule
- Enemy: Cube
- Spawn-gate (open/closed states)

**VFX**
- Enemy-destroy burst

**Audio**
- Enemy spawn SFX
- Enemy destroyed SFX

---

## Phase 7 — Full Round Set, DOH Boss, and Polish

Expand to the full game: the complete round progression, the final boss, scoring meta-systems, and presentation polish.

- Full 33-round progression driven by data-defined brick layouts (one layout file per round).
- Final round: the **DOH** boss — a face that spawns enemies and absorbs roughly 16 ball hits; defeating it triggers victory.
- Extra life awarded at each 20,000-point threshold; high-score tracking and an ending sequence.
- Intro story text / attract screen.
- Presentation polish: ball trail, screen shake on impacts, smooth round transitions.

**Bevy patterns introduced:** data-driven level loading from `assets/` (RON layout files), a dedicated boss state machine, and computed/derived state for end-of-game conditions.

### Assets

**2D Sprites**
- Remaining round layouts (data-defined brick arrangements, rounds 2–32)
- DOH boss (with animation frames)
- Ending / victory art

**VFX**
- Ball trail
- Screen-shake impact feedback
- DOH defeat explosion

**Audio**
- Title theme
- Intro story cue
- Ending / victory theme
- Full SFX balance pass

**UI**
- Intro story screen
- High-score table
- Victory / ending screen
