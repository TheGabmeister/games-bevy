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
- **rand 0.9** — RNG for AI and effects

## What This Is

A playable Joust clone rendered with primitive meshes (no sprite assets). Features: flap-based movement, joust combat (higher rider wins), egg lifecycle (defeated enemies drop eggs that hatch into stronger tiers), lava kills, wave progression, 1–2 player support, HUD, start screen, and game-over flow.

## Architecture

### Module Layout

- **`main.rs`** — App setup, plugin registration, system set ordering, camera, arena spawn hook
- **`constants.rs`** — All tunable values as named constants (window size, speeds, radii, scoring, platform layout, Z layers)
- **`components.rs`** — Marker components, data components (Velocity, FacingDirection, etc.), and `Message` types
- **`resources.rs`** — Shared game state (`GameState`), cached `SharedMeshes`/`SharedMaterials` handles, wave definitions (`WaveDef`)
- **`states.rs`** — `AppState` enum, `PlayState` sub-state, `GameSet` system sets
- **Domain modules** — One module per gameplay domain, each exposing a Plugin:
  - `player.rs` — spawning, input, respawn
  - `enemy.rs` — spawning, AI (wander/pursue)
  - `physics.rs` — gravity, drag, velocity, platform collision, screen wrap
  - `combat.rs` — joust resolution, egg collection/hatching, lava kills, scoring, game-over check
  - `waves.rs` — game reset, wave intro/clear transitions
  - `ui.rs` — start screen, HUD, game-over overlay
  - `rendering.rs` — shared mesh/material setup, rider visual hierarchy, wing/facing/egg animations
  - `effects.rs` — death particles

### State Machine

Two-level state machine:

- **`AppState`**: `StartScreen` → `Playing` → `GameOver` (top-level)
- **`PlayState`** (sub-state of `Playing`): `WaveIntro` → `WaveActive` → `WaveClear` → loops back to `WaveIntro`

Most gameplay systems gate on `PlayState::WaveActive`. Some (invincibility tick, respawn, HUD updates) run across all of `AppState::Playing`.

### System Set Ordering

Systems run in a strict chain defined in `main.rs`:

```
GameSet::Input → GameSet::Ai → [ApplyDeferred] → GameSet::Physics → GameSet::Combat → [ApplyDeferred] → GameSet::Progression
```

`ApplyDeferred` flushes are inserted between Ai/Physics and Combat/Progression so later sets observe commands from earlier ones. Combat internally chains additional `ApplyDeferred` flushes between its sub-systems (joust → egg collection → egg hatch → lava kill → scoring → game over).

### Message System

Inter-system communication uses Bevy 0.18's buffered messages (not events):
- `JoustKillMessage` — emitted on any kill (joust or lava), consumed by effects for death particles
- `ScoreMessage` — point awards, consumed by score/extra-life handler
- `PlayerDiedMessage` — life loss, consumed by life handler

Messages are registered in `main.rs` with `add_message::<T>()` and accessed via `MessageWriter<T>`/`MessageReader<T>`.

### Entity Cleanup

Entities use `DespawnOnExit(AppState::Playing)` or `DespawnOnExit(PlayState::WaveIntro)` for automatic cleanup when leaving a state — no manual despawn systems needed.

### Input Bindings

- **Player 1**: A/D or Arrow Left/Right to move, W/Space/Arrow Up to flap
- **Player 2**: J/L to move, I to flap
- **Menus**: Space to start/restart, 2 to toggle player count

## Coding Rules

- New tunable values go in `constants.rs`, not inline magic numbers
- New shared mutable game state goes in `resources.rs`
- New ECS marker/data types go in `components.rs`
- New messages must be registered in `main.rs` with `add_message::<T>()`
- Prefer extending an existing domain plugin over registering ad hoc systems in `main.rs`
- Assign new systems to the appropriate `GameSet` and gate with the correct state
- When spawning entities for a state, attach `DespawnOnExit` for cleanup
- Use marker components for entity classification (e.g., `#[derive(Component)] struct Player;`)

### Query Filters

Use Bevy's query filters for performance and correctness:
- `With<T>`/`Without<T>` to narrow queries without reading a component's data
- `Changed<T>` to run logic only when a component is mutated
- `Added<T>` to detect newly added components

### Assets

All current visuals are code-built primitive meshes. Mesh and material handles are cached in `SharedMeshes`/`SharedMaterials` resources (created at startup). If file assets are introduced later, keep paths as plain relative strings for `asset_server.load(...)` and store repeated handles in a resource.

## Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`. Cast with `as u32` if constants are `f32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`, not `bevy::render::camera`.
- Use `ApplyDeferred` (struct) not `apply_deferred` (no such function) for command flushing between systems.
- 2D rendering uses `Camera2d`, `Mesh2d`, `MeshMaterial2d`, `Sprite`.
- Use `DespawnOnExit(state)` for state-scoped entity cleanup.
- Use `Message` derive + `MessageWriter<T>`/`MessageReader<T>` for buffered inter-system messaging (registered with `add_message::<T>()`).
- `ChildOf` relationship is used to query parent from child (e.g., `wings.Query<(&ChildOf, ...)>` then `parents.get(parent.get())`).
