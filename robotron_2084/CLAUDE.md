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

The entire game lives in `src/main.rs` (~1700 lines, Phase 1 of the SPEC's growth path). It is organized into clearly delimited sections by comments:

1. **Constants** — all tunable values (speeds, radii, cooldowns, counts) at the top
2. **States** — `AppState` (StartScreen → Playing → GameOver) and `PlayState` sub-state (WaveIntro → WaveActive, plus PlayerDeath, Paused)
3. **Components** — marker components (`Player`, `Enemy`, `Grunt`, `Killable`, `DamagesPlayer`, `Confined`, `WaveEntity`) and data components (`Velocity`, `CollisionRadius`, `Lifetime`, etc.)
4. **Resources** — `GameState` (score/lives/wave), `GameAssets` (shared mesh/material handles), `WaveState` (timers), `ScreenShake`
5. **Wave Definition** — `wave_definition(wave)` returns enemy counts and speed multiplier per wave
6. **System functions** — grouped by domain: startup, start screen, playing setup, wave management, player systems, enemy AI systems, projectile systems, movement, combat, death, pause, effects (particles/popups/shake), HUD, game over

### Key Patterns

- **Marker-driven collision**: `DamagesPlayer` marks anything that kills the player. `Killable` marks enemies that count toward wave clear. `Confined` marks entities clamped to arena bounds. `WaveEntity` marks entities despawned between waves.
- **Shared asset resource**: All meshes and materials are created once in `setup_assets` and stored in `GameAssets`. Entity spawning clones handles from this resource.
- **Player spawn reuse**: `spawn_player_entity()` is a helper called from both initial spawn and respawn-after-death.
- **Score helper**: `award_score()` handles score increment + extra life threshold check in one place.
- **Wave scaling**: `wave_definition()` computes enemy counts programmatically rather than using a data table.
- **Collision**: Circle-circle overlap (`distance_squared < (r1+r2)^2`) for all gameplay collision. No physics engine.
- **High score persistence**: Saved to `highscore.txt` next to the executable.

### State Machine

- Gate gameplay systems with `.run_if(in_state(PlayState::WaveActive))`
- Use `OnEnter`/`OnExit` for spawn/cleanup symmetry
- Use `DespawnOnExit(AppState::Playing)` on gameplay entities for automatic cleanup
- Use `DespawnOnExit(PlayState::WaveIntro)` for wave overlay UI
- `WaveEntity` is used for per-wave cleanup (enemies, projectiles, humans, electrodes) via explicit despawn in `despawn_wave_entities`

### Future Module Split (Phase 2)

When the file grows too large, extract into: `states.rs`, `constants.rs`, `components.rs`, `resources.rs`, then domain plugins (`player.rs`, `enemy.rs`, `combat.rs`, `effects.rs`, `ui.rs`). See SPEC.md Section 4.

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
