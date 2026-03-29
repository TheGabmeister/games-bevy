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
- **rand 0.9** — RNG for spawning, wander AI, particles
- **Rust Edition 2024**

## Architecture

The game is split into domain-specific modules with Bevy plugins:

```
src/
  main.rs          — app setup, camera, plugin registration, system set ordering
  constants.rs     — all tunable values (speeds, radii, cooldowns, counts)
  states.rs        — AppState, PlayState, GameSet system sets
  components.rs    — all ECS marker and data components
  resources.rs     — GameAssets, GameState, WaveState, ScreenShake, high score I/O, setup_assets
  arena.rs         — ArenaPlugin: borders, velocity, confinement, lifetime despawn
  player.rs        — PlayerPlugin: movement, aim, fire, invincibility, respawn
  enemy.rs         — EnemyPlugin: all enemy AI, spawner logic, projectile steering
  human.rs         — HumanPlugin: human wander behavior
  combat.rs        — CombatPlugin: collision detection, damage resolution, wave clear
  waves.rs         — WavePlugin: wave definitions, spawn orchestration, state transitions
  effects.rs       — EffectsPlugin: particles, score popups, screen shake
  ui.rs            — UiPlugin: start screen, HUD, pause, wave overlay, game over
```

### Key Patterns

- **Plugin-per-domain**: Each gameplay domain is a Bevy plugin that registers its own systems.
- **System sets for ordering**: `GameSet::Input → Movement → Confinement → Combat → Resolution` chained during `WaveActive`. Always-on systems (effects, HUD) run with `run_if(in_state(AppState::Playing))`.
- **Marker-driven collision**: `DamagesPlayer` marks anything that kills the player. `Killable` marks enemies that count toward wave clear. `Confined` marks entities clamped to arena bounds. `WaveEntity` marks entities despawned between waves.
- **Shared asset resource**: All meshes and materials are created once in `setup_assets` and stored in `GameAssets`. Entity spawning clones handles from this resource.
- **Player spawn reuse**: `spawn_player_entity()` is a helper called from both initial spawn and respawn-after-death.
- **Score helper**: `GameState::award_score()` handles score increment + extra life threshold check in one place.
- **Wave scaling**: `wave_definition()` computes enemy counts programmatically rather than using a data table.
- **Collision**: Circle-circle overlap (`distance_squared < (r1+r2)^2`) for all gameplay collision. No physics engine.
- **High score persistence**: Saved to `highscore.txt` next to the executable, written once on game over (not every frame).

### State Machine

```
AppState: StartScreen → Playing → GameOver
PlayState (sub-state of Playing): WaveIntro → WaveActive → WaveClear → WaveIntro...
                                              ↕ Paused
                                              → PlayerDeath → WaveActive / WaveClear / GameOver
```

- Gate gameplay systems via `GameSet` (inherits `run_if(in_state(PlayState::WaveActive))`)
- Use `OnEnter`/`OnExit` for spawn/cleanup symmetry
- Use `DespawnOnExit(AppState::Playing)` on gameplay entities for automatic cleanup
- Use `DespawnOnExit(PlayState::WaveIntro)` for wave overlay UI
- `WaveEntity` is used for per-wave cleanup via explicit despawn in `despawn_wave_entities`

### Cross-Module Dependencies

- `combat.rs` imports from `effects.rs` (spawn_particles, spawn_score_popup) and `waves.rs` (wave_definition)
- `enemy.rs` imports from `waves.rs` (wave_definition)
- All domain modules import from `constants.rs`, `components.rs`, `resources.rs`, `states.rs`

## Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`.
- `WindowResolution` is **not in the prelude** — import with `use bevy::window::WindowResolution;`.
- `ScalingMode` is in `bevy::camera::ScalingMode`, not `bevy::render::camera`.
- Use `ApplyDeferred` (struct) not `apply_deferred` (no such function) for command flushing between systems.
- 2D rendering uses `Camera2d`, `Mesh2d`, `MeshMaterial2d`, `Sprite`.
- `ChildBuilder` is **not in the prelude**. Inline child-spawning logic inside `.with_children(|parent| { ... })` closures.
- `ColorMaterial::from_color(color)` works for creating materials from a `Color`.
- `Text2d::new("text")` for world-space text, paired with `TextFont` and `TextColor`.
- Primitive 2D shapes: `Circle::new(r)`, `Capsule2d::new(r, len)`, `RegularPolygon::new(circumradius, sides)`, `Ellipse::new(hw, hh)`.
- **Bundle tuple size limit**: `commands.spawn((...))` fails with more than ~15 elements. Split with `.spawn(first_batch).insert(rest)`.

### Timers

- Tick with `timer.tick(time.delta())` each frame.
- Check with `timer.is_finished()`, **not** `timer.finished()` (`finished` is a private field).
- `timer.just_finished()` is true only on the tick the timer completed.

### Bloom / HDR

- Bloom component is `Bloom` (not `BloomSettings`). Presets: `Bloom::NATURAL`, `Bloom::OLD_SCHOOL`.
- Import: `use bevy::{core_pipeline::tonemapping::{DebandDither, Tonemapping}, post_process::bloom::Bloom};`
- `ColorMaterial` has **no** `emissive` field. Use `Color` values > 1.0 for bloom glow (e.g., `Color::srgb(5.0, 1.0, 0.2)`).

### State-Scoped Entities

- `DespawnOnExit<S: States>` (and `DespawnOnEnter<S: States>`) — replaces old `StateScoped`.
- Register sub-states: `app.init_state::<AppState>().add_sub_state::<PlayState>();`
- Define with `#[derive(SubStates)]` and `#[source(AppState = AppState::Playing)]`.
