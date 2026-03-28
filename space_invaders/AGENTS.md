# AGENTS.md

This repository is a small Bevy game project built with Rust.

## Scope

- Stay within this repository.
- Do not read or modify parent directories.
- Prefer small, behavior-preserving changes unless the task clearly asks for a refactor.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- Game entry point: [src/main.rs](src/main.rs)
- Main game modules:
  - [src/game/gameplay.rs](src/game/gameplay.rs)
  - [src/game/presentation.rs](src/game/presentation.rs)

## Project Shape

- `main.rs` configures Bevy plugins, window setup, clear color, and the 2D camera.
- `gameplay.rs` owns state, gameplay systems, ECS components, collision logic, and tests.
- `presentation.rs` owns visuals and HUD/UI construction.

## Preferred Patterns

- Follow Bevy 0.18 APIs and patterns.
- Prefer state-driven setup/teardown with `DespawnOnExit(...)` for state-scoped entities.
- Keep gameplay logic in `gameplay.rs` and rendering/UI concerns in `presentation.rs`.
- Use `FixedUpdate` for deterministic gameplay simulation.
- Use targeted ordering via system sets, `.before()`, `.after()`, or small `.chain()` groups instead of one large serialized schedule.
- Keep systems focused and composable; split oversized systems when a change would otherwise grow them further.
- Prefer simple ECS data components over large object-style structs.

## Coding Rules

- Use ASCII unless the file already requires Unicode.
- Prefer `rg` for file and text search.
- Use `apply_patch` for manual file edits.
- Do not revert user changes that are unrelated to the task.
- Avoid destructive git commands such as `git reset --hard`.

## Validation

Run these after code changes when applicable:

- `cargo fmt`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

## Notes For Future Agents

- The repo should stay `clippy`-clean under `-D warnings`.
- Tests live in `src/game/gameplay.rs`.
- If you add state-specific entities, attach the correct `DespawnOnExit(ScreenState::...)`.
- If you touch scheduling, preserve the current gameplay flow across:
  - cooldown ticking
  - player actions
  - formation behavior
  - movement
  - collision handling
  - score / hit resolution
