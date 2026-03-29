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
- The project is now a modular Lode Runner prototype, not a minimal Bevy starter

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
- The grid module currently has `34` unit tests; keep them passing when changing parsing or traversal behavior.
- If you cannot run validation, say so explicitly.

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Prefer targeted changes that keep iteration fast.

## Design Source Of Truth

- `SPEC.md` is the current design target for the project.
- The codebase may lag behind the spec in places; align changes with the current user request and the current implementation.
- When the spec and code disagree, do not blindly "fix" code to match the spec unless the task calls for it.

## Current Project Layout

- `src/main.rs`: app wiring, state registration, schedule setup, camera startup, level spawning
- `src/states.rs`: `AppState` and `PlayState`
- `src/components.rs`: ECS gameplay and UI marker/data components
- `src/constants.rs`: gameplay tuning, colors, sizing, timers
- `src/resources.rs`: `GameState`, `DeathTimer`
- `src/grid.rs`: tile model, hole overlay, parser, BFS pathfinding, coordinate helpers, unit tests
- `src/levels.rs`: embedded ASCII-authored levels
- `src/player.rs`: player input and dig initiation
- `src/movement.rs`: interpolation, gravity, hole ticking, transform sync
- `src/guard.rs`: guard AI and path-following
- `src/gameplay.rs`: gold collection, death flow, exit checks, restart flow
- `src/render.rs`: shared meshes/materials
- `src/ui.rs`: start screen, HUD, pause overlay, level complete, game over
- `.cargo/config.toml`: shared cargo target-dir configuration
- `SPEC.md`: target design and architecture guidance

## Current Runtime Behavior

The game is no longer just a starter scene. The current app already has:

- `AppState` flow for `StartScreen`, `Playing`, `Restarting`, `LevelComplete`, and `GameOver`
- `PlayState` flow for `Running`, `Paused`, and `Dying`
- a persistent `Camera2d` with tone mapping and bloom
- embedded ASCII levels parsed into a `LevelGrid`
- primitive-rendered tiles, player, guards, gold, and hidden ladders
- player movement, climbing, falling, and digging
- hole open/closing behavior via a `HoleMap` overlay
- dig visuals that spawn when a hole actually opens
- guard BFS pathfinding
- gold collection and hidden ladder reveal
- HUD updates, pause overlay, death flow, level-complete screen, and game-over screen
- play-state cleanup for both state-scoped entities and play-scoped resources

Before making assumptions, inspect the existing modules. The repo is already past the "Hello, World!" stage.

## Architecture Guidance

Prefer the current gameplay-domain split over collapsing logic back into `main.rs`.

Keep responsibilities aligned like this:

- `grid.rs`: traversal rules, parser, and level-structure helpers
- `movement.rs`: movement progression and gravity
- `player.rs`: player intent and player-only actions
- `guard.rs`: guard-only decision making
- `gameplay.rs`: game rules that span entities or states
- `ui.rs`: screen and HUD composition
- `render.rs`: mesh/material setup and rendering support

If the game grows further, likely next additions are:

- `effects.rs` for particles or screen shake
- more focused level/game-rule helpers if `gameplay.rs` gets too large

Prefer small, domain-owned modules over ad hoc systems spread through `main.rs`.

## Bevy Conventions To Follow

- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry.
- Gate state-specific systems with `run_if(in_state(...))`.
- Keep the main camera persistent unless a task explicitly requires a different lifetime.
- Use `DespawnOnExit(...)` consistently for state-scoped entities, or explicit cleanup systems, but do not mix styles carelessly for the same entity group.
- Prefer resources for shared mutable state and messages for buffered cross-system communication if new communication paths are added.
- Use explicit ordering with `.after(...)`, `.chain()`, or system sets where frame order matters.
- Use `Res<T>::is_changed()` or `resource_changed::<T>` for resource-driven updates; do not use `Changed<T>` for resources.
- Remove play-scoped resources on `OnExit(AppState::Playing)` so stale level state does not leak into menus or restarts.

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

## Gameplay And Level Notes

- Levels are embedded ASCII strings in `src/levels.rs`.
- `Tile` data lives in `src/grid.rs`; gold, guards, and holes are not base tiles.
- Hidden ladders are authored in the level format and revealed at runtime.
- BFS pathfinding already exists in `src/grid.rs`; if changing guard pathing, keep the algorithm naming accurate.
- The project currently favors deterministic behavior over fancy AI heuristics.
- Level parsing is intentionally strict: rows must be rectangular, there must be exactly one `P`, and unsupported characters should be treated as authoring errors.

If you change level parsing, traversal rules, hole behavior, or pathfinding, update or add tests in `src/grid.rs`.

## Rendering, UI, And Asset Notes

- Current rendering uses primitive meshes and `ColorMaterial`, not sprite assets.
- The current prototype does not depend on the `assets/` folder for gameplay.
- UI uses Bevy UI for menus, overlays, and HUD.
- If you want glowing title treatment or bloom-heavy presentation, prefer world-space presentation elements for that effect and keep HUD readability independent from bloom.
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
2. Check `SPEC.md` if the task depends on intended design rather than current behavior.
3. Implement the change in the owning file or module.
4. Extract new helpers/constants/resources only when the code size justifies it.
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
- design target: `SPEC.md`
