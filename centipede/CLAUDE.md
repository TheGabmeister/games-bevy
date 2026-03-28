# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Centipede-style arcade game built with Bevy 0.18.1 in Rust. Single binary, ~1900 lines across 12 source files. Favor readable, deterministic, easy-to-extend changes.

## Build & Verify

```bash
cargo run                                              # Run the game
cargo fmt                                              # Format code
cargo check                                            # Check compilation
cargo clippy --all-targets --all-features -- -D warnings  # Lint
```

Run `cargo fmt`, `cargo check`, and `cargo clippy` before finishing non-trivial changes. There are no automated tests; verification is visual via `cargo run`.

Build artifacts go to `D:/cargo-target-dir` (configured in `.cargo/config.toml`).

## Architecture

The game uses a plugin-per-subsystem architecture with a 3-state state machine (`MainMenu -> Playing -> GameOver`). Each plugin registers systems into a shared 4-phase gameplay loop defined in `src/scheduling.rs`:

```
GameplaySet::Input -> Movement -> Collision -> Cleanup
```

All gameplay systems run only in `AppState::Playing` via `run_if(in_state(...))`.

**Key modules:**
- `main.rs` — App setup, plugin registration, full game state reset on entering Playing
- `constants.rs` — All magic numbers: grid dimensions (25x30, 28px cells), speeds, timers, scoring values
- `components.rs` — ECS components (13 types: markers, grid positions, enemy state)
- `resources.rs` — Global state: Score, Lives, Wave, MushroomGrid (HashMap for O(1) lookup), spawn timers, CentipedeChains (HashMap<chain_id, Vec<Entity>>)
- `centipede.rs` — Chain-based movement on a discrete grid; chains split on hit, heads can poison-rush downward
- `collision.rs` — Circle-based overlap checks; single pass with early breaks after hits
- `enemies.rs` — Flea/Spider/Scorpion each capped at one active instance, with spawn timers

**Two coordinate systems coexist:** centipede/mushrooms use discrete grid positions (`GridPos`), while player/bullets/enemies use continuous world coordinates (`Transform`). Conversion helpers are in `constants.rs`.

## Conventions

- Use `DespawnOnExit(AppState::Playing)` on all gameplay entities to prevent leaks between sessions
- Place new order-sensitive systems in the appropriate `GameplaySet` phase
- Reuse `constants.rs` for gameplay values; don't duplicate numbers
- Use change detection for UI updates, not per-frame rebuilds
- Keep systems focused; prefer simple components and direct queries over abstractions
- Use shared spawn helpers when multiple systems create the same entity type
- Don't bundle unrelated refactors with gameplay fixes
- If a change affects gameplay rules or feel, mention that clearly
