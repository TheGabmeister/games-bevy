# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Scope

This project lives inside a monorepo. Stay scoped to this directory only — do not read, edit, or run git commands that reference parent or sibling directories.

## Build & Run Commands

```bash
cargo run          # Build and run the game
cargo build        # Build only
cargo check        # Fast type-check (use for most code changes)
cargo clippy       # Lint (use when changing API patterns broadly)
```

Target dir is redirected to `D:/cargo-target-dir` via `.cargo/config.toml`.

## Dev Profile

Dependencies compile at `opt-level = 3` while the main crate uses `opt-level = 1` — fast iteration on game code with performant dependencies.

## Tech Stack

- **Bevy 0.18.1** (with `dynamic_linking` feature) — ECS game engine
- **Rust Edition 2024**

## Architecture

This is a template for 2D arcade-style Bevy games. When building out from the hello world starter, follow this modular layout:

- **`main.rs`** — App setup, plugin registration, system scheduling with explicit ordering
- **`constants.rs`** — All tunable values as named constants (window size, speeds, radii, scoring)
- **`components.rs`** — Marker components for entity types + data components (Velocity, FacingDirection, etc.)
- **`resources.rs`** — Shared game state (score, lives, wave progression)
- **`states.rs`** — `AppState` enum driving a state machine (StartScreen → Playing → GameOver)
- **Domain modules** — One module per gameplay domain (player, enemy, combat, ui, audio, etc.), each exposing a Plugin

### State Machine Pattern

Systems should be state-aware:
- Gate gameplay systems with `.run_if(in_state(AppState::Playing))`
- Use `OnEnter`/`OnExit` for spawn/cleanup symmetry
- Use `.after()` chains where frame ordering matters

### Coding Rules

- New tunable values go in `constants.rs`, not inline magic numbers
- New shared mutable game state goes in `resources.rs`
- New ECS marker/data types go in `components.rs`
- Prefer extending an existing domain plugin over registering ad hoc systems in `main.rs`
- When spawning entities on `OnEnter`, define matching cleanup on `OnExit`
- Use marker components for entity classification (e.g., `#[derive(Component)] struct Player;`)

### Assets

Asset paths are plain relative strings passed to `asset_server.load(...)` — keep them aligned with files under `assets/`. Existing assets are space-shooter themed sprites and audio.

## Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`. Cast with `as u32` if constants are `f32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`, not `bevy::render::camera`.
- Use `ApplyDeferred` (struct) not `apply_deferred` (no such function) for command flushing between systems.
- 2D rendering uses `Camera2d`, `Mesh2d`, `MeshMaterial2d`, `Sprite`.
