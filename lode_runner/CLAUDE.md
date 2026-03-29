# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Scope

This project lives inside a monorepo. Stay scoped to this directory only — do not read, edit, or run git commands that reference parent or sibling directories.

## Build & Run Commands

```bash
cargo run              # Build and run the game
cargo build            # Build only
cargo check            # Fast type-check (use for most code changes)
cargo clippy           # Lint (use when changing API patterns broadly)
cargo test             # Run all tests (grid module has 34 unit tests)
cargo test bfs         # Run tests matching a name pattern
```

Target dir is redirected to `D:/cargo-target-dir` via `.cargo/config.toml`.

## Dev Profile

Dependencies compile at `opt-level = 3` while the main crate uses `opt-level = 1` — fast iteration on game code with performant dependencies.

## Tech Stack

- **Bevy 0.18.1** (with `dynamic_linking` feature) — ECS game engine
- **Rust Edition 2024**

## Architecture

Lode Runner clone with grid-based movement, digging, guard AI, and multi-level progression.

### Module Layout

Each domain module exports `pub` system functions. `main.rs` does all system registration — keeping the full execution order visible in one place.

**Core data modules:**
- **`constants.rs`** — All tunable values: window size, speeds, durations, colors, scoring
- **`components.rs`** — ECS marker components (`Player`, `Guard`, `Gold`) and data components (`GridPosition`, `MovementState`, `Hole`, HUD markers)
- **`resources.rs`** — Shared mutable game state (`GameState` for score/lives/level, `DeathTimer`)
- **`states.rs`** — `AppState` enum and `PlayState` sub-state
- **`grid.rs`** — `LevelGrid` tile map, `HoleMap` overlay, movement validation (`can_enter`, `can_move_horizontal`, `is_supported`, `can_dig`), BFS pathfinding, level parsing

**Domain modules (pub system functions, no scheduling):**
- **`player.rs`** — `player_input` (sets `MovementState::Digging`; hole entity is spawned by `advance_movement` on dig completion)
- **`guard.rs`** — `guard_ai` (BFS toward player each AI tick)
- **`movement.rs`** — `advance_movement`, `tick_holes`, `apply_gravity`, `sync_transforms`
- **`gameplay.rs`** — `collect_gold`, `check_guard_trap`, `check_player_death`, `check_exit`, death/restart
- **`ui.rs`** — All screens (start, pause, HUD, level complete, game over)
- **`render.rs`** — `RenderAssets` resource, `setup_render_assets`
- **`levels.rs`** — Level string data (`LEVELS` array)

**Orchestrator:**
- **`main.rs`** — App setup, camera, `spawn_level`, `cleanup_play_resources`, all `.add_systems()` calls with set/chain/run_if ordering

### State Machine

```
StartScreen → Playing → LevelComplete → Playing (next level) → GameOver
                  ↕           ↑                                     ↑
             Restarting ───┘ (death w/ lives)          (death w/o lives or all levels done)
```

`PlayState` is a sub-state of `AppState::Playing` with variants: `Running`, `Paused`, `Dying`.

The `Restarting` state is **transient** — it exists only to trigger `OnExit(Playing)` cleanup then immediately re-enters `Playing` to respawn the level. `OnExit(Playing)` does two things: `DespawnOnExit` removes all play-scoped entities, and `cleanup_play_resources` removes play-time resources (`LevelGrid`, `HoleMap`, `DeathTimer`). Score and lives persist across restarts; gold and exit status reset.

### System Set Pipeline

Gameplay systems run under `PlayState::Running` in four ordered sets:

1. **`GameSet::Input`** — Player keyboard handling, dig initiation
2. **`GameSet::Simulate`** — `advance_movement` → `tick_holes` → `guard_ai` → `apply_gravity` (chained)
3. **`GameSet::Resolve`** — `collect_gold` → `check_guard_trap` → `check_player_death` → `check_exit` (chained)
4. **`GameSet::Presentation`** — `sync_transforms`, `update_hud`

Systems outside the pipeline (pause input, death tick, screen inputs) use `.run_if(in_state(...))` directly.

### Grid-Based Movement

Entities move cell-to-cell via `MovementState`, not continuous physics. `GridPosition` is the authoritative position; `Transform` is derived in `sync_transforms` by lerping between `from`/`to` based on `progress`. Movement states: `Idle`, `Moving`, `Falling`, `Climbing`, `Digging`, `Trapped`.

`LevelGrid` is the base tile map (never mutated after parse, except `reveal_hidden_ladders`). `HoleMap` is a separate overlay that tracks dug holes without modifying the base grid — this keeps level resets trivial.

### Level Format

Levels are inline `&str` constants (28x16 grid). Characters:
- `.` empty, `#` brick (diggable), `=` concrete (indestructible)
- `H` ladder, `-` bar (monkey bar), `^` hidden ladder (revealed when all gold collected)
- `$` gold, `S` gold on ladder, `P` player spawn, `G` guard spawn

Rows are top-to-bottom in the string but bottom-up in grid coordinates (y=0 is the floor). `parse_level` handles the flip. Validation: all rows must be the same width, exactly one `P` is required, and unknown characters panic.

### Coding Rules

- New tunable values go in `constants.rs`, not inline magic numbers
- New shared mutable game state goes in `resources.rs`
- New ECS marker/data types go in `components.rs`
- Use marker components for entity classification (e.g., `#[derive(Component)] struct Player;`)

### Events and Observers

- `EventWriter<T>`, `EventReader<T>`, and `App::add_event::<T>()` are **not available**. Use a shared `Resource` with a `Vec` to pass data between systems instead of the event system.
- Use `Observer` and `Trigger` for one-shot reactions to entity lifecycle or custom game events.

### Timers

Use `Timer` with `Res<Time>` for cooldowns, spawn intervals, and delays — do not use frame-counting. Store timers in components (per-entity) or resources (global). Tick them with `timer.tick(time.delta())` each frame.

### Query Filters

- `With<T>`/`Without<T>` to narrow queries without reading a component's data
- `Changed<T>` to run logic only when a component is mutated
- `Added<T>` to detect newly added components

### Assets

Asset paths are plain relative strings passed to `asset_server.load(...)` — keep them aligned with files under `assets/`. Store `Handle<T>` in a resource when an asset is used repeatedly to avoid redundant loads. This project uses a `RenderAssets` resource to pre-create all meshes and materials at startup.

## Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`. Cast with `as u32` if constants are `f32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`, not `bevy::render::camera`.
- Use `ApplyDeferred` (struct) not `apply_deferred` (no such function) for command flushing between systems.
- 2D rendering uses `Camera2d`, `Mesh2d`, `MeshMaterial2d`, `Sprite`.
- `ChildBuilder` is **not in the prelude**. Avoid naming it as a type in function signatures. Instead, inline child-spawning logic inside `.with_children(|parent| { ... })` closures. Nested `.with_children` calls may fail type inference — flatten children under one parent instead.
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

### State-Scoped Entities

- `StateScoped` was renamed to `DespawnOnExit<S: States>` (and `DespawnOnEnter<S: States>`).
- Usage: `commands.spawn((MyComponent, DespawnOnExit(AppState::Playing)));`
- Entities are automatically despawned when the state exits (or enters, respectively).

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
