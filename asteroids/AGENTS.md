# AGENTS.md

## Project

- Name: `asteroids`
- Language: Rust
- Engine: Bevy `0.18.1`
- Crate type: single binary game prototype

This repository is a small Asteroids-style game built with Bevy. Keep changes simple, idiomatic, and easy to grow as the project expands.

## Run And Verify

- Run the game with `cargo run`
- Format with `cargo fmt`
- Check compilation with `cargo check`
- Lint with `cargo clippy --all-targets --all-features`

Before finishing non-trivial changes, run `cargo fmt` and `cargo check` at minimum.

## Architecture

- `src/main.rs`: app setup, window config, shared constants, startup systems, global scheduling
- `src/components.rs`: ECS components and marker types
- `src/resources.rs`: long-lived resources such as game progression and cached handles
- `src/spawn.rs`: shared spawn helpers and asteroid utility functions
- `src/ship.rs`: player input, shooting, bullet cleanup, invincibility updates
- `src/asteroids.rs`: asteroid-specific behavior and wave progression
- `src/collision.rs`: collision handling and gameplay consequences
- `src/ui.rs`: HUD and game-over flow
- `src/state.rs`: app state enum

## Bevy Conventions

- Prefer plugins and focused systems over large all-in-one systems.
- Keep gameplay flow inside the configured `GameSet` order:
  `Input -> Movement -> Collision -> Cleanup`
- If a system depends on another system's side effects, make ordering explicit with `.chain()`, `.before()`, or `.after()`.
- Use `run_if(in_state(...))` for state-specific behavior instead of branching inside systems.
- Put reusable spawn logic in `src/spawn.rs`.
- Keep `GameData` for actual game progression state only. Do not mix per-frame transient timers into it.
- Prefer Bevy `Timer` for cooldowns, lifetimes, and temporary status effects.
- Cache meshes and materials in resources and clone handles when spawning entities.
- Prefer simple ECS data and plain components over premature abstractions.

## Gameplay Rules To Preserve

- Window size is fixed at `800x600` and currently non-resizable.
- Screen wrapping assumes the fixed playfield constants in `src/main.rs`.
- The ship should spawn at the center with temporary invincibility.
- The game starts in `AppState::Playing` and transitions to `GameOver` when lives reach zero.
- Restart flow is handled from the game-over state and should reset score, lives, wave, and ship state cleanly.

## Coding Guidelines

- Match the existing module layout unless there is a clear benefit to restructuring.
- Prefer adding small components/resources over hiding logic in deeply shared structs.
- Keep systems readable and direct; this codebase favors clarity over cleverness.
- Avoid hard-coding duplicate gameplay values in multiple files. Reuse the shared constants.
- If adding new order-sensitive gameplay, document the intended set or transition point.
- If you introduce a new resource or component that changes every frame, consider how it affects change detection.

## Performance Expectations

- This is a small game, so straightforward code is preferred.
- Avoid unnecessary allocations in hot paths when an equally simple approach exists.
- Collision is currently simple and acceptable for this scale, but avoid making it dramatically more expensive without a reason.

## When Making Changes

- Preserve existing user changes in the worktree unless explicitly asked to modify them.
- Do not add unrelated refactors.
- If a change affects gameplay behavior, mention the behavior change clearly in the final summary.
- If you cannot verify a change locally, say so explicitly.
