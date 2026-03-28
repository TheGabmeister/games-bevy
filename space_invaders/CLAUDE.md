# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
cargo build                  # dev build (opt-level 1, deps at opt-level 3)
cargo run                    # run the game
cargo fmt                    # format code
cargo test --lib             # run unit tests (all tests are in gameplay.rs)
cargo clippy --all-targets --all-features -- -D warnings   # lint (must stay clean under -D warnings)
```

## Architecture

Bevy 0.18.1 Space Invaders game, Rust edition 2024. Three source files with strict separation of concerns:

- **main.rs** -- Bevy app setup: window (600x800), disabled audio, 2D camera, loads `SpaceInvadersPlugin`.
- **game/gameplay.rs** -- All game logic, ECS components, resources, state machine, collision, spawning, input, and tests (~1400 lines). This is where nearly all development happens.
- **game/presentation.rs** -- Visual rendering: sprite builders (colored rectangles), HUD text, UI overlays for each screen state (~550 lines).
- **game/mod.rs** -- Glue: exports submodules, defines window/game constants, wraps plugins.

### State Machine

`ScreenState` enum drives the game: `Title` -> `Playing` -> `WaveTransition` -> back to `Playing` (or `GameOver` -> `Title`). State-scoped entities use `DespawnOnExit(ScreenState::...)` for automatic cleanup.

### Fixed Update System Sets (60 Hz, deterministic)

Systems execute in this order via `GameplayFixedSet`:
1. **Timers** -- tick all cooldowns
2. **Player** -- respawn, move, fire
3. **Formation** -- advance invader grid, invader fire, UFO spawn
4. **Movement** -- apply velocity to all dynamic entities
5. **Collision** -- AABB projectile collisions, out-of-bounds cleanup
6. **Resolve** -- process ScoreEvent/PlayerHitEvent messages

Variable-rate Update handles input caching, state transitions (title/game-over input, wave clear detection, wave transition timer).

### Key Resources

- **GameConfig** -- all tuning constants (speeds, sizes, positions, limits)
- **SessionState** -- score, lives, wave number
- **FormationState** -- invader movement direction, step timer, fire column cursor
- **Cooldowns** -- reusable timers (player fire, invader fire, UFO spawn, respawn, wave transition)
- **PlayerIntent** -- cached input (move axis, fire requested)

### Key Components

`Player`, `Invader` (row_kind + grid position), `Projectile` (owner), `ShieldCell`, `Ufo`, `Velocity`, `Collider` (AABB size).

### Event Communication

`ScoreEvent` and `PlayerHitEvent` decouple collision detection from score/life resolution.

## Development Conventions

- Keep gameplay logic in gameplay.rs, rendering/UI in presentation.rs.
- Use `FixedUpdate` for deterministic simulation; variable `Update` only for input and state transitions.
- Prefer `DespawnOnExit` for state-scoped entities.
- Use system sets with `.before()`/`.after()` ordering, not one large serialized chain.
- Simple ECS data components over large object structs.
- If touching scheduling, preserve the six-phase gameplay flow (timers -> player -> formation -> movement -> collision -> resolve).
- Tests use Bevy's `App` testing with `MinimalPlugins` + `StatesPlugin`; add new tests in the existing `#[cfg(test)]` module in gameplay.rs.
