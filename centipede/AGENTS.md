# AGENTS.md

## Project

- Name: `centipede`
- Language: Rust
- Engine: Bevy `0.18.1`
- Crate type: single binary game prototype

This repository is a small Centipede-style arcade game built with Bevy. Favor changes that are readable, deterministic, and easy to extend without overengineering.

## Run And Verify

- Run the game with `cargo run`
- Format with `cargo fmt`
- Check compilation with `cargo check`
- Lint with `cargo clippy --all-targets --all-features -- -D warnings`

Before finishing non-trivial changes, run `cargo fmt`, `cargo check`, and `cargo clippy --all-targets --all-features -- -D warnings`.

## Architecture

- `src/main.rs`: app setup, window config, shared resource initialization, state reset flow
- `src/scheduling.rs`: shared gameplay system sets and frame-phase ordering
- `src/states.rs`: app state enum
- `src/constants.rs`: grid, speed, timing, score, and playfield constants
- `src/components.rs`: ECS components and marker types
- `src/resources.rs`: long-lived gameplay resources and timers
- `src/player.rs`: player spawn, input, shooting, and respawn flow
- `src/bullet.rs`: bullet movement and cleanup
- `src/mushroom.rs`: mushroom spawning and mushroom appearance helpers
- `src/centipede.rs`: centipede spawning, chain movement, poisoned-mushroom behavior, and wave progression
- `src/enemies.rs`: flea, spider, and scorpion spawning and movement
- `src/collision.rs`: collision handling and gameplay consequences
- `src/ui.rs`: HUD, main menu, and game-over UI flow

## Bevy Conventions

- Prefer focused plugins and systems over large all-in-one gameplay systems.
- Keep gameplay flow inside the configured `GameplaySet` order:
  `Input -> Movement -> Collision -> Cleanup`
- If a system depends on another system's side effects, make ordering explicit with `.chain()`, `.before()`, or `.after()`.
- Use `run_if(in_state(...))` or schedule-level conditions for state-specific behavior instead of branching everywhere inside systems.
- Use `DespawnOnExit(AppState::Playing)` for gameplay entities that should be cleaned up on state exit.
- Prefer shared spawn helpers when multiple systems create the same entity shape or setup.
- Use resources for truly global state and timers, not as a replacement for per-entity components.
- Prefer simple components and direct queries over clever abstractions unless duplication is clearly becoming a problem.

## Gameplay Rules To Preserve

- Window size is fixed by `WINDOW_WIDTH` and `WINDOW_HEIGHT` in `src/constants.rs`, and the window is currently non-resizable.
- The game should begin in `AppState::MainMenu`, enter `Playing` from Enter, and transition to `GameOver` when lives reach zero.
- A new run should fully reset score, lives, wave, chain IDs, mushroom lookup state, spawn timers, and respawn state.
- Extra lives should be awarded repeatedly at the configured score interval, not just once.
- Poisoned mushrooms should affect centipede behavior, and split chains should preserve their correct movement direction.
- Gameplay entities should not leak across `Playing` sessions.

## Coding Guidelines

- Match the existing module layout unless a clear architectural benefit justifies moving code.
- Reuse `src/constants.rs` instead of duplicating gameplay values across files.
- Keep systems readable and straightforward; this repo favors clarity over abstraction-heavy patterns.
- If adding a new order-sensitive mechanic, place it in the appropriate `GameplaySet` and document the dependency in code if it is not obvious.
- When touching UI, prefer change detection over rebuilding text every frame unnecessarily.
- If adding new resources or components that change often, consider how they affect change detection and scheduling.

## Performance Expectations

- This is a small game, so prioritize correctness and clarity first.
- Avoid needless allocations or repeated scans in hot paths when a simple cached approach is available.
- Collision logic is intentionally simple for this scale; do not introduce a much more expensive approach without a concrete reason.

## When Making Changes

- Preserve user changes already present in the worktree unless explicitly asked to modify them.
- Do not bundle unrelated refactors with gameplay fixes.
- If a change affects gameplay rules or feel, mention that behavior change clearly in the final summary.
- If you could not verify a change by running checks or a playable build, say so explicitly.
