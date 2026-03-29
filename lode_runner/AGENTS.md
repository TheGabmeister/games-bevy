# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\lode_runner`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- `bevy` is enabled with the `dynamic_linking` feature
- The project is a modular Lode Runner prototype, not a minimal Bevy starter

## Build And Validation

Use these commands from the project root:

```powershell
cargo run
cargo build
cargo check
cargo clippy
cargo test
```

Validation expectations:

- Run `cargo check` for most code changes.
- Run `cargo test` when changing grid logic, parsing, movement rules, or other code with existing unit tests.
- Run `cargo clippy` when changing broader patterns, Bevy API usage, or shared infrastructure.
- The grid module currently has `34` unit tests; keep them passing when changing parsing, traversal, digging, or pathfinding behavior.
- Docs-only changes usually do not require cargo validation.
- If you cannot run validation, say so explicitly.

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Prefer targeted changes that keep iteration fast.

## Source Of Truth

- There is currently no `SPEC.md` in this repo.
- Treat the current codebase and the direct user request as the primary source of truth.
- `CLAUDE.md` exists in the repo root as companion guidance, but verify any prose guidance against the current code before relying on it.
- When documentation and code disagree, do not blindly "fix" code to match prose unless the task calls for it.

## Current Project Layout

- `src/main.rs`: app wiring, system sets, state registration, camera startup, level spawning, play-resource cleanup
- `src/states.rs`: `AppState` and `PlayState`
- `src/components.rs`: ECS gameplay, movement, hole, AI, and UI marker/data components
- `src/constants.rs`: gameplay tuning, colors, sizes, timers, scores, and window constants
- `src/resources.rs`: `GameState` and `DeathTimer`
- `src/grid.rs`: tile model, hole overlay, parser, traversal helpers, BFS pathfinding, coordinate helpers, unit tests
- `src/levels.rs`: embedded ASCII-authored levels
- `src/player.rs`: player input, movement intent, and dig initiation
- `src/movement.rs`: interpolation, gravity, hole ticking, trap handling, transform sync
- `src/guard.rs`: guard AI and path-following decisions
- `src/gameplay.rs`: gold collection, ladder reveal, death flow, exit checks, restart flow
- `src/render.rs`: primitive meshes and `ColorMaterial` setup
- `src/ui.rs`: start screen, HUD, pause overlay, level-complete screen, game-over screen
- `.cargo/config.toml`: shared cargo target-dir configuration
- `CLAUDE.md`: additional repo guidance

## Current Runtime Behavior

The app already has:

- `AppState` flow for `StartScreen`, `Playing`, `Restarting`, `LevelComplete`, and `GameOver`
- `PlayState` flow for `Running`, `Paused`, and `Dying`
- ordered gameplay sets in `Update`: `Input`, `Simulate`, `Resolve`, and `Presentation`
- a persistent `Camera2d` with custom clear color, tone mapping, and bloom
- embedded ASCII levels parsed into a `LevelGrid`
- primitive-rendered tiles, player, guards, gold, and hidden ladders
- player movement, climbing, falling, and digging
- hole open/closing behavior via a `HoleMap` overlay plus spawned hole visuals
- guard BFS pathfinding with timer-driven decision updates
- gold collection, score tracking, hidden ladder reveal, death flow, level transitions, and a win/game-over ending
- HUD updates and pause/transition overlays
- play-state cleanup for both state-scoped entities and play-scoped resources

Before making assumptions, inspect the existing modules. The repo is already well past the "Hello, World!" stage.

## Architecture Guidance

Prefer the current gameplay-domain split over collapsing logic back into `main.rs`.

Keep responsibilities aligned like this:

- `grid.rs`: traversal rules, parsing, tile semantics, and pathfinding helpers
- `movement.rs`: movement progression, gravity, hole timers, and world-position sync
- `player.rs`: player-only input and actions
- `guard.rs`: guard-only decision making
- `gameplay.rs`: cross-entity rules and app/play-state transitions
- `ui.rs`: menus, overlays, and HUD composition
- `render.rs`: mesh/material setup and rendering support

Prefer small, domain-owned modules over ad hoc systems spread through `main.rs`.

## Bevy Conventions To Follow

- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry.
- Gate state-specific systems with `run_if(in_state(...))`.
- Keep the main camera persistent unless a task explicitly requires a different lifetime.
- Use `DespawnOnExit(...)` consistently for state-scoped entities, or explicit cleanup systems, but do not mix styles carelessly for the same entity group.
- Prefer resources for shared mutable state and Bevy 0.18 messages for buffered communication if new communication paths are added.
- Use explicit ordering with `.after(...)`, `.chain()`, or system sets where frame order matters.
- Use `Res<T>::is_changed()` or `resource_changed::<T>` for resource-driven updates; do not use `Changed<T>` for resources.
- Remove play-scoped resources on `OnExit(AppState::Playing)` so stale level state does not leak into menus or restarts.
- Preserve the existing `States` plus `SubStates` structure unless a task clearly requires changing it.

## Coding Rules For This Repo

- Make the smallest coherent change that solves the task.
- Do not rewrite working structure just to make it "cleaner".
- Keep module boundaries aligned to gameplay domains.
- Put new tunable values in `src/constants.rs` instead of scattering magic numbers.
- Add new shared mutable state to `src/resources.rs` unless the state is tightly owned by a single domain and clearly belongs elsewhere.
- Add ECS marker/data types to `src/components.rs` unless a more local placement is clearly better.
- Extend existing modules before creating new ones without a strong reason.
- When adding state-scoped entities, define how they are cleaned up.
- Keep temporary hole state separate from the base tile grid. Do not rewrite authored level tiles just to represent open holes.
- Preserve the grid-first gameplay model; do not introduce continuous physics for core movement/collision.
- Keep traversal rules consistent across player input, gravity, and guard pathfinding. Unsupported actors should fall instead of gaining sideways movement in midair.

## Gameplay, Controls, And Level Notes

- Levels are embedded ASCII strings in `src/levels.rs`.
- `Tile` data lives in `src/grid.rs`; gold, guards, player spawn, and holes are not base tiles.
- Hidden ladders are authored in the level format and revealed at runtime when all gold is collected.
- BFS pathfinding already exists in `src/grid.rs`; if changing guard pathing, keep the algorithm naming accurate.
- The project currently favors deterministic behavior over fancy AI heuristics.
- `GameState` tracks score, lives, current level, remaining gold, and whether the exit is unlocked.
- Start screen input uses `Space` and also resets `GameState` by inserting `GameState::default()`.
- Pause input uses `Escape`.
- Player movement uses `A/D` or arrow keys, climbing uses `W/S` or arrow keys, and digging uses `Q` or `Z` for left and `E` or `X` for right.

Current level characters:

- `.` empty
- `#` brick
- `=` concrete
- `H` ladder
- `-` bar
- `^` hidden ladder
- `$` gold on empty space
- `S` gold on a ladder tile
- `P` player spawn, exactly one required
- `G` guard spawn

Level parsing is intentionally strict:

- rows must be rectangular
- there must be exactly one `P`
- unsupported characters are authoring errors

If you change level parsing, traversal rules, hole behavior, or pathfinding, update or add tests in `src/grid.rs`.

## Rendering, UI, And Asset Notes

- Current rendering uses primitive meshes and `ColorMaterial`, not sprite assets.
- The current prototype does not depend on the `assets/` folder for gameplay.
- UI uses Bevy UI for menus, overlays, and HUD.
- Keep HUD readability independent from bloom-heavy presentation.
- Keep asset paths as plain relative strings if assets are introduced later.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`.
- Use `ApplyDeferred` if later systems in the same schedule must observe queued commands.
- 2D rendering uses current Bevy APIs such as `Camera2d`, `Mesh2d`, and `MeshMaterial2d`.
- For buffered communication added in new work, prefer Bevy 0.18 messages (`add_message`, `MessageWriter`, `MessageReader`) instead of older event-reader/event-writer patterns.

## Preferred Change Pattern

1. Inspect the current code and local module boundaries before making assumptions.
2. Check the relevant owning modules and any repo docs that still exist before changing behavior.
3. Implement the change in the owning file or module.
4. Extract new helpers, constants, or resources only when the code size justifies it.
5. Run validation, usually `cargo check`, plus `cargo test` for gameplay-rule changes.
6. Summarize what changed and any remaining risks.

## Good First Places To Look

- app wiring and schedule setup: `src/main.rs`
- state flow: `src/states.rs`
- level parser, traversal rules, BFS, tests: `src/grid.rs`
- player behavior: `src/player.rs`
- movement and holes: `src/movement.rs`
- guard behavior: `src/guard.rs`
- game rules and transitions: `src/gameplay.rs`
- UI flow: `src/ui.rs`
- repo guidance: `CLAUDE.md`
