# CLAUDE.md

This file provides guidance to Claude Code when working in this repository.

## Scope

- Stay scoped to this project directory only.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Build And Validation

Run these commands from the project root:

```bash
cargo run
cargo build
cargo check
cargo clippy
```

Validation expectations:

- Run `cargo check` for most code changes.
- Run `cargo clippy` when changing API usage or broad architectural patterns.
- If validation cannot be run, say so explicitly.

Target output is redirected to `D:/cargo-target-dir` via `.cargo/config.toml`.

## Dev Profile

- The main crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Prefer small, iteration-friendly changes over broad rewrites.

## Tech Stack

- Rust edition `2024`
- Bevy `0.18.1`
- `bevy` is enabled with the `dynamic_linking` feature

## Current Repository State

The project is no longer a single-file hello-world starter, but it is still early scaffolding rather than a full game.

Current files in `src/`:

- `main.rs`
- `constants.rs`
- `components.rs`
- `messages.rs`
- `resources.rs`
- `states.rs`

Current implementation already includes:

- Window configuration
- Camera setup with bloom and tonemapping
- `AppState` and `PlayState`
- Core resource initialization
- Cross-system message definitions in `messages.rs`

The actual platformer loop, world, player, enemies, HUD, and menus are still mostly to be built.

## Architecture Direction

This repository is a **Mario-style 2D platformer**, not a generic arcade/shooter template.

When the codebase grows, prefer this structure:

- `main.rs` for app setup, plugin registration, and high-level wiring
- `constants.rs` for tunable values such as window size, physics values, timers, and palette choices
- `components.rs` for ECS markers and shared gameplay data components
- `resources.rs` for shared mutable game state such as score, lives, timer, and world metadata
- `states.rs` for `AppState`, `PlayState`, and related state helpers
- `messages.rs` for cross-system message types
- Domain modules such as `player`, `level`, `enemies`, `items`, `ui`, and `effects`

Prefer small domain plugins over growing `main.rs` indefinitely.

## State Machine Pattern

The current state model is:

- `AppState::StartScreen`
- `AppState::Playing`
- `AppState::LevelClear`
- `AppState::GameOver`

The current play sub-state model is:

- `PlayState::Running`
- `PlayState::Paused`
- `PlayState::Dying`
- `PlayState::Respawning`
- `PlayState::Cutscene`

Guidance:

- Gate gameplay systems with `.run_if(in_state(AppState::Playing))`
- Gate active gameplay logic more narrowly with `PlayState` where needed
- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry
- Prefer `DespawnOnExit(...)` for state-bound entities
- Use `.after(...)` or ordered `SystemSet`s where frame ordering matters

## Cross-System Communication

For new work in Bevy `0.18.1`, prefer Bevy's **message system** over ad hoc `Resource<Vec<_>>` queues:

- Register with `app.add_message::<T>()`
- Send with `MessageWriter<T>`
- Read with `MessageReader<T>`

Use `Observer` and `Trigger` when reactive observer-style behavior is a better fit than schedule-polled messages.

Note:

- `src/messages.rs` contains the project's cross-system message types.
- Prefer message-based communication over custom queue resources for new work.

## Timers

Use `Timer` with `Res<Time>` for cooldowns, delays, and animation timing.

- Store timers in components for per-entity timing
- Store timers in resources for global timing
- Tick timers with `timer.tick(time.delta())`
- Use `timer.is_finished()`
- Do not use frame-counting for gameplay timing

## Coding Rules

- Make the smallest coherent change that solves the task
- Do not rewrite working code just to make it "cleaner"
- Put new tunable values in `constants.rs` once that module is the obvious home
- Put new shared mutable game state in `resources.rs` once that module is the obvious home
- Put shared marker/data ECS types in `components.rs` once that module is the obvious home
- Prefer extending an existing domain plugin over adding many ad hoc systems to `main.rs`
- Use marker components for entity classification
- Keep module boundaries aligned to platformer gameplay domains

## Query Filters

Use query filters for both clarity and performance:

- `With<T>` and `Without<T>` to narrow queries without reading component data
- `Changed<T>` when logic should run only after mutation
- `Added<T>` for newly inserted components

## Assets

This project currently does **not** rely on checked-in game assets for core gameplay.

If assets are introduced later:

- Use plain relative paths with `asset_server.load(...)`
- Keep references aligned with files under `assets/`
- Store reused `Handle<T>` values in a resource when appropriate

Do not design new gameplay features assuming an asset pipeline already exists.

## Bevy 0.18.1 API Notes

- `despawn()` is recursive by default; do not use `despawn_recursive()`
- `WindowResolution::new(width, height)` expects `u32`
- `ScalingMode` is in `bevy::camera::ScalingMode`
- Use `ApplyDeferred` rather than a nonexistent `apply_deferred` helper function
- 2D rendering uses current APIs such as `Camera2d`, `Sprite`, `Mesh2d`, and `MeshMaterial2d`
- `WindowResolution` is not in the prelude; import it from `bevy::window::WindowResolution`
- `Text2d::new("text")` works for world-space text such as score popups
- `ColorMaterial` does not have an `emissive` field

### Bloom And HDR

- Use `Bloom`, not `BloomSettings`
- Typical import:

```rust
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
};
```

- Bright colors can drive bloom without a dedicated emissive material field
- Keep the scene readable even if bloom is removed or toned down

### State-Scoped Entities

- `DespawnOnExit<S: States>` and `DespawnOnEnter<S: States>` are available in Bevy `0.18.1`
- Use them for entities that should be cleaned up automatically on state transitions

Example:

```rust
commands.spawn((MyComponent, DespawnOnExit(AppState::Playing)));
```

### SubStates

Use sub-states for gameplay flow inside `AppState::Playing`.

Example:

```rust
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
enum PlayState {
    #[default]
    Running,
    Paused,
}
```

Register with:

```rust
app.init_state::<AppState>().add_sub_state::<PlayState>();
```

## Preferred Change Pattern

1. Inspect the current code and module boundaries before making assumptions.
2. Implement the change in the owning file or module.
3. Extract new modules only when the code is large enough to justify them.
4. Run validation, usually `cargo check`.
5. Summarize what changed and any remaining risks.

## Good First Places To Look

- App boot and current wiring: `src/main.rs`
- State definitions: `src/states.rs`
- Shared game data: `src/resources.rs`
- Shared ECS types: `src/components.rs`
- Cross-system communication types: `src/messages.rs`
- Build output location: `.cargo/config.toml`
- Dependency configuration: `Cargo.toml`
