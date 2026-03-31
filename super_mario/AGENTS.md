# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\super_mario`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- Level data is loaded through `ron` + `serde` asset deserialization
- `bevy` is enabled with the `dynamic_linking` feature
- The package name in `Cargo.toml` is currently `bevy_template`
- Current app state: Super Mario prototype covering `TASKS.md` Phases 1-13 plus Phase 14.1 and 14.5, with RON-backed multi-level progression, overworld decorations, shared keyboard/gamepad input, block interactions, mushrooms/growth, Fire Mario + fireballs, Goomba gameplay, Koopa shell mechanics, Starman, 1-Up mushrooms, and a scripted flagpole-to-next-level sequence implemented through small domain plugins

## Build And Validation

Use these commands from the project root:

```powershell
cargo run
cargo build
cargo check
cargo clippy
cargo test
```

Validation expectations:

- Run `cargo check` for most code changes.
- Run `cargo clippy` when changing patterns that may affect API usage or code quality broadly.
- `cargo test` is available, including the level-RON generation test in `src/level.rs`.
- For docs-only changes, validation is optional.
- If you cannot run validation, say so explicitly.

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Keep iteration-friendly workflows in mind; prefer targeted changes over broad rewrites.

## Current Project Layout

- `src/main.rs`: top-level app setup, resource/state initialization, message registration, gameplay set ordering, asset loader registration, and plugin registration
- `src/input.rs`: gathers keyboard and gamepad input into the shared `ActionInput` resource during `PreUpdate`
- `src/assets.rs`: startup-time creation of shared mesh/material handles stored in the `GameAssets` resource, grouped into nested asset structs with `spawn()` helpers
- `src/decoration.rs`: decoration asset setup plus overworld decoration spawning for clouds, bushes, and hills
- `src/components.rs`: shared ECS components such as `Player`, `Velocity`, `FacingDirection`, `Grounded`, `Tile`, `TileType`, enemy markers, power-up markers, shell state, decoration markers, and HUD markers
- `src/constants.rs`: tunable constants for window size, camera behavior, physics, z-layers, timer values, death values, block interactions, power-ups, decorations, enemies, fireballs, starman, and flagpole/castle flow
- `src/collision.rs`: shared AABB overlap logic and tile-collision resolution used by the player and moving enemies/items
- `src/level.rs`: custom `.level.ron` asset type/loader, active level handle, spawner registry, level grid resource, grid/world coordinate helpers, hardcoded level generators for dev/testing, and the level load/spawn systems
- `src/player.rs`: player input consumption, gravity, movement, tile collision, left-camera clamp, pit death detection, skid visuals, and player-face positioning
- `src/death.rs`: death animation flow for `PlayState::Dying`, including respawn or game-over transition
- `src/level_complete.rs`: flagpole collision handling and the scripted flagpole-to-castle level-complete sequence, followed by level advancement
- `src/camera.rs`: camera setup, reset, and follow logic
- `src/ui.rs`: start screen, level-transition screen, HUD, countdown timer, pause overlay, game-over UI flow, and score popup helpers
- `src/enemy/mod.rs`: shared enemy plugin wiring, activation, shared enemy physics, score popups, and common damage helper
- `src/enemy/goomba.rs`: Goomba plugin, spawner registration, and Mario-Goomba or star-power interactions
- `src/enemy/koopa.rs`: Koopa plugin, spawner registration, shell assets, and Mario-Koopa/Mario-shell/shell-enemy or star-power interactions
- `src/block.rs`: block-hit processing, block bounce, brick break particles, coin pop, floating-coin collection, and question-block content selection between mushroom, Fire Flower, Starman, and 1-Up
- `src/powerup.rs`: mushroom/Fire Flower/Starman emergence and collection, fireball systems, growth animation, shrink/invincibility handling, star power, ducking, and 1-Up collection
- `src/resources.rs`: `GameData`, `GameTimer`, score/coin messages, `SpawnPoint`, death-animation resources, pending block-hit state, level-complete animation state, `LevelList`, and level-transition timer state
- `src/states.rs`: `AppState`, `PlayState`, and gameplay `SystemSet` definitions
- `assets/levels/`: current level data assets such as `test.level.ron`, `1-1.level.ron`, and `1-2.level.ron`
- `TASKS.md`: implementation roadmap for the Mario clone
- `SPEC.md`: gameplay and behavior spec for future expansion
- `.cargo/config.toml`: shared cargo target-dir configuration

## Current Runtime Behavior

The current app is a runnable Mario prototype covering `TASKS.md` Phases 1-13 plus Phase 14.1 and 14.5:

- `DefaultPlugins` are registered with a configured primary window.
- The window is `800x600` and titled `Super Mario Bros`.
- `ClearColor` defaults to a light blue sky tone, but the active level asset can override the background color.
- `AppState` currently includes `StartScreen`, `LevelTransition`, `Playing`, and `GameOver`.
- `PlayState` currently includes `Running`, `Dying`, `Paused`, `LevelComplete`, and `Growing`.
- `GameplaySet` ordering is used to separate input, physics, camera, and late-frame systems while `PlayState::Running` is active, so gameplay freezes cleanly during pause, death, growth transitions, and the scripted flagpole sequence.
- Startup initializes `GameData`, `GameTimer`, `SpawnPoint`, `LevelList`, `ScoreEvent`/`CoinEvent` messages, shared `GameAssets`, decoration assets, the custom `LevelAssetLoader`, and a `SpawnerRegistry`.
- `src/input.rs` gathers keyboard and any connected gamepad input into a unified `ActionInput` resource; gameplay and menus consume that resource instead of reading devices directly.
- The start screen transitions into `AppState::LevelTransition` on confirm input, then into play after a short level-transition timer.
- The level-transition screen displays the current world and remaining lives.
- `LevelList` currently cycles between `levels/1-1.level.ron` and `levels/1-2.level.ron`.
- Startup and level transitions load the current level path into a `LevelHandle`; `spawn_level` reads that asset and falls back to the hardcoded `level_test()` grid if the asset is not ready.
- `assets/levels/1-1.level.ron`, `assets/levels/1-2.level.ron`, and `assets/levels/test.level.ron` exist, while `src/level.rs` still keeps hardcoded generators plus a test that can regenerate some RON files.
- Entering `AppState::Playing` builds the `LevelGrid`, applies level metadata for timer, world name, background color, and theme, records the `SpawnPoint`, spawns the world from the loaded grid through registry-driven tile/entity spawners, conditionally spawns decorations, and creates the HUD.
- `LevelData` supports `time`, `world_name`, `background_color`, `gravity_multiplier`, and `theme` metadata in the RON format. The current runtime applies timer, world name, background color, and theme; `gravity_multiplier` exists in the format but is not yet used by gameplay systems.
- Level themes currently distinguish overworld from underground presentation: overworld levels spawn clouds, bushes, and hills, while underground levels skip those outdoor decorations and can use darker backgrounds.
- Level tiles currently use characters for spawn point, ground, bricks, coin question blocks, mushroom/power-up question blocks, Starman blocks, 1-Up blocks, solid stair blocks, pipe pieces, Goombas, Koopas, floating coins, and flagpole segments.
- Colored primitive meshes are spawned from shared asset handles for world tiles, Mario, the player face, skid state, power-ups, fireballs, starman, 1-Up mushrooms, flagpole/castle pieces, brick particles, and score popups, while enemy modules own their own visuals and spawner registration.
- Mario spawns at the `S` tile position with `Player`, `PlayerSize`, `CollisionSize`, `Velocity`, `FacingDirection`, and `Grounded`.
- Horizontal movement includes acceleration/deceleration, air control, run speed, jumping, jump-cut behavior, and a left-edge clamp so Mario cannot move behind the camera.
- Big Mario and Fire Mario can duck while grounded; ducking shrinks the collision box and blocks horizontal movement until the player stands back up.
- Mario also has a face child mesh and a skid visual when reversing direction at speed.
- `src/collision.rs` owns the shared neighborhood-based AABB tile-collision resolver used by the player and moving enemies/items.
- Gravity, velocity integration, tile collision, grounded probing, pit death, and death animation/respawn are active.
- The death flow is owned by `src/death.rs`, which handles the pause/bounce animation, life loss, respawn, and game-over transition.
- The camera follows Mario horizontally with smooth lerp, uses a dead-zone offset, never scrolls left, and clamps to level bounds.
- The HUD displays score, coins, world name, and countdown timer during play.
- `GameData::add_coin()` awards an extra life every 100 coins.
- Escape or controller start pauses gameplay and shows a pause overlay.
- Goombas spawn from `G` tiles, activate near the camera, patrol, collide with tiles, fall off ledges, can be stomped, and damage Mario on side/bottom contact unless star power is active.
- Koopas spawn from `K` tiles, patrol like Goombas, turn into shells when stomped, can be kicked, bounce off walls, and support chain enemy-kill scoring while moving.
- Moving shell contact damages Mario on side contact, while stomping a moving shell stops it.
- Enemy contact respects player size: Small Mario dies, Big Mario or Fire Mario shrink through the growth/shrink transition, and temporary invincibility frames make Mario flash and ignore enemy hits.
- Mario head hits are tracked through `PendingBlockHit`, selecting the closest hittable block when multiple blocks overlap.
- `?` blocks release coins, `M` blocks release a mushroom for Small Mario or a Fire Flower otherwise, `T` blocks release a Starman, and `L` blocks release a 1-Up mushroom. Used question blocks become `E` in the level grid.
- Bricks bump for Small Mario and break into particles for Big Mario or Fire Mario.
- Bumping a brick can also kill active enemies standing on top of that block.
- Floating `C` coins are placed in the level and collected on contact.
- Mushrooms emerge upward from blocks, then move using shared enemy-style ground physics until collected.
- Collecting a mushroom awards score and starts the `PlayState::Growing` animation when Mario is Small.
- Fire Flowers emerge upward from blocks, can be collected when fully emerged, upgrade Big Mario to Fire Mario, and swap Mario's material to the fire palette.
- Fire Mario can shoot up to two fireballs; fireballs travel horizontally, bounce on ground contact, despawn on wall hit or falling out of bounds, and kill Goombas or convert Koopas into shells.
- Starman emerges from blocks, starts bouncing with enemy-style movement, and grants temporary star power on collection.
- Star power cycles Mario through flashing colors and lets Mario defeat enemies on contact while active.
- 1-Up mushrooms can emerge from blocks and grant an extra life on collection.
- Touching the flagpole starts `PlayState::LevelComplete`: Mario snaps to the pole, slides down while the flag descends, walks to the castle, tallies remaining time into score, then advances `LevelList` and transitions to the next level screen.
- Flagpole and castle are rendered from primitive meshes when the level contains `F` tiles.
- Automatic progression between the current 1-1 and 1-2 levels is implemented. Manual level selection, moving platforms, warp pipes, screen shake, combo stomp scoring, and sprite/audio asset pipelines are still not implemented.

When making changes, align your work with what actually exists in the repo rather than assuming later phases from `TASKS.md` are already present.

## Architecture Guidance For Future Expansion

As this prototype grows toward the roadmap in `TASKS.md`, prefer this structure:

- `src/main.rs`: app setup, plugin registration, and high-level wiring
- `src/input.rs`: device aggregation and input normalization into shared action resources
- `src/assets.rs`: shared runtime-created meshes/materials and any future reusable visual asset handles
- `src/decoration.rs`: non-interactive level decoration assets and spawn rules
- `src/constants.rs`: tunable values such as window size, physics, player speeds, enemy values, decoration spacing, and UI sizing
- `src/components.rs`: marker and data ECS components shared across domains
- `src/collision.rs`: reusable collision math and tile-resolution helpers
- `src/level.rs`: level asset definitions/loaders, coordinate helpers, registry-based spawning support, and future level progression helpers
- `src/resources.rs`: shared mutable game-wide state and animation resources
- `src/states.rs`: `AppState`, sub-state enums, and system-set definitions
- Domain modules such as `src/player.rs`, `src/death.rs`, `src/level_complete.rs`, `src/camera.rs`, `src/ui.rs`, `src/block.rs`, `src/powerup.rs`, and `src/enemy/`
- Add new domain files only when a feature clearly outgrows the existing modules
- Prefer building future level progression on the `.level.ron` asset flow under `assets/levels/` rather than re-expanding hardcoded grids

Prefer extending the existing domain plugins over growing `main.rs` indefinitely.

## Bevy Conventions To Follow

- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry once states are introduced.
- Gate gameplay systems with `run_if(in_state(...))` once an `AppState` exists.
- Use resources for cross-system shared state.
- Use messages for buffered inter-system score/coin communication.
- Normalize raw device input in `src/input.rs` and consume the shared action resource elsewhere.
- Use marker components for entity categories.
- Use explicit system ordering with `.after(...)` where frame ordering matters.
- Keep temporary prototype systems simple, but convert magic-number behavior into constants when the pattern stabilizes.

## Coding Rules For This Repo

- Make the smallest coherent change that solves the task.
- Do not rewrite working structure just to make it "cleaner".
- Preserve the existing split between `main.rs`, `input.rs`, `assets.rs`, `decoration.rs`, `components.rs`, `constants.rs`, `collision.rs`, `level.rs`, `player.rs`, `death.rs`, `level_complete.rs`, `camera.rs`, `block.rs`, `powerup.rs`, `ui.rs`, `resources.rs`, `states.rs`, and `src/enemy/`.
- Add new tunable values to `src/constants.rs` instead of scattering magic numbers.
- Add shared marker/data ECS types to `src/components.rs` instead of growing `main.rs` with inline definitions.
- Keep shared mesh/material setup in `src/assets.rs` instead of recreating identical Bevy assets across gameplay systems.
- Keep non-interactive scenery logic in `src/decoration.rs`.
- Keep input normalization in `src/input.rs` instead of re-reading devices throughout gameplay code.
- Keep shared collision math and tile-resolution logic in `src/collision.rs` instead of duplicating it in player/enemy/item systems.
- Keep level asset definitions, level loading, grid/world conversion helpers, and the spawner registry in `src/level.rs` instead of duplicating them in gameplay systems.
- Keep cross-system state in `src/resources.rs` and state enums/system sets in `src/states.rs`.
- Keep block-hit logic in `src/block.rs`, power-up and player-size transitions in `src/powerup.rs`, death flow in `src/death.rs`, level-complete scripting in `src/level_complete.rs`, and enemy-specific interactions inside `src/enemy/`.
- If you add or edit level content, prefer updating the `.level.ron` files in `assets/levels/` to match the active level-loading path.
- If you add new level characters or spawnable entities, update the owning module's spawner registration and keep `src/level.rs` as the shared registry/dispatch layer.
- If you add new level metadata such as themes or environment modifiers, keep the asset schema in `src/level.rs` and route the owning behavior to the appropriate domain module.
- Prefer extending an existing domain plugin over registering more ad hoc systems from `main.rs`.
- When spawning entities tied to `AppState` or `PlayState`, define the matching cleanup path using `DespawnOnExit` or `OnExit` behavior if they should not persist.

## UI And Asset Notes

- Current visuals still use Bevy 2D primitives (`Rectangle`, `Mesh2d`, `MeshMaterial2d`) rather than sprite assets, but those shared handles are centralized in `src/assets.rs`, `src/decoration.rs`, and the enemy asset resources in `src/enemy/`.
- The current level flow is data-driven through `.level.ron` files under `assets/levels/`, deserialized into row strings and converted into the fixed runtime grid in `src/level.rs`.
- Level spawning is registry-based: base tiles/entities are registered in `src/level.rs`, while enemy modules register their own tile-character spawners during startup.
- Level themes are currently lightweight metadata: `theme = "overworld"` enables clouds, bushes, and hills, while `theme = "underground"` suppresses those overworld decorations.
- UI is currently built with Bevy's component-based UI and `Text`/`Text2d`.
- `AssetServer` is part of the level-loading flow; do not assume sprite/audio assets are part of the flow yet.
- `assets/levels/` is actively used, while the rest of the prototype visuals are still built procedurally with meshes and materials.
- When assets are introduced, keep paths as plain relative strings passed to `asset_server.load(...)`.
- Keep asset references aligned with files under `assets/`.
- Match new art/audio usage to the Mario project direction described in `TASKS.md` and `SPEC.md`, not the old shooter template language.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`.
- Use `ApplyDeferred` rather than a nonexistent `apply_deferred` helper function.
- Use `add_message`, `MessageWriter`, and `MessageReader` for buffered messages instead of the old event names.
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
- Shared action input: `src/input.rs`
- Shared render assets: `src/assets.rs`
- Decoration assets and overworld scenery: `src/decoration.rs`
- Shared ECS types: `src/components.rs`
- Tunable gameplay values: `src/constants.rs`
- Shared collision helpers: `src/collision.rs`
- Level asset loading, RON format, grid helpers, metadata, and spawn registry: `src/level.rs`
- Player movement, skid behavior, and pit-death detection: `src/player.rs`
- Death animation and respawn/game-over flow: `src/death.rs`
- Flagpole detection, time tally, and level advancement: `src/level_complete.rs`
- Camera behavior: `src/camera.rs`
- Block and collectible interactions: `src/block.rs`
- Power-up, growth, star power, and invincibility flow: `src/powerup.rs`
- Enemy plugin wiring and shared behavior: `src/enemy/mod.rs`
- Goomba interactions: `src/enemy/goomba.rs`
- Koopa and shell interactions: `src/enemy/koopa.rs`
- UI/state transitions, HUD, and level-transition screen: `src/ui.rs`
- Shared runtime state and progression resources: `src/resources.rs`
- App/play state definitions: `src/states.rs`
- Current level assets: `assets/levels/`
- Build output location: `.cargo/config.toml`
- Dependency/runtime configuration: `Cargo.toml`
- Planned feature order: `TASKS.md`
- Project behavior targets: `SPEC.md`
- Available art/audio for later phases: `assets/`
