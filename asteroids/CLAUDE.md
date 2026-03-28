# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Rust Asteroids game built with Bevy 0.18.1 (Rust 2024 edition). Single binary, single dependency (`bevy`).

## Build & Verify

```bash
cargo run              # run the game
cargo check            # check compilation
cargo fmt              # format code
cargo clippy --all-targets --all-features  # lint
```

Run `cargo fmt` and `cargo check` at minimum before finishing non-trivial changes. No test suite exists; verification is manual via `cargo run`.

## Architecture

Plugin-based ECS with four domain plugins registered in `main()`:

- **ShipPlugin** (`ship.rs`) - player input, shooting, bullet cleanup, invincibility
- **AsteroidPlugin** (`asteroids.rs`) - asteroid rotation, wave progression
- **CollisionPlugin** (`collision.rs`) - circle-circle hit detection, fragmentation, scoring, life loss
- **UiPlugin** (`ui.rs`) - HUD (score/lives) with change detection, game-over overlay and restart

Supporting modules:
- `main.rs` - app setup, window config, all gameplay constants, startup system, global systems (movement, screen wrap), `GameSet` ordering
- `components.rs` - ECS components and markers (`Ship`, `Asteroid`, `Bullet`, `Velocity`, `Lifetime`, `Invincible`, UI markers)
- `resources.rs` - `GameData` (score/lives/wave), `ShootCooldown`, `GameAssets` (cached mesh/material handles)
- `spawn.rs` - shared spawn helpers for ship, bullets, asteroids
- `state.rs` - `AppState` enum (`Playing`, `GameOver`)

### System Ordering

All gameplay systems run within `GameSet`, enforced in this order:

```
Input -> Movement -> Collision -> Cleanup
```

State-specific behavior uses `run_if(in_state(...))`, not internal branching.

### Key Patterns

- **GameAssets resource**: meshes/materials are created once at startup and cloned on spawn. Do not recreate assets per entity.
- **Invincibility**: modeled as a timer component. Collision queries use `Without<Invincible>` to exclude protected ships.
- **HUD updates**: gated behind `game_data.is_changed()` to respect Bevy's change detection.
- **Collision deduplication**: uses `HashSet` to prevent one bullet hitting multiple asteroids simultaneously.
- **Ship forward direction**: `transform.up().truncate()` gives the forward vector (nose of the triangle).
- **Z-ordering**: ship at z=1, asteroids/bullets at z=0.

## Bevy Conventions

- Prefer plugins and focused systems over monolithic systems.
- New order-sensitive systems must be placed in the correct `GameSet` variant or have explicit ordering (`.chain()`, `.before()`, `.after()`).
- Put reusable spawn logic in `spawn.rs`.
- `GameData` is for game progression state only; use Bevy `Timer` components for per-entity cooldowns/lifetimes.
- Prefer simple components over premature abstractions.

## Gameplay Rules To Preserve

- Fixed 800x600 window, non-resizable. Screen wrapping uses constants from `main.rs`.
- Ship spawns at center with temporary invincibility.
- State flow: `Playing` -> `GameOver` (on 0 lives) -> restart resets score/lives/wave/ship.
- Asteroids fragment: Large -> 2 Medium -> 2 Small each.
- Wave progression: +1 asteroid per wave, capped at 8.

## Coding Guidelines

- Match existing module layout unless there's a clear benefit to restructuring.
- Reuse shared constants from `main.rs`; don't duplicate gameplay values across files.
- Keep systems readable and direct; clarity over cleverness.
- If adding a new resource/component that changes every frame, consider change detection impact.
- If a change affects gameplay behavior, mention it clearly.
