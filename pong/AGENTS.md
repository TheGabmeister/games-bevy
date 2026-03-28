# AGENTS.md

## Project

- Name: `bevy-template`
- Language: Rust
- Engine: Bevy `0.18.1`
- Crate type: single binary game prototype

This repository is a small Pong-style game built with Bevy. Favor changes that keep the code easy to read, easy to verify, and easy to extend without adding unnecessary abstraction.

## Run And Verify

- Run the game with `cargo run`
- Format with `cargo fmt`
- Check compilation with `cargo check`
- Lint with `cargo clippy --all-targets --all-features -- -D warnings`
- Run tests with `cargo test`

Before finishing non-trivial changes, run `cargo fmt`, `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.

## Architecture

- `src/main.rs`: app setup, window config, shared resource initialization, plugin registration
- `src/state.rs`: app state enum for `Menu`, `Playing`, and `Winner`
- `src/components.rs`: ECS components, marker types, gameplay config, score, winner state, and gameplay events
- `src/audio.rs`: audio asset loading, looping background music, and paddle-hit sound effect observer
- `src/systems/mod.rs`: camera setup, state transition wiring, gameplay system-set ordering, and state cleanup
- `src/systems/menu.rs`: menu text spawn and start-input handling
- `src/systems/gameplay.rs`: gameplay entity spawning, paddle movement, ball movement, collisions, scoring, HUD updates, and gameplay logic tests
- `src/systems/winner.rs`: winner screen spawn and return-to-menu input handling

## Bevy Conventions

- Keep state-specific behavior scheduled with `OnEnter`, `OnExit`, and `run_if(in_state(...))` rather than branching inside systems.
- Keep gameplay frame flow inside the configured `GameplaySet` order:
  `Movement -> Collision -> Scoring -> Presentation`
- If a system depends on another system's side effects, make the dependency explicit with sets, `.before()`, `.after()`, or `.chain()`.
- Cache shared meshes and materials in resources and clone handles when spawning gameplay entities.
- Prefer focused systems and simple components over large multi-purpose systems.
- Use resources only for truly global state such as match configuration, score, winner state, and cached asset handles.
- Prefer change detection for UI updates instead of rewriting text every frame unnecessarily.
- Keep audio event-driven where possible; paddle-hit SFX currently uses an observer on `PaddleHitEvent`.

## Gameplay Rules To Preserve

- Window size is fixed at `960x540` and currently non-resizable.
- The game flow is `Menu -> Playing -> Winner -> Menu`.
- Starting a match from the menu should reset score and clear the previous winner.
- Gameplay entities should be cleaned up when leaving `Phase::Playing`.
- The ball should reset to center after a point unless that point ends the match.
- Paddle collisions should reflect the ball away from the paddle and increase its speed by the configured hit bonus.
- The winner screen should show the winning side and final score, then return to menu on Enter.

## Coding Guidelines

- Match the existing module layout unless a clear architectural benefit justifies restructuring.
- Keep constants centralized in `src/systems/gameplay.rs` unless there is enough growth to justify a dedicated constants module.
- Reuse `MatchConfig` instead of duplicating gameplay values in multiple places.
- Keep systems direct and readable; this repo favors clarity over clever abstractions.
- If adding new gameplay logic that is order-sensitive, place it in the correct `GameplaySet` and document the dependency in code if it is not obvious.
- When adding tests, prefer focused logic tests for pure helpers and state transitions.

## Performance Expectations

- This is a small arcade prototype, so prioritize correctness and maintainability first.
- Avoid repeated asset creation during state transitions when the assets can be cached once and reused.
- Avoid needless allocations or repeated full scans in hot paths when a simpler cached approach is equally clear.

## When Making Changes

- Preserve user changes already present in the worktree unless explicitly asked to modify them.
- Do not bundle unrelated refactors with gameplay fixes.
- If a change affects gameplay feel or rules, mention that behavior change clearly in the final summary.
- If you could not verify a change with local checks or a playable run, say so explicitly.
