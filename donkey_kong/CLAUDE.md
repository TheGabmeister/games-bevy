# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Scope

This project lives inside a monorepo. Stay scoped to this directory only — do not read, edit, or run git commands that reference parent or sibling directories.

## Build & Run Commands

```bash
cargo run          # Build and run the game
cargo build        # Build only
cargo check        # Fast type-check without full compilation
cargo clippy       # Lint
```

## Dev Profile

Dependencies compile at `opt-level = 3` while the main crate uses `opt-level = 1` — this gives fast iteration on game code while keeping dependency performance reasonable.

## Tech Stack

- **Bevy 0.18.1** — ECS game engine
- **Rust Edition 2024**

## Game Overview

## Architecture

### State Machine

`AppState`: `StartScreen → Playing → Dying → Playing (retry) / GameOver` and `Playing → WaveTally → Playing (next wave) / WinScreen`.

- `LevelPlugin` owns the `OnEnter(Playing)` lifecycle — it cleans up, spawns the stage, and then spawns the HUD in a chained sequence.
- `UiPlugin` owns all screen UI (start, game over, win, death overlay, wave tally) and the HUD update loop.

### Plugin Responsibilities

| Plugin | Systems |
|---|---|
| `LevelPlugin` | Stage spawn/cleanup on state entry, dying entry, entity lifecycle |
| `PlayerPlugin` | Input reading, walking/jumping/falling/climbing movement, hammer timer |
| `HazardsPlugin` | DK throw AI, barrel rolling/falling/ladder descent, fireball patrol/pursuit |
| `CombatPlugin` | All collision checks, scoring, bonus timer, bonus items, goal detection |
| `UiPlugin` | All screen spawning/cleanup, HUD updates, death sequence animation, wave tally |

### Entity Tagging

- `StageEntity` — permanent stage geometry (girders, ladders, DK, Pauline, oil drum). Cleared only on full reset (new run, game over, win).
- `AttemptEntity` — per-attempt entities (barrels, fireballs). Cleared on death retry and wave advance.
- Hammer pickups and bonus items are managed separately by their respective systems.

### Data Flow

- `StageData` (resource) — static level geometry: girder definitions, ladder definitions, spawn points. Created once at startup.
- `WaveConfig` (resource) — current wave's difficulty parameters. Replaced on wave advance.
- `RunData` (resource) — score, lives, wave number. Reset on new run.
- `WaveRuntime` (resource) — bonus timer, elapsed time, bonus item status, RNG. Reset on new wave or new run.
- `GameMeshes` / `GameMaterials` (resources) — shared handles created once at startup to avoid per-entity allocations.

### Coordinate System

- Logical playfield: 224×256, centered at origin. Camera uses `ScalingMode::Fixed` to map this to the 672×768 window.
- All gameplay coordinates, collision, and entity placement use this logical space.
- Girders are defined as line segments with `left` and `right` endpoints. `girder_surface_y()` interpolates the Y at any X along the girder.

### Key Bevy Conventions

- Marker components for entity classification (e.g., `#[derive(Component)] struct Player;`)
- `.run_if(in_state(AppState::Playing))` for state-gated systems
- `.after()` chains for system ordering within update phases
- Resources (`Res<T>`, `ResMut<T>`) for shared mutable game state
- 2D rendering with `Camera2d`, `Mesh2d`, `MeshMaterial2d`
- `Projection::Orthographic(...)` is the component wrapper (not `OrthographicProjection` directly)

### Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — it despawns the entity and all children. Do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`. Cast with `as u32` if your constants are `f32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`, not `bevy::render::camera`.
- Use `ApplyDeferred` (struct) not `apply_deferred` (no such function) when chaining systems that need command flushing.
