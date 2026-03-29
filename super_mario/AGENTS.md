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
- Current app state: playable Mario-style slice through early enemy/power-up work, with level spawning, player movement, block interactions, enemies, shells, and mushrooms already implemented

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

- `src/main.rs`: app bootstrap, window/camera setup, state initialization, resource initialization, and gameplay message registration
- `src/constants.rs`: tunable values for windowing, physics, player/enemy tuning, scoring, and palette constants
- `src/components.rs`: shared ECS marker and data components for player, enemies, items, tiles, and collision/power state
- `src/level.rs`: data-driven World 1-1-inspired layout, primitive world spawning, camera bounds/follow, and temporary boot-to-playing bridge
- `src/player.rs`: player spawning, input, jump/gravity, collision resolution, facing updates, and invulnerability flashing
- `src/blocks.rs`: block hit handling, bump animations, score popups, coin pops, brick debris, mushroom emergence, mushroom movement, and mushroom collection
- `src/enemies.rs`: Goomba/Koopa spawning, enemy movement, shell behavior, player collision handling, stomp effects, and enemy score popups
- `src/resources.rs`: shared mutable game data, level bounds, player start data, and enemy spawn definitions
- `src/states.rs`: `AppState` and `PlayState`
- `src/messages.rs`: cross-system gameplay messages plus a few forward-looking message types not fully consumed yet
- `.cargo/config.toml`: shared cargo target-dir configuration

## Current Runtime Behavior

The current app is a playable platformer slice, not just scaffolding:

- `DefaultPlugins` are registered with custom window configuration.
- A `Camera2d` is spawned at startup with bloom and tonemapping configured.
- `AppState` and `PlayState` are initialized.
- Shared game data and gameplay message types are registered.
- A World 1-1-inspired level is spawned from Rust data using primitive meshes and colors, including gaps, pipes, a staircase, a flagpole, and a castle.
- The app currently boots directly into `AppState::Playing` as a temporary bridge until the real start screen phase is implemented.
- A player entity spawns, moves left/right, jumps, collides with solids, drives a side-scrolling camera, can grow to Big Mario, and flashes during post-hit invulnerability.
- Question blocks, brick blocks, and hard blocks respond to upward hits; question blocks can spawn coins or mushrooms, and Big Mario can break bricks.
- Goombas and Koopas spawn from level data, patrol, reverse on collision, can be stomped, and Koopas transition into kickable shells.
- Moving shells can defeat enemies, and enemy/block interactions already spawn simple world-space score popups and particle-like transient effects.
- `GameData` currently tracks score, coins, lives, timer, and world label, including `100`-coin extra-life behavior, but there is not yet a screen-space HUD or active timer countdown.
- There is still no start menu, pause flow, flagpole/castle completion sequence, respawn/game-over loop, or audio lifecycle yet.

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
- Domain modules such as `src/player.rs`, `src/level.rs`, `src/blocks.rs`, `src/enemies.rs`, `src/items.rs`, `src/ui.rs`, and `src/effects.rs`

Prefer small domain plugins over growing `main.rs` indefinitely once the game has more than a handful of systems. The current repo already follows this pattern with `LevelPlugin`, `PlayerPlugin`, `BlocksPlugin`, and `EnemyPlugin`.

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

- Screen-space UI is not implemented yet.
- The project does not currently rely on checked-in assets for core gameplay.
- If assets are introduced later, keep asset paths as plain relative strings passed to `asset_server.load(...)`.
- Do not design new gameplay features assuming an asset pipeline already exists.
- Current visuals are mostly primitive-driven via `Mesh2d`, `MeshMaterial2d<ColorMaterial>`, and `Text2d` for world-space labels and block markers.

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
- Enemy behavior: `src/enemies.rs`
- Level layout and spawn data: `src/level.rs`
- Cross-system gameplay messages: `src/messages.rs`
- Build output location: `.cargo/config.toml`
- Dependency/runtime configuration: `Cargo.toml`
