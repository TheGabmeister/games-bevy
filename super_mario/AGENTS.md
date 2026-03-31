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
- Current app state: Super Mario prototype covering `TASKS.md` Phases 1-9, with state flow, HUD, death/lives handling, block interactions, mushrooms/growth, Goomba gameplay, and Koopa shell mechanics implemented using small domain plugins

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

- `src/main.rs`: top-level app setup, resource/state initialization, set ordering, and plugin registration
- `src/components.rs`: shared ECS components such as `Player`, `Velocity`, `FacingDirection`, `Grounded`, `Tile`, `TileType`, enemy markers, power-up markers, shell state, and HUD markers
- `src/constants.rs`: tunable constants for window size, camera behavior, physics, z-layers, timer values, death values, block interactions, power-ups, and enemy values
- `src/collision.rs`: shared AABB overlap logic and tile-collision resolution used by the player and moving enemies/items
- `src/level.rs`: level grid resource, grid/world coordinate helpers, solid-tile queries, Level 1-1 data, and the level spawn system for `AppState::Playing`
- `src/player.rs`: player input, gravity, movement, tile collision, left-camera clamp, pit death, and death-animation/respawn flow
- `src/camera.rs`: camera setup, reset, and follow logic
- `src/ui.rs`: start screen, HUD, countdown timer, pause overlay, and game-over UI flow
- `src/enemy/mod.rs`: shared enemy plugin wiring, activation, shared enemy physics, score popups, and common damage helper
- `src/enemy/goomba.rs`: Mario-Goomba stomp/damage behavior
- `src/enemy/koopa.rs`: Mario-Koopa, Mario-shell, and shell-enemy interactions
- `src/block.rs`: block-hit processing, block bounce, brick break particles, coin pop, and floating-coin collection
- `src/powerup.rs`: mushroom emergence/collection, growth animation, shrink/invincibility handling, and ducking
- `src/resources.rs`: `GameData`, `SpawnPoint`, death-animation resources, pending block-hit state, and cached player meshes
- `src/states.rs`: `AppState`, `PlayState`, and gameplay `SystemSet` definitions
- `assets/`: available project assets for later phases, still not wired into the current prototype
- `TASKS.md`: implementation roadmap for the Mario clone
- `SPEC.md`: gameplay and behavior spec for future expansion
- `.cargo/config.toml`: shared cargo target-dir configuration

## Current Runtime Behavior

The current app is a runnable Mario prototype covering `TASKS.md` Phases 1-9:

- `DefaultPlugins` are registered with a configured primary window.
- The window is `800x600` and titled `Super Mario Bros`.
- `ClearColor` is set to a light blue sky tone.
- `AppState` currently includes `StartScreen`, `Playing`, and `GameOver`.
- `PlayState` currently includes `Running`, `Dying`, `Paused`, `LevelComplete`, and `Growing`.
- `GameplaySet` ordering is used to separate input, physics, camera, and late-frame systems while `PlayState::Running` is active, so gameplay freezes cleanly during pause, death, and growth transitions.
- A `Camera2d` is spawned at startup with an orthographic scale derived from `CAMERA_VISIBLE_HEIGHT`.
- The camera follows Mario horizontally with smooth lerp, uses a dead-zone offset, never scrolls left, and clamps to level bounds.
- Entering `AppState::Playing` spawns Level 1-1 from tile data in `src/level.rs`, resets `GameData`, records the `SpawnPoint`, caches player meshes in a resource, and creates the HUD.
- Level 1-1 data is generated in `src/level.rs` as a `211 x 15` grid and stored as a `LevelGrid` resource.
- Level tiles currently use characters for spawn point, ground, bricks, coin question blocks, mushroom question blocks, solid stair blocks, pipe pieces, Goombas, Koopas, floating coins, and a reserved `F` marker in the grid for future flagpole work.
- Colored primitive meshes are spawned for ground, bricks, question blocks, used blocks, solid stair blocks, pipes, Mario, Goombas, Koopas, mushrooms, floating coins, shell forms, brick particles, and score popups.
- Mario spawns at the `S` tile position with `Player`, `PlayerSize`, `CollisionSize`, `Velocity`, `FacingDirection`, and `Grounded`.
- Horizontal movement includes acceleration/deceleration, air control, run speed with Shift, jumping, jump-cut behavior, and a left-edge clamp so Mario cannot move behind the camera.
- Big Mario can duck while grounded; ducking shrinks the collision box and blocks horizontal movement until the player stands back up.
- `src/collision.rs` owns the shared neighborhood-based AABB tile-collision resolver used by the player and moving enemies/items.
- Gravity, velocity integration, tile collision, grounded probing, pit death, and death animation/respawn are active.
- The start screen transitions into play on Enter, and the game-over screen returns to the start screen on Enter.
- The HUD displays score, coins, world name, and countdown timer during play.
- `GameData::add_coin()` awards an extra life every 100 coins.
- Escape pauses gameplay and shows a pause overlay.
- Goombas spawn from `G` tiles, activate near the camera, patrol, collide with tiles, fall off ledges, can be stomped, and kill Mario on side/bottom contact.
- Koopas spawn from `K` tiles, patrol like Goombas, turn into shells when stomped, can be kicked, bounce off walls, and support chain enemy-kill scoring while moving.
- Moving shell contact damages Mario on side contact, while stomping a moving shell stops it.
- Enemy contact now respects player size: Small Mario dies, Big Mario shrinks through the growth/shrink transition, and temporary invincibility frames make Mario flash and ignore enemy hits.
- Mario head hits are tracked through `PendingBlockHit`, selecting the closest hittable block when multiple blocks overlap.
- `?` blocks release coins, `M` blocks release mushrooms, used question blocks become `E` in the level grid, bricks bump for Small Mario, and bricks break into particles for Big Mario.
- Bumping a brick can also kill active enemies standing on top of that block.
- Floating `C` coins are placed in the level and collected on contact.
- Mushrooms emerge upward from blocks, then move using shared enemy-style ground physics until collected.
- Collecting a mushroom awards score and starts the `PlayState::Growing` animation when Mario is Small.
- Fire Flower, fireballs, flagpole/castle rendering, `LevelComplete` gameplay flow, time-bonus tallying, decorations, asset-driven rendering, and external level-file loading are not implemented yet.

When making changes, align your work with what actually exists in the repo rather than assuming later phases from `TASKS.md` are already present.

## Architecture Guidance For Future Expansion

As this prototype grows toward the roadmap in `TASKS.md`, prefer this structure:

- `src/main.rs`: app setup, plugin registration, and high-level wiring
- `src/constants.rs`: tunable values such as window size, physics, player speeds, enemy values, and UI sizing
- `src/components.rs`: marker and data ECS components shared across domains
- `src/collision.rs`: reusable collision math and tile-resolution helpers
- `src/level.rs`: level data, coordinate helpers, and tile/level spawning support
- `src/resources.rs`: shared mutable game-wide state and animation resources
- `src/states.rs`: `AppState`, sub-state enums, and system-set definitions
- Domain modules such as `src/player.rs`, `src/camera.rs`, `src/ui.rs`, `src/block.rs`, `src/powerup.rs`, and `src/enemy/`
- Add new domain files such as `src/fire.rs` or `src/level_complete.rs` only when a feature clearly outgrows the existing modules

Prefer extending the existing domain plugins over growing `main.rs` indefinitely.

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
- Preserve the existing split between `main.rs`, `components.rs`, `constants.rs`, `collision.rs`, `level.rs`, `player.rs`, `camera.rs`, `block.rs`, `powerup.rs`, `ui.rs`, `resources.rs`, `states.rs`, and `src/enemy/`.
- Add new tunable values to `src/constants.rs` instead of scattering magic numbers.
- Add shared marker/data ECS types to `src/components.rs` instead of growing `main.rs` with inline definitions.
- Keep shared collision math and tile-resolution logic in `src/collision.rs` instead of duplicating it in player/enemy/item systems.
- Keep level layout data and grid/world conversion helpers in `src/level.rs` instead of duplicating them in gameplay systems.
- Keep cross-system state in `src/resources.rs` and state enums/system sets in `src/states.rs`.
- Keep block-hit logic in `src/block.rs`, power-up and player-size transitions in `src/powerup.rs`, and enemy-specific interactions inside `src/enemy/`.
- Prefer extending an existing domain plugin over registering more ad hoc systems from `main.rs`.
- When spawning entities tied to `AppState` or `PlayState`, define the matching cleanup path using `DespawnOnExit` or `OnExit` behavior if they should not persist.

## UI And Asset Notes

- Current visuals use Bevy 2D primitives (`Rectangle`, `Mesh2d`, `MeshMaterial2d`) rather than sprite assets.
- The current level is also data-driven through tile characters in `src/level.rs`; keep new level features aligned with that approach unless the task explicitly changes it.
- UI is currently built with Bevy's component-based UI and `Text`/`Text2d`.
- Asset loading is still not wired up; do not assume `AssetServer` is already part of the flow.
- `assets/` exists for later phases, but the current prototype still builds visuals procedurally with meshes and materials.
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

- App boot and plugin wiring: `src/main.rs`
- Shared ECS types: `src/components.rs`
- Tunable gameplay values: `src/constants.rs`
- Shared collision helpers: `src/collision.rs`
- Level data and grid helpers: `src/level.rs`
- Player movement and death flow: `src/player.rs`
- Camera behavior: `src/camera.rs`
- Block and collectible interactions: `src/block.rs`
- Power-up, growth, and invincibility flow: `src/powerup.rs`
- Enemy plugin wiring and shared behavior: `src/enemy/mod.rs`
- Goomba interactions: `src/enemy/goomba.rs`
- Koopa and shell interactions: `src/enemy/koopa.rs`
- UI/state transitions: `src/ui.rs`
- Shared runtime state: `src/resources.rs`
- App/play state definitions: `src/states.rs`
- Build output location: `.cargo/config.toml`
- Dependency/runtime configuration: `Cargo.toml`
- Planned feature order: `TASKS.md`
- Project behavior targets: `SPEC.md`
- Available art/audio for later phases: `assets/`
