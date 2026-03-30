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

- **Bevy 0.18.1** — ECS game engine
- **Rust Edition 2024**
- **serde + ron** — data-driven content loaded from `assets/data/*.ron`

## Architecture

Zelda-like top-down action game using primitive 2D shapes (no sprite assets). One-plugin-per-domain organization.

### Plugin Map

| Plugin | File | Role |
|--------|------|------|
| `GameStatePlugin` | `game_state.rs` | State transitions (Boot→Title→Playing⇄Paused/GameOver), `Inventory` init |
| `ItemsPlugin` | `items.rs` | Loads `assets/data/items.ron` into `ItemTable` resource at startup |
| `CameraPlugin` | `camera.rs` | Fixed orthographic camera, `ScalingMode::FixedVertical(240)` |
| `PrimitiveRenderingPlugin` | `rendering.rs` | Color palette, mesh/material spawn helpers, reactive `Label` → `Text2d` system |
| `RoomPlugin` | `room.rs` | Room loading, transitions, perimeter walls, doors, obstacles, pickups, secrets |
| `InputPlugin` | `input.rs` | Keyboard + gamepad → `InputActions` resource |
| `PlayerPlugin` | `player.rs` | Player spawn, movement, facing indicator |
| `CombatPlugin` | `combat.rs` | Sword attacks, damage resolution, knockback, invulnerability, death |
| `CollisionPlugin` | `collision.rs` | AABB overlap, split-axis movement vs `StaticBlocker` entities |
| `UiPlugin` | `ui.rs` | HUD (hearts, counters, equipped item), title/pause/game-over overlays |
| `EnemyPlugin` | `enemy.rs` | Stub — enemies are spawned by rooms but have no AI |
| `AudioPlugin` | `audio.rs` | Stub — no sound |

### Shared Modules (no plugin)

- **`constants.rs`** — All tunable values: room dimensions (256×176), HUD height (64), window scale (4×), collision unit (8), door anchors, entry offsets, Z-depth layers
- **`components.rs`** — All ECS components: `Player`, `Enemy`, `Wall`, `Door`, `StaticBlocker`, `Velocity`, `Health`, `Facing`, `Hitbox`/`Hurtbox`/`SolidBody`, `Damage`, `Knockback`, `SwordAttack`, `InvulnerabilityTimer`, `Lifetime`, `RoomEntity`, `Label`, `PickupKind`
- **`resources.rs`** — Global state: `Score`, `PlayerVitals`, `Inventory`, `EquippedItem`, `CurrentRoom`, `RoomTransitionState`, `RoomPersistence`, `RoomId` enum, `ExitDirection` enum
- **`states.rs`** — `AppState` enum: `Boot`, `Title`, `Playing`, `PausedInventory`, `GameOver`

### Data-Driven Items

Item properties (label, description, color, radius, pickup effect) are defined in `assets/data/items.ron` and loaded at startup into `ItemTable` (a `Resource`). The `PickupKind` enum still lives in Rust (adding a new kind requires recompile), but all tunable data is in the RON file.

- `items::ItemTable::lookup(kind)` → `&ItemData` (label, color, radius, effect)
- `items::apply_pickup_effect(&effect, inventory, health, vitals)` — applies the data-driven `PickupEffect`
- `PickupEffect` variants: `AddRupees(n)`, `RestoreHealth(n)`, `AddBombs(n)`, `AddKeys(n)`, `HeartContainer`

When adding a new item type: add variant to `PickupKind`, add a `PickupEffect` variant if needed, add an entry to `items.ron`.

### Label System

Attach `Label("text".into())` to any entity — the `PrimitiveRenderingPlugin` reactively spawns a `Text2d` child via `Added<Label>` detection in `PostUpdate`. Labels inherit parent transforms and despawn automatically.

### State Machine Flow

```
Boot → Title → Playing ⇄ PausedInventory
                  ↓
              GameOver → Playing (continue) or Title (quit)
```

- Gate gameplay systems with `.run_if(in_state(AppState::Playing))`
- Use `OnEnter`/`OnExit` for spawn/cleanup symmetry
- Prefer `DespawnOnExit(AppState::Playing)` on entities that should auto-despawn when leaving a state

### Inventory & HUD

- `Inventory` resource tracks `rupees`, `bombs`, `keys`, `equipped: Option<EquippedItem>`
- `PlayerVitals` tracks `current_health`/`max_health` persistently across room transitions
- HUD (UI nodes with absolute positioning) shows hearts, rupee/bomb/key counters, and equipped item
- Inventory persists through death/continue, resets on new game (Title)
- Equipped item cycles with attack key (Z) while paused

### Room System

5 overworld rooms (`RoomId` enum): Center, North, South, East, West. Each room spawns: floor, perimeter walls with door openings, obstacles (`StaticBlocker`), enemies, pickups.

- **Transitions**: triggered when player crosses room edge (6-unit padding). Locked for 0.2s to prevent re-entry. Player repositioned at inverse door entry.
- **Persistence** (`RoomPersistence`): unique pickups stay collected, secrets stay revealed, temporary pickups reset on re-entry.
- **Messages**: `LoadRoomMessage` requests load, `RoomLoadedMessage` confirms completion.
- **Typed pickups**: each pickup has a `PickupKind` component; visual properties and effects looked up from `ItemTable`.

### Combat System Ordering

Systems are grouped into `SystemSet`s executed in order:
1. `AttackSpawn` — sword entity created on attack input
2. `AttackResolve` — tick lifetimes, detect sword↔hurtbox AABB overlap
3. `Damage` — tick invulnerability, resolve enemy contact → player damage + knockback
4. `Death` — check player health → GameOver transition

Sword: lives 0.12s, despawns after one hit or timeout. Player invulnerability: 0.75s after hit. Knockback: 140 units/s, decayed via lerp.

### Collision

Split-axis AABB: velocity applied X then Y separately against all `StaticBlocker` entities, allowing wall-sliding. Player pushed to blocker edge on overlap.

### Screen Layout

- Screen center is world origin (0, 0)
- Room shifted down by HUD height: `ROOM_ORIGIN = (0, -32)`
- HUD strip at top of screen
- Perimeter walls are 16 units thick; door openings are 32 units wide
- Entry offsets 24 units inside from door anchors

### Z-Depth Layers (`render_layers` module in constants.rs)

Background (-20) → Floor (0) → Walls (10) → Entities (20) → Pickups (30) → Projectiles (40) → UI Background (90) → UI (100) → Debug (200)

### Coding Rules

- New tunable values go in `constants.rs`, not inline magic numbers
- New shared mutable game state goes in `resources.rs`
- New ECS marker/data types go in `components.rs`
- New content data goes in `assets/data/*.ron`; keep game logic in Rust, tunables in RON
- Prefer extending an existing domain plugin over registering ad hoc systems in `main.rs`
- Use `.after()` chains where frame ordering matters; for 10+ systems, group into `SystemSet`s and order at the set level

## Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`. Cast with `as u32` if constants are `f32`.
- `WindowResolution` is **not in the prelude** — import with `use bevy::window::WindowResolution;`.
- `OrthographicProjection` is **not** a standalone `Component` — it is wrapped in `Projection` (an enum). To set a custom orthographic projection on a camera, build the struct and convert: `Projection::from(OrthographicProjection { scale: 0.33, ..OrthographicProjection::default_2d() })`. Spawn it alongside `Camera2d` to override the default.
- `ScalingMode` is in `bevy::camera::ScalingMode`, not `bevy::render::camera`.
- Use `ApplyDeferred` (struct) not `apply_deferred` (no such function) for command flushing between systems.
- 2D rendering uses `Camera2d`, `Mesh2d`, `MeshMaterial2d`, `Sprite`.
- `ChildBuilder` no longer exists — replaced by `ChildSpawnerCommands` (in the prelude). The `.with_children(|parent| { ... })` pattern still works; the closure parameter is now `&mut ChildSpawnerCommands`. Nested `.with_children` calls may fail type inference — flatten children under one parent instead.
- `ColorMaterial::from_color(color)` works for creating `ColorMaterial` from a `Color`.
- `Text2d::new("text")` for world-space text, paired with `TextFont` and `TextColor`.
- Primitive 2D shapes for `Mesh2d`: `Circle::new(radius)`, `Capsule2d::new(radius, middle_length)`, `RegularPolygon::new(circumradius, sides)`, `Ellipse::new(half_w, half_h)`.

### Events / Messages

- `EventWriter<T>`, `EventReader<T>`, `App::add_event::<T>()` were **renamed** in Bevy 0.17+:
  - `MessageWriter<M>` / `MessageReader<M>` for buffered inter-system messaging
  - `App::add_message::<M>()` to register a message type
- Use `Observer` and `Trigger` for one-shot reactions to entity lifecycle or custom game events.

### Timers

Use `Timer` with `Res<Time>` for cooldowns and delays — not frame-counting. The check method is `timer.is_finished()`, **not** `timer.finished()` (`finished` is a private field).

### Bloom / HDR

- The bloom component is `Bloom`, not `BloomSettings` (renamed).
- Import: `use bevy::{core_pipeline::tonemapping::{DebandDither, Tonemapping}, post_process::bloom::Bloom};`
- `Bloom` has presets: `Bloom::NATURAL`, `Bloom::OLD_SCHOOL`, `Bloom::ANAMORPHIC`.
- `ColorMaterial` has **no** `emissive` field. Use `Color` values > 1.0 directly for glow.

### State-Scoped Entities

- `StateScoped` was renamed to `DespawnOnExit<S: States>` (and `DespawnOnEnter<S: States>`).
- Usage: `commands.spawn((MyComponent, DespawnOnExit(AppState::Playing)));`

### SubStates

- Define with `#[derive(SubStates)]` and `#[source(ParentState = ParentState::Variant)]`.
- Register: `app.init_state::<AppState>().add_sub_state::<PlayState>();`
- Sub-states only exist when the source state matches; removed automatically otherwise.
