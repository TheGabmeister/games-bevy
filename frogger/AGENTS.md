# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\frogger`.

## Scope

- This project lives inside a monorepo. Stay scoped to this directory only.
- Do not read, edit, or run git commands that reference parent or sibling directories.
- If git is necessary, prefer file-scoped or directory-scoped commands so unrelated monorepo changes stay out of scope.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- Single-screen 2D Frogger game using Bevy ECS, Bevy UI, primitive-rendered visuals, and lightweight gameplay effects

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
- Run `cargo clippy` when changing patterns that may affect API usage or code quality broadly.
- Run `cargo test` when touching shared logic with unit coverage or when adding new helper/resource logic.
- If you cannot run validation, say so explicitly.

## Dev Profile

- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Keep iteration-friendly workflows in mind; prefer targeted changes over broad rewrites.

## Project Layout

- `src/main.rs`: app bootstrap, window configuration, resource and message registration, camera setup
- `src/gameplay.rs`: ordered scheduling for the `Playing` state
- `src/states.rs`: `AppState` state machine
- `src/constants.rs`: window, grid, gameplay, rendering, and UI tuning values plus grid/world helper functions
- `src/components.rs`: marker and data components for gameplay, UI, and effects
- `src/resources.rs`: shared game-wide resources such as `GameData`, `FrogTimer`, `LevelState`, and gameplay messages
- `src/player.rs`: frog spawn, input, hop animation, facing/rotation, and forward-progress scoring
- `src/lanes.rs`: road/river lane spawning, world rows, lane movement, wrapping, and bay visuals
- `src/collision.rs`: riding logic, collisions, bounds checks, bay resolution, timer, respawn, and level clear
- `src/ui.rs`: start screen, HUD, game over screen, and state transitions from UI input
- `src/effects.rs`: death flashes and score popups driven by Bevy messages

## Current Game Flow

The current state machine is:

`StartScreen -> Playing -> GameOver -> StartScreen`

Important behavior tied to state transitions:

- `StartScreen`: title/menu UI exists, `Space` starts the run
- `Playing`: frog, lane objects, HUD, and gameplay effects are active
- `Playing`: road collisions, river support, home bays, score, lives, timer, and level progression all run here
- `GameOver`: final score UI exists, `Space` or `Enter` returns to `StartScreen`

When adding features, decide first whether they are:

- state-specific systems gated with `run_if(in_state(...))`
- enter/exit lifecycle systems attached to `OnEnter(...)` or `OnExit(...)`
- persistent resources that survive state changes
- messages that cross system or module boundaries within a frame

## Bevy Conventions Used Here

- Prefer small domain plugins over a large `main.rs`
- Keep cross-module gameplay ordering in `src/gameplay.rs` instead of expanding `main.rs`
- Use marker components for entity categories like `Frog`, `Vehicle`, `Platform`, `HomeBay`, `LaneObject`, gameplay roots, and UI roots
- Use resources for cross-system shared state like `GameData`, `FrogTimer`, and `LevelState`
- Use Bevy messages for lightweight cross-system notifications such as `FrogDeath`, `FrogBayFilled`, `SpawnDeathFlash`, and `SpawnScorePopup`
- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry
- Use explicit system ordering with `.after(...)` where frame ordering matters
- Keep gameplay systems gated behind `AppState::Playing` unless they intentionally span menus

## Coding Rules For This Repo

- Put new tunable values in `src/constants.rs` instead of scattering magic numbers.
- Add new shared mutable game state to `src/resources.rs`.
- Add new message types to `src/resources.rs` unless there is a strong module-local reason to keep them elsewhere.
- Add marker/data ECS types to `src/components.rs` unless a component is tightly local and clearly better kept nearby.
- Keep module boundaries aligned to gameplay domains.
- Prefer extending an existing plugin in the relevant module over registering ad hoc systems from `main.rs`.
- If a change affects update ordering across modules, adjust `src/gameplay.rs` deliberately rather than relying on incidental plugin order.
- When spawning entities on `OnEnter`, also define the matching cleanup path on `OnExit` if the entities should not persist.
- Reuse helper methods on shared resources before duplicating reset, scoring, timer, or progression logic.
- Reuse coordinate helpers such as `grid_to_world` and `world_x_to_col` before introducing new conversions.
- Preserve the current simple architecture unless the task requires a broader refactor.

## UI And Asset Notes

- UI uses Bevy UI nodes and text directly, without a custom theme system.
- Text sizes and text colors live in `src/constants.rs`.
- Core gameplay does not depend on external sprite or audio assets.
- Prefer primitive-rendered visuals (`Sprite`, `Mesh2d`, `Text2d`, UI nodes) unless a task explicitly introduces asset-backed content.
- HUD currently includes score, high score, level, lives, timer bar, and a short status message.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- The old event API has been replaced here with `MessageWriter<T>` and `MessageReader<T>`. Register messages with `app.add_message::<T>()`.
- The codebase uses Bevy's current component-style UI and `Camera2d`.
- The project uses `Mesh2d` and `MeshMaterial2d<ColorMaterial>` for simple gameplay visuals and effects.
- `ChildBuilder` is not in the prelude; avoid exposing it in signatures when inline `.with_children(...)` logic will do.

## Working Style

- Make the smallest coherent change that solves the task.
- Do not rewrite working gameplay structure just to make it "cleaner".
- Preserve user changes that are unrelated to the task.
- If you find conflicting local edits in the same area you need to touch, stop and surface the conflict.

## Preferred Change Pattern

1. Inspect the relevant module boundaries and state interactions.
2. Implement the change in the owning module.
3. Update `src/constants.rs`, `src/components.rs`, or `src/resources.rs` if the change introduces shared concepts.
4. Update `src/gameplay.rs` if the change needs explicit ordering relative to other systems.
5. Run validation, usually `cargo check`.
6. Run `cargo clippy` or `cargo test` when the change touches shared logic, APIs, helpers, or messages.
7. Summarize what changed and any remaining risks.

## Good First Places To Look

- Input or movement bug: `src/player.rs`
- Lane count, spawn behavior, or bay visuals: `src/lanes.rs` and `src/constants.rs`
- Collision, drowning, timer, respawn, or progression logic: `src/collision.rs`
- Ordered gameplay execution across modules: `src/gameplay.rs`
- Menu/HUD/game over behavior: `src/ui.rs`
- Score popups or death flash behavior: `src/effects.rs`
- App boot, plugins, resources, or message registration: `src/main.rs`
