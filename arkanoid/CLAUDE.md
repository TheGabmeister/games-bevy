# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Scope

This project lives inside a monorepo. Stay scoped to this directory only — do not read, edit, or run git commands that reference parent or sibling directories. Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Build & Run Commands

```bash
cargo run          # Build and run the game
cargo build        # Build only
cargo check        # Fast type-check (use for most code changes)
cargo clippy       # Lint (use when changing API patterns broadly)
```

Run `cargo check` for most code changes and `cargo clippy` when changing API patterns broadly. If you cannot run validation, say so explicitly.

Launch the game with `cargo run`, **not** the built `.exe` directly: the `dynamic_linking` feature needs the Bevy dylib on PATH (which Cargo sets up), and `cargo run` sets the working directory to the crate root so `assets/` resolves. The window is interactive — a headless/CI run can only confirm it starts up and builds the schedule without panicking, not that gameplay behaves.

Target dir is redirected to `D:/cargo-target-dir` via `.cargo/config.toml`.

## Dev Profile

Dependencies compile at `opt-level = 3` while the main crate uses `opt-level = 1` — fast iteration on game code with performant dependencies.

## Tech Stack

- **Bevy 0.18.1** (with `dynamic_linking` feature) — ECS game engine
- **Rust Edition 2024**
- **`ron` + `serde`** — only for loading hand-authored level layouts from `assets/levels/*.ron`. Their versions in `Cargo.toml` are pinned to match the ones Bevy already pulls in (so no extra builds); bump them together if Bevy's change.

There is no automated test suite (it's a game). Validate with `cargo check`/`cargo clippy` and by running it — see below.

## Architecture

A from-scratch recreation of the classic **Arkanoid** (Taito, 1986), built as a Bevy learning exercise. One-plugin-per-domain organization.

**`PLAN.md` is the source of truth for scope and sequencing** — a 7-phase roadmap (vertical slice → systems depth → content). It also carries the canonical asset manifest (file paths + pixel dimensions under `assets/`). Read the relevant phase before adding a feature; build only on systems from earlier phases.

Module map (✅ = implemented, ◻ = still a stub awaiting its phase). Phases 1–6 are done; Phase 7 is pending.

- **`main.rs`** — App setup, 600×800 **portrait** window config, state registration (`AppState` + `PlayState` sub-state), resource init (`Score`/`Lives`/`Round`/`BallSpeed`/`GameAssets`/`Levels`; `PaddleMode`/`CapsuleDirector` are init'd by `PowerupPlugin`), plugin registration, camera + playfield border spawn
- **`constants.rs`** — All tunable values as named constants (playfield bounds, paddle/ball sizing, speeds, brick grid, lives + ready-timer, z-layers)
- **`components.rs`** — Marker + data components (`Velocity`, `Paddle { half_width }`, `Ball { stuck }`, `Brick { points, hits_remaining, max_hits }`, `BrickColor`, `BrickKind`, `Silver`/`Indestructible` markers, `PowerupKind`, `Capsule`, `Laser`, `WarpGate`, `VfxAnim`, `EnemyKind`, `Enemy`, `SpawnGate`)
- **`resources.rs`** ✅ — Shared game state: `Score { current, high }`, `Lives`, `Round`, `BallSpeed` (per-serve speed ramp), `PaddleMode` (active paddle power-up), `CapsuleDirector` (capsule-drop schedule), `EnemySpawnTimer` + `EnemyDirector` (enemy spawn cadence/rotation)
- **`states.rs`** ✅ — `AppState` (`StartScreen → Playing → GameOver`) + `PlayState` sub-state (`Ready → Serving → Running`) under `Playing`
- **`assets.rs`** ✅ — `GameAssets`, the central preloaded-handle registry (see **Asset Registry** below)
- **`input.rs`** ✅ — `InputPlugin`; keyboard/gamepad → `InputActions` resource (no mouse — keyboard + gamepad only)
- **`player.rs`** ✅ — `PlayerPlugin`; spawns the Vaus, clamped paddle control (clamp uses the paddle's dynamic `half_width`)
- **`ball.rs`** ✅ — `BallPlugin`; spawn/serve/launch/integrate the ball, the in-round speed ramp (`accelerate_ball`, reset on serve), and Catch-power-up release (`release_caught_balls`)
- **`bricks.rs`** ✅ — `BrickPlugin`; per-round brick grid spawn (layouts from the `Levels` resource; colored/silver/gold via `BrickKind`), silver damage-frame feedback, scoring, round-clear detection (excludes indestructible gold)
- **`levels.rs`** ✅ — `Levels` resource: brick layouts loaded at startup from `assets/levels/round-*.ron` (RON → `Vec<Vec<String>>`, in round order). Currently rounds 1–2; the full 33-round set is Phase 7
- **`collision.rs`** ✅ — `CollisionPlugin`; ball↔wall/paddle/brick reflection (silver durability, indestructible gold, Catch sticks instead of bouncing), multi-ball-aware bottom loss, triggers `BallLost`, emits `BounceSound`
- **`powerups.rs`** ✅ — `PowerupPlugin`; capsule drop schedule (`CapsuleDirector`) + fall + paddle-catch, the `CapsuleCaught` observer applying the seven effects (Catch/Laser/Expand/Disruption/Slow/Break/Player), laser fire + bolt flight + bolt↔brick collision, the Break warp gate, and power-up reset on life loss
- **`vfx.rs`** ✅ — `VfxPlugin`; reusable one-shot animated VFX flipbook (`spawn_vfx` + `VfxAnim`), self-despawning after the last frame
- **`flow.rs`** ✅ — `GameFlowPlugin`; run-level orchestration: lives, the ready/serve flow, start/restart, game-over, and the `BallLost` observer (`stick_ball` keeps one ball, despawning multi-ball extras)
- **`ui.rs`** ✅ — `UiPlugin`; HUD (score / high / round / life icons), "ROUND n READY" banner, title + game-over screens
- **`audio.rs`** ✅ — `AudioPlugin`; plays SFX from `BounceSound` messages
- **`debug.rs`** ✅ — `DebugPlugin`; dev keys (reads the keyboard directly, bypassing `InputActions` by design): **F1** toggles a collider overlay, **F2** destroys all bricks, **F3** drops a capsule (cycling through power-ups)
- **`enemy.rs`** ✅ — `EnemyPlugin`; top-gate spawning on a timer, three enemy types with distinct wandering descents (Pyramid weave / Molecule zig-zag / Cube slow drift) + looping animation, ball/laser destruction with the `EnemySpawned`/`EnemyDestroyed` observers, bottom-exit despawn

### Game State Flow

`flow.rs` (`GameFlowPlugin`) owns run-level orchestration, but the flow as a whole is spread across the systems that cause each transition — find a transition by grepping for the `NextState` it sets or the resource/event it touches, not by following a call stack.

- **`AppState`**: `StartScreen` —(launch)→ `Playing` —(lives exhausted)→ `GameOver` —(launch)→ `StartScreen`.
- **`PlayState`** (exists only inside `Playing`): `Ready` (intro banner + `ReadyTimer`) → `Serving` (ball parked on paddle) —(launch)→ `Running` (live play). Round cleared → back to `Ready`; ball lost with lives remaining → back to `Serving`.
- Per-round bricks spawn `OnEnter(PlayState::Ready)` keyed to `Round`. `start_run` (`OnEnter(Playing)`) resets `Lives`/`Score`/`Round` — this relies on Bevy running a parent state's `OnEnter` **before** its sub-state's.
- Ball-off-bottom: `collision.rs` calls `commands.trigger(BallLost)`; the observer in `flow.rs` spends a life and routes to `Serving` or `GameOver`.
- Gating: movement and collision run on `in_state(PlayState::Running)`; the paddle and `ball_follow_paddle` run through all of `Playing`.

### Physics & System Ordering

Gameplay physics runs in **`FixedUpdate`** for determinism, ordered across plugins with `.after()`:
`paddle_control` → `ball_follow_paddle` → `ball_movement` → `ball_collision`. One-shot input reactions (e.g. `ball_launch`) run in `Update`. Systems referenced across modules are `pub`.

### Input Abstraction

Raw devices are read **only** in `input.rs` (`PreUpdate`) and translated into the `InputActions` resource. All gameplay systems read `InputActions`, never `ButtonInput`/`Gamepad` directly. `pressed` for held actions (move), `just_pressed` for one-shots (launch).

### Asset Registry

Asset handles live in the **`GameAssets`** resource (`assets.rs`), grouped into nested category structs (`assets.sprites.vaus`, `assets.sfx.wall_bounce`). Handles are preloaded once via `FromWorld` + `init_resource`; call sites `.clone()` a handle rather than calling `asset_server.load(...)`. This keeps asset references type-checked (a typo is a compile error). Add a field per phase as features land — don't preload assets nothing references yet. **Level data** is loaded from hand-authored RON files in `assets/levels/round-*.ron` into the `Levels` resource (`levels.rs`) at startup via `std::fs` + `ron` in `FromWorld` (rounds 1–2 so far; the full 33-round set is Phase 7). The sprite/audio asset registry itself stays in Rust.

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

- Make the smallest coherent change that solves the task — don't rewrite working structure just to make it "cleaner"
- Preserve the current plugin/module split unless the task clearly calls for a restructure
- New tunable values go in `constants.rs`, not inline magic numbers
- New shared mutable game state goes in `resources.rs`
- New ECS marker/data types go in `components.rs`
- Prefer extending an existing domain plugin over registering ad hoc systems in `main.rs`
- Use marker components for entity classification (e.g., `#[derive(Component)] struct Player;`)
- Keep `cargo clippy` clean. Idiomatic Bevy queries/systems sometimes trip `clippy::type_complexity` (multi-filter queries like `Query<.., (With<A>, Changed<B>)>`) or `clippy::too_many_arguments` (8+ system params). When the system genuinely needs that shape, put `#[allow(clippy::type_complexity)]` / `#[allow(clippy::too_many_arguments)]` on the function rather than contorting it — this is the existing convention (see `bricks.rs`, `powerups.rs`).

### Query Filters

Use Bevy's query filters for performance and correctness:
- `With<T>`/`Without<T>` to narrow queries without reading a component's data
- `Changed<T>` to run logic only when a component is mutated
- `Added<T>` to detect newly added components

### Assets

Load assets through the `GameAssets` registry (see **Asset Registry** above), not ad hoc `asset_server.load(...)` calls in gameplay systems. Asset file paths (the strings inside `assets.rs`) must stay aligned with files under `assets/` and with the manifest in `PLAN.md`.

### Asset Generation Pipeline

All **art and audio** under `assets/` is **generated**, not hand-authored — `tools/generate_assets.py` is the source of truth. (The exception is `assets/levels/*.ron`, which are hand-authored level data — see **Asset Registry**.) The generator emits SVGs and rasterizes them to PNG via **Inkscape** (sprites, sprite sheets, VFX, UI art), and synthesizes audio via **ffmpeg** (SFX as `aevalsrc` tone expressions; music as Python-rendered WAV → OGG, kept short). Run `python tools/generate_assets.py` to regenerate (Inkscape + ffmpeg must be on PATH); PNG export is skipped when the SVG is unchanged, but SFX/music always re-render.

- **To change an asset, edit the generator** — not the `.png`/`.svg`/`.ogg`. A regen overwrites hand-edited files. (You can hand-edit an SVG + re-rasterize for a one-off, but mirror it back into the generator or it will be lost.)
- The playfield is **portrait 600×800**: the border frame and full-screen UI art are authored at 600×800 and the brick grid is 9 columns. Keep `generate_assets.py`, the `assets/` files, `constants.rs`, and the `PLAN.md` manifest consistent when changing dimensions.

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

### State-Scoped Entities

- `StateScoped` was renamed to `DespawnOnExit<S: States>` (and `DespawnOnEnter<S: States>`).
- Usage: `commands.spawn((MyComponent, DespawnOnExit(AppState::Playing)));`
- Entities are automatically despawned when the state exits (or enters, respectively).

### SubStates

- Define with `#[derive(SubStates)]` and a `#[source(ParentState = ParentState::Variant)]` attribute:
  ```rust
  // This is the project's actual sub-state (see Game State Flow above).
  #[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
  #[source(AppState = AppState::Playing)]
  enum PlayState {
      #[default]
      Ready,
      Serving,
      Running,
  }
  ```
- Register: `app.init_state::<AppState>().add_sub_state::<PlayState>();`
- Sub-states only exist when the source state matches; they are removed automatically otherwise.
- `ComputedStates` also exists for read-only derived states (`app.add_computed_state::<T>()`).
## Local Python

- Python is available at `C:\Users\Admin\AppData\Local\Python\pythoncore-3.14-64\python.exe`.
