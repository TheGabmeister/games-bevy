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

## Architecture Patterns

Preferred modular architecture:

- **`main.rs`** — App setup, plugin registration, system scheduling with explicit ordering
- **`components.rs`** — Marker components for entity types + data components (Velocity, FacingDirection, WorldPosition, etc.)
- **`constants.rs`** — All magic numbers as named constants (window size, speeds, radii, scoring)
- **`resources.rs`** — Shared game state (score, lives, wave progression)
- **`states.rs`** — `AppState` enum driving a state machine (StartScreen → Playing → GameOver pattern)
- **Domain modules** — One module per system domain (player, enemies, collision, spawning, ui, etc.)

### Key Bevy Conventions

- Marker components for entity classification (e.g., `#[derive(Component)] struct Player;`)
- `.run_if(in_state(AppState::Playing))` for state-gated systems
- `.after()` chains for system ordering within update phases
- `Startup` systems for initial entity spawning, `Update` for game loop
- Resources (`Res<T>`, `ResMut<T>`) for shared mutable game state
- 2D rendering with `Camera2d`, `Mesh2d`, `MeshMaterial2d`, `Sprite`

### Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — it despawns the entity and all children. Do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`. Cast with `as u32` if your constants are `f32`.
