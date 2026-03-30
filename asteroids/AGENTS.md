# AGENTS.md

## Project

- Name: `asteroids`
- Language: Rust
- Edition: `2024`
- Engine: Bevy `0.18.1`
- Crate type: single binary game prototype

This repository is a small Asteroids-style game built with Bevy. Keep changes simple, idiomatic, and easy to grow as the project expands.

## Run And Verify

- Run the game with `cargo run`
- Format with `cargo fmt`
- Check compilation with `cargo check`
- Lint with `cargo clippy --all-targets --all-features`

Run `cargo fmt` and `cargo check` at minimum before finishing non-trivial changes. There is no automated test suite; gameplay verification is manual via `cargo run`.

## Architecture

The game is organized around four domain plugins registered in `src/main.rs`:

- `ShipPlugin` in `src/ship.rs`: shooting in `Update`, ship rotation/thrust in `FixedUpdate`, bullet lifetime cleanup, invincibility timer updates
- `AsteroidPlugin` in `src/asteroids.rs`: asteroid rotation and wave progression
- `CollisionPlugin` in `src/collision.rs`: bullet/asteroid and ship/asteroid collision handling, score updates, asteroid fragmentation
- `UiPlugin` in `src/ui.rs`: HUD setup and updates, game-over overlay, restart flow

Supporting modules:

- `src/main.rs`: app setup, window config, shared gameplay constants, startup spawning, global movement and screen wrapping, `GameSet` ordering
- `src/components.rs`: ECS components and markers, `AsteroidSize`, `AsteroidDestroyed` event, UI marker components
- `src/resources.rs`: `GameData` for score/lives/wave and `GameAssets` for cached meshes/materials
- `src/spawn.rs`: shared spawn helpers for ship, bullets, asteroids, waves, fragments, plus asteroid utility helpers
- `src/state.rs`: `AppState` enum

## System Scheduling

Systems are intentionally split across two schedules:

- `Update`: `GameSet::Input` for systems that rely on `just_pressed`, especially shooting
- `FixedUpdate`: `GameSet::Movement -> GameSet::Collision -> GameSet::Cleanup` for physics, collision, timer ticking, and wave management

Keep this distinction intact:

- Input based on `just_pressed(...)` should stay in `Update`
- Physics, movement, collision, and timer-driven cleanup should stay in `FixedUpdate`
- If a system depends on another system's side effects, make ordering explicit with `.chain()`, `.before()`, or `.after()`
- Prefer `run_if(in_state(...))` for state-specific behavior instead of branching inside systems

## Bevy Conventions

- Prefer plugins and focused systems over large all-in-one systems
- Put reusable spawn logic in `src/spawn.rs`
- Keep `GameData` for actual game progression state only; do not mix per-frame timers into it
- Prefer Bevy `Timer` components for cooldowns, lifetimes, invincibility, and other temporary effects
- Cache meshes and materials in `GameAssets` and clone handles when spawning entities
- Prefer simple ECS data and plain components over premature abstractions
- Derive `Reflect` on new components to match the existing codebase pattern
- Use observers when one gameplay event fans out to multiple independent responses

## Gameplay Rules To Preserve

- Window size is fixed at `800x600` and is non-resizable
- Screen wrapping assumes the fixed playfield constants in `src/main.rs`
- The ship spawns at the center with temporary invincibility
- The game starts in `AppState::Playing` and transitions to `GameOver` when lives reach zero
- Restart flow is handled from the game-over state and should reset score, lives, wave, and ship state cleanly
- Large asteroids split into 2 medium asteroids, and medium asteroids split into 2 small asteroids
- Wave progression adds 1 starting asteroid per wave and caps at 8
- The ship should render above asteroids, and bullets should spawn slightly ahead of the ship nose

## Coding Guidelines

- Match the existing module layout unless there is a clear benefit to restructuring
- Reuse shared constants from `src/main.rs`; do not duplicate gameplay values across files
- Keep systems readable and direct; this codebase favors clarity over cleverness
- If adding new order-sensitive gameplay, document the intended schedule and set placement
- If you introduce a new resource or component that changes every frame, consider how it affects change detection
- If a change affects gameplay behavior, mention the behavior change clearly in the final summary

## Performance Expectations

- This is a small game, so straightforward code is preferred
- Avoid unnecessary allocations in hot paths when an equally simple approach exists
- Collision is currently simple and acceptable for this scale, but avoid making it dramatically more expensive without a clear reason

## When Making Changes

- Preserve existing user changes in the worktree unless explicitly asked to modify them
- Do not add unrelated refactors
- If you cannot verify a change locally, say so explicitly
