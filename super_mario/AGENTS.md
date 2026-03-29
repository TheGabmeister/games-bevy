# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\super_mario`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- `bevy` is enabled with the `dynamic_linking` feature
- Current app state: early platformer scaffolding, not yet a playable Mario clone

## Build And Validation

Use these commands from the project root:

```powershell
cargo run
cargo build
cargo check
cargo clippy
```

Validation expectations:

- Run `cargo check` for most code changes.
- Run `cargo clippy` when changing patterns that may affect API usage or code quality broadly.
- If you cannot run validation, say so explicitly.

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Keep iteration-friendly workflows in mind; prefer targeted changes over broad rewrites.

## Current Project Layout

- `src/main.rs`: app bootstrap, window/camera setup, state initialization, and message registration
- `src/constants.rs`: tunable values and palette constants
- `src/components.rs`: shared ECS marker and data components
- `src/resources.rs`: shared mutable game data scaffolding
- `src/states.rs`: `AppState` and `PlayState`
- `src/messages.rs`: cross-system gameplay messages
- `.cargo/config.toml`: shared cargo target-dir configuration

## Current Runtime Behavior

The current app is scaffolding, not yet a playable game:

- `DefaultPlugins` are registered with custom window configuration.
- A `Camera2d` is spawned at startup with bloom and tonemapping configured.
- `AppState` and `PlayState` are initialized.
- Shared game data and gameplay message types are registered.
- There is no world generation, player controller, collision system, HUD, menu flow, or gameplay loop yet.

When making changes, align your work with what actually exists in the repo rather than assuming the larger game architecture is already implemented.

## Architecture Guidance For Future Expansion

This project is a Mario-style 2D platformer, not a generic arcade/shooter template.

Prefer this structure as the project grows:

- `src/main.rs`: app setup, plugin registration, high-level wiring
- `src/constants.rs`: tunable values such as window size, speeds, jump forces, timers, and colors
- `src/components.rs`: marker and data ECS components
- `src/resources.rs`: shared mutable game-wide state
- `src/states.rs`: `AppState`, `PlayState`, and state-related helpers
- `src/messages.rs`: cross-system gameplay messages
- Domain modules such as `src/player.rs`, `src/level.rs`, `src/enemies.rs`, `src/items.rs`, `src/ui.rs`, and `src/effects.rs`

Prefer small domain plugins over growing `main.rs` indefinitely once the game has more than a handful of systems.

## Bevy Conventions To Follow

- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry once states are introduced.
- Gate gameplay systems with `run_if(in_state(...))` once an `AppState` exists.
- Use `PlayState` to gate active gameplay more narrowly inside `AppState::Playing`.
- Use resources for cross-system shared state.
- Use messages for schedule-driven cross-system communication.
- Use marker components for entity categories.
- Use explicit system ordering with `.after(...)` where frame ordering matters.

## Messages And Observers

- For cross-system gameplay communication, prefer Bevy `Message`s:
  - register with `app.add_message::<T>()`
  - write with `MessageWriter<T>`
  - read with `MessageReader<T>`
- Prefer messages for gameplay data flow such as score, particles, damage, and camera shake.
- Use observer `Event`s only when reactive trigger-style behavior is a better fit than schedule-polled messages.

## Coding Rules For This Repo

- Make the smallest coherent change that solves the task.
- Do not rewrite working structure just to make it "cleaner".
- Keep module boundaries aligned to gameplay domains once modules are introduced.
- Put new tunable values in `src/constants.rs` instead of scattering magic numbers once that module exists.
- Add new shared mutable game state to `src/resources.rs` once that module exists.
- Add shared marker/data ECS types to `src/components.rs` once that module exists.
- Add cross-system gameplay messages to `src/messages.rs` once that module exists.
- Prefer extending an existing domain plugin over registering many ad hoc systems from `main.rs`.
- When spawning entities tied to a state, define the matching cleanup path on `OnExit` if they should not persist.

## UI And Asset Notes

- UI currently uses Bevy's component-based UI directly.
- The project does not currently rely on checked-in assets for core gameplay.
- If assets are introduced later, keep asset paths as plain relative strings passed to `asset_server.load(...)`.
- Do not design new gameplay features assuming an asset pipeline already exists.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`.
- Use `ApplyDeferred` rather than a nonexistent `apply_deferred` helper function.
- 2D rendering uses current Bevy APIs such as `Camera2d`, `Sprite`, `Mesh2d`, and `MeshMaterial2d`.
- `WindowResolution` is not in the prelude; import it from `bevy::window::WindowResolution`.
- In Bevy 0.18, old queue-style event usage is now modeled with messages (`add_message`, `MessageWriter`, `MessageReader`).

## Preferred Change Pattern

1. Inspect the current code and local module boundaries before making assumptions.
2. Implement the change in the owning file or module.
3. Extract constants/resources/components/messages only when the code has grown enough to justify it.
4. Run validation, usually `cargo check`.
5. Summarize what changed and any remaining risks.

## Good First Places To Look

- App boot or current behavior: `src/main.rs`
- State definitions: `src/states.rs`
- Shared game data: `src/resources.rs`
- Shared ECS types: `src/components.rs`
- Cross-system gameplay messages: `src/messages.rs`
- Build output location: `.cargo/config.toml`
- Dependency/runtime configuration: `Cargo.toml`
