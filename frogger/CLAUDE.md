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
- `EventWriter<T>`, `EventReader<T>`, and `App::add_event::<T>()` are **not available**. Use a shared `Resource` with a `Vec` to pass data between systems instead of the event system.
- `ChildBuilder` is **not in the prelude**. Avoid naming it as a type in function signatures. Instead, inline child-spawning logic inside `.with_children(|parent| { ... })` closures. Nested `.with_children` calls may fail type inference — flatten children under one parent instead.
- `ColorMaterial::from_color(color)` works for creating `ColorMaterial` from a `Color`.
- `Text2d::new("text")` works for world-space text (score popups, etc.), paired with `TextFont` and `TextColor`.
- Primitive 2D shapes for `Mesh2d`: `Circle::new(radius)`, `Capsule2d::new(radius, middle_length)`, `RegularPolygon::new(circumradius, sides)`, `Ellipse::new(half_w, half_h)`. The capsule is vertical by default — rotate with `Quat::from_rotation_z(FRAC_PI_2)` for horizontal.
- Systems with many parameters (6+) still work with `.after()` ordering as long as all parameter types resolve correctly.
