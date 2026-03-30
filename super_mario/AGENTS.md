# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\super_mario`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- `bevy` is enabled with the `dynamic_linking` feature
- The package name in `Cargo.toml` is currently `bevy_template`
- Current app state: early Super Mario-style prototype, not a generic Bevy starter and not yet a full platformer implementation

## Build And Validation

Use these commands from the project root:

```powershell
cargo run
cargo build
cargo check
cargo clippy
```

Validation expectations:

- Run `cargo check` for most code changes.
- Run `cargo clippy` when changing patterns that may affect API usage or code quality broadly.
- For docs-only changes, validation is optional.
- If you cannot run validation, say so explicitly.

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Keep iteration-friendly workflows in mind; prefer targeted changes over broad rewrites.

## Current Project Layout

- `src/main.rs`: app setup, startup scene spawning, and current movement/physics systems
- `src/components.rs`: ECS components such as `Player`, `Velocity`, `FacingDirection`, `Grounded`, and `Ground`
- `src/constants.rs`: tunable constants for window size, camera scale, physics, player dimensions, and temporary ground dimensions
- `assets/`: available project assets for later phases, not yet wired into the current prototype
- `TASKS.md`: implementation roadmap for the Mario clone
- `SPEC.md`: gameplay and behavior spec for future expansion
- `.cargo/config.toml`: shared cargo target-dir configuration

## Current Runtime Behavior

The current app is a Phase 1-style platformer prototype:

- `DefaultPlugins` are registered with a configured primary window.
- The window is `800x600` and titled `Super Mario Bros`.
- A `Camera2d` is spawned at startup with an orthographic scale derived from `CAMERA_VISIBLE_HEIGHT`.
- `ClearColor` is set to a light blue sky tone.
- Mario is represented by a red rectangle mesh with `Player`, `Velocity`, `FacingDirection`, and `Grounded` components.
- The ground is represented by a single brown rectangle mesh.
- Horizontal input supports `A`/`D` and arrow keys.
- Gravity, velocity integration, and simple ground collision are active.
- The update pipeline is currently chained as `player_input -> apply_gravity -> apply_velocity -> ground_collision`.
- Jumping, acceleration/deceleration, tile maps, enemies, states, HUD, and asset-driven rendering are not implemented yet.

When making changes, align your work with what actually exists in the repo rather than assuming later phases from `TASKS.md` are already present.

## Architecture Guidance For Future Expansion

As this prototype grows toward the roadmap in `TASKS.md`, prefer this structure:

- `src/main.rs`: app setup, plugin registration, and high-level wiring
- `src/constants.rs`: tunable values such as window size, physics, player speeds, and UI sizing
- `src/components.rs`: marker and data ECS components shared across domains
- `src/resources.rs`: shared mutable game-wide state once it exists
- `src/states.rs`: `AppState` and sub-state enums once state flow is introduced
- `src/level.rs`: level data and tile spawning once tile maps are introduced
- Domain modules such as `src/player.rs`, `src/camera.rs`, `src/enemy.rs`, `src/ui.rs`, `src/audio.rs`, and `src/combat.rs` as systems grow

Prefer small domain plugins over growing `main.rs` indefinitely once the game has more than a handful of systems.

## Bevy Conventions To Follow

- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry once states are introduced.
- Gate gameplay systems with `run_if(in_state(...))` once an `AppState` exists.
- Use resources for cross-system shared state.
- Use marker components for entity categories.
- Use explicit system ordering with `.after(...)` where frame ordering matters.
- Keep temporary prototype systems simple, but convert magic-number behavior into constants when the pattern stabilizes.

## Coding Rules For This Repo

- Make the smallest coherent change that solves the task.
- Do not rewrite working structure just to make it "cleaner".
- Preserve the existing split between `main.rs`, `components.rs`, and `constants.rs`.
- Add new tunable values to `src/constants.rs` instead of scattering magic numbers.
- Add shared marker/data ECS types to `src/components.rs` instead of growing `main.rs` with inline definitions.
- Introduce new modules when they clearly own a gameplay domain, not preemptively.
- Prefer extending the existing system chain or a focused plugin over ad hoc scattered registration.
- When spawning entities tied to a future state, define the matching cleanup path on `OnExit` if they should not persist.

## UI And Asset Notes

- Current visuals use Bevy 2D primitives (`Rectangle`, `Mesh2d`, `MeshMaterial2d`) rather than sprite assets.
- Asset loading is not wired up yet; do not assume `AssetServer` is already part of the flow.
- When assets are introduced, keep paths as plain relative strings passed to `asset_server.load(...)`.
- Keep asset references aligned with files under `assets/`.
- Match new art/audio usage to the Mario project direction described in `TASKS.md` and `SPEC.md`, not the old shooter template language.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`.
- Use `ApplyDeferred` rather than a nonexistent `apply_deferred` helper function.
- 2D rendering uses current Bevy APIs such as `Camera2d`, `Sprite`, `Mesh2d`, and `MeshMaterial2d`.

## Preferred Change Pattern

1. Inspect the current code and local module boundaries before making assumptions.
2. Check `TASKS.md` or `SPEC.md` when the requested behavior depends on planned Mario mechanics.
3. Implement the change in the owning file or module.
4. Extract constants, components, resources, or modules only when the code has grown enough to justify it.
5. Run validation when code changes were made, usually `cargo check`.
6. Summarize what changed and any remaining risks.

## Good First Places To Look

- App boot and current gameplay loop: `src/main.rs`
- Shared ECS types: `src/components.rs`
- Tunable gameplay values: `src/constants.rs`
- Build output location: `.cargo/config.toml`
- Dependency/runtime configuration: `Cargo.toml`
- Planned feature order: `TASKS.md`
- Project behavior targets: `SPEC.md`
- Available art/audio for later phases: `assets/`
