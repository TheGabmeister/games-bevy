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

Super Mario Bros clone. Placeholder geometry (colored rectangles), no sprite assets, no state machine yet.

### Current Structure

- **`main.rs`** — App setup, all systems (no plugins yet). Systems run in an explicit `.chain()`: `player_input → apply_gravity → apply_velocity → tile_collision → camera_follow`
- **`constants.rs`** — All tunable values (window, camera, player dimensions, physics, z-layers, level origin)
- **`components.rs`** — `Player` marker, `Velocity`, `FacingDirection`, `Grounded`, `Tile` marker, `TileType` enum
- **`level.rs`** — Level 1-1 grid (211×15 chars built programmatically), `LevelGrid` resource for collision lookups, coordinate conversion (`tile_to_world`, `world_to_col`, `world_to_row`)

### Coordinate System

The level grid uses row 0 = top, row 14 = bottom. World space has Y increasing upward. `LEVEL_ORIGIN_X/Y` anchors tile (col=0, row=14) at world position (0, -120). Conversion functions in `level.rs` translate between grid and world coordinates.

### Tile Collision

Collision uses the `LevelGrid` resource (char grid) for O(1) tile lookups — not entity queries. `is_solid()` checks the char directly. The `tile_collision` system converts Mario's AABB to grid neighborhood (~12 tiles), resolves overlaps by smallest-penetration-axis push-out, and uses a 1-pixel grounded probe below Mario's feet.

### Physics Model

Dual gravity: `GRAVITY_ASCENDING` (600) while rising, `GRAVITY_DESCENDING` (980) while falling — gives the classic Mario "floaty jump, fast fall" feel. Terminal velocity is capped. Horizontal movement uses acceleration/deceleration (not instant). Shift key toggles walk/run speed.

### Camera

Uses `OrthographicProjection` scaled to show ~267×200 world units (NES-like resolution) in an 800×600 window. Camera follows Mario horizontally with a dead zone (scrolls when Mario reaches the right third), never scrolls left (one-way), and clamps to level bounds. Camera Y is fixed.

### Target Layout (as the game grows)

- **`resources.rs`** — Shared game state (score, lives, wave progression)
- **`states.rs`** — `AppState` enum driving a state machine (StartScreen → Playing → GameOver)
- **Domain modules** — One module per gameplay domain (player, enemy, level, ui, audio, etc.), each exposing a Plugin

### State Machine Pattern

Systems should be state-aware:
- Gate gameplay systems with `.run_if(in_state(AppState::Playing))`
- Use `OnEnter`/`OnExit` for spawn/cleanup symmetry
- Prefer `DespawnOnExit(AppState::Playing)` on entities that should auto-despawn when leaving a state — this eliminates most manual cleanup systems
- Use `.after()` chains where frame ordering matters; for 10+ systems, group into `SystemSet`s (e.g., `MovementSet`, `CollisionSet`) and order at the set level

### Messages and Observers

- `EventWriter<T>`, `EventReader<T>`, and `App::add_event::<T>()` were **renamed** in Bevy 0.17. Use the new names:
  - `MessageWriter<M>` / `MessageReader<M>` for buffered inter-system messaging
  - `App::add_message::<M>()` to register a message type
  - Messages still auto-double-buffer and clean up — do **not** roll your own `Resource<Vec<T>>` workaround.
- Use `Observer` and `Trigger` for one-shot reactions to entity lifecycle or custom game events — these replace boilerplate `Added<T>`/`RemovedComponents<T>` query patterns.

### Timers

Use `Timer` with `Res<Time>` for cooldowns, spawn intervals, and delays — do not use frame-counting. Store timers in components (per-entity) or resources (global). Tick them with `timer.tick(time.delta())` each frame.
- The check method is `timer.is_finished()`, **not** `timer.finished()` (`finished` is a private field).
- `WindowResolution` is **not in the prelude** — import with `use bevy::window::WindowResolution;`.

### Coding Rules

- New tunable values go in `constants.rs`, not inline magic numbers
- New shared mutable game state goes in `resources.rs`
- New ECS marker/data types go in `components.rs`
- Prefer extending an existing domain plugin over registering ad hoc systems in `main.rs`
- Use marker components for entity classification (e.g., `#[derive(Component)] struct Player;`)

### Query Filters

Use Bevy's query filters for performance and correctness:
- `With<T>`/`Without<T>` to narrow queries without reading a component's data
- `Changed<T>` to run logic only when a component is mutated
- `Added<T>` to detect newly added components

### Assets

Asset paths are plain relative strings passed to `asset_server.load(...)` — keep them aligned with files under `assets/`. Store `Handle<T>` in a resource when an asset is used repeatedly to avoid redundant loads. 

## Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`. Cast with `as u32` if constants are `f32`.
- `OrthographicProjection` is **not** a standalone `Component` — it is wrapped in `Projection` (an enum). To set a custom orthographic projection on a camera, build the struct and convert: `Projection::from(OrthographicProjection { scale: 0.33, ..OrthographicProjection::default_2d() })`. Spawn it alongside `Camera2d` to override the default.
- `ScalingMode` is in `bevy::camera::ScalingMode`, not `bevy::render::camera`.
- Use `ApplyDeferred` (struct) not `apply_deferred` (no such function) for command flushing between systems.
- 2D rendering uses `Camera2d`, `Mesh2d`, `MeshMaterial2d`, `Sprite`.
- `ChildBuilder` no longer exists — it was replaced by `ChildSpawnerCommands` (which **is** in the prelude). The `.with_children(|parent| { ... })` pattern still works; the closure parameter is now `&mut ChildSpawnerCommands`. Nested `.with_children` calls may fail type inference — flatten children under one parent instead.
- `ColorMaterial::from_color(color)` works for creating `ColorMaterial` from a `Color`.
- `Text2d::new("text")` works for world-space text (score popups, etc.), paired with `TextFont` and `TextColor`.
- Primitive 2D shapes for `Mesh2d`: `Circle::new(radius)`, `Capsule2d::new(radius, middle_length)`, `RegularPolygon::new(circumradius, sides)`, `Ellipse::new(half_w, half_h)`. The capsule is vertical by default — rotate with `Quat::from_rotation_z(FRAC_PI_2)` for horizontal.
- Systems with many parameters (6+) still work with `.after()` ordering as long as all parameter types resolve correctly.

### Bloom / HDR

- The bloom component is `Bloom`, not `BloomSettings` (renamed).
- Import: `use bevy::{core_pipeline::tonemapping::{DebandDither, Tonemapping}, post_process::bloom::Bloom};`
- `Bloom` has presets: `Bloom::NATURAL`, `Bloom::OLD_SCHOOL`, `Bloom::ANAMORPHIC`.
- Camera setup for bloom:
  ```rust
  commands.spawn((
      Camera2d,
      Camera {
          clear_color: ClearColorConfig::Custom(Color::BLACK),
          ..default()
      },
      Tonemapping::TonyMcMapface,
      Bloom::default(),
      DebandDither::Enabled,
  ));
  ```
- `ColorMaterial` has **no** `emissive` field. To make shapes glow with bloom, use `Color` values > 1.0 directly (e.g., `Color::srgb(5.0, 1.0, 0.2)`). The bloom post-process extracts bright regions above its threshold.

### SubStates

- Define with `#[derive(SubStates)]` and a `#[source(ParentState = ParentState::Variant)]` attribute:
  ```rust
  #[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
  #[source(AppState = AppState::Playing)]
  enum PlayState {
      #[default]
      Running,
      Paused,
  }
  ```
- Register: `app.init_state::<AppState>().add_sub_state::<PlayState>();`
- Sub-states only exist when the source state matches; they are removed automatically otherwise.
- `ComputedStates` also exists for read-only derived states (`app.add_computed_state::<T>()`).