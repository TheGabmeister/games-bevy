# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\zelda`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- `bevy` is enabled with the `dynamic_linking` feature
- `serde` and `ron` are used for lightweight data-driven content
- Current app state: modular Bevy prototype with explicit app-state flow, primitive-shape room rendering, data-driven rooms/items/drops, overworld + cave/shop/hint + first dungeon slice, inventory/dialogue HUD flow, a playable combat loop, and continue/game-over flow, but not a full Zelda feature set yet

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

- `src/main.rs`: app bootstrap, plugin registration, window configuration, and high-level wiring
- `src/game_state.rs`: `GameStatePlugin` with top-level app-state transitions including game-over continue flow
- `src/camera.rs`: `CameraPlugin` that spawns the fixed-height 2D room camera
- `src/input.rs`: `InputPlugin` plus the `InputActions` resource populated from keyboard and gamepad input
- `src/constants.rs`: canonical room-space, HUD, entry offset, logical resolution, and render-layer constants
- `src/components.rs`: shared ECS gameplay components such as `Velocity`, `Player`, `Enemy`, `Health`, `Hitbox`, `Hurtbox`, `SwordAttack`, timers, `PickupKind`, `Npc`, `ShopItem`, and `PushBlock`
- `src/rendering.rs`: shared primitive mesh/material helpers and world color conventions
- `src/resources.rs`: shared resources such as room state, room persistence, transition lock state, room/dungeon metadata, player vitals, inventory/equipped-item state, and dialogue state
- `src/items.rs`: `ItemsPlugin`, item/drop-table loading from RON, and shared pickup-effect application helpers
- `src/room.rs`: `RoomPlugin` with room-table loading, room lifecycle, overworld/cave/shop/hint/dungeon spawning, door/secret/shop interactions, and room-local persistence interactions
- `src/combat.rs`: `CombatPlugin` with sword spawning, damage resolution, invulnerability, knockback, death, and continue reset flow
- `src/states.rs`: `AppState` definition for top-level flow
- `src/player.rs`: `PlayerPlugin` with player spawning, movement input, facing, and facing-indicator updates
- `src/enemy.rs`: `EnemyPlugin` stub
- `src/collision.rs`: `CollisionPlugin` with player-vs-world collision and knockback-aware movement resolution
- `src/ui.rs`: title, HUD, dialogue overlay, paused inventory, and game-over UI overlays
- `src/audio.rs`: `AudioPlugin` stub
- `assets/data/items.ron`: data-driven pickup/item definitions loaded at startup
- `assets/data/drops.ron`: enemy drop table definitions loaded at startup
- `assets/data/rooms.ron`: unified room definitions for overworld, caves, shop/hint rooms, and the current dungeon slice
- `assets/`: currently contains gameplay data, but no themed art or audio assets yet
- `SPEC.md`: long-term feature target and staging plan; use it to distinguish current implementation from intended future scope
- `.cargo/config.toml`: shared cargo target-dir configuration

## Current Runtime Behavior

The current app is now a playable prototype shell rather than a starter scene:

- `DefaultPlugins` are registered.
- The window uses a logical `256x240` layout scaled to a default `1024x960` desktop window.
- The app registers `GameStatePlugin`, `ItemsPlugin`, `CameraPlugin`, `PrimitiveRenderingPlugin`, `RoomPlugin`, `InputPlugin`, `PlayerPlugin`, `EnemyPlugin`, `CombatPlugin`, `CollisionPlugin`, `UiPlugin`, and `AudioPlugin`.
- `CameraPlugin` spawns a `Camera2d` with `ScalingMode::FixedVertical` using the logical screen height.
- `GameStatePlugin` initializes `AppState`, advances from `Boot` to `Title`, and currently supports `Title -> Playing -> PausedInventory -> GameOver -> Playing/Title`.
- `ItemsPlugin` loads both `ItemTable` and `DropTable` resources from `assets/data/items.ron` and `assets/data/drops.ron` during startup using `std::fs` + `ron`.
- `RoomPlugin` loads a `RoomTable` from `assets/data/rooms.ron` and spawns the world from data rather than hardcoded room-specific spawn functions.
- The current world includes a small overworld graph, a sword cave, a shop cave, a hint cave, and a first dungeon slice with locked/shutter doors, staircase transitions, dungeon pickups, and Triforce progress.
- Rooms use primitive walls, doors, blockers, pickups, enemies, NPCs, shop offers, staircases, and secrets; temporary room entities reset on reload, while unique pickups, secrets, and dungeon doors persist through `RoomPersistence`.
- Secret interactions currently include hidden staircases, burnable bushes, bombable walls, and push-block reveals.
- `UiPlugin` spawns title text, a playing HUD for hearts/rupees/bombs/keys/equipped item, a dialogue overlay, a pause inventory overlay with dungeon progress, and a game-over continue overlay using Bevy UI text/nodes.
- `InputPlugin` initializes an `InputActions` resource and updates it during `PreUpdate`.
- Keyboard and gamepad input currently support Zelda-style movement, attack, item-use, pause, confirm, and cancel actions.
- `Inventory` tracks rupees, bombs, keys, sword ownership, and the currently equipped item; `DungeonState` tracks current dungeon, dungeon keys, map/compass ownership, boss defeat, room clear state, and Triforce pieces.
- `PlayerPlugin` spawns the player on room load, updates 4-direction movement/facing, and shows a centered triangle facing indicator.
- `CollisionPlugin` resolves player movement against static blockers and applies knockback-aware movement.
- `CombatPlugin` spawns short-lived sword attack entities only when the sword has been acquired, applies enemy contact damage, handles invulnerability/knockback/death, and spawns temporary enemy drops from the drop table.
- Pickup collection applies item effects through shared item data; health pickups update both live `Health` and persisted `PlayerVitals`, while dungeon pickups update `DungeonState`.
- `EnemyPlugin` and `AudioPlugin` are still placeholders; current enemies are spawned by `RoomPlugin` as simple room-local test entities.

When making changes, align your work with what actually exists in the repo rather than assuming a larger gameplay architecture is already implemented.

## Architecture Guidance For Near-Term Expansion

The project has already started moving toward a plugin-per-domain layout with shared support modules. Prefer continuing that direction:

- Keep `src/main.rs` focused on app setup, plugin registration, and high-level wiring.
- Extend the existing domain modules before adding new ad hoc systems to `main.rs`.
- Keep input concerns in `src/input.rs`.
- Keep item/drop-table loading and pickup effect rules in `src/items.rs`.
- Keep room schema loading, room spawning, secret logic, shop interactions, and staircase/door transitions in `src/room.rs`.
- Use the existing `src/constants.rs`, `src/components.rs`, `src/resources.rs`, and `src/states.rs` modules instead of recreating those concepts inside domain files.
- Add new gameplay domains as separate modules/plugins when they own distinct behavior.

The current shared foundation is intentionally small:

- `src/constants.rs` currently holds canonical room-space constants, logical resolution, door anchors, entry offsets, and render layers.
- `src/components.rs` currently holds the core shared gameplay markers/data used by movement, combat, pickups, NPC/shop interaction, and push-block secrets.
- `src/rendering.rs` currently holds shared rectangle/circle spawning helpers and the initial world color palette.
- `src/resources.rs` currently holds room id/transition/persistence state plus room type/door/dungeon/dialogue state, the current player-health snapshot, and inventory state used by HUD/pause/continue flow.
- `src/items.rs` currently holds the item metadata loader, enemy drop table loader, and shared pickup-effect rules.
- `src/room.rs` currently owns the unified room schema and the runtime translation from `assets/data/rooms.ron` into spawned entities and interactions.
- `src/states.rs` currently defines `AppState` with `Boot`, `Title`, `Playing`, `PausedInventory`, and `GameOver`.

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
- Put reusable primitive-shape spawning and shared color conventions in `src/rendering.rs`.
- Put shared ECS marker/data types in `src/components.rs`.
- Put shared mutable game-wide state in `src/resources.rs`.
- Put pickup metadata, effect definitions, and item/drop-table loading in `src/items.rs`; keep `assets/data/items.ron` and `assets/data/drops.ron` aligned with the Rust data model.
- Extend `src/states.rs` when adding real state-driven flow.
- Keep room-scale camera behavior in `src/camera.rs` rather than reconfiguring projection ad hoc from gameplay modules.
- Keep room loading, schema translation, room adjacency, persistence classification, screen-edge/staircase transition rules, dungeon door behavior, room-local dialogue/shop logic, and pickup placement in `src/room.rs`.
- Keep sword/damage/death/continue logic in `src/combat.rs` unless the task clearly calls for a broader combat refactor.
- Prefer adding or editing room content in `assets/data/rooms.ron` instead of hardcoding new room setup in Rust.
- When gameplay entities become state-scoped, define the matching cleanup path on `OnExit` or use Bevy's state-based despawn helpers.

## UI And Asset Notes

- UI currently uses Bevy's component-based UI directly.
- Asset paths should remain plain relative strings passed to `asset_server.load(...)`.
- Keep asset references aligned with files under `assets/`.
- The room, item, and drop tables currently load from `assets/data/*.ron` via `std::fs::read_to_string(...)` rather than `AssetServer`; preserve or consciously refactor that behavior.
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
- Top-level state flow and continue/game-over wiring: `src/game_state.rs`
- Camera setup and logical viewport: `src/camera.rs`
- Room lifecycle, schema loading, transitions, dungeon/shop/secret logic, pickups, and persistence behavior: `src/room.rs`
- Combat loop, sword hits, player damage, and death/continue rules: `src/combat.rs`
- Player spawning and movement/facing updates: `src/player.rs`
- Input mapping and action resource: `src/input.rs`
- Item metadata, drop tables, and pickup effects: `src/items.rs`, `assets/data/items.ron`, and `assets/data/drops.ron`
- Room content and current world graph: `assets/data/rooms.ron`
- Shared constants/components/resources/state definitions: `src/constants.rs`, `src/components.rs`, `src/rendering.rs`, `src/resources.rs`, `src/states.rs`
- Long-term intended scope versus current implementation: `SPEC.md`
- Build output location: `.cargo/config.toml`
- Dependency/runtime configuration: `Cargo.toml`
- Available assets and data: `assets/`
