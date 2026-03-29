# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\robotron_2084`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- `bevy` is enabled with the `dynamic_linking` feature
- `rand 0.9` is used for spawn placement, AI wander, and particles

## Build And Validation

Use these commands from the project root:

```powershell
cargo run
cargo build
cargo check
cargo clippy --all-targets --all-features
```

Validation expectations:

- Run `cargo check` for most code changes.
- Run `cargo clippy --all-targets --all-features` when touching APIs, scheduling, or shared patterns.
- If you cannot run validation, say so explicitly.

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Prefer targeted iterations over broad rewrites so Bevy compile times stay reasonable.

## Current Project Layout

- `src/main.rs`: app bootstrap, window/camera setup, plugin registration, game-set ordering
- `src/constants.rs`: tunable values for arena size, movement, collisions, waves, FX, and UI timing
- `src/components.rs`: ECS markers and gameplay data components
- `src/resources.rs`: shared assets, game state, timers, screen shake, and high-score persistence
- `src/states.rs`: `AppState`, `PlayState`, and ordered `GameSet`s
- `src/arena.rs`: borders, velocity application, confinement, lifetime cleanup
- `src/player.rs`: player spawn, movement, aiming, firing, invincibility blink, respawn
- `src/enemy.rs`: enemy AI, spawner children, enemy projectiles, missile steering, shell bounces
- `src/human.rs`: human wandering
- `src/combat.rs`: rescue logic, collision handling, damage resolution, wave clear checks
- `src/waves.rs`: wave definitions, wave spawn orchestration, state transitions
- `src/effects.rs`: particles, score popups, camera shake
- `src/ui.rs`: start screen, HUD, pause overlay, game over flow
- `assets/`: available art/audio assets for later expansion

## Current Runtime Behavior

The project is already a playable Robotron-style prototype, not a starter scene.

- `AppState` flows through `StartScreen`, `Playing`, and `GameOver`.
- `PlayState` handles `WaveIntro`, `WaveActive`, `WaveClear`, `PlayerDeath`, and `Paused`.
- The arena, player, enemies, humans, waves, HUD, particles, and high-score persistence are implemented.
- Rendering uses primitive 2D meshes plus bright `ColorMaterial`s, bloom, and a fixed-vertical 2D camera.
- Gameplay uses manual circle-circle collision instead of a physics engine.

## Architecture Notes

- The repo already follows the recommended domain-plugin structure. Prefer extending the owning plugin instead of putting new systems back into `main.rs`.
- `GameSet::Input -> Movement -> Confinement -> Combat -> Resolution` is chained only while `PlayState::WaveActive`.
- Use `OnEnter` / `OnExit` for spawn-cleanup symmetry when entities are owned by a state.
- Use `WaveEntity` for entities that should be cleared between waves without leaving `AppState::Playing`.
- Keep cross-system shared state in resources rather than duplicating counters across modules.

## Bevy Conventions To Follow

- `despawn()` is recursive by default. Do not use removed APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`.
- When later systems must observe despawns or inserts from earlier systems in the same schedule, use `ApplyDeferred` explicitly.
- Use `DespawnOnExit` for state-owned entities and UI instead of frame-by-frame cleanup systems when possible.
- Prefer marker components for categories such as `Player`, `Killable`, `DamagesPlayer`, `EnemyProjectile`, and `WaveEntity`.

## Coding Rules For This Repo

- Make the smallest coherent change that solves the task.
- Do not rewrite working gameplay structure only to make it look cleaner.
- Keep module boundaries aligned to gameplay domains.
- Add new tunables to `src/constants.rs` instead of scattering magic numbers.
- Add new shared state to `src/resources.rs`.
- Add reusable markers/data components to `src/components.rs`.
- If a state owns an entity, define the matching cleanup path as part of the same change.

## Good First Places To Look

- App wiring: `src/main.rs`
- Gameplay state flow: `src/states.rs`, `src/waves.rs`
- Collision and scoring: `src/combat.rs`
- AI and spawns: `src/enemy.rs`
- UI flow: `src/ui.rs`
- Build output location: `.cargo/config.toml`
