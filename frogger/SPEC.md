# Frogger Technical Specification

Status: Draft, revised on 2026-03-29

This document defines the target game for this repository and records the current implementation baseline. The repository now contains a playable Frogger MVP, so this spec focuses on documenting the implemented core loop, the remaining polish opportunities, and the architecture conventions we want to preserve.

## 1. Product Summary

Build a single-screen 2D Frogger-style arcade game in Rust with Bevy 0.18.1.

The target experience is:

- Grid-based frog movement with readable, responsive hop timing.
- Road hazards, river platforms, home bays, score, lives, and per-life timer.
- State-driven menus and HUD using Bevy UI.
- Simple, clean visuals that do not depend on external image assets for the core game.

Core design goals:

- Preserve the readable, deliberate feel of classic Frogger.
- Keep the architecture small and modular.
- Favor deterministic, testable gameplay rules over decorative complexity.
- Make the MVP playable without relying on an `assets/` folder.

## 2. Current Repository Baseline

As of 2026-03-29, the repository already implements the main Frogger play loop with primitive visuals and no required assets.

Current implemented behaviors:

- `StartScreen -> Playing -> GameOver -> StartScreen` state flow.
- Portrait window at `720 x 960`.
- Grid-based frog movement with one-cell hops on `WASD` or arrow keys.
- Five traffic lanes, five river lanes, five home bays, score, lives, and per-life timer.
- Successful bay fills award score and respawn the frog.
- Filling all five bays advances the level and increases lane speed.
- The game runs without external image or audio assets.

Current code facts:

- The playfield is built from colored Bevy sprites and Bevy UI nodes.
- The game uses `src/lanes.rs` for world and lane objects and `src/collision.rs` for rule resolution.
- High score is currently in-memory only.
- Core gameplay is single-screen and deterministic, with no save system or content streaming.

Implication:

- The project is in a playable MVP state.
- The next meaningful milestone is polish, tests, and targeted gameplay refinement rather than a template conversion.

## 3. Scope Definition

### 3.1 MVP Scope

The MVP should include:

- Start screen.
- One complete Frogger play loop.
- Frog spawning at the bottom safe zone.
- Five road lanes with moving hazards.
- One safe median row.
- Five river lanes with rideable platforms.
- Five home bays at the top.
- Lives system.
- Per-life timer.
- Score and level tracking.
- Game-over flow and restart.

### 3.2 Post-MVP Polish

The following are desirable but not required for MVP:

- Bloom and emissive-style highlights.
- Score popups and lightweight particles.
- Screen shake on death.
- Animated turtles that dive.
- Bonus fly and lady-frog mechanics.
- Ambient menu background animation.
- Shorter feedback loops for level-clear and death moments.
- Small automated tests around scoring, helpers, and progression math.

### 3.3 Explicit Non-Goals For The First Pass

Do not require these for the first playable Frogger version:

- Networked play.
- Save system beyond optional high score persistence.
- Complex AI.
- Texture-heavy art pipeline.
- Large-scale refactors unrelated to the gameplay conversion.

## 4. Technical Constraints

- Language: Rust 2024 edition.
- Engine: Bevy 0.18.1.
- Architecture: plugin-based modules with state-gated systems.
- Validation: `cargo check` should pass during active implementation and `cargo clippy` should stay clean for routine refactors.
- Runtime dependency goal: core gameplay should work without image or audio assets.

Asset policy for the Frogger conversion:

- Prefer primitive shapes, solid-color sprites, meshes, or UI nodes for core visuals.
- Audio is optional in MVP.
- If audio is added later, treat it as enhancement, not a blocker for basic playability.

## 5. Target Gameplay Loop

The intended run loop is:

1. Player starts on the title screen.
2. Press `Space` or `Enter` to begin.
3. Frog spawns in the bottom safe zone with full timer.
4. Player hops upward, sideways, or downward one cell at a time.
5. Road collisions, drowning, timeout, invalid home landing, or leaving the playfield cost a life.
6. Reaching an empty home bay banks score and respawns the frog.
7. Filling all five home bays clears the level and increases difficulty.
8. Losing the final life transitions to `GameOver`.
9. Press `Space` or `Enter` to return to `StartScreen`.

## 6. World Layout

### 6.1 Window

Implemented gameplay window:

- Width: `720`
- Height: `960`
- Orientation: portrait

Rationale:

- Frogger benefits from vertical progression.
- Portrait layout creates clearer separation between safe zones, lanes, and HUD.

Note:

- The current code already uses this portrait layout.

### 6.2 Grid

Target playfield grid:

- `13` playable columns
- `13` gameplay rows
- `CELL_SIZE = 48.0`

This yields a `624 x 624` gameplay area centered in the window, leaving room for top HUD and bottom timer/lives UI.

### 6.3 Row Plan

Rows are indexed bottom-to-top:

| Row | Type | Notes |
|---|---|---|
| 0 | Start safe zone | Frog spawn row |
| 1 | Road lane 1 | Slow traffic |
| 2 | Road lane 2 | Medium traffic |
| 3 | Road lane 3 | Fast traffic |
| 4 | Road lane 4 | Long vehicles |
| 5 | Road lane 5 | Fast traffic |
| 6 | Median safe zone | Mid-run recovery row |
| 7 | River lane 1 | Rideable logs |
| 8 | River lane 2 | Rideable turtles or logs |
| 9 | River lane 3 | Rideable logs |
| 10 | River lane 4 | Rideable turtles or logs |
| 11 | River lane 5 | Rideable logs |
| 12 | Home row | Five goal bays |

### 6.4 Home Bays

Target home-row behavior:

- Five evenly spaced bays.
- Bay openings accept a frog only if currently empty.
- Landing in a filled bay kills the frog.
- Landing on the wall between bays kills the frog.
- Filled bays remain occupied until level completion.

## 7. Core Mechanics

### 7.1 Frog Movement

Movement rules:

- One hop per input.
- Inputs: arrow keys or `WASD`.
- Movement is grid-locked, not freeform.
- Each hop moves exactly one cell.
- Horizontal and vertical movement are both allowed.
- The frog cannot move outside the playable column range.

Recommended feel targets:

- Hop duration: `0.12` to `0.16` seconds.
- Optional one-input buffer while hopping.
- Input should use `just_pressed`, not held auto-repeat.

### 7.2 Road Hazards

Road hazards:

- Move horizontally at lane-specific speeds.
- Wrap at screen edges.
- Kill the frog on contact.

MVP hazard set:

- Short car
- Medium truck
- Long truck

### 7.3 River Platforms

River platforms:

- Move horizontally at lane-specific speeds.
- Carry the frog while overlapping.
- Cause death if the frog is in a river row and not supported.
- Wrap at screen edges.

MVP platform set:

- Short log
- Medium log
- Long log

Optional post-MVP platform:

- Turtle groups with dive cycle

### 7.4 Death Conditions

The frog loses a life when:

- Hit by a vehicle.
- In a river row without a rideable platform.
- Carried fully beyond the screen bounds.
- Timer reaches zero.
- Landing in a filled home bay.
- Landing on a home-row wall segment.

### 7.5 Respawn Rules

On death:

1. Consume one life.
2. If lives remain, respawn a fresh frog in row `0`.
3. Reset the per-life timer.
4. Preserve the current level state, including already-filled home bays.
5. If no lives remain, transition to `GameOver`.

### 7.6 Level Completion

When all five home bays are filled:

1. Award level-clear bonus.
2. Increment level.
3. Reset all home bays to empty.
4. Increase lane speeds within a capped multiplier.
5. Spawn a fresh frog with full timer.

## 8. Scoring

Recommended score model:

| Event | Points |
|---|---|
| Forward hop to a new furthest row | +10 |
| Reaching an empty home bay | +50 |
| Remaining time on successful home landing | +10 per second |
| Level clear | +1000 |

Optional score events:

| Event | Points |
|---|---|
| Fly bonus | +200 |
| Lady frog rescue | +200 |
| Extra life threshold | +1 life at 10000 score |

Important rule:

- Forward-hop score should only reward progress beyond the furthest row reached during the current life.

## 9. Game State Model

### 9.1 Top-Level States

Keep the current top-level state machine:

`StartScreen -> Playing -> GameOver -> StartScreen`

This matches the existing architecture and is sufficient for MVP.

### 9.2 State Responsibilities

`StartScreen`

- Spawn title UI.
- Optionally show a lightweight animated background.
- Wait for `Space` or `Enter`.

`Playing`

- Spawn world, frog, HUD, lane objects, and gameplay resources.
- Run all movement, collision, timer, scoring, and level logic.

`GameOver`

- Show final score and reached level.
- Wait for restart input.

### 9.3 Level Completion Handling

Do not create a separate top-level `LevelComplete` state unless implementation complexity demands it.

Preferred approach:

- Keep level completion inside `Playing`.
- Use a short timer resource or sub-state-like flag for the between-level pause.

## 10. Data Model

### 10.1 Shared Resources

The target shared resource model should include at least:

```rust
struct GameData {
    score: u32,
    high_score: u32,
    lives: u32,
    level: u32,
    filled_bays: [bool; 5],
    max_row_this_life: i32,
}

struct FrogTimer {
    remaining_secs: f32,
}

struct LevelState {
    speed_multiplier: f32,
    celebrating: bool,
}
```

Current status:

- The repo stores score, high score, lives, level, filled bays, and furthest-row progress in `GameData`.
- `FrogTimer` tracks the remaining life timer and `LevelState` tracks speed scaling plus level-clear pacing.

### 10.2 Core Components

Expected component set:

```rust
struct Frog;
struct Vehicle;
struct Platform;
struct HomeBay { index: usize }
struct FilledBay;
struct Velocity(Vec2);
struct GridPosition { col: i32, row: i32 }
struct HopState { from: Vec2, to: Vec2, progress: f32 }
struct LaneObject;
struct GameplayEntity;
```

Optional later components:

```rust
struct TurtleDiveState;
struct ScorePopup;
struct Lifetime;
struct ShakeCamera;
```

## 11. Architecture Plan

### 11.1 Current Modules

Current checked-in modules:

- `src/main.rs`
- `src/gameplay.rs`
- `src/states.rs`
- `src/constants.rs`
- `src/components.rs`
- `src/resources.rs`
- `src/player.rs`
- `src/lanes.rs`
- `src/collision.rs`
- `src/ui.rs`

### 11.2 Recommended Target Ownership

The target design should still stay close to the current modular style:

- `main.rs`: app bootstrap, camera, plugin registration
- `gameplay.rs`: ordered gameplay scheduling for the `Playing` state
- `states.rs`: top-level app states
- `constants.rs`: gameplay and UI tuning values
- `components.rs`: ECS marker and data components
- `resources.rs`: score, lives, level, timer, bay state
- `player.rs`: frog spawn, hop input, hop animation, respawn helpers
- `lanes.rs`: road and river lane spawning plus home-bay visuals
- `collision.rs`: collisions, drowning, bay resolution, level clear checks
- `ui.rs`: title screen, HUD, game-over screen

Implementation note:

- The repository has already taken the minimal-churn path by adding lane and collision modules directly, without reviving the old shooter-template names.

### 11.3 System Scheduling

Within `Playing`, the recommended order is:

1. Read input.
2. Start hop if possible.
3. Advance active hop animation.
4. Move lane objects.
5. Apply riding movement.
6. Resolve out-of-bounds conditions.
7. Check vehicle collision.
8. Check water support.
9. Check home-bay landing.
10. Tick frog timer.
11. Apply death or respawn.
12. Update score and HUD.
13. Check level completion.

The important behavioral requirement is not the exact function names, but that water checks happen after platform movement and after riding movement.

## 12. Rendering Direction

### 12.1 MVP Visual Strategy

Use simple rendered shapes instead of external sprite assets.

Recommended choices:

- Colored rectangles for vehicles, walls, timer bar, road, and safe zones.
- Rounded or rectangular frog body built from one or more primitive shapes.
- Solid-color logs.
- Simple circles or rounded capsules if turtle groups are implemented.

### 12.2 Color Direction

Suggested palette:

- Background: deep blue-black
- Safe zones: muted green
- Road: dark gray
- River: deep blue
- Frog: bright green
- Vehicles: high-contrast saturated colors
- HUD text: white or pale yellow

### 12.3 Audio

Audio is optional for MVP.

If retained:

- Keep it simple.
- Do not make missing audio assets block gameplay startup.
- Prefer a fallback path where the game still runs silently.

## 13. Known Issues In The Previous Spec

The previous version of this document had several problems:

- It described a game state that no longer matched the repository.
- It omitted the fact that the current code already implements Frogger gameplay with primitive visuals.
- It referred to a shooter-template baseline that has already been replaced.
- It referred to missing modules and stale migration concerns.
- It mixed current implementation details with aspirational future ideas.
- It contained text-encoding corruption, making symbols and arrows hard to read.
- It specified many polish-heavy systems before documenting the current playable baseline.

This revision fixes those issues by separating:

- current baseline
- target Frogger requirements
- optional polish
- implementation roadmap

## 14. Migration Plan

Recommended implementation sequence:

### Phase 1: Stabilize And Document

- Keep the current playable loop intact.
- Sync the spec with the actual module layout and implemented behaviors.
- Add lightweight tests for helper math and progression state.

### Phase 2: Gameplay Polish

- Improve level-clear presentation.
- Add clearer HUD and status feedback.
- Tune lane spacing and collision feel based on playtesting.

### Phase 3: Optional Enhancements

- Add optional audio.
- Add bonus entities and diving turtles.
- Add menu/background polish.

## 15. Acceptance Criteria

The Frogger MVP is complete when all of the following are true:

- The player can start a run from `StartScreen`.
- The frog moves one grid cell per input.
- Vehicles move continuously and kill on contact.
- River platforms move continuously and carry the frog.
- The frog dies in river rows when unsupported.
- The frog can reach and fill five home bays.
- Lives decrease correctly and game over triggers at zero.
- Timer resets on respawn and successful home landing.
- Score updates correctly during play.
- Clearing all five bays advances the level.
- Restart from `GameOver` returns to `StartScreen`.
- The game runs without requiring missing image or audio assets.

Current status:

- The repository meets these MVP criteria in code, though it still benefits from more testing and polish.

## 16. Testing Checklist

Manual checks:

- Start screen accepts `Space` or `Enter`.
- Frog cannot move outside the playfield.
- One key press results in one hop.
- Vehicle collision is detected while the frog is moving.
- Riding movement keeps the frog aligned and controllable.
- Water death does not trigger incorrectly during a hop.
- Filled bay re-entry causes death.
- Wall landing in the home row causes death.
- Last life transitions to `GameOver`.
- Restart resets run-specific data.

Nice-to-have automated tests:

- Grid-to-world and world-to-grid conversion helpers.
- Furthest-row scoring logic.
- Speed multiplier cap logic.

Current coverage target:

- Helper math and speed-cap tests are a good first target because they validate logic that is easy to regress and cheap to run.

## 17. Open Decisions

These items still matter for polish planning:

- Whether turtles are MVP or post-MVP.
- Whether optional audio should be added at all.
- Whether high score persists only in memory or to disk.

Until those are decided, the default should be:

- logs only for MVP river platforms
- silent gameplay is acceptable
- in-memory high score only
