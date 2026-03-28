# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\adventure`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling projects unless the user explicitly asks.
- Treat unrelated local changes as user-owned.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- Single-binary 2D game prototype inspired by Atari Adventure

## Run And Verify

Use these commands from the project root:

```powershell
cargo run
cargo fmt
cargo check
cargo test
cargo clippy -- -D warnings
```

Validation expectations:

- Run `cargo check` for most code changes.
- Run `cargo test` when changing room logic, world data, or gameplay rules.
- Run `cargo clippy -- -D warnings` before finishing non-trivial work.
- If you cannot run a validation step, say so clearly.

## Project Layout

- `src/main.rs`: app bootstrap, window config, state setup, plugin registration, shared system sets
- `src/setup.rs`: startup camera, game reset, world spawning, gameplay entity cleanup
- `src/world.rs`: room graph, constants, wall generation, shared world resources, room-data tests
- `src/components.rs`: ECS marker/data components and shared system params
- `src/rooms.rs`: room transitions, wall syncing, room visibility, background color updates
- `src/player.rs`: movement, inventory handling, gate interaction, win condition
- `src/enemies.rs`: dragon behavior, swallow flow, sword combat, bat behavior
- `src/ui.rs`: title/game over/win UI, HUD updates, menu state transitions

## Current State Flow

The current state machine is:

`Title -> Playing -> Swallowed -> GameOver -> Title`

There is also a `Win` state reached from gameplay.

Important behavior tied to states:

- `Title`: title text exists and `Space` or `Enter` starts the game
- `Playing`: world entities, room walls, HUD, movement, enemies, and interactions are active
- `Swallowed`: swallow animation runs before transitioning to `GameOver`
- `GameOver`: failure UI exists and restart input returns to `Title`
- `Win`: victory UI exists and restart input returns to `Title`

## Scheduling And Bevy Conventions

- Keep gameplay order aligned with the shared system sets in `src/main.rs`:
  `Prepare -> Movement -> Interaction -> Room -> Enemies -> WinCheck -> Presentation`
- If a system depends on another system's side effects, make ordering explicit with system sets, `.chain()`, `.before()`, or `.after()`.
- Prefer `OnEnter` and `OnExit` for spawn/cleanup symmetry instead of mixing lifecycle logic into always-running systems.
- Prefer focused domain plugins over registering many gameplay systems directly in `main.rs`.
- Use resources for truly global state like current room, room walls, inventory, and cached materials.
- Use components for per-entity gameplay data.
- Prefer change detection or `run_if(...)` conditions when updates only need to happen after state changes.

## Coding Rules For This Repo

- Preserve the plugin-based structure unless the task clearly requires changing it.
- Put new shared ECS types in `src/components.rs` unless they are tightly local to one module.
- Put new map/layout data and wall-generation helpers in `src/world.rs`.
- Keep room-related transitions and visibility logic in `src/rooms.rs`.
- Keep player interaction rules in `src/player.rs` and enemy rules in `src/enemies.rs`.
- Reuse `GameEntity` for gameplay entities that should be cleaned up when returning to the title screen.
- Avoid duplicating gameplay state in both resources and components unless there is a strong reason.
- Prefer simple deterministic logic over clever abstractions.

## Gameplay Rules To Preserve

- The window is currently `800x600`.
- The player starts in room `1` with an empty inventory.
- Gates are defined from `WorldMap` room data, not a second hard-coded source.
- Carrying the chalice into room `0` wins the game.
- Carrying the dot through the north wall of room `6` leads to the secret room.
- Returning to the title screen should despawn gameplay entities and allow a clean restart.

## Tests And Data Safety

- Keep `src/world.rs` tests passing when editing room topology or wall generation.
- If you change exits, gates, or interior walls, update or extend the room-data tests.
- Do not silently change room connectivity or item placement without calling it out.

## Working Style

- Make the smallest coherent change that solves the task.
- Do not bundle unrelated refactors with gameplay fixes.
- Preserve user changes in the worktree.
- If a task changes gameplay feel or rules, mention that clearly in the final summary.
