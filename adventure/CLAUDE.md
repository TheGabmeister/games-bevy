# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Validate

```sh
cargo check                    # fast compile check
cargo test                     # run unit tests (world.rs room-data tests)
cargo clippy -- -D warnings    # lint (run before finishing non-trivial work)
cargo fmt                      # format
cargo run                      # launch the game (800x600 window)
```

Build output goes to `D:/cargo-target-dir` (set in `.cargo/config.toml`). Dev profile uses opt-level 1 for the binary, opt-level 3 for dependencies.

Run `cargo test` when changing room topology, world data, or gameplay rules. Run `cargo clippy -- -D warnings` before finishing non-trivial work.

## Architecture

Single-binary Bevy 0.18.1 2D game (Rust 2024 edition) inspired by Atari Adventure.

### Plugin Structure

All gameplay is registered through domain plugins in `main.rs`:

- **SetupPlugin** (`setup.rs`): camera, game reset, entity spawning/cleanup
- **RoomsPlugin** (`rooms.rs`): room transitions, wall syncing, visibility, background color
- **PlayerPlugin** (`player.rs`): movement, inventory, gate interaction, bridge/magnet mechanics, win condition
- **EnemiesPlugin** (`enemies.rs`): dragon AI, swallow animation, sword combat, bat behavior, bat dragon revival
- **UiPlugin** (`ui.rs`): title/game-over/win screens, HUD, menu state transitions

Shared ECS types live in `components.rs`. Map data, constants, wall generation, and world resources live in `world.rs`.

### State Machine

`Title -> Playing -> Swallowed -> GameOver -> Title` (also `Playing -> Win -> Title`)

### System Sets (Playing State)

Systems execute in this order via `PlayingSet`:

`Prepare -> Movement -> Interaction -> Room -> Enemies -> WinCheck -> Presentation`

New systems must be placed in the appropriate set. If a system depends on another's side effects, use explicit ordering (`.before()`, `.after()`, `.chain()`).

### OnEnter(Playing) Sets

`PlayingEnterSet::Reset -> SpawnWorld -> RoomState -> Ui`

## Conventions

- Tag gameplay entities with `GameEntity` for automatic cleanup on return to title screen.
- Gate definitions are derived from `WorldMap` room data, not hard-coded elsewhere.
- Use resources for global state (current room, inventory, room walls). Use components for per-entity data.
- Prefer `OnEnter`/`OnExit` for spawn/cleanup symmetry.
- Prefer change detection or `run_if(...)` when updates only need to happen after state changes.
- Keep the smallest coherent change; don't bundle unrelated refactors.

## Gameplay Invariants

- Player starts in room 1 with empty inventory. Carrying the chalice into room 0 wins.
- Carrying the dot through the north wall of room 6 reaches the secret room (room 13).
- The world has 14 rooms (0-13). Room 13 is the easter egg room.
- Do not silently change room connectivity or item placement.
- If you change exits, gates, or interior walls, update or extend the `world.rs` tests.
