# AGENTS.md

Guidance for coding agents working in this repository.

## Project Summary

This is a single-binary Bevy 0.18.1 2D game in Rust 2024 inspired by Atari Adventure.

Code is organized by domain plugin:

- `src/setup.rs`: camera setup, game reset, world spawning, cleanup
- `src/rooms.rs`: room transitions, walls, room visibility, background color
- `src/player.rs`: movement, inventory, item interactions, gates, win condition
- `src/enemies.rs`: dragons, swallow state, sword combat, bat behavior
- `src/ui.rs`: title/game-over/win screens and in-game HUD
- `src/components.rs`: shared ECS components, resources, messages, system params
- `src/world.rs`: room data, geometry constants, wall building, world resources, tests

## Build And Validate

Run these commands from the repo root:

```sh
cargo check
cargo test
cargo clippy -- -D warnings
cargo fmt
cargo run
```

Useful notes:

- Run `cargo test` after changing room topology, world data, or gameplay rules.
- Run `cargo clippy -- -D warnings` before finishing non-trivial work.
- Build artifacts go to `D:/cargo-target-dir` via `.cargo/config.toml`.

## State And Scheduling

App state flow:

- `Title -> Playing -> Swallowed -> GameOver -> Title`
- `Playing -> Win -> Title`

During `AppState::Playing`, systems are scheduled in this order:

- `Prepare`
- `Movement`
- `Interaction`
- `Room`
- `Enemies`
- `WinCheck`
- `Presentation`

On entering `Playing`, setup is ordered as:

- `Reset`
- `SpawnWorld`
- `RoomState`
- `Ui`

When adding systems, put them in the correct set. Only add explicit ordering such as `.before()`, `.after()`, or `.chain()` when a real dependency exists.

## ECS Conventions

- Tag gameplay entities with `GameEntity` so they can be cleaned up when returning to the title screen.
- Prefer components for per-entity state and resources for truly global state.
- Prefer `OnEnter` and `OnExit` for spawn/cleanup symmetry.
- Prefer `run_if(...)` and change detection when updates only need to happen after state changes.
- Keep system functions module-private unless there is a clear reason to expose them.
- Keep changes scoped; do not bundle unrelated refactors.

## Current Gameplay Invariants

- The player starts in room `1` with an empty inventory.
- Bringing the chalice into room `0` wins the game.
- Carrying the dot through the north wall of room `6` reaches secret room `13`.
- The world has 14 rooms total: `0..=13`.
- Do not silently change room connectivity, gates, or item placement.
- If you change exits, gates, or interior walls, update or extend the tests in `src/world.rs`.

## Messages

Gameplay messages are defined in `src/components.rs`:

- `ItemPickedUp`
- `ItemDropped`
- `GateOpened`
- `DragonKilled`
- `PlayerSwallowed`

Producers register them with `app.add_message::<M>()` and emit them with `MessageWriter<M>`. If you introduce new messages, add at least one consumer or remove the abstraction if it is not needed yet.

## Practical Editing Notes

- Preserve the plugin-based structure unless there is a strong reason to reorganize it.
- Keep world data centralized in `src/world.rs` where possible.
- Avoid duplicating room/gameplay state across multiple sources of truth unless synchronization is intentional and documented.
- Prefer deriving presentation from gameplay state instead of storing redundant visual-only state.
