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

- **ShipPlugin** (`ship.rs`) - shooting (Update), rotation/thrust (FixedUpdate), bullet lifetime, invincibility
- **AsteroidPlugin** (`asteroids.rs`) - asteroid rotation, wave progression
- **CollisionPlugin** (`collision.rs`) - circle-circle hit detection, observers for scoring and fragmentation, life loss
- **UiPlugin** (`ui.rs`) - HUD (score/lives) with change detection, game-over overlay and restart

Supporting modules:
- `main.rs` - app setup, window config, all gameplay constants, startup system, global systems (movement, screen wrap), `GameSet` ordering across Update and FixedUpdate
- `components.rs` - ECS components and markers (`Ship`, `Asteroid`, `Bullet`, `Velocity`, `Lifetime`, `Invincible`, `ShootCooldown`, UI markers), `AsteroidDestroyed` event
- `resources.rs` - `GameData` (score/lives/wave), `GameAssets` (cached mesh/material handles)
- `spawn.rs` - shared spawn helpers for ship, bullets, asteroids
- `state.rs` - `AppState` enum (`Playing`, `GameOver`)

### System Ordering

Systems are split across two schedules:

- **Update**: `GameSet::Input` — systems that read `just_pressed` (shooting). HUD update runs in Update without a GameSet.
- **FixedUpdate**: `GameSet::Movement` → `GameSet::Collision` → `GameSet::Cleanup` — all physics, collision detection, timer ticks, and wave management.

```
Update:    Input (shoot)          HUD update
FixedUpdate: Movement -> Collision -> Cleanup
```

Systems that only read `pressed()` (rotation, thrust) are safe in FixedUpdate. Systems that use `just_pressed()` (shooting) must stay in Update to avoid missed or doubled inputs.

State-specific behavior uses `run_if(in_state(...))`, not internal branching.

### Key Patterns

- **GameAssets resource**: meshes/materials are created once at startup and cloned on spawn. Do not recreate assets per entity.
- **ShootCooldown component**: per-ship weapon cooldown, spawned as part of the ship entity (not a global resource). Ticked in Update alongside `just_pressed` checks.
- **Invincibility**: modeled as a timer component. Collision queries use `Without<Invincible>` to exclude protected ships.
- **Asteroid destruction observers**: `bullet_asteroid_collision_system` triggers `AsteroidDestroyed` events via `commands.trigger()`. Two independent observers respond: one for scoring, one for fragment spawning. Ship-asteroid collision remains inline (single code path, not a fan-out).
- **Frame-rate independent drag**: `velocity *= SHIP_DRAG.powf(60.0 * dt)` — the constant `0.97` was tuned as a per-frame factor at 60fps; the `powf` formulation makes it timestep-independent.
- **HUD updates**: gated behind `game_data.is_changed()` to respect Bevy's change detection.
- **Collision deduplication**: uses `HashSet` to prevent one bullet hitting multiple asteroids simultaneously.
- **Ship forward direction**: `transform.up().truncate()` gives the forward vector (nose of the triangle).
- **Z-ordering**: ship at z=1, asteroids/bullets at z=0.
- **Reflect derives**: all components derive `Reflect` for compatibility with Bevy debugging/inspector tools.

## Bevy Conventions

- Prefer plugins and focused systems over monolithic systems.
- Physics and collision systems belong in `FixedUpdate`. Input that uses `just_pressed` belongs in `Update`.
- New order-sensitive systems must be placed in the correct `GameSet` variant or have explicit ordering (`.chain()`, `.before()`, `.after()`).
- Put reusable spawn logic in `spawn.rs`.
- `GameData` is for game progression state only; use Bevy `Timer` components for per-entity cooldowns/lifetimes.
- Prefer simple components over premature abstractions.
- Derive `Reflect` on new components.
- Use observers (`commands.trigger()` + `add_observer()`) when a single event fans out to multiple independent responses. Keep inline handling for single-consumer logic.

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
