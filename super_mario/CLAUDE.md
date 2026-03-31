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
cargo test         # Run all tests
cargo test <name>  # Run a single test by name (e.g. cargo test generate_level_ron_files)
```

Target dir is redirected to `D:/cargo-target-dir` via `.cargo/config.toml`.

## Dev Profile

Dependencies compile at `opt-level = 3` while the main crate uses `opt-level = 1` — fast iteration on game code with performant dependencies.

## Tech Stack

- **Bevy 0.18.1** (with `dynamic_linking` feature) — ECS game engine
- **Rust Edition 2024**
- **ron** / **serde** — level data serialization (RON files under `assets/levels/`)

## Architecture

Super Mario Bros clone. Placeholder geometry (colored rectangles/ellipses), no sprite assets.

### Module Structure

- **`main.rs`** — App setup, plugin registration, `GameplaySet` ordering, asset loader registration
- **`assets.rs`** — `GameAssets` resource — all shared `Handle<Mesh>` / `Handle<ColorMaterial>` for every entity type, initialized once on `Startup`
- **`player.rs`** — `PlayerPlugin` — input, gravity, velocity, tile collision, death animation, flagpole collision, level-complete scripted sequence
- **`camera.rs`** — `CameraPlugin` — camera setup, reset on level enter, smooth follow
- **`enemy/`** — `EnemyPlugin` (registered from `enemy/mod.rs`)
  - **`mod.rs`** — shared physics (walk, gravity, velocity, tile collision, activation, despawn), squish timer, score popups, `mario_take_damage` helper
  - **`goomba.rs`** — `mario_goomba_collision` (stomp → squish)
  - **`koopa.rs`** — `mario_koopa_collision` (stomp → shell), `mario_shell_collision` (kick/stop), `shell_enemy_collision` (chain kills)
- **`block.rs`** — `BlockPlugin` — head-hit processing (`PendingBlockHit` resource), `?`/`M` block content release (mushroom for Small, fire flower for Big/Fire), brick bump/break, block bounce animation, coin pop, brick particles, floating coin collection
- **`powerup.rs`** — `PowerUpPlugin` — mushroom/fire flower emerge+collection, fireball shooting+physics+enemy collision, growth/shrink animation (`PlayState::Growing`), invincibility flashing, ducking
- **`collision.rs`** — Shared collision helpers: `aabb_overlap()` for entity-vs-entity overlap, `resolve_tile_collisions()` for AABB-vs-grid (used by player, enemies, and fireballs)
- **`ui.rs`** — `UiPlugin` — start screen, game over screen, HUD (score/coins/world/timer), pause overlay, countdown timer
- **`level.rs`** — `LevelData` asset + `LevelAssetLoader` (RON files), `LevelGrid` resource, coordinate conversion, `spawn_level` system, hardcoded fallback grids (`level_test`, `level_1_1`)
- **`states.rs`** — `AppState`, `PlayState` sub-state, `GameplaySet` enum
- **`resources.rs`** — `GameData` (score, coins, lives, timer), `SpawnPoint`, `DeathAnimation`, `PendingBlockHit`, `LevelCompleteAnimation`
- **`components.rs`** — All ECS marker/data types (Player, Velocity, Grounded, Tile, Goomba, KoopaTroopa, Shell, Mushroom, FireFlower, Fireball, CollisionSize, PlayerSize, FlagpoleFlag, Castle, HUD markers, etc.)
- **`constants.rs`** — All tunable values (window, camera, player, physics, enemies, blocks, power-ups, fireballs, flagpole, z-layers)

### Coordinate System

The level grid uses row 0 = top, row 14 = bottom. World space has Y increasing upward. `LEVEL_ORIGIN_X/Y` anchors tile (col=0, row=14) at world position (0, -120). Conversion functions in `level.rs` translate between grid and world coordinates.

### Tile Collision

Collision uses the `LevelGrid` resource (char grid) for O(1) tile lookups — not entity queries. `is_solid()` checks the char directly. The shared `resolve_tile_collisions()` in `collision.rs` handles the AABB-vs-grid loop for both player and enemies: it converts an entity's AABB to a grid neighborhood (~12 tiles), resolves overlaps by smallest-penetration-axis push-out, uses a 1-pixel grounded probe, and returns head-hit info. A `WallAction` enum controls horizontal collision behavior (`Stop` for player, `Reverse` for enemies). Entity-vs-entity overlap uses `aabb_overlap()` from the same module.

### Enemy Activation

Enemies are spawned inactive (no `EnemyActive` component). The `enemy_activation` system adds `EnemyActive` when an enemy scrolls within one tile of the camera's right edge. Once activated, enemies stay active permanently. All enemy physics systems filter `With<EnemyActive>`.

### Block Interactions

Mario hitting a block from below is detected in `player.rs` `tile_collision` and stored in the `PendingBlockHit` resource (single optional hit per frame, consumed by `process_block_hits` in `block.rs`). The `LevelGrid` char is the source of truth: `?` → coin pop, `M` → mushroom spawn, `B` → bump (small) or break (big). After hit, `?`/`M` become `E` (solid but not hittable). Brick break despawns the tile entity and sets grid to `.`. `TilePos` components on `?`/`M`/`B` tiles enable entity lookup by grid position.

### Power-up System

**Mushrooms** emerge from `M` blocks (when Small Mario) with a rising `MushroomEmerging` animation, then receive `EnemyWalker` + `EnemyActive` to reuse enemy physics. Collection triggers `PlayState::Growing` which freezes gameplay while flashing between sizes.

**Fire Flowers** emerge from `M` blocks (when Big/Fire Mario) and stay stationary. Collection sets `PlayerSize::Fire` and swaps the player material to white via `GameAssets.player_fire_mat`.

**Fireballs** (J/E key, Fire Mario only, max 2 on screen) travel at constant horizontal speed with gravity and bounce off ground via `resolve_tile_collisions`. Despawn on wall hit or off-screen. Kill Goombas on contact; turn Koopas into stationary shells.

**PlayerSize** has three variants: `Small`, `Big`, `Fire`. Fire Mario takes damage → Small (skips Big). The shrink animation in `growth_animation_system` resets material to `player_normal_mat` to handle Fire → Small color change. `GameAssets` holds both mesh and material handles for the player.

`CollisionSize` on the player is updated dynamically (small=16, big=32, ducking=16). Damage shrink grants 2s `Invincible` (visibility flashing, enemy pass-through).

### Koopa & Shell Mechanics

Koopa Troopa patrols like Goomba. Stomp despawns the Koopa and spawns a `Shell` entity (stationary, `EnemyWalker` with speed=0). Kick sets speed=`SHELL_SPEED`, stomp sets speed=0. Moving shells kill enemies on contact with chain scoring (200×2^n). Shell reuses enemy physics via `EnemyWalker`/`EnemyActive` — wall bounce handled automatically by `enemy_tile_collision`.

### Physics Model

Dual gravity: `GRAVITY_ASCENDING` (600) while rising, `GRAVITY_DESCENDING` (980) while falling — gives the classic Mario "floaty jump, fast fall" feel. Terminal velocity is capped. Horizontal movement uses acceleration/deceleration (not instant). Shift key toggles walk/run speed.

### Camera

Uses `OrthographicProjection` scaled to show ~267×200 world units (NES-like resolution) in an 800×600 window. Camera follows Mario horizontally with a dead zone (scrolls when Mario reaches the right third), never scrolls left (one-way), and clamps to level bounds. Camera Y is fixed.

### State Machine

- **`AppState`**: `StartScreen → Playing → GameOver` (cycles via Enter key)
- **`PlayState`** (sub-state of `Playing`): `Running`, `Dying`, `Paused`, `LevelComplete`, `Growing`
- Gate gameplay systems with `.run_if(in_state(PlayState::Running))` (not just `AppState::Playing`)
- Use `OnEnter`/`OnExit` for spawn/cleanup symmetry
- Prefer `DespawnOnExit(AppState::Playing)` on entities that should auto-despawn when leaving a state
- New domain modules should expose a `Plugin` — register in `main.rs`

### System Ordering (GameplaySet)

Cross-plugin system ordering is configured once in `main.rs` via `GameplaySet`:

```
Input → Physics → Camera → Late   (chained, run_if PlayState::Running)
```

Each plugin drops its systems into the appropriate set:
- **Input**: `player_input`, `enemy_activation`, `fireball_shoot`
- **Physics**: player gravity/velocity/collision chain, enemy walk/gravity/velocity/collision chain, `fireball_physics`
- **Camera**: `camera_follow`
- **Late**: `check_pit_death`, `flagpole_collision`, `countdown_timer`, `mario_goomba_collision`, `mario_koopa_collision`, `mario_shell_collision`, `shell_enemy_collision`, `enemy_despawn_out_of_bounds`, `process_block_hits`, `floating_coin_collection`, `mushroom_collection`, `fire_flower_collection`, `fireball_enemy_collision`

Systems outside the chain (HUD update, pause input, squish timer, score popups, block/coin/brick animations, mushroom/fire flower emerge, growth animation, invincibility, ducking, `level_complete_system`) use direct `run_if` conditions.

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
- **Disjoint queries**: When a system has multiple queries that access the same component (especially `&mut`), Bevy requires proof they can never match the same entity. `With<Goomba>` vs `With<KoopaTroopa>` is **not** sufficient — add explicit `Without<KoopaTroopa>` to the Goomba query and vice versa. Always cross-exclude enemy type markers (`Goomba`, `KoopaTroopa`, `Shell`) on queries that share mutable component access.

### Level Complete Sequence

Flagpole collision (in `player.rs`, Late set) triggers `PlayState::LevelComplete` and creates a `LevelCompleteAnimation` resource. The `level_complete_system` (runs during `LevelComplete`) drives four phases: `SlideDown` (snap to pole X, move down), `WalkToCastle` (walk right), `TimeTally` (rapidly decrement timer, add score at 50 pts/tick), `Done` (2s pause, then transition to `StartScreen`). The flag entity (`FlagpoleFlag`) slides down in sync with Mario. Camera does not follow during LevelComplete (GameplaySet requires Running).

### Assets

**GameAssets** (`assets.rs`): A single `Resource` holding every shared `Handle<Mesh>` and `Handle<ColorMaterial>` — tiles, player, enemies, shells, mushrooms, fire flowers, fireballs, flagpole, castle, particles, coins. Created once on `Startup` via `init_game_assets`. All systems that spawn entities take `Res<GameAssets>` and `.clone()` the handles — no inline `meshes.add()` / `materials.add()`.

**Level data** (`assets/levels/*.level.ron`): RON files loaded via a custom `LevelAssetLoader`. Format is `( rows: ["..row0..", "..row1..", ...] )` — 15 rows of 211 chars using the tile legend. `LevelHandle` resource stores the active level's `Handle<LevelData>`. `spawn_level` reads the loaded asset, falling back to the hardcoded grid if not yet loaded. Generate RON files from existing grids with `cargo test generate_level_ron_files`.

To switch which level is loaded, change the path in `load_level()` in `level.rs`.

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