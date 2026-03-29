# CLAUDE.md

This file provides project guidance for coding agents working in this repository.

## Scope

Stay scoped to `c:\dev\games-bevy\robotron_2084` only. Do not read, edit, or run git commands against parent or sibling directories.

## Build And Run

```powershell
cargo run
cargo build
cargo check
cargo clippy --all-targets --all-features
```

Target output is redirected to `D:/cargo-target-dir` by `.cargo/config.toml`.

## Stack

- Rust edition `2024`
- Bevy `0.18.1` with `dynamic_linking`
- `rand 0.9`

## Architecture

The project is already split into gameplay-domain plugins:

```text
src/
  main.rs
  constants.rs
  components.rs
  resources.rs
  states.rs
  arena.rs
  player.rs
  enemy.rs
  human.rs
  combat.rs
  waves.rs
  effects.rs
  ui.rs
```

Primary ownership:

- `main.rs`: app setup, window/camera config, plugin registration
- `resources.rs`: shared assets, game state, wave timers, high-score persistence
- `states.rs`: `AppState`, `PlayState`, `GameSet`
- `combat.rs`: collisions, scoring, rescue, damage, wave completion
- `waves.rs`: spawn orchestration and play-state transitions

## State Flow

```text
AppState: StartScreen -> Playing -> GameOver
PlayState: WaveIntro -> WaveActive -> WaveClear -> WaveIntro ...
                                     -> PlayerDeath -> WaveActive / WaveClear / GameOver
                                     -> Paused
```

## Bevy Notes

- `despawn()` is recursive by default.
- `WindowResolution::new(width, height)` takes `u32`.
- `ScalingMode` lives in `bevy::camera::ScalingMode`.
- Use `ApplyDeferred` when later systems in the same schedule need to observe queued despawns or inserts from earlier systems.
- `DespawnOnExit` is preferred for state-owned cleanup.
- `Mesh2d`, `MeshMaterial2d<ColorMaterial>`, and `Text2d` are the current rendering approach.

## Current Gameplay Summary

- Keyboard controls use `WASD` for movement and arrow keys for aim/fire.
- The arena is single-screen and rectangular.
- Collision is manual circle-circle overlap; there is no physics crate.
- Waves include grunts, hulks, brains, progs, spheroids, enforcers, quarks, tanks, electrodes, and humans.
- HUD, pause, game over, particles, score popups, bloom, and screen shake are implemented.

## Repo Conventions

- Prefer small coherent changes over architecture churn.
- Put new tunables in `constants.rs`.
- Put new shared state in `resources.rs`.
- Put reusable ECS markers/data in `components.rs`.
- Extend the owning domain plugin instead of growing `main.rs`.
- Run `cargo check` after most changes and `cargo clippy --all-targets --all-features` when touching shared patterns.
