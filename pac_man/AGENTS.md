# AGENTS.md

This repository is a small Bevy 2D game project implementing a Pac-Man-style prototype.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`

## Project Layout

- `src/main.rs`
  Starts the Bevy app, configures the window, and adds `PacmanPlugin`.
- `src/game/mod.rs`
  Main plugin wiring. This is the source of truth for schedules, states, and system ordering.
- `src/game/components.rs`
  ECS components and lightweight marker types.
- `src/game/resources.rs`
  Global resources, especially `GameSession`, `GameMeshes`, `GameMaterials`, and `RoundState`.
- `src/game/constants.rs`
  Gameplay tuning values, colors, z-layers, fixed timestep, and the built-in ASCII level map.
- `src/game/level.rs`
  ASCII level parsing and grid/world coordinate helpers.
- `src/game/logic.rs`
  Pure gameplay helper logic that is easy to unit test.
- `src/game/systems/`
  Systems split by responsibility:
  - `setup.rs`: asset/resource initialization and entity spawning
  - `flow.rs`: round lifecycle and state transitions
  - `input.rs`: player/restart input
  - `simulation.rs`: fixed-step gameplay systems
  - `presentation.rs`: HUD and visual sync

## Architecture Rules

- Prefer Bevy `States` over hand-rolled enum resources for high-level game flow.
- Keep gameplay simulation in `FixedUpdate`.
- Keep rendering, animation, and UI sync in `Update`.
- Use system sets and `run_if` conditions instead of one large chained schedule when possible.
- Keep pure logic out of systems when it can live in `logic.rs` or `level.rs`.
- Prefer small focused modules over one large `systems.rs`.
- Treat `mod.rs` as orchestration, not a place for gameplay logic.

## Entity Lifetime Conventions

- `LevelEntity` is for static level geometry that persists for the whole app lifetime.
- `RoundEntity` is for gameplay entities that are recreated when a round/game restarts.
- HUD entities are persistent and should not be tied to round resets.

## State Model

The game currently uses `RoundState`:

- `Ready`
- `Playing`
- `Won`
- `GameOver`

Important expectations:

- Entering `Ready` resets actor positions and round timers.
- `Playing` runs fixed-step simulation.
- `Won` and `GameOver` stop simulation and allow restart input.

## Scheduling Model

Current intent:

- `Startup`
  - create shared meshes/materials
  - spawn camera/HUD/static level
  - spawn round entities
- `Update`
  - handle input
  - update visuals and HUD
- `FixedUpdate`
  - advance timers
  - plan ghost movement
  - move actors
  - resolve pellets/collisions

When adding new systems, place them in the schedule that matches their job instead of defaulting to `Update`.

## Code Style Guidance

- Keep components data-oriented and lightweight.
- Avoid storing view-only data in gameplay components unless there is a strong reason.
- Prefer `Single<...>` only when the code genuinely assumes one matching entity exists.
- Add `Name` components for important spawned entities to help debugging.
- Add unit tests for pure helpers and parsing logic.
- Keep comments short and only where they add real clarity.

## Testing and Validation

Run these before finishing meaningful changes:

```powershell
cargo check
cargo clippy
cargo test
```

For behavior-sensitive changes, also run:

```powershell
cargo run
```

## Change Guidance For Future Agents

- Preserve the state-driven architecture in `src/game/mod.rs`.
- If a change introduces more gameplay modes, extend the Bevy state model instead of adding more boolean flags.
- If a system starts mixing simulation and presentation concerns, split it.
- If helper logic becomes testable without ECS, move it out of systems and add tests.
- Do not reintroduce a monolithic `systems.rs` file.
