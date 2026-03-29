# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Scope

Stay scoped to `c:\dev\games-bevy\robotron_2084` only. Do not read, edit, or run git commands against parent or sibling directories.

## Build & Run

```bash
cargo run                                # Build and run
cargo check                              # Fast type-check (use for most changes)
cargo clippy --all-targets --all-features  # Lint (use when touching shared patterns or scheduling)
```

Target output is redirected to `D:/cargo-target-dir` by `.cargo/config.toml`. Dev profile uses `opt-level = 1` for game code and `opt-level = 3` for dependencies.

## Stack

- Rust edition `2024`, Bevy `0.18.1` (with `dynamic_linking`), `rand 0.9`

## Architecture

Domain-plugin structure. Each gameplay domain owns a Bevy `Plugin` that registers its systems.

```text
src/
  main.rs        — app bootstrap, camera, plugin registration, GameSet ordering
  constants.rs   — all tunable values (speeds, radii, cooldowns, timings)
  components.rs  — ECS marker and data components
  resources.rs   — GameAssets, GameState, WaveState, ScreenShake, high-score I/O, setup_assets
  states.rs      — AppState, PlayState (sub-state), GameSet (system sets)
  arena.rs       — ArenaPlugin: borders, velocity, confinement, lifetime despawn, OOB bullets
  player.rs      — PlayerPlugin: movement, aim, fire, invincibility blink, respawn
  enemy.rs       — EnemyPlugin: all enemy AI, spawner children, projectile steering, shell bounces
  human.rs       — HumanPlugin: human wander
  combat.rs      — CombatPlugin: rescue, bullet hits, damage, electrode/hulk/brain collisions, wave clear
  waves.rs       — WavePlugin: wave definitions, spawn orchestration, state transitions (intro/clear/death)
  effects.rs     — EffectsPlugin: particles, score popups, screen shake
  ui.rs          — UiPlugin: start screen, HUD, pause overlay, game over
```

### System Set Ordering

During `PlayState::WaveActive`, systems run in a chained set order configured in `main.rs`:

```
GameSet::Input → Movement → Confinement → Combat → Resolution
```

Systems that must run in all play states (effects, HUD, invincibility) use `run_if(in_state(AppState::Playing))` without a set. Within `GameSet::Combat`, the chain uses `ApplyDeferred` barriers to ensure despawn commands are visible to later collision systems.

### Cross-Module Dependencies

- `combat.rs` imports `spawn_particles` and `spawn_score_popup` from `effects.rs`
- `enemy.rs` and `combat.rs` import `wave_definition` from `waves.rs`
- All domain modules import from `constants`, `components`, `resources`, `states`

### State Machine

```
AppState: StartScreen → Playing → GameOver
PlayState (sub-state of Playing):
    WaveIntro → WaveActive → WaveClear → WaveIntro ...
                WaveActive → PlayerDeath → WaveActive / WaveClear / GameOver
                WaveActive ↔ Paused
```

### Key Patterns

- **Marker-driven collision**: `DamagesPlayer`, `Killable`, `Confined`, `WaveEntity` drive behavior via queries.
- **Shared asset resource**: `GameAssets` holds all mesh/material handles, created once in `setup_assets`.
- **Score via method**: `GameState::award_score()` handles increment + extra life threshold.
- **Circle-circle collision**: `distance_squared < (r1+r2)^2` everywhere. No physics crate.
- **High score persistence**: written to `highscore.txt` once on game over, not every frame.

## Repo Conventions

- Prefer small coherent changes over architecture churn.
- Put new tunables in `constants.rs`, new shared state in `resources.rs`, new markers/data in `components.rs`.
- Extend the owning domain plugin instead of growing `main.rs`.
- If a state owns an entity, define the cleanup path in the same change.

## Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(w, h)` takes `u32`. Not in the prelude — import `bevy::window::WindowResolution`.
- `ScalingMode` is in `bevy::camera::ScalingMode`.
- Use `ApplyDeferred` (struct) for command flushing between systems in the same schedule.
- **Bundle tuple size limit**: `commands.spawn((...))` fails above ~15 elements. Split with `.spawn(batch1).insert(batch2)`.
- `DespawnOnExit<S: States>` replaces old `StateScoped`. Register sub-states with `app.add_sub_state::<PlayState>()`.
- Rendering: `Mesh2d`, `MeshMaterial2d<ColorMaterial>`, `Text2d` for world-space text.
- Bloom: `Bloom` component (not `BloomSettings`). `ColorMaterial` has no `emissive` — use `Color` values > 1.0 for glow.
- Timers: tick with `timer.tick(time.delta())`, check `timer.is_finished()` (not `.finished()`), `timer.just_finished()` fires once.
