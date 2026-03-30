# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\bevy_template`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- `bevy` is enabled with the `dynamic_linking` feature
- Current app state: modular Bevy prototype with starter scene wiring plus early shared foundation modules, not a full game yet

## Build And Validation

Use these commands from the project root:

```powershell
cargo run
cargo build
cargo check
cargo clippy
```

Validation expectations:

- Run `cargo check` for most Rust code changes.
- Run `cargo clippy` when changing patterns that may affect API usage or code quality broadly.
- If you cannot run validation, say so explicitly.

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Keep iteration-friendly workflows in mind; prefer targeted changes over broad rewrites.

## Current Project Layout

- `src/main.rs`: app bootstrap, plugin registration, and starter scene setup
- `src/input.rs`: `InputPlugin` plus the `InputActions` resource populated from keyboard and gamepad input
- `src/constants.rs`: shared tunables such as window size
- `src/components.rs`: shared ECS components such as `Velocity`
- `src/resources.rs`: shared resources such as `Score`
- `src/states.rs`: `AppState` definition for future state-driven flow
- `src/player.rs`: `PlayerPlugin` stub
- `src/enemy.rs`: `EnemyPlugin` stub
- `src/collision.rs`: `CollisionPlugin` stub
- `src/ui.rs`: `UiPlugin` stub
- `src/audio.rs`: `AudioPlugin` stub
- `assets/`: currently present but empty
- `.cargo/config.toml`: shared cargo target-dir configuration

## Current Runtime Behavior

The current app is still a starter scene with early module wiring:

- `DefaultPlugins` are registered.
- The app registers `InputPlugin`, `PlayerPlugin`, `EnemyPlugin`, `CollisionPlugin`, `UiPlugin`, and `AudioPlugin`.
- `setup` spawns a `Camera2d`.
- `setup` also spawns a centered `Hello, World!` UI text node.
- `InputPlugin` initializes an `InputActions` resource and updates it during `PreUpdate`.
- Keyboard input supports left/right movement, clockwise/counterclockwise rotation, soft drop, and hard drop.
- Gamepad input merges into the same `InputActions` resource.
- `constants`, `components`, `resources`, and `states` modules exist, but their types are not meaningfully wired into app setup or gameplay yet.
- `PlayerPlugin`, `EnemyPlugin`, `CollisionPlugin`, `UiPlugin`, and `AudioPlugin` are currently placeholders with no systems registered yet.

When making changes, align your work with what actually exists in the repo rather than assuming a larger gameplay architecture is already implemented.

## Architecture Guidance For Near-Term Expansion

The project has already started moving toward a plugin-per-domain layout with shared support modules. Prefer continuing that direction:

- Keep `src/main.rs` focused on app setup, plugin registration, and high-level wiring.
- Extend the existing domain modules before adding new ad hoc systems to `main.rs`.
- Keep input concerns in `src/input.rs`.
- Use the existing `src/constants.rs`, `src/components.rs`, `src/resources.rs`, and `src/states.rs` modules instead of recreating those concepts inside domain files.
- Add new gameplay domains as separate modules/plugins when they own distinct behavior.

The current shared foundation is intentionally small:

- `src/constants.rs` currently holds window sizing constants.
- `src/components.rs` currently holds `Velocity`.
- `src/resources.rs` currently holds `Score`.
- `src/states.rs` currently defines `AppState` with `StartScreen`, `Playing`, and `GameOver`.

If you add more gameplay, prefer extending those files before introducing parallel duplicates elsewhere.

## Bevy Conventions To Follow

- Use resources for cross-system shared state such as input snapshots, score, or timers.
- Use marker/data components for entity categories and per-entity state.
- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry once states are introduced.
- Gate gameplay systems with `run_if(in_state(...))` once an `AppState` exists.
- Use explicit ordering with `.after(...)` where frame ordering matters.
- Prefer `Timer` over frame counting for cooldowns and intervals.

## Coding Rules For This Repo

- Make the smallest coherent change that solves the task.
- Do not rewrite working structure just to make it "cleaner".
- Preserve the current plugin/module split unless the task clearly calls for a restructure.
- Keep shared input mapping logic centralized in `src/input.rs`.
- Put new tunable values in `src/constants.rs` instead of scattering magic numbers.
- Put shared ECS marker/data types in `src/components.rs`.
- Put shared mutable game-wide state in `src/resources.rs`.
- Extend `src/states.rs` when adding real state-driven flow.
- When gameplay entities become state-scoped, define the matching cleanup path on `OnExit` or use Bevy's state-based despawn helpers.

## UI And Asset Notes

- UI currently uses Bevy's component-based UI directly.
- Asset paths should remain plain relative strings passed to `asset_server.load(...)`.
- Keep asset references aligned with files under `assets/`.
- Do not assume themed art or audio already exists; check `assets/` before wiring references.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`.
- Use `ApplyDeferred` rather than a nonexistent `apply_deferred` helper function.
- 2D rendering uses current Bevy APIs such as `Camera2d`, `Sprite`, `Mesh2d`, and `MeshMaterial2d`.
- `WindowResolution` is not in the prelude; import it from `bevy::window::WindowResolution`.

## Preferred Change Pattern

1. Inspect the current code and local module boundaries before making assumptions.
2. Implement the change in the owning file or module.
3. Extract shared constants/resources/components only when the added complexity is justified.
4. Run validation, usually `cargo check`, for Rust code changes.
5. Summarize what changed and any remaining risks.

## Good First Places To Look

- App boot and plugin wiring: `src/main.rs`
- Input mapping and action resource: `src/input.rs`
- Shared constants/components/resources/state definitions: `src/constants.rs`, `src/components.rs`, `src/resources.rs`, `src/states.rs`
- Build output location: `.cargo/config.toml`
- Dependency/runtime configuration: `Cargo.toml`
- Available assets, if any: `assets/`
