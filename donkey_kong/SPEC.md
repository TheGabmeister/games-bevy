# Donkey Kong (1981) 25m Stage - Game Specification

## 1. Purpose

This document defines the target design for converting the current Bevy template into a single-stage Donkey Kong style game.

It serves two jobs:

1. Describe the intended player-facing game behavior.
2. Describe the implementation boundaries clearly enough that the repo can evolve toward this design without guesswork.

This spec intentionally supersedes the current shooter-template gameplay.

## 2. Current Repo Baseline

At the time of writing, the repository still contains the Bevy template baseline:

- `AppState`: `StartScreen -> Playing -> GameOver`
- Free-movement player ship with laser shooting
- Static enemy row
- Sprite and audio asset usage
- Minimal score-only HUD

The target game in this spec differs substantially:

- Single 25m Donkey Kong stage
- Primitive-shape rendering only
- No sprites
- No audio
- Multi-wave progression
- Platform, ladder, and hazard logic

Any implementation should treat the current code as scaffolding, not as the authoritative game design.

## 3. Design Goals

- Recreate the feel of the original 25m stage using simple geometric rendering.
- Keep the implementation small and readable in the style of the current repo.
- Prefer deterministic, tunable gameplay systems over physics-heavy simulation.
- Use Bevy ECS state and lifecycle patterns consistently.

## 4. Non-Goals

- Full arcade-perfect reproduction of every timing quirk.
- Multiple stages beyond 25m.
- Sprite animation, texture atlases, or audio playback.
- Persistent save data.
- A generalized platforming engine.

## 5. Core Decisions

The original draft had several inconsistencies. The rules below resolve them.

- Rendering is shape-based only. Existing sprite and audio modules should be removed.
- Fireballs persist during a life attempt, but are cleared on player death and on wave transition.
- The game uses 5 waves on the same stage, then a win screen.
- The game requires expanding `AppState` beyond the current three-state template (see section 9).
- The bonus timer does not reset on death within the same wave, but does reset on wave advance.
- Hammers respawn after death because each death resets the current attempt on the same wave.
- Broken ladders block the player as specified, but fireballs ignore broken-ladder restrictions.
- Barrels always roll downhill on sloped girders and reverse horizontal direction on each girder transition.

## 6. Presentation

### 6.1 Window and Playfield

- Primary logical playfield: `224 x 256`
- Recommended rendered scale: `3x`
- Recommended gameplay window: `672 x 768`
- Background color: dark navy, approximately `#0A0A2E`

Implementation note:

- The game should use one logical coordinate system for gameplay regardless of final window size.
- If the team keeps a larger desktop window for development, center the playfield and keep gameplay coordinates defined by the logical stage rectangle.

### 6.2 HUD Region

- HUD lives inside the top portion of the playfield.
- Required fields:
  - current score
  - session high score
  - lives
  - wave number
  - bonus timer

### 6.3 Visual Style

- All visible entities are rendered from primitive shapes.
- Use flat fills with no texture assets.
- Avoid outlines unless needed for readability.
- Favor a limited arcade-like palette.

Implementation note:

- In Bevy terms, prefer colored 2D meshes or other sprite-free primitives.
- Do not depend on image assets for gameplay entities.

Suggested palette:

| Element | Color |
|---|---|
| Background | `#0A0A2E` |
| Girders | `#D03030` |
| Ladders | `#00E8E8` |
| Player | `#E04040` |
| Player with hammer | `#FFD700` |
| Donkey Kong | `#8B4513` |
| Pauline | `#FF69B4` |
| Barrel | `#CD853F` |
| Blue barrel | `#4169E1` |
| Fireball | `#FF4500` |
| Oil drum | `#404040` |
| HUD text | `#FFFFFF` |

## 7. Coordinate Model

The implementation must define one explicit world-space convention and use it everywhere.

- World origin: center of the logical playfield.
- Positive X: right.
- Positive Y: up.
- Entity placement and collision are computed in world coordinates.
- The level module owns the authoritative stage geometry.

Required geometry data types:

- `GirderSegment { start: Vec2, end: Vec2, thickness: f32 }`
- `Ladder { x: f32, y_bottom: f32, y_top: f32, kind: LadderKind }`
- `LadderKind = Full | Broken`

`Broken` ladders have a visible gap and block player traversal entirely. Fireballs ignore this restriction and may still use broken ladders. (The original arcade had partial-access ladders; this game simplifies to a binary full/broken model.)
- `SpawnPoint` values for player, DK, Pauline, oil drum, hammer pickups, and bonus items
- `GoalZone` near Pauline

## 8. Level Layout

### 8.1 Stage Shape

The game recreates the classic 25m stage with:

- 6 main girders from bottom to top
- alternating slopes on the middle girders
- DK at the top-left
- Pauline on a small perch above DK
- oil drum at the lower-left

### 8.2 Platform Rules

Platform numbering is bottom to top. Approximate vertical spacing between adjacent girders is `40 px` (logical). This spacing drives fall damage thresholds and jump reach calculations.

1. Platform 1: near-flat ground, full width, oil drum and player spawn on the left.
2. Platform 2: slopes down left-to-right (left end is higher).
3. Platform 3: slopes down right-to-left (right end is higher).
4. Platform 4: slopes down left-to-right.
5. Platform 5: slopes down right-to-left.
6. Platform 6: short top platform for DK, with Pauline above on a separate perch.

### 8.3 Ladders

- Target ladder count: 9 total
- Mix of full and broken ladders
- Broken ladders should meaningfully shape routing, not just decorate the stage
- At least:
  - 2 broken ladders in the middle of the stage
  - 1 full ladder that offers a shorter but riskier route near hazards

Implementation deliverable:

- Exact girder endpoints and ladder placements must live in one static stage definition rather than being scattered across systems.

### 8.4 Pickups and Key Placements

- Player spawn: lower-left, to the right of the oil drum
- DK: upper-left platform
- Pauline: above DK in the goal area
- Hammer A: lower-middle portion of the stage
- Hammer B: upper-middle portion of the stage
- Bonus item spawn: one fixed mid-stage position

## 9. Game States

The current repo state machine is too small for the target game. The target `AppState` is:

- `StartScreen`
- `Playing`
- `Dying`
- `WaveTally`
- `GameOver`
- `WinScreen`

### 9.1 State Responsibilities

`StartScreen`

- Show title, high score, and start prompt
- Reset run-scoped data on entry
- Preserve session high score across runs
- Space starts wave 1

`Playing`

- All gameplay systems active
- Player, hazards, pickups, HUD, and timer are live

`Dying`

- Freeze active gameplay (barrels, fireballs, DK, and the bonus timer all pause)
- Run death flash and pause
- Transition either back to `Playing` (with attempt reset) or to `GameOver`

Note: `AppState::Dying` is a game-level state that gates all gameplay systems. The player's locomotion state (section 11.1) also has a `Dying` variant, but this is cosmetic — it drives the death animation while `AppState::Dying` controls the game freeze. Entering `AppState::Dying` should set the player locomotion to `Dying` and prevent all other locomotion transitions.

`WaveTally`

- Freeze action after player reaches Pauline
- Convert remaining bonus timer into score
- Advance to next wave or win

`GameOver`

- Show final score and high score
- Space returns to `StartScreen`

`WinScreen`

- Show completion message and final score
- Space returns to `StartScreen`

## 10. Shared Resources

Required shared resources:

- `SessionData`
  - `high_score: u32`
- `RunData`
  - `score: u32`
  - `lives: u8`
  - `wave: u8`
  - `extra_life_awarded: bool`
- `WaveRuntime`
  - `bonus_timer: i32`
  - `elapsed_wave_time: f32`
  - attempt-seeded random source or equivalent deterministic generator
  - collected or expired state for each bonus item
- `DeathSequence`
  - `elapsed: f32` — time since entering `AppState::Dying`
  - `cause: DeathCause` — barrel, fireball, fall, or timer (used for any cause-specific logic or display)
- `WaveConfig` (one static config per wave, indexed by wave number)
  - per-wave throw interval
  - blue barrel frequency
  - barrel speed multiplier
  - max fireballs
  - fireball speed multiplier

## 11. Player

### 11.1 Player States

The player uses mutually exclusive locomotion states:

- `Walking` — on a girder, responding to horizontal input
- `Jumping` — upward arc of a jump (positive vertical velocity)
- `Falling` — downward arc of a jump, or walking off an edge (negative vertical velocity, no girder support)
- `Climbing` — on a ladder, responding to vertical input
- `Dying` — death animation playing, no input accepted

Transitions:
- `Walking` → `Jumping` (Space pressed), `Falling` (walked off edge), `Climbing` (up/down at ladder), `Dying` (hazard hit)
- `Jumping` → `Falling` (vertical velocity becomes negative)
- `Falling` → `Walking` (landed on girder), `Dying` (fatal fall distance exceeded or hazard hit)
- `Climbing` → `Walking` (reached top or bottom of ladder)

Hammer is a timed power-up flag, not a separate locomotion state.

### 11.2 Movement Rules

- Walk speed: about `60 px/s` (constant regardless of slope — the player moves at the same speed uphill and downhill, but follows the slope angle)
- Climb speed: about `40 px/s`
- Gravity: about `600 px/s^2`
- Jump impulse: about `220 px/s` (yields a max jump height of approximately `40 px`, enough to clear barrels but not skip girders)
- No air control after jump start — horizontal velocity is locked at the moment of jump
- Walking on a girder follows the girder slope
- Walking off an edge enters `Falling`

### 11.3 Controls

Supported bindings:

| Action | Keys |
|---|---|
| Move left | Left Arrow, A |
| Move right | Right Arrow, D |
| Climb up | Up Arrow, W |
| Climb down | Down Arrow, S |
| Jump | Space |

### 11.4 Ladder Interaction

- Ladder grab zone width: same as ladder visual width (`16 px`), centered on the ladder X coordinate.
- The player must be within the grab zone horizontally to initiate climbing.
- Up enters from the bottom if the ladder allows it (i.e., the ladder is `Full`).
- Down enters from the top if the ladder allows it (i.e., the ladder is `Full`).
- `Broken` ladders block all player entry (both up and down).
- Entering a ladder should snap or quickly lerp the player to ladder center.
- Horizontal movement is disabled while climbing.
- The player cannot climb while the hammer is active.
- Releasing the climb direction while on a ladder: the player stays on the ladder (does not fall).

### 11.5 Fall Damage

- Safe short drops are allowed (walking off a girder edge onto the girder below).
- Falling more than roughly one full girder spacing (`40 px`) causes death. Suggested fatal threshold: `36 px` (allows small tolerance for walking off short edges).
- Fall damage is measured by vertical distance from the last supported walking position.

## 12. Donkey Kong

- DK is a static stage actor with a simple throw animation state machine.
- Visual states and approximate durations:
  - `Idle` — default between throws
  - `WindUp` — barrel raised, lasts about `0.5 s`
  - `Throwing` — throw release, lasts about `0.3 s`, barrel spawns at the start of this state
- DK stands on the top platform and spawns barrels onto the top route.
- Barrel spawn point: at DK's right side, on the girder surface of platform 6.

Throw timing by wave:

| Wave | Throw interval | Blue barrel frequency |
|---|---|---|
| 1 | 3.0 s | every 5th |
| 2 | 2.5 s | every 4th |
| 3 | 2.0 s | every 4th |
| 4 | 1.8 s | every 3rd |
| 5 | 1.5 s | every 3rd |

## 13. Barrels

### 13.1 Spawn and Movement

- Barrels spawn from DK at the barrel spawn point (see section 12).
- They roll along the active girder, always moving downhill along the slope. On flat surfaces (platform 1), barrels roll in the direction they were already traveling.
- Base roll speed starts around `80 px/s` before wave multipliers.
- At girder ends, barrels drop to the next valid girder below at a fall speed of about `200 px/s` (distinct from player gravity — barrels fall at a fixed speed, not accelerating).
- Barrels reverse horizontal direction on each girder transition (they zigzag down the stage).

### 13.2 Ladder Behavior

When a barrel crosses a ladder entry:

- Normal barrel: about `30%` chance to descend a full ladder
- Blue barrel: always descends if a valid full ladder is available
- Broken ladders are never used by any barrel type (normal, blue, or wild)
- Barrel descent decisions are evaluated only once per ladder crossing so a barrel cannot jitter in place and repeatedly reroll

### 13.3 Wild Barrels

- About `10%` of normal barrels are flagged wild on spawn.
- Wild barrels use the same pathing but add a small visible vertical bounce (amplitude about `4 px`, frequency about `3 Hz`). This is purely visual — the collision circle stays centered on the barrel's logical position on the girder.
- Wild barrels keep the same gameplay collision radius.

### 13.4 Oil Drum Interaction

- A barrel that reaches the oil drum on platform 1 despawns.
- On despawn at the oil drum, a fireball spawn check occurs (see section 14.1).
- Barrels that roll off the right edge of platform 1 (past the oil drum) also despawn, but do not trigger a fireball check.

### 13.5 Wave Speed Scaling

| Wave | Barrel speed multiplier |
|---|---|
| 1 | 1.00 |
| 2 | 1.10 |
| 3 | 1.20 |
| 4 | 1.30 |
| 5 | 1.40 |

## 14. Fireballs

### 14.1 Spawn Rules

- The oil drum may spawn a fireball when a barrel reaches it.
- Spawn chance: `50%`
- Respect per-wave max active fireballs.

### 14.2 Behavior

Fireballs use simple pursuit-biased movement:

- Patrol on current girder
- Favor movement toward the player most of the time
- Consider climbing when reaching ladders
- May use full or broken ladders (unlike barrels, fireballs ignore ladder restrictions)
- On spawn from the oil drum, fireballs start moving to the right (away from the drum)

Suggested behavior weights:

- Horizontal pursuit bias: `70%`
- Ladder usage toward player level: `70%`

### 14.3 Fireball Speeds

- Base patrol speed: about `40 px/s`
- Base climb speed: about `30 px/s`

Wave scaling:

| Wave | Max fireballs | Fireball speed multiplier |
|---|---|---|
| 1 | 2 | 1.00 |
| 2 | 3 | 1.00 |
| 3 | 4 | 1.10 |
| 4 | 5 | 1.15 |
| 5 | 5 | 1.25 |

### 14.4 Lifetime Clarification

Fireballs:

- persist during normal play within a single life attempt
- are cleared on player death
- are cleared on wave completion
- are not carried between waves

## 15. Hammer

### 15.1 Pickup Rules

- Two fixed hammer pickups exist per wave.
- Each hammer can be collected once per attempt.
- After player death, both hammers are restored for the restarted attempt.

### 15.2 Active Effects

- Duration: about `10 s`
- Player color changes to yellow (see palette)
- A larger smash radius surrounds the player — hammer hit zone extends about `20 px` in the player's facing direction and `12 px` vertically above the player center
- Climbing is disabled
- Jumping remains allowed
- Touching a barrel or fireball with the hammer hit zone destroys it

### 15.3 Expiration

- Last 3 seconds flash between normal and hammer colors
- Hammer expires cleanly without preserving any combo state

### 15.4 Hammer Scoring

- Barrel smashed: `300`
- Fireball smashed: `500`

## 16. Scoring

### 16.1 Score Values

| Event | Score |
|---|---|
| Jump one barrel | 100 |
| Jump two or more overlapping barrels in one jump window | 300 |
| Smash barrel with hammer | 300 |
| Smash fireball with hammer | 500 |
| Bonus item 1 | 300 |
| Bonus item 2 | 500 |
| Bonus item 3 | 800 |
| Remaining wave timer on clear | 1 point per timer unit |

### 16.2 Jump-Over Rules

A barrel counts as jumped when all of the following are true:

1. The player is in `Jumping` or `Falling` (the upward and downward arcs of a jump both count).
2. The player's feet (bottom of the `16 x 22` body AABB) are above the top of the barrel (barrel center Y + barrel radius).
3. The barrel's center is within the player's horizontal footprint (player center X ± `16 px`, slightly wider than the body to feel fair).
4. That barrel has not already scored during the current jump.

Clarification:

- If two or more barrels are cleared in the same scoring window, award `300` total, not `100` plus `300`.
- The scoring window starts when the player enters `Jumping` and ends when the player lands (enters `Walking` or `Climbing`). All barrels scored in that window count as one event.

### 16.3 Bonus Timer

- Start each wave at `5000`
- Decrease by `100` every `2 s`
- Show current value in the HUD
- Add remaining value to score during `WaveTally`
- Do not reset on death within the same wave
- If the timer reaches `0`, trigger death

### 16.4 Bonus Items

- One fixed bonus-item spawn location
- Three items appear by sequence, not simultaneously

Spawn plan:

- at `20 s`: purse worth `300`
- at `40 s`: hat worth `500`
- at `60 s`: parasol worth `800`

Each item:

- lasts `15 s`
- despawns if uncollected
- is collected by overlap
- remains collected for the rest of the wave after pickup

If the player dies:

- bonus-item timing continues from the current `elapsed_wave_time`
- already collected items stay collected
- expired items stay expired

## 17. Lives and High Score

### 17.1 Lives

- Starting lives: `3` (the player can die 3 times before game over — on the 3rd death, lives reaches `0` and the game ends)
- One extra life at `10,000` score (awarded once per run, tracked by `extra_life_awarded`)
- Maximum lives: `5`

### 17.2 High Score

- Session-only high score
- Stored independently from run-scoped gameplay data
- Updated on `GameOver` and `WinScreen`
- Displayed on start, game over, and win screens

## 18. Death and Respawn

### 18.1 Death Causes

- Barrel collision without hammer protection
- Fireball collision without hammer protection
- Fatal fall
- Bonus timer expiration

### 18.2 Death Sequence

1. Enter `Dying`
2. Freeze active gameplay
3. Flash player for about `1 s`
4. Hold for about `1 s`
5. Deduct one life
6. If lives remain, reset the wave attempt and return to `Playing`
7. If no lives remain, go to `GameOver`

### 18.3 Attempt Reset After Death

When the player loses a life but still has lives remaining:

- player returns to spawn
- all barrels are cleared
- all fireballs are cleared
- hammer pickups are restored
- DK throw cadence restarts
- bonus items remain in whatever collected or expired state matches the current wave clock
- bonus timer keeps its current value

## 19. Wave Progression

- Total waves: `5`
- All waves use the same stage geometry
- Only timing and hazard pressure change between waves
- Reaching Pauline triggers `WaveTally`
- After wave 5 tally, transition to `WinScreen`

### 19.1 Wave Clear Reset

When advancing from one wave to the next:

- clear all hazards and pickups from the previous wave
- reset the player to the wave spawn point
- reset hammer availability
- reset bonus-item sequence
- reset wave elapsed time
- reset bonus timer to its starting value
- load the next wave's `WaveConfig`

### 19.2 Goal Detection

- The wave clears when the player's body overlaps Pauline's goal zone while `AppState` is `Playing`
- Goal detection is only active during `AppState::Playing` — it is inherently disabled during `Dying` and `WaveTally`
- If the bonus timer reaches `0` on the exact same frame the player reaches the goal, the goal takes priority (the player clears the wave)

## 20. Collision Model

This game uses explicit geometric collision, not a physics engine.

Required collision types:

| Check | Shape A | Shape B |
|---|---|---|
| Player vs girder | AABB (`16 x 22`) | line segment (girder) |
| Player vs ladder grab zone | AABB center X | point-in-range (ladder X ± `8`) |
| Player vs barrel | AABB (`16 x 22`) | circle (radius `7`) |
| Player vs fireball | AABB (`16 x 22`) | circle (radius `5`) |
| Hammer zone vs barrel | AABB (see 15.2) | circle (radius `7`) |
| Hammer zone vs fireball | AABB (see 15.2) | circle (radius `5`) |
| Barrel vs girder | circle center | point-on-segment |
| Fireball vs girder | circle center | point-on-segment |
| Player vs pickup | AABB overlap | AABB overlap |
| Player vs goal zone | AABB overlap | AABB overlap |

Rules:

- Prefer simple AABB and circle checks. AABB-vs-circle tests should use closest-point-on-AABB distance.
- Support resolution priority should favor the highest valid girder under the player within a small vertical tolerance (about `4 px`).
- All thresholds must be constants, not inline magic numbers.
- Use one consistent collision shape per entity category to avoid frame-to-frame rule changes.

## 21. Rendering and Entity Shapes

Target approximate dimensions in logical pixels:

| Entity | Size |
|---|---|
| Player | `16 x 22` |
| Barrel radius | `7` |
| Fireball radius | `5` |
| DK body | about `40 x 36` |
| Pauline | about `12 x 20` |
| Hammer pickup | about `12 x 12` |
| Oil drum | about `20 x 24` |
| Goal zone | about `24 x 28` (centered on Pauline) |
| Girder thickness | `8` |
| Ladder width | `16` |
| Bonus item | about `10 x 10` |

The implementation may tune these, but the final values belong in `constants.rs`.

## 22. Recommended Module Ownership

To stay close to the current repo style, use a small set of focused modules.

Required modules to keep:

- `main.rs`
- `states.rs`
- `constants.rs`
- `components.rs`
- `resources.rs`
- `player.rs`
- `ui.rs`

Recommended additions or repurposes:

- `level.rs`
  - owns girders, ladders, spawn points, and goal zone
- `enemy.rs`
  - may be repurposed into DK and hazard spawning, or replaced by `hazards.rs`
- `combat.rs`
  - owns collisions, scoring events, hammer destruction, and win/death triggers

Cleanup note:

- `audio.rs` is not needed for the target game and should be removed.
- The `Laser`, `Enemy`, `Music`, and `Velocity` components from the template should be removed once superseded.
- The package name in `Cargo.toml` should be changed from `bevy_template` to `donkey_kong`.
- `WINDOW_TITLE` in `constants.rs` should be updated to `"Donkey Kong"` (currently `"Bevy 2D Template"`).

## 23. System Ordering

Recommended update order while in `Playing`:

1. read input
2. update player intent and movement state
3. resolve ladder entry and climbing
4. apply player movement and gravity
5. update DK throw timer and spawn barrels
6. move barrels and resolve barrel ladder decisions
7. move fireballs and resolve fireball ladder decisions
8. update hammer timer and hammer visuals
9. resolve collisions and scoring
10. trigger death, wave clear, or pickup events
11. tick bonus timer
12. update HUD

Important implementation note:

- Scoring and death should be based on events or ordered systems so the player cannot both die and score the same interaction twice in one frame.

## 24. Constants Checklist

All tunable values should live in `constants.rs`.

Minimum expected categories:

- window and playfield dimensions
- color constants
- player movement values
- girder and ladder dimensions
- barrel behavior values
- fireball behavior values
- hammer timings and hit radius
- scoring values
- bonus timer values
- life and extra-life values
- per-wave configuration values
- death-sequence timings

## 25. Validation Plan

Minimum validation after implementation work:

1. `cargo check`
2. `cargo clippy` after broad refactors or API-heavy changes
3. `cargo run`

Manual gameplay checks:

1. Start screen appears and Space starts a new run.
2. The stage renders using primitive geometry, not sprites.
3. Player walks slopes correctly and cannot leave the playfield incorrectly.
4. Ladder entry and broken-ladder restrictions behave consistently.
5. Jumps are committed and have no air steering.
6. DK throws barrels on schedule.
7. Normal and blue barrel ladder behavior differs as designed.
8. Oil drum can spawn fireballs within max-count limits.
9. Hammer disables climbing and destroys hazards.
10. Timer persists after death within the same wave.
11. Death sequence resets the attempt correctly.
12. Reaching Pauline advances the wave.
13. Wave 5 completion reaches `WinScreen`.
14. Score, lives, wave, and high score display correctly.

## 26. Implementation Notes for This Repo

To keep the work manageable in this repository:

- Make the conversion in small vertical slices.
- Replace sprite-based entities with shape-based entities early.
- Introduce level geometry before adding advanced hazard AI.
- Prefer adding one new system group at a time and validating with `cargo check`.
- Keep data ownership obvious: geometry in `level`, shared run data in `resources`, tuning in `constants`.

## 27. Out of Scope Unless Requested Later

- cutscenes
- intermission screens
- save files
- leaderboard persistence
- multiple stage themes
- full arcade attract mode
