# Lode Runner - Bevy 0.18.1 Game Spec

## 1. Purpose

This document defines the target design for a 2D Lode Runner-style game built in Bevy 0.18.1.

It is written for this repo as it exists today:

- The current project is still a minimal Bevy starter in `src/main.rs`.
- The game architecture described here is a target architecture, not the current implementation.
- Changes should be introduced incrementally. Do not split the codebase into many modules until the game logic is large enough to justify it.

## 2. Product Direction

### High-level goal

Build a clean, readable, grid-based Lode Runner interpretation with:

- tile-accurate movement
- digging and hole regeneration
- guard pursuit
- gold collection
- hidden escape ladder reveal
- multiple levels

### Visual direction

The first playable version should use Bevy-native 2D rendering and simple shapes. Primitive-only rendering is acceptable for MVP and keeps the game focused on mechanics.

### Non-goals for MVP

- no dependency on the existing `assets/` folder
- no audio requirement
- no sprite animation pipeline
- no save system
- no custom editor

Polish such as bloom, particles, screen shake, and title presentation should come after the core rules are solid.

## 3. Design Principles

### Gameplay principles

- Prefer deterministic, grid-first gameplay over continuous physics.
- Preserve the core Lode Runner loop before adding modern twists.
- Treat every gameplay rule as data-driven where practical.
- Favor predictable behavior over cinematic behavior.

### Bevy principles

- Keep one clear source of truth for logic state.
- Use `States`, `OnEnter`, and `OnExit` for lifetime management.
- Gate gameplay systems with `run_if(in_state(...))`.
- Use messages for buffered gameplay communication and triggered events for immediate reactions.
- Avoid over-engineering the initial version.

## 4. Current Repo Alignment

The repo currently contains:

- `src/main.rs` with `DefaultPlugins`, a `Camera2d`, and a centered `Hello, World!`
- Bevy `0.18.1`
- Rust edition `2024`
- `dynamic_linking` enabled for Bevy

This spec assumes we will grow from that starter. It should not imply that the full module layout already exists.

## 5. Architecture Summary

### Recommended file growth path

Start small and split only when the code becomes hard to reason about.

Suggested module layout once needed:

```text
src/
  main.rs
  constants.rs
  states.rs
  components.rs
  resources.rs
  grid.rs
  levels.rs
  player.rs
  enemy.rs
  holes.rs
  ui.rs
  rendering.rs
  camera.rs
  effects.rs
```

### Module responsibilities

- `main.rs`: app setup, plugin registration, schedule wiring
- `constants.rs`: tunable values only
- `states.rs`: `AppState`, optional play sub-state, state helpers
- `components.rs`: ECS marker and data components
- `resources.rs`: mutable cross-system state
- `grid.rs`: static tile data, passability, coordinate helpers
- `levels.rs`: level definitions, parsing, validation, loading
- `player.rs`: input buffering, player actions, dig requests
- `enemy.rs`: guard decision-making and movement
- `holes.rs`: hole lifecycle and regeneration
- `ui.rs`: HUD and menus
- `rendering.rs`: primitive spawning, visual sync
- `camera.rs`: persistent camera setup
- `effects.rs`: optional particles and screen shake

## 6. Core Gameplay Rules

### Movement rules

| Context | Allowed actions |
|---|---|
| Supported on solid ground | Move left/right, dig left/right, climb down if ladder below |
| On ladder | Move up/down/left/right |
| On bar | Move left/right, optionally drop |
| Unsupported in air | Fall vertically |
| Standing on trapped guard | Treat as supported ground |

### Important movement constraints

- There is no jump.
- Movement is tile-by-tile.
- Visual motion can be interpolated, but gameplay occupancy remains grid-based.
- Falling is automatic when support is lost.

### Digging rules

- The dig target is the brick diagonally below-left or below-right of the player.
- Digging fails if the target is not a diggable brick.
- Digging fails if the side cell at player height is blocked and the design requires reach space.
- Digging is not allowed while falling.
- Dug holes are temporary and eventually regenerate into bricks.

### Gold and level completion

- Gold is represented as entities, not as tile data.
- The exit ladder becomes active only after all level gold is collected.
- The player finishes the level by reaching the active escape route.

### Guard rules

- Guards do not dig.
- Guards follow the same traversal rules as the player for ladders, bars, and falling.
- Guards can fall into holes and become trapped temporarily.
- A trapped guard acts as support for entities above it.

## 7. Canon vs Variant Rules

The previous version of this spec mixed classic-inspired rules with modern rule changes while also claiming to preserve all original mechanics. That was inconsistent.

This spec separates baseline rules from optional variants.

### MVP baseline

Use the classic-compatible baseline unless the project intentionally chooses otherwise:

- player is not expected to rely on "safe holes" as a core feature
- guard trapping is essential
- escape ladder reveal is mandatory
- deaths should reset the level state in a predictable way

### Optional variants

Treat these as explicit configuration choices, not hidden assumptions:

- player dies if caught in a regenerating hole
- collected gold persists across death
- guards respawn at random valid spawn points instead of original spawns

If a variant is chosen, document it clearly in code and UI because it changes the feel of the game.

## 8. State Model

### App states

```rust
#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum AppState {
    #[default]
    StartScreen,
    Playing,
    LevelComplete,
    GameOver,
}
```

### Optional play sub-state

```rust
#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash, Default)]
#[source(AppState = AppState::Playing)]
enum PlayState {
    #[default]
    Running,
    Paused,
    Dying,
}
```

### State usage

- use `AppState` for screen-level transitions
- use `PlayState` only if pause/death sequencing actually needs it
- keep gameplay entities scoped to `AppState::Playing`
- keep menu entities scoped to their own app states

### Cleanup strategy

Use one of these patterns consistently:

1. `DespawnOnExit(StateVariant)` for state-scoped entity groups
2. explicit cleanup systems on `OnExit(...)`

Do not mix both styles arbitrarily for the same entity category.

## 9. Camera Strategy

The earlier spec scoped the main camera to `AppState::Playing`, but also wanted world-space visuals on non-playing screens. That was contradictory.

### Recommended approach

Keep one persistent `Camera2d` alive across the whole app unless there is a strong reason not to.

Good defaults:

- spawn the camera once at startup
- configure bloom and tone mapping on that camera
- keep HUD readability independent from bloom

### Why

- start screen effects can reuse the same camera
- level-complete world effects still render
- fewer camera lifecycle bugs

## 10. Inter-System Communication

The earlier spec was directionally right that old buffered event APIs changed in Bevy 0.18, but its replacement advice was incomplete.

### Buffered gameplay communication

For queue-like cross-system communication, use messages:

```rust
#[derive(Message)]
enum GameplayMessage {
    GoldCollected { pos: IVec2 },
    GuardKilled { pos: IVec2 },
    PlayerDied,
    ExitUnlocked,
}

app.add_message::<GameplayMessage>();
```

Systems can then use:

- `MessageWriter<GameplayMessage>`
- `MessageReader<GameplayMessage>`

This is the right Bevy 0.18 replacement for many old event-reader/event-writer patterns.

### Immediate reactions

For immediate, entity-focused reactions, use triggered events and observers:

```rust
#[derive(EntityEvent)]
struct GuardExploded {
    entity: Entity,
}
```

Use this sparingly for localized reactions such as:

- spawn a burst effect
- attach a one-shot animation
- react to a direct entity-targeted occurrence

### When not to use observers

Do not make observers the default communication mechanism for all gameplay rules. Most gameplay flow here is easier to debug with ordinary systems plus messages/resources.

## 11. Scheduling and Ordering

### Suggested system sets

Use gameplay-oriented names instead of `Render`, which can be confused with Bevy's actual render pipeline.

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet {
    Input,
    Simulate,
    Resolve,
    Presentation,
}
```

### Recommended order

```rust
app.configure_sets(
    Update,
    (
        GameSet::Input,
        GameSet::Simulate.after(GameSet::Input),
        GameSet::Resolve.after(GameSet::Simulate),
        GameSet::Presentation.after(GameSet::Resolve),
    ),
);
```

### Responsibilities

- `Input`: gather player intent
- `Simulate`: movement, gravity, digging, hole timers, AI
- `Resolve`: pickups, death, win checks, state changes
- `Presentation`: transform sync, animation, HUD, optional effects

### Deferred commands

Do not rely on vague assumptions about when deferred commands become visible.

Best practice:

- if a later system in the same schedule must observe queued spawns or despawns, add an explicit `ApplyDeferred`
- otherwise keep systems loosely coupled enough that same-frame command visibility does not matter

## 12. Data Model

### Static tiles vs dynamic entities

Keep the tile grid as the source of truth for level structure:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tile {
    Empty,
    Brick,
    Concrete,
    Ladder,
    Bar,
    HiddenLadder,
}
```

Do not store dynamic objects like gold or guards in the tile enum.

Track these separately:

- player spawn
- guard spawns
- gold positions
- active holes

### Hole state

Do not permanently mutate the base level data to represent temporary holes. Keep hole state separate from static tiles.

Preferred model:

```rust
#[derive(Resource)]
struct LevelGrid {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

#[derive(Component)]
struct Hole {
    cell: IVec2,
    phase: HolePhase,
    timer: Timer,
}
```

Why this is better:

- the authored level remains intact
- reset logic is simpler
- rendering can derive the visible state from base tile plus hole overlay

### Occupancy model

Dynamic occupancy should come from queries or a short-lived occupancy cache, not by rewriting the base tile map every frame.

## 13. Coordinate System

### Grid convention

- grid origin: bottom-left
- player, guards, gold, and holes use grid coordinates for gameplay
- world-space transforms are derived from grid coordinates

### Helper

```rust
fn grid_to_world(pos: IVec2, width: usize, height: usize, cell_size: f32) -> Vec2 {
    let offset_x = -(width as f32 * cell_size) * 0.5 + cell_size * 0.5;
    let offset_y = -(height as f32 * cell_size) * 0.5 + cell_size * 0.5;
    Vec2::new(
        offset_x + pos.x as f32 * cell_size,
        offset_y + pos.y as f32 * cell_size,
    )
}
```

Keep this logic in one place and reuse it everywhere.

## 14. Components and Resources

### Core components

```rust
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Guard;

#[derive(Component)]
struct Gold;

#[derive(Component, Copy, Clone)]
struct GridPosition(pub IVec2);

#[derive(Component)]
struct SpawnPoint(pub IVec2);
```

### Movement state

```rust
#[derive(Component)]
enum MovementState {
    Idle,
    Moving { from: IVec2, to: IVec2, progress: f32 },
    Falling { from: IVec2, to: IVec2, progress: f32 },
    Climbing { from: IVec2, to: IVec2, progress: f32 },
    Digging { side: HorizontalDir, timer: Timer },
    Trapped { timer: Timer },
}
```

### Shared resources

```rust
#[derive(Resource, Default)]
struct GameState {
    score: u32,
    lives: u32,
    current_level: usize,
    collected_gold: u32,
    total_gold: u32,
    exit_unlocked: bool,
}
```

Additional resources may include:

- `LevelGrid`
- `LevelCatalog`
- `ScreenShake`
- `InputBuffer`

## 15. Input Handling

### Default mapping

| Action | Keys |
|---|---|
| Move left | `A`, `Left` |
| Move right | `D`, `Right` |
| Move up | `W`, `Up` |
| Move down | `S`, `Down` |
| Dig left | `Q`, `Z` |
| Dig right | `E`, `X` |
| Pause | `Escape` |
| Restart level | `R` |

### Input buffering

Use at most a 1-deep buffered action for the MVP.

That gives:

- responsive directional feel
- predictable next action
- no long action queue

## 16. Movement and Physics Rules

### Simulation model

Movement is discrete for gameplay but interpolated for visuals.

Recommended flow:

1. read intended action
2. validate against grid rules
3. enter a movement state
4. update interpolation over time
5. commit `GridPosition` when the step completes
6. apply gravity if support is gone

### Support checks

An entity is supported when any of these is true:

- solid tile below
- ladder behavior says it should remain attached
- trapped guard below
- bottom boundary acts as floor

Centralize support and passability checks in `grid.rs` so movement, AI, and hole logic all agree.

### Collision rules

Use deterministic tile occupancy rules. Avoid continuous collision bodies for gameplay.

Important cases:

- player enters guard cell -> death
- guard enters player cell -> death
- pass-through swap in the same step also counts as collision
- two guards should not occupy the same non-bar cell

## 17. Hole System

### Hole lifecycle

```text
Dig request -> dig animation -> hole open -> hole closing -> brick restored
```

### Requirements

- only bricks can be dug
- each hole has its own timer
- guard trap duration must be shorter than total hole lifetime
- regeneration behavior must be deterministic

### Recommended implementation

- store active holes as entities or a dedicated resource map
- keep base tile data unchanged
- treat `HolePhase::Closing` as non-enterable

### Edge cases to define explicitly in code

- two guards around one hole
- gold dropped into or above a hole
- hole closes while something occupies the cell
- dig request on an already-open hole

Do not leave these as emergent behavior.

## 18. Guard AI

### AI target

For MVP, guards should pursue the player with simple grid pathfinding and deterministic fallback behavior.

### Recommendation

Use BFS on the traversable movement graph.

Why BFS is enough here:

- level sizes are small
- movement is tile-based
- shortest-path behavior is easy to reason about

### Important simplification

Do not add weighted costs while still calling the algorithm BFS. If weighted path bias is desired later, switch intentionally to Dijkstra or A*.

### MVP guard behavior

- recalculate path on a timer, not every frame
- if no path exists, continue with a simple fallback rule
- when trapped, stop pathfinding until escape
- when respawning, use validated spawn points

### Respawn guidance

Prefer deterministic respawn rules first:

- original spawn point if clear
- otherwise first valid spawn from a prevalidated list

Random top-row respawn can be added later if it is part of the intended design.

## 19. Level Data

### Format

Use ASCII-authored levels embedded in Rust for the MVP.

```text
. = Empty
# = Brick
= = Concrete
H = Ladder
- = Bar
$ = Gold on empty
S = Gold on ladder
P = Player spawn
G = Guard spawn
^ = Hidden ladder
```

### Parse result

The parser should produce:

- base tile grid
- player spawn
- guard spawn list
- gold position list

### Validation rules

Validate every authored level at load time or in tests:

- exactly one `P`
- all rows same width
- bottom row fully concrete
- at least one gold piece
- at least one reachable completion path
- every guard spawn is valid
- hidden ladder cells are placed intentionally

### Reachability testing

The earlier spec discussed unreachable gold only after runtime events. Add validation earlier too:

- authored levels should be solvable before runtime
- any dynamic gold re-drop logic should still attempt to keep the level completable

## 20. Rendering

### Primitive-only approach

Primitive rendering is acceptable and Bevy-friendly for this project.

Use shared meshes and materials through a resource:

```rust
#[derive(Resource)]
struct RenderAssets {
    brick_mesh: Handle<Mesh>,
    brick_material: Handle<ColorMaterial>,
    gold_mesh: Handle<Mesh>,
    gold_material: Handle<ColorMaterial>,
}
```

### Recommended rendering rules

- reuse shared mesh handles
- reuse shared material handles when per-instance color is not needed
- keep Z-layers explicit
- keep tile visuals derived from gameplay state, not the reverse

### Player and guard visuals

Child entities are fine here. The previous spec warned against ordinary child usage too strongly.

Acceptable approaches:

- `with_children(...)`
- `children![...]`

Choose whichever keeps the code clearer.

## 21. UI and Text

### HUD

Use Bevy UI for:

- score
- lives
- level number
- gold counter

### Change detection

Do not use `Changed<GameState>` in a query. `GameState` is a resource, not a component.

Use one of these patterns instead:

```rust
fn update_hud(game_state: Res<GameState>) {
    if !game_state.is_changed() {
        return;
    }
}
```

or gate the system:

```rust
update_hud.run_if(resource_changed::<GameState>)
```

### Bloom and UI

The previous spec said UI was not affected by bloom, then described bloom-heavy UI text. That was inconsistent.

Use this rule:

- gameplay HUD must remain readable without bloom
- if a glowing title is desired, prefer `Text2d` or another world-space presentation element

## 22. Visual Effects

Effects should be optional polish, not core dependencies.

Recommended order:

1. no effects
2. subtle gold pickup burst
3. light screen shake
4. title and completion polish

### Bloom guidance

If bloom is enabled:

- keep intensity restrained
- verify the game remains readable with and without it
- use bright world-space colors intentionally, not everywhere

## 23. Constants

Keep tunables centralized once a constants module exists.

Example starter values:

```rust
const CELL_SIZE: f32 = 40.0;
const GRID_WIDTH: usize = 28;
const GRID_HEIGHT: usize = 16;

const PLAYER_MOVE_SPEED: f32 = 6.0;
const PLAYER_CLIMB_SPEED: f32 = 4.0;
const PLAYER_FALL_SPEED: f32 = 10.0;

const GUARD_MOVE_SPEED: f32 = 4.5;
const GUARD_CLIMB_SPEED: f32 = 3.5;
const GUARD_FALL_SPEED: f32 = 10.0;

const DIG_DURATION: f32 = 0.25;
const HOLE_OPEN_DURATION: f32 = 4.5;
const HOLE_CLOSE_DURATION: f32 = 0.5;
const GUARD_TRAP_DURATION: f32 = 3.0;
```

These are starting points only and should be tuned in playtesting.

## 24. Implementation Plan

Each phase should leave the project runnable.

### Phase 1 - Game shell

- replace the hello-world UI
- add app state
- add persistent camera
- add one hardcoded level render

Validation:

- `cargo check`

### Phase 2 - Grid movement

- player spawn
- tile passability
- walking, climbing, falling
- grid-to-world interpolation

Validation:

- `cargo check`
- movement helper unit tests

### Phase 3 - Digging and holes

- dig validation
- hole timers
- regeneration
- trap behavior

Validation:

- `cargo check`
- hole rule tests

### Phase 4 - Gold and level completion

- gold entities
- collection
- hidden ladder reveal
- level end transition

Validation:

- `cargo check`
- parser and completion tests

### Phase 5 - Guards

- guard movement
- BFS pursuit
- trap and respawn behavior
- player death on contact

Validation:

- `cargo check`
- `cargo clippy`

### Phase 6 - UI and polish

- HUD
- menus
- pause
- optional particles and bloom

Validation:

- `cargo check`
- `cargo clippy`

## 25. Testing Strategy

Add focused tests for logic that is easy to regress:

- level parser
- tile passability
- support checks
- dig target validation
- hole regeneration timing
- hidden ladder reveal
- guard path graph generation

Where possible, keep gameplay logic in plain functions that can be tested without rendering.

## 26. Best Practices Checklist

- keep the main camera persistent
- keep static level data separate from temporary hole state
- use messages for buffered gameplay communication
- use observers only for targeted immediate reactions
- use resource change detection correctly
- avoid introducing many modules before they are needed
- prefer deterministic rules over clever but hard-to-debug behavior
- validate levels and rule helpers with tests

## 27. Summary of Key Corrections

This spec intentionally fixes several issues from the prior version:

- clarified that this repo is still a starter, not an existing modular game
- replaced contradictory camera/state lifetime guidance with a persistent camera plan
- corrected Bevy 0.18 communication guidance toward messages plus triggered events
- removed the invalid `Changed<GameState>` resource pattern
- removed the misleading warning against normal child entity usage
- resolved the bloom/UI contradiction
- separated baseline gameplay rules from optional modern variants
- removed the BFS-plus-weighting inconsistency
- moved temporary hole data out of the base tile grid model

This version should be a more reliable implementation target for Bevy 0.18.1 and a better fit for the current repo.
