# Lode Runner — Modernized 2D (Bevy 0.18.1)

## Overview

A modernized 2D recreation of Lode Runner (1983) built with Bevy using only primitive shapes — no sprites, textures, or audio. The game preserves all original mechanics (grid movement, hole-digging, ladder/bar traversal, enemy AI, gold collection) while adding modern visual polish through bloom, particles, screen shake, and smooth interpolated movement.

---

## 1. Core Mechanics Reference

### What Makes Lode Runner Unique
- **No jump button.** Movement is walk, climb, and fall only.
- **Hole-digging** is the primary offensive/defensive tool — dig bricks to trap enemies or create paths.
- **Gold collection** gates level completion — all gold must be collected before the escape ladder appears.
- **Enemies (guards)** pursue the player with pathfinding AI, carry/drop gold, and respawn when killed.

### Movement Rules
| Context | Allowed Actions |
|---|---|
| On solid ground (brick, concrete, enemy-in-hole) | Walk left/right, dig left/right, climb down if ladder below |
| On ladder | Move up/down/left/right |
| On bar (overhead rail) | Move left/right, release to fall |
| In air (no support) | Fall straight down until landing |
| On top of trapped enemy | Walk across as if solid ground |

### Hole-Digging Rules
- Player presses dig-left or dig-right.
- The brick **diagonally below and to that side** is targeted.
- Dig fails if: target is not a diggable brick, target position is occupied by concrete/ladder/bar, the space directly beside the player (at player level on the dig side) is a solid brick (player needs an open space to reach down).
- Digging can be performed while standing, on a ladder, or on a bar.
- Digging **cannot** be performed while falling.
- Dug hole has a lifetime timer; after it expires, the brick regenerates.
- Regeneration is **lethal to enemies** inside the hole cell. The enemy is killed and respawns at the top of the level.
- Regeneration is **also lethal to the player** — unlike some ports, this version kills the player if caught in a filling hole (adds skill ceiling).
- The player **cannot** walk through a partially-regenerated brick.

### Enemy Behavior
- Guards chase the player using grid-aware pathfinding.
- They can climb ladders, traverse bars, and fall.
- When a guard falls into a hole, it becomes **stuck** for a duration (shorter than the hole's total lifetime, so the hole fills after the guard could escape).
- A stuck guard acts as walkable ground for other entities.
- Guards carrying gold drop it when they fall into a hole. Gold sits at the top of the hole (cell above). If that cell is solid/occupied, gold reappears at the guard's original spawn position.
- When killed by hole regeneration, a guard respawns at a random position along the top row after a short delay.
- Guards **cannot** dig holes.

### Gold Collection & Level Completion
- Player walks over gold to collect it. Gold that is on the ground (uncollected) and gold carried by guards both count as **uncollected** for escape purposes.
- Guards can also pick up gold (walk over it). They carry it internally.
- When all gold is in the player's possession (none on the ground, none carried by guards), the **escape ladder** appears (hidden ladders become visible at the top of the level).
- Player must reach the top of the screen via the escape ladder to complete the level.
- If a guard is carrying gold when killed, the gold reappears at the hole location (cell above the hole).

---

## 2. Technical Architecture

### Dependencies

```toml
[dependencies]
bevy = { version = "0.18.1", features = ["dynamic_linking"] }
rand = "0.9"
```

`rand` is needed for AI tie-breaking, particle velocity randomization, and guard respawn position selection.

### Module Layout

Each domain module exposes a `Plugin` that is registered in `main.rs`.

```
src/
  main.rs          — App setup, plugin registration, state init, system set configuration
  constants.rs     — All tunable values (grid size, speeds, timers, colors, window)
  components.rs    — ECS marker + data components (Player, Guard, GridPosition, MovementState, etc.)
  resources.rs     — Shared game state (score, lives, current level, gold remaining, message queues)
  states.rs        — AppState enum, PlayState sub-state
  grid.rs          — Grid data structure, tile types, level loading, spatial queries (GridPlugin)
  player.rs        — Player movement, input handling, dig initiation (PlayerPlugin)
  enemy.rs         — Guard AI, pathfinding, stuck-in-hole state, respawn (EnemyPlugin)
  holes.rs         — Hole lifecycle (dig animation, open state, timer, regeneration, kill check) (HolePlugin)
  physics.rs       — Gravity/falling, grid-based collision resolution, movement interpolation (PhysicsPlugin)
  rendering.rs     — Primitive shape spawning, visual updates, color mapping (RenderingPlugin)
  effects.rs       — Particles, screen shake, score popups, trails (EffectsPlugin)
  levels.rs        — Level definitions (data), level loading/parsing, escape ladder logic (LevelPlugin)
  ui.rs            — HUD (score, lives, level), start screen, game over screen (UiPlugin)
  camera.rs        — Camera setup, bloom/HDR config (CameraPlugin)
```

**Plugin registration in main.rs:**
```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                resolution: WindowResolution::new(
                    WINDOW_WIDTH as u32,  // WindowResolution takes u32, not f32
                    WINDOW_HEIGHT as u32,
                ),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_sub_state::<PlayState>()
        .configure_sets(Update, (
            GameSet::Input,
            GameSet::Movement.after(GameSet::Input),
            GameSet::Physics.after(GameSet::Movement),
            GameSet::GameLogic.after(GameSet::Physics),
            GameSet::Animation.after(GameSet::GameLogic),
            GameSet::Render.after(GameSet::Animation),
        ))
        .add_plugins((
            CameraPlugin,
            GridPlugin,
            PlayerPlugin,
            EnemyPlugin,
            HolePlugin,
            PhysicsPlugin,
            RenderingPlugin,
            EffectsPlugin,
            LevelPlugin,
            UiPlugin,
        ))
        .run();
}
```

### State Machine

```
StartScreen → Playing → LevelComplete → Playing (next level)
                ↓                           ↓
           PlayerDeath              GameOver (if no levels left)
                ↓
       Playing (respawn) or GameOver (0 lives)
```

**States:**
```rust
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum AppState {
    #[default]
    StartScreen,
    Playing,
    LevelComplete,
    GameOver,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
enum PlayState {
    #[default]
    Running,
    Paused,
    Dying,       // death animation playing
}
```

**State gating rules:**
- All gameplay systems use `.run_if(in_state(PlayState::Running))` — this automatically means `AppState::Playing` is active (sub-state only exists when source state matches).
- Pause overlay systems use `.run_if(in_state(PlayState::Paused))`.
- Death animation systems use `.run_if(in_state(PlayState::Dying))`.
- Effect/particle update systems use `.run_if(in_state(AppState::Playing))` (they run in all play sub-states so particles don't freeze on pause/death).

**Level transitions:** When `AppState::Playing` exits to `LevelComplete`, all entities tagged with `DespawnOnExit(AppState::Playing)` are auto-despawned (the entire level grid, player, guards, gold, holes, particles). The `LevelComplete` state spawns its own transition UI with `DespawnOnExit(AppState::LevelComplete)`. When transitioning back to `Playing` for the next level, `OnEnter(AppState::Playing)` rebuilds the level from scratch.

### Inter-System Communication

**Bevy 0.18.1 constraint:** `EventWriter<T>`, `EventReader<T>`, and `App::add_event::<T>()` are **not available**. Use these patterns instead:

**Pattern 1 — Message queue resource** for data that multiple systems produce/consume within a frame:
```rust
#[derive(Resource, Default)]
struct GameMessages {
    gold_collected: Vec<IVec2>,       // grid positions of collected gold
    guard_killed: Vec<(IVec2, Entity)>, // hole position + guard entity
    guard_fell_in_hole: Vec<Entity>,
    player_died: bool,
    escape_triggered: bool,
}
```
Systems write to `GameMessages` fields. A drain system at the end of each frame clears the vecs. Insert ordering: producing systems run in `GameSet::GameLogic`, consuming systems run in `GameSet::Animation` or `GameSet::Render` (after producers).

**Pattern 2 — Observer/Trigger** for one-shot entity lifecycle reactions:
```rust
// When gold entity is despawned after collection, trigger a visual effect
commands.entity(gold_entity).observe(|trigger: Trigger<OnRemove>, ...| {
    // spawn particles at gold position
});
```
Use Observers for: gold pickup effects, guard death effects, player death effects. These replace boilerplate `Added<T>` / `RemovedComponents<T>` query patterns.

### Grid Representation

The grid is the single source of truth for game logic. Visual entities mirror grid state.

```rust
#[derive(Resource)]
struct LevelGrid {
    width: usize,
    height: usize,
    cells: Vec<CellType>,          // row-major, (0,0) = bottom-left
    original_cells: Vec<CellType>, // for hole regeneration: what was here before digging
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellType {
    Empty,
    Brick,            // diggable
    Concrete,         // indestructible
    Ladder,
    Bar,              // overhead rail
    HiddenLadder,     // becomes Ladder when all gold collected
    HoleDug,          // currently open hole (was Brick)
    HoleRegenerating, // filling back in
}
```

**Gold is NOT a cell type.** Gold is a separate entity placed on top of a tile (usually `Empty` or `Ladder`). The level format encodes gold positions separately from the base tile. See Section 5 for details.

**Coordinate system:** Grid `(0,0)` is at the bottom-left. World coordinates are computed as:
```rust
fn grid_to_world(grid_pos: IVec2, grid_width: usize, grid_height: usize) -> Vec2 {
    let offset_x = -(grid_width as f32 * CELL_SIZE) / 2.0 + CELL_SIZE / 2.0;
    let offset_y = -(grid_height as f32 * CELL_SIZE) / 2.0 + CELL_SIZE / 2.0;
    Vec2::new(
        grid_pos.x as f32 * CELL_SIZE + offset_x,
        grid_pos.y as f32 * CELL_SIZE + offset_y,
    )
}
```
This centers the grid on the screen with `(0,0)` at bottom-left. The camera sits at the world origin.

### Entity Model

All gameplay entities use `DespawnOnExit(AppState::Playing)` for automatic cleanup on state transitions.

| Entity | Components |
|---|---|
| Player | `Player`, `GridPosition`, `MovementState`, `FacingDirection`, `AnimationState`, `DespawnOnExit(AppState::Playing)` |
| Guard | `Guard`, `GridPosition`, `MovementState`, `FacingDirection`, `AnimationState`, `CarriedGold(Option<IVec2>)`, `AiState`, `SpawnPosition`, `DespawnOnExit(AppState::Playing)` |
| Gold | `Gold`, `GridPosition`, `OriginalPosition`, `DespawnOnExit(AppState::Playing)` |
| Hole | `Hole { timer: Timer, phase: HolePhase }`, `GridPosition`, `DespawnOnExit(AppState::Playing)` |
| Tile visuals | `TileVisual`, `GridPosition`, `DespawnOnExit(AppState::Playing)` |

**`GridPosition`** stores the entity's logical grid cell. **`MovementState`** tracks whether the entity is idle, moving between cells (with interpolation progress 0.0..1.0), falling, climbing, or hanging.

```rust
#[derive(Component)]
struct GridPosition { x: i32, y: i32 }

#[derive(Component)]
struct SpawnPosition(IVec2); // guard's original spawn point, for gold fallback

#[derive(Component)]
struct OriginalPosition(IVec2); // gold's level-defined position, for unreachable fallback

#[derive(Component)]
enum MovementState {
    Idle,
    Moving { from: IVec2, to: IVec2, progress: f32 },
    Falling { from: IVec2, progress: f32 },
    // target is always (from.x, from.y - 1) — falling is one cell straight down per step
    Climbing { from: IVec2, to: IVec2, progress: f32 },
    Digging { side: HorizontalDir, timer: Timer },
    StuckInHole { timer: Timer },
}
```

### Movement & Interpolation

Grid movement is **discrete** (tile to tile) but **visually smooth**.

1. Player presses a direction -> system checks if the target cell is passable.
2. If passable, `MovementState` transitions to `Moving { from, to, progress: 0.0 }`.
3. Each frame, `progress += speed * delta_time`. Speed is in cells/sec, so a speed of 6.0 means one cell takes ~0.167s. The entity's `Transform` is lerped between `grid_to_world(from)` and `grid_to_world(to)`.
4. When `progress >= 1.0`, `GridPosition` is set to `to`, and `MovementState` returns to `Idle`.
5. Gravity check runs after every state transition: if the entity has no support below, enter `Falling`.

**Critical rule:** An entity's `GridPosition` only updates when movement **completes**. During transition, the entity logically occupies the `from` cell for collision purposes.

**Pass-through collision:** Two entities moving toward each other (A's `to` == B's `from` AND B's `to` == A's `from`) are treated as colliding — this prevents the player and a guard from phasing through each other in a single frame.

**Falling:** Falling does not require input. Each fall segment (one cell down) is its own interpolation. The target is always `(from.x, from.y - 1)`. Fall speed is faster than walk speed. When landing (support detected below after a fall step completes), `MovementState` returns to `Idle`.

### Input Mapping

| Action | Default Key |
|---|---|
| Move left | `A` or `ArrowLeft` |
| Move right | `D` or `ArrowRight` |
| Move up (climb) | `W` or `ArrowUp` |
| Move down (climb/fall) | `S` or `ArrowDown` |
| Dig left | `Q` or `Z` |
| Dig right | `E` or `X` |
| Pause | `Escape` |
| Restart level | `R` |

---

## 3. Hole-Digging Deep Dive

This is the most complex system, so it gets its own section.

### Hole Lifecycle

```
[Player digs] -> DigAnimation (0.3s) -> HoleOpen (starts timer) -> HoleRegenerating (0.5s) -> BrickRestored
```

**Total hole lifetime:** `HOLE_OPEN_DURATION (5.0s) + HOLE_REGEN_DURATION (0.5s) = 5.5s` from when the hole opens to when the brick is fully restored. Guards escape after `GUARD_STUCK_DURATION (3.5s)`, giving them a 2.0s window before regeneration begins.

### Phases

1. **Dig animation:** The brick visually crumbles. Player is locked in `Digging` movement state. The brick cannot be interacted with during the animation. The timer is ticked via `timer.tick(time.delta())`.
2. **Hole open:** The cell becomes `HoleDug` (empty, passable). An open-duration timer starts. Entities can fall into it.
3. **Hole regenerating:** At open-timer expiry, the cell enters `HoleRegenerating`. This is a ~0.5s visual transition where the brick fills back in from the sides. **Any entity whose GridPosition matches this cell when the phase begins OR during the phase is killed.** `HoleRegenerating` cells are impassable (entities cannot enter them).
4. **Brick restored:** Cell returns to `Brick`. The `Hole` entity is despawned. The tile visual entity is restored to full opacity/scale.

### Edge Cases

| Scenario | Resolution |
|---|---|
| Dig a hole while an enemy stands on the target brick | Dig succeeds; enemy immediately begins falling into the hole. |
| Two enemies fall into the same hole | Only one can occupy the cell. The second enemy lands *on top* of the stuck enemy (the stuck enemy counts as ground). If the hole is only 1 deep, the second enemy sits at ground level and is NOT stuck. |
| Player digs two adjacent bricks | Both become independent holes with independent timers. |
| Enemy stuck in hole, carrying gold | Gold is dropped and appears at the cell above the hole. If that cell is solid/occupied, gold reappears at the guard's `SpawnPosition`. |
| Hole regenerates while gold is inside | Gold is pushed up one cell. If that cell is solid, gold returns to its `OriginalPosition`. Gold is never destroyed. |
| Player in a hole when it regenerates | Player dies. (Prevents exploiting holes as safe shelters indefinitely.) |
| Digging through the bottom row | Not possible — bottom row must be concrete. Level validation enforces this. |
| Dig target has a ladder or bar | Dig fails. Ladders and bars are not diggable. |
| Digging while on a bar | Allowed. Player digs the brick diagonally below-left or below-right from their bar position. The bar itself is not affected. |
| Rapid double-dig to the same brick | Second dig is ignored — the cell is no longer `Brick` so the dig precondition fails. |
| Guard escaping hole exactly as it regenerates | If the guard's stuck timer expires on the same frame the hole begins regenerating, the guard escapes. Process guard escape BEFORE hole phase transitions (system ordering: `EnemyMovement` before `HoleUpdate`). |

---

## 4. Enemy AI & Pathfinding

### Approach: BFS on the Movement Graph

The grid forms a **directed movement graph** — not all cells connect to all neighbors:
- Horizontal edges exist between adjacent cells that have ground support below (brick, concrete, stuck enemy, or bottom edge).
- Upward edges exist only on ladder cells (to another ladder or empty cell above).
- Downward edges exist on ladders, or into empty/air cells (falling).
- Bar cells connect horizontally to adjacent bar cells or empty cells (dropping off end of bar).
- Edges into `HoleRegenerating` cells do not exist (impassable).

Each guard runs BFS from their position to the player's position on this graph every N frames (controlled by a per-guard `Timer` with `AI_UPDATE_INTERVAL` duration). This is cheap for typical level sizes (28x16 = 448 nodes).

### AI Differentiation

To avoid all guards clumping on the same path:
- **Jitter:** Each guard's AI timer is offset by a random fraction of `AI_UPDATE_INTERVAL` at spawn time (using `rand`), so they don't all update on the same frame.
- **Tie-breaking:** When BFS finds multiple equal-length paths, each guard uses a seeded `rand` value to break ties differently.
- **Reluctance zones:** Guards that recently escaped a hole have a temporary bias *away* from holes — BFS treats hole-adjacent cells as having +2 cost for 3 seconds after escaping.

### Guard States

```
Patrolling -> Chasing -> Falling -> StuckInHole -> Escaping -> Chasing
                                        |
                                    Killed -> Respawning -> Chasing
```

- **Chasing:** Following BFS path toward player.
- **Falling:** No support, falling down. Not controllable.
- **StuckInHole:** Timer counting down via `timer.tick(time.delta())`. Entity is immobile and acts as ground.
- **Escaping:** Stuck timer expired, guard moves up one cell (standard movement interpolation).
- **Killed:** Hole regenerated on top of guard. Entity is hidden (set `Visibility::Hidden`). A respawn timer starts. After `GUARD_RESPAWN_DELAY`, the entity is moved to a random empty cell along the top row (using `rand`) and made visible again.
- **Patrolling:** Fallback when BFS can't find a path to the player. Guard paces back and forth.

**Guard-guard collision:** A guard cannot move into a cell occupied by another guard's `GridPosition`, except on bar cells (where overlap is allowed, matching the original). If the only BFS path goes through another guard, the guard waits in place until the path clears (re-evaluated on next AI tick).

### Pathfinding Edge Cases

| Scenario | Resolution |
|---|---|
| Player is in a walled-off section | BFS returns no path. Guard enters Patrolling. |
| Multiple guards choose the same narrow path | They queue up. Leading guard moves; trailing guard waits. |
| Guard on a bar above a newly dug hole | Guard continues on bar (bar provides support). Only falls if guard leaves the bar into unsupported space. |
| Guard reaches a ledge with no floor | Falls. AI doesn't predict falls — it follows the graph, and gravity handles the rest. |

---

## 5. Level Data Format

### Text-Based Level Encoding

Levels are defined as ASCII grids stored in a Rust module (no external file I/O needed).

**Gold is an overlay, not a tile.** Gold can sit on top of `Empty` or `Ladder` cells. The level format uses `$` to mean "gold on empty" and encodes gold-on-ladder as a separate pass. For simplicity, the primary format convention:

```
Legend:
  '.' = Empty
  '#' = Brick (diggable)
  '=' = Concrete (indestructible)
  'H' = Ladder
  '-' = Bar (overhead rail)
  '$' = Gold (on empty tile — the underlying cell is Empty)
  'P' = Player spawn (on empty tile)
  'G' = Guard spawn (on empty tile)
  '^' = Hidden ladder (escape route, revealed when all gold collected)
  'S' = Gold on ladder (both a Ladder cell AND a gold entity)
```

The parser produces:
- `CellType` grid (with `$` -> `Empty`, `P` -> `Empty`, `G` -> `Empty`, `S` -> `Ladder`)
- Separate `Vec<IVec2>` for gold positions (from `$` and `S`)
- `player_spawn: IVec2` (from `P`)
- `Vec<IVec2>` for guard spawns (from `G`)

Example 28x16 level:
```
............................
....-----.....-----.......^^
....H....#####....H.......^H
....$....#....$...H..$$..^H
====H====######===H======^H
....H.........G...H......^H
....H...------....H......^H
..$.H...H....#####H......^H
====H===H====######======^H
........H.........G......^H
...------H...####........^H
...H.....H..#...$........^H
===H=====H==######===H===^H
...H.....H..........H....^H
...H..P..H....G.$..GH....^H
============================
```

All rows are exactly 28 characters wide. The bottom row is all concrete (`=`). The rightmost column contains hidden ladders (`^`) as the escape route.

### Level Storage

A `const` array of level strings parsed at `OnEnter(AppState::Playing)`:

```rust
struct LevelData {
    width: usize,
    height: usize,
    grid: Vec<CellType>,
    player_spawn: IVec2,
    guard_spawns: Vec<IVec2>,
    gold_positions: Vec<IVec2>,
    total_gold: usize,
}
```

### Level Design Constraints (Enforced at Parse Time)
- Exactly 1 player spawn (`P`) per level.
- Bottom row must be entirely concrete (no falling off the bottom).
- At least 1 gold piece.
- At least 1 hidden ladder column for escape.
- Player spawn must not be inside a solid tile.
- All rows must be the same width.
- Guard spawns must be on empty tiles with ground support.

---

## 6. Rendering (Primitive Shapes Only)

### Shared Mesh & Material Handles

Creating duplicate `Mesh` and `ColorMaterial` assets for every tile is wasteful. Store shared handles in a resource initialized at startup:

```rust
#[derive(Resource)]
struct RenderAssets {
    brick_mesh: Handle<Mesh>,
    brick_material: Handle<ColorMaterial>,
    concrete_mesh: Handle<Mesh>,
    concrete_material: Handle<ColorMaterial>,
    gold_mesh: Handle<Mesh>,
    gold_material: Handle<ColorMaterial>,
    bar_mesh: Handle<Mesh>,
    bar_material: Handle<ColorMaterial>,
    // ... per tile type + player/guard body parts + particle
    particle_mesh: Handle<Mesh>,
    // Materials for particles are per-instance (varying alpha), but mesh is shared
}
```

Initialize by adding meshes and materials to `ResMut<Assets<Mesh>>` and `ResMut<Assets<ColorMaterial>>`:
```rust
let brick_mesh = meshes.add(Rectangle::new(CELL_SIZE - 2.0, CELL_SIZE - 2.0));
let brick_material = materials.add(ColorMaterial::from_color(Color::srgb(0.7, 0.35, 0.1)));
```

### Tile Visuals

Each tile type maps to a distinct `Mesh2d` + `MeshMaterial2d<ColorMaterial>` combination:

| Tile | Shape | Color | Notes |
|---|---|---|---|
| Brick | `Rectangle` | Warm brown (`srgb(0.7, 0.35, 0.1)`) | Two overlapping rects — darker larger one behind for border effect |
| Concrete | `Rectangle` | Gray (`srgb(0.5, 0.5, 0.55)`) | Cross-hatch detail via two thin overlapping rects at 45deg |
| Ladder | Two vertical thin rects + horizontal rungs | Yellow-green (`srgb(0.6, 0.8, 0.2)`) | Rungs are small horizontal rects spaced evenly |
| Bar | Thin horizontal rect | Cyan (`srgb(0.3, 0.8, 0.9)`) | Positioned at top of cell |
| Gold | `RegularPolygon::new(radius, 6)` | Bright HDR gold (`srgb(5.0, 3.5, 0.5)`) | HDR color triggers bloom; gentle scale pulse animation |
| Hidden Ladder | Invisible until triggered | Same as Ladder when revealed | Fade-in via alpha interpolation on reveal |
| Empty | Nothing rendered | — | |
| Dug Hole | Brick visual with animated scale | Brown fading to transparent | Scale shrinks to 0 during dig, grows back during regen |

**Spawning a tile entity:**
```rust
commands.spawn((
    Mesh2d(render_assets.brick_mesh.clone()),
    MeshMaterial2d(render_assets.brick_material.clone()),
    Transform::from_translation(grid_to_world(pos, width, height).extend(Z_BRICKS)),
    TileVisual,
    GridPosition { x: pos.x, y: pos.y },
    DespawnOnExit(AppState::Playing),
));
```

### Player & Guard Visuals

The player and guards are composed of multiple child primitives forming a simple humanoid:
- **Head:** `Circle::new(head_radius)`
- **Body:** `Rectangle`
- **Legs:** Two thin `Rectangle`s
- **Arms:** Two thin `Rectangle`s

**Important (Bevy 0.18.1):** `ChildBuilder` is not in the prelude. Do NOT use nested `.with_children()` calls — they can fail type inference. Instead, flatten all children under one parent:

```rust
// Spawn parent entity
let player_entity = commands.spawn((
    Player,
    GridPosition { x: spawn.x, y: spawn.y },
    MovementState::Idle,
    FacingDirection::Right,
    AnimationState::default(),
    Transform::from_translation(grid_to_world(spawn, w, h).extend(Z_PLAYER)),
    Visibility::Inherited,
    DespawnOnExit(AppState::Playing),
)).id();

// Spawn all body parts as flat children of the parent
commands.entity(player_entity).with_children(|parent| {
    // Head
    parent.spawn((
        Mesh2d(render_assets.head_mesh.clone()),
        MeshMaterial2d(render_assets.player_material.clone()),
        Transform::from_xyz(0.0, BODY_HEIGHT / 2.0 + HEAD_RADIUS, 0.0),
        BodyPart::Head,
    ));
    // Body
    parent.spawn((
        Mesh2d(render_assets.body_mesh.clone()),
        MeshMaterial2d(render_assets.player_material.clone()),
        Transform::default(),
        BodyPart::Body,
    ));
    // Left leg, right leg, left arm, right arm — all flat under this one parent
    // ...
});
```

**Player color:** Bright green HDR (`srgb(0.2, 4.0, 0.4)`) — bloom makes them always visible.
**Guard color:** Red/crimson HDR (`srgb(4.0, 0.3, 0.3)`) — visually distinct from player.
**Guard carrying gold:** A small gold `Circle` child entity is made visible on the guard's body.

### Animation System

Animations are purely programmatic — no sprite sheets.

```rust
#[derive(Component, Default)]
struct AnimationState {
    current: AnimationType,
    frame_timer: Timer,
    frame_index: usize,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AnimationType {
    #[default]
    Idle,
    WalkLeft,
    WalkRight,
    ClimbUp,
    ClimbDown,
    HangBar,
    DigLeft,
    DigRight,
    Falling,
    Stuck,      // guard stuck in hole
    Death,
}
```

Each animation type defines how child `BodyPart` transforms are updated per frame:
- **Walking:** Legs alternate spread via rotation; arms swing slightly.
- **Climbing:** Arms reach up alternately; legs step.
- **Digging:** One arm extends to the side and swings downward.
- **Falling:** Arms rotated up, legs together.
- **Death:** Not animated per-frame — body parts are detached and converted to particles (see Effects).

The animation system queries `(Entity, &AnimationState, &Children)` on the parent, then for each child queries `(&BodyPart, &mut Transform)` to apply positional/rotational offsets.

---

## 7. Visual Effects

### Bloom / HDR Setup

```rust
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
};

// Camera setup
commands.spawn((
    Camera2d,
    Camera {
        clear_color: ClearColorConfig::Custom(Color::BLACK),
        ..default()
    },
    Tonemapping::TonyMcMapface,
    Bloom::OLD_SCHOOL,
    DebandDither::Enabled,
    DespawnOnExit(AppState::Playing),
));
```

Gold, player, and guards use `Color::srgb()` values > 1.0 to trigger bloom. `ColorMaterial` has **no** `emissive` field — the bloom post-process extracts bright regions above its threshold directly from the color value.

### Particle System (Lightweight Custom)

No external particle crate — a simple custom system using a shared mesh handle:

```rust
#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime: Timer,
    drag: f32,
}
```

Particles reuse the shared `render_assets.particle_mesh` (a small `Circle::new(2.0)`). Each particle gets its own `MeshMaterial2d<ColorMaterial>` for individual alpha/color (unavoidable for per-particle fade). Keep particle count bounded — a `particle_count: usize` field in a resource, checked before spawning. Max ~200 alive at once.

Each frame, the particle system:
1. Ticks `lifetime` via `particle.lifetime.tick(time.delta())`.
2. Applies `velocity * time.delta()` to `Transform.translation`.
3. Applies drag: `velocity *= 1.0 - drag * time.delta()`.
4. Fades alpha based on lifetime remaining (requires mutating the `ColorMaterial` asset).
5. Despawns particles where `lifetime.finished()`.

**Particle triggers:**
| Event | Effect |
|---|---|
| Brick dug | 8-12 brown particles burst from dig location |
| Gold collected | 6-10 gold sparkle particles + score popup |
| Guard killed (hole fill) | 12-16 red particles |
| Player death | 20+ green particles (body "shatters") |
| Level complete | Continuous gold particle shower from top |
| Hole regenerating | Particles slowly drift inward toward hole center |

### Screen Shake

```rust
#[derive(Resource, Default)]
struct ScreenShake {
    timer: Timer,
    intensity: f32,
}
```

When triggered, set `timer` and `intensity`. Each frame (while timer is active), offset the camera's `Transform.translation` by a random `Vec2` within the intensity radius (using `rand`), with intensity decaying linearly as the timer progresses. Reset the camera offset to zero when the timer finishes.

Triggers: player death (strong), guard killed (mild), level complete (medium).

### Score Popups

When gold is collected, spawn a world-space text entity:
```rust
commands.spawn((
    Text2d::new(format!("+{}", SCORE_GOLD)),
    TextFont { font_size: 20.0, ..default() },
    TextColor(Color::srgb(5.0, 3.5, 0.5)),  // HDR gold for bloom glow
    Transform::from_translation(world_pos.extend(Z_PARTICLES)),
    ScorePopup { velocity: Vec2::new(0.0, SCORE_POPUP_RISE_SPEED), lifetime: Timer::from_seconds(SCORE_POPUP_DURATION, TimerMode::Once) },
    DespawnOnExit(AppState::Playing),
));
```

The popup floats upward and fades out. Since `TextColor` doesn't support alpha animation natively, track alpha in the `ScorePopup` component and update `TextColor` each frame.

### Trail Effect (Optional)

When the player moves, spawn a faint copy of the player's visual at the previous position. Implemented as a ring buffer of 3-4 trail entities with decreasing alpha. Each trail entity is a simple rectangle (not the full humanoid) for performance.

---

## 8. Camera

### Fixed Camera (Default)

For standard 28x16 levels, the entire grid fits on screen. The camera is fixed at the world origin, centered on the grid.

Window size: `WINDOW_WIDTH x WINDOW_HEIGHT` (1120x640). Note: `WindowResolution::new()` takes `u32`, so constants must be cast: `WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)`.

The HUD is rendered as Bevy UI nodes (separate from world-space rendering, not affected by camera position or bloom).

### Scrolling Camera (Future Extension)

If a level exceeds the viewport, the camera tracks the player with smooth damping:
```rust
let target = grid_to_world(player_grid_pos, width, height);
let current = camera_transform.translation.truncate();
let new_pos = current.lerp(target, 5.0 * time.delta_secs());
let clamped = new_pos.clamp(min_bound, max_bound);
camera_transform.translation = clamped.extend(camera_transform.translation.z);
```

Not needed for initial levels — all 28x16 levels fit in the fixed camera.

---

## 9. UI / UX

### HUD (During Play)

Rendered as Bevy UI nodes, not world-space entities. Uses `DespawnOnExit(AppState::Playing)`.

```
+--------------------------------------------+
|  SCORE: 12500   LIVES: ooo   LEVEL: 03    |
|                                            |
|              [GAME GRID]                   |
|                                            |
|                                            |
|                            GOLD: 5/12      |
+--------------------------------------------+
```

- **Score:** Top-left. Increases on gold collection and guard kills.
- **Lives:** Top-center. Shown as small green circles (matching player head color).
- **Level:** Top-right. Current level number.
- **Gold counter:** Bottom-right. "Collected / Total" — helps the player know when escape is close.

HUD text uses `Text` + `TextFont` + `TextColor` inside UI `Node`s. Update text reactively using `Changed<GameState>` query filter on the `GameState` resource (avoids updating every frame).

### Start Screen

- Title "LODE RUNNER" in large `Text` with bright HDR `TextColor` for bloom glow.
- "PRESS ENTER TO START" — blinking via a `Timer` that toggles `Visibility`.
- Background: a decorative grid of bricks/gold rendered as world-space entities with `DespawnOnExit(AppState::StartScreen)`.
- Level select (optional): display unlocked levels 1-N, arrow keys to choose.

### Game Over Screen

- "GAME OVER" with heavy bloom (`TextColor` HDR red).
- Final score displayed.
- "PRESS ENTER TO RESTART" or "PRESS ESCAPE FOR MENU".
- Uses `DespawnOnExit(AppState::GameOver)`.

### Level Complete Transition

1. `AppState` transitions to `LevelComplete`. All `Playing` entities auto-despawn via `DespawnOnExit`.
2. "LEVEL COMPLETE" text fades in with `DespawnOnExit(AppState::LevelComplete)`.
3. Gold particle shower effect (spawned fresh in `LevelComplete` state).
4. After `LEVEL_COMPLETE_PAUSE` (2s), increment current level in `GameState` resource and transition to `AppState::Playing`. `OnEnter(AppState::Playing)` rebuilds the next level.
5. If no more levels, transition to `AppState::GameOver` instead.

### Pause Screen

- `PlayState` transitions to `Paused`.
- All gameplay systems stop (they're gated with `run_if(in_state(PlayState::Running))`).
- Spawn a semi-transparent full-screen `Node` with `BackgroundColor` and "PAUSED" text.
- Uses `DespawnOnExit(PlayState::Paused)` — Bevy auto-despawns the overlay when unpausing.
- Resume with Escape.

### Death Sequence

1. `PlayState` transitions to `Dying`.
2. Player body parts are detached and converted to particles (scatter outward, ~1s).
3. Screen shake (strong).
4. Brief pause (0.5s timer in a `Dying`-state system).
5. If lives > 0: decrement lives in `GameState`, restore `LevelGrid` holes to bricks (using `original_cells`), reset guard positions to `SpawnPosition`, respawn player at spawn point, transition to `PlayState::Running`.
6. If lives == 0: transition to `AppState::GameOver`.

**Important:** Death does NOT reset collected gold. The player keeps gold progress across deaths within a level. Only guard positions and hole states reset.

---

## 10. Scoring

| Event | Points |
|---|---|
| Gold collected | 250 |
| Guard killed (hole regen) | 500 |
| Level completed | 1000 + (remaining_lives * 100) |

Stored in a `GameState` resource:
```rust
#[derive(Resource)]
struct GameState {
    score: u32,
    lives: u32,
    current_level: usize,
    gold_collected: u32,
    gold_total: u32,
    all_gold_collected: bool, // triggers hidden ladder reveal
}
```

---

## 11. System Ordering

Execution order matters for deterministic behavior. All gameplay systems are gated with `.run_if(in_state(PlayState::Running))` unless noted otherwise.

### System Sets

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet {
    Input,
    Movement,
    Physics,
    GameLogic,
    Animation,
    Render,
}
```

**Configuration in main.rs:**
```rust
app.configure_sets(Update, (
    GameSet::Input,
    GameSet::Movement.after(GameSet::Input),
    GameSet::Physics.after(GameSet::Movement),
    GameSet::GameLogic.after(GameSet::Physics),
    GameSet::Animation.after(GameSet::GameLogic),
    GameSet::Render.after(GameSet::Animation),
));
```

### Systems by Set

**`GameSet::Input`** — `.run_if(in_state(PlayState::Running))`
1. `read_player_input` — read keyboard, buffer intended action

**`GameSet::Movement`** — `.run_if(in_state(PlayState::Running))`
2. `update_player_movement` — process player movement/dig initiation
3. `update_enemy_ai` — run BFS on timer tick, decide guard actions
4. `update_enemy_movement` — process guard movement, escape from holes

**`GameSet::Physics`** — `.run_if(in_state(PlayState::Running))`
5. `apply_gravity` — check all entities for unsupported positions, start falling
6. `update_hole_timers` — tick hole timers, handle phase transitions, kill entities in regenerating holes

**`GameSet::GameLogic`** — `.run_if(in_state(PlayState::Running))`
7. `check_collisions` — player-gold pickup, player-guard death, guard-gold pickup, pass-through detection
8. `check_gold_completion` — if all gold collected, reveal hidden ladders (set `all_gold_collected` in `GameState`, update `LevelGrid` cells from `HiddenLadder` to `Ladder`)
9. `check_escape` — if `all_gold_collected` and player reaches top row, trigger level complete
10. `drain_messages` — clear `GameMessages` vecs for next frame

**`GameSet::Animation`** — `.run_if(in_state(AppState::Playing))` (runs in all play sub-states)
11. `update_animations` — update `AnimationState` and child `BodyPart` transforms
12. `update_tile_visuals` — sync brick visuals for holes (scale/alpha animation)

**`GameSet::Render`** — `.run_if(in_state(AppState::Playing))` (runs in all play sub-states)
13. `sync_grid_to_transform` — sync `GridPosition` + `MovementState` interpolation -> `Transform` for all entities
14. `update_effects` — update particles, screen shake, score popups
15. `update_hud` — update HUD text from `GameState` (use `Changed<GameState>` filter to avoid per-frame updates)

### Command Flushing

Systems in `GameSet::GameLogic` may despawn entities (guard killed, gold collected). Systems in `GameSet::Animation` read those entities. Bevy applies commands at set boundaries by default in 0.18.1. If commands are not visible in the next set, insert an explicit `ApplyDeferred` schedule node:

```rust
app.add_systems(Update, ApplyDeferred.after(GameSet::GameLogic).before(GameSet::Animation));
```

---

## 12. Edge Cases & Concerns

### Gameplay Edge Cases

1. **Digging beneath yourself:** Player stands on a brick, digs left-down. The brick below-left is removed. Player does NOT fall (they're still standing on their current brick). But if the player then walks left, they fall into the hole they dug.

2. **Chain digging escape routes:** Player digs a series of holes to create a temporary path through brick, then the holes fill behind them. This is intentional and a core strategy. The timer must be tuned so this is tight but achievable.

3. **Guard escaping hole exactly as it regenerates:** System ordering resolves this: `update_enemy_movement` (escape logic) runs in `GameSet::Movement`, before `update_hole_timers` in `GameSet::Physics`. If both timers expire on the same frame, the guard escapes first.

4. **Two guards, one hole:** Second guard lands on the stuck guard's head (it's solid). The second guard is NOT stuck — it's standing on "ground." When the first guard escapes or dies, the second guard then falls in (if the hole is still open). Gravity system detects the lost support.

5. **Gold in a sealed area:** If a guard carries gold into an area that becomes sealed (e.g., holes fill around it), the gold may become unreachable. **Resolution:** When a guard dies, run a quick BFS reachability check from the player's spawn position to the gold's drop location. If unreachable, teleport the gold to the guard's `SpawnPosition`. If that's also unreachable, fall back to the gold's `OriginalPosition`.

6. **Player and guard moving through each other:** Detected by the pass-through rule in `check_collisions`: if player's `(from, to)` swaps with a guard's `(from, to)`, treat it as a collision. Player dies.

7. **Digging the brick you're standing next to, not below:** The dig target is always the brick **diagonally below** — one cell to the side AND one cell down from the player. The player does NOT dig the cell directly beside them at their own level.

8. **Rapid input during movement interpolation:** Buffer exactly one input. When the current move completes, the buffered input executes immediately. New inputs overwrite the buffer (only the most recent pending input is kept).

9. **Frame-rate independence:** All timers and movement speeds use `Res<Time>` deltas via `time.delta()` and `time.delta_secs()`, never frame counting. Movement is discrete (one grid cell per step), so variable frame rate cannot cause cell-skipping — it only affects how smooth the visual interpolation looks.

10. **Guard on a ladder when floor below is dug:** The guard is on a ladder cell, which provides its own support — the guard does NOT fall. Only when the guard steps off the ladder onto the now-missing floor does gravity apply.

### Technical Concerns

1. **Entity count:** A 28x16 grid = 448 tiles. Many are empty (no entity). Typical level: ~200 tile entities, ~30 body part entities (player + 4 guards), ~20 gold entities, up to ~200 particles. Peak ~450 entities. Well within Bevy's capabilities.

2. **Pathfinding cost:** BFS on 448 nodes is negligible. Running it for 5 guards ~6 times/second = 30 BFS/second. Not a concern.

3. **Mesh/material asset count:** Shared handles in `RenderAssets` keep unique mesh count low (~15 shapes). Material count is higher due to per-particle fade materials — mitigate by capping particles at 200 and reusing despawned particle material handles via a pool (or accept the minor churn for simplicity).

4. **Hole visual consistency:** Don't despawn the brick entity when digging — instead, animate its `Transform.scale` to zero during dig and back to one during regeneration. This avoids spawn/despawn complexity and keeps the entity available for the regeneration animation.

5. **Grid-visual sync:** The `LevelGrid` resource must always be updated BEFORE rendering systems read from it. The system set ordering (GameLogic before Render) enforces this. The `ApplyDeferred` between GameLogic and Animation ensures despawned entities aren't queried.

6. **Z-ordering:** Explicit Z values in `Transform.translation.z`:
   ```rust
   const Z_BACKGROUND: f32 = 0.0;
   const Z_BRICKS: f32 = 1.0;
   const Z_LADDERS: f32 = 2.0;
   const Z_GOLD: f32 = 3.0;
   const Z_GUARDS: f32 = 4.0;
   const Z_PLAYER: f32 = 5.0;
   const Z_PARTICLES: f32 = 6.0;
   ```
   Bevy UI is rendered in a separate pass and is not affected by world-space Z.

7. **State transition timing:** `DespawnOnExit(AppState::Playing)` despawns ALL level entities when exiting `Playing`. This is correct for `LevelComplete` transitions (we rebuild the next level from scratch). For death/respawn, we stay in `AppState::Playing` (only `PlayState` changes to `Dying` then back to `Running`), so entities persist.

### Design Tradeoffs

1. **Grid-based collision vs. continuous physics:** Grid-based is correct for this game. Continuous physics would add complexity with no benefit — Lode Runner's design assumes discrete tile occupancy. **Chosen: grid-based.**

2. **One entity per tile vs. tilemap batching:** Individual entities per tile is simpler and sufficient for ~200 non-empty tiles. A custom tilemap renderer would be more performant but adds significant complexity. **Chosen: individual entities.** Could optimize later if needed.

3. **AI update frequency:** Every frame is wasteful; every 30 frames is sluggish. ~6 times/second is responsive without waste. Each guard on a different timer offset to spread the cost. **Chosen: timer-based with jitter.**

4. **Hole regeneration kill — include player?** Classic Lode Runner does NOT kill the player in holes. Including it adds difficulty and a meaningful skill ceiling (no parking in holes). **Chosen: kill the player.** This is the "modernized" take. Can be made a difficulty toggle later.

5. **Level data as code vs. external files:** Embedding levels as `const &str` in Rust means recompiling to add levels but avoids file I/O, path issues, and parsing errors. For 5-10 levels, this is fine. **Chosen: levels in code.**

6. **Input buffering depth:** No buffer = dropped inputs at speed. 1-deep buffer = responsive without confusion. Deep buffer = delayed unpredictable actions. **Chosen: 1-deep buffer** (most recent input overwrites).

7. **Per-particle material handles vs. shared:** Fading particles need individual alpha values, requiring separate `ColorMaterial` assets. A pooling system adds complexity. For a 200-particle cap the churn is acceptable. **Chosen: individual materials, no pool, capped count.**

---

## 13. Constants (Initial Values)

```rust
// Window
const WINDOW_TITLE: &str = "Lode Runner";
const CELL_SIZE: f32 = 40.0;
const GRID_WIDTH: usize = 28;
const GRID_HEIGHT: usize = 16;
const WINDOW_WIDTH: f32 = CELL_SIZE * GRID_WIDTH as f32;   // 1120.0
const WINDOW_HEIGHT: f32 = CELL_SIZE * GRID_HEIGHT as f32;  // 640.0
// Note: WindowResolution::new() takes u32 — cast with `as u32` at the call site.

// Speeds (cells per second — progress += speed * delta_secs per frame)
const PLAYER_MOVE_SPEED: f32 = 6.0;   // one cell in ~0.167s
const PLAYER_CLIMB_SPEED: f32 = 4.0;  // one cell in 0.25s
const PLAYER_FALL_SPEED: f32 = 10.0;  // one cell in 0.1s
const GUARD_MOVE_SPEED: f32 = 4.5;    // slightly slower than player
const GUARD_CLIMB_SPEED: f32 = 3.5;
const GUARD_FALL_SPEED: f32 = 10.0;

// Timers (seconds)
const DIG_DURATION: f32 = 0.3;
const HOLE_OPEN_DURATION: f32 = 5.0;
const HOLE_REGEN_DURATION: f32 = 0.5;
// Total hole lifetime: HOLE_OPEN_DURATION + HOLE_REGEN_DURATION = 5.5s
const GUARD_STUCK_DURATION: f32 = 3.5;    // must be < HOLE_OPEN_DURATION (guard escapes 1.5s before regen)
const GUARD_RESPAWN_DELAY: f32 = 2.0;
const DEATH_ANIMATION_DURATION: f32 = 1.0;
const LEVEL_COMPLETE_PAUSE: f32 = 2.0;

// Scoring
const SCORE_GOLD: u32 = 250;
const SCORE_GUARD_KILL: u32 = 500;
const SCORE_LEVEL_COMPLETE: u32 = 1000;
const SCORE_LIFE_BONUS: u32 = 100;

// Gameplay
const STARTING_LIVES: u32 = 5;
const AI_UPDATE_INTERVAL: f32 = 1.0 / 6.0; // ~6 times per second per guard

// Z-layers
const Z_BACKGROUND: f32 = 0.0;
const Z_BRICKS: f32 = 1.0;
const Z_LADDERS: f32 = 2.0;
const Z_GOLD: f32 = 3.0;
const Z_GUARDS: f32 = 4.0;
const Z_PLAYER: f32 = 5.0;
const Z_PARTICLES: f32 = 6.0;

// Effects
const SCREEN_SHAKE_DEATH_INTENSITY: f32 = 8.0;
const SCREEN_SHAKE_DEATH_DURATION: f32 = 0.4;
const SCREEN_SHAKE_KILL_INTENSITY: f32 = 3.0;
const SCREEN_SHAKE_KILL_DURATION: f32 = 0.2;
const MAX_PARTICLES: usize = 200;
const SCORE_POPUP_DURATION: f32 = 0.8;
const SCORE_POPUP_RISE_SPEED: f32 = 60.0;

// Body part dimensions (fractions of CELL_SIZE)
const HEAD_RADIUS: f32 = CELL_SIZE * 0.15;
const BODY_WIDTH: f32 = CELL_SIZE * 0.25;
const BODY_HEIGHT: f32 = CELL_SIZE * 0.3;
const LIMB_WIDTH: f32 = CELL_SIZE * 0.06;
const LIMB_LENGTH: f32 = CELL_SIZE * 0.2;
```

---

## 14. Implementation Order

A suggested build sequence, each phase producing a runnable game:

### Phase 1 — Static Grid Rendering
- `constants.rs`, `states.rs`, `grid.rs`, `rendering.rs`, `camera.rs`
- Define `AppState`, `PlayState`, register states
- Parse a hardcoded level string into `LevelGrid`
- Create `RenderAssets` resource with shared mesh/material handles
- Render all tiles as `Mesh2d` + `MeshMaterial2d` with correct colors and Z-layers
- Camera with bloom/HDR setup
- **Milestone:** A static, visually appealing level displayed on screen with bloom.

### Phase 2 — Player Movement
- `components.rs`, `resources.rs`, `player.rs`, `physics.rs`
- Player entity spawn with humanoid body parts (flattened children)
- Grid-based movement with smooth interpolation (`MovementState`)
- Gravity system (falling when unsupported)
- Ladder climbing, bar traversal
- Input handling with 1-deep buffer
- System set configuration with `run_if` state gating
- `grid_to_world` coordinate conversion for `Transform` sync
- **Milestone:** Player can navigate a level (walk, climb, hang, fall).

### Phase 3 — Hole Digging
- `holes.rs`
- Dig precondition checks (target is brick, side is clear, not falling)
- Dig animation via `MovementState::Digging` timer
- Hole lifecycle: open -> timer -> regenerating -> restored
- Brick visual animation (scale to zero, scale back)
- Player death if caught in regenerating hole
- Dig particle burst (first use of particle system)
- **Milestone:** Player can dig holes, watch them fill, and die if caught.

### Phase 4 — Gold & Level Completion
- `levels.rs`, gold collection logic in `GameSet::GameLogic`
- Gold entities with HDR bloom glow and scale pulse animation
- Gold counter in HUD (`ui.rs` started)
- `GameState` resource tracking collected/total gold
- `GameMessages` resource for inter-system communication
- Hidden ladder reveal when `all_gold_collected` flips
- Escape detection (player reaches top row while hidden ladders are revealed)
- Score popups via `Text2d`
- Gold collect particles
- **Milestone:** A completable level with gold, score, and escape.

### Phase 5 — Enemy AI
- `enemy.rs`
- Guard entity spawning with humanoid body parts
- BFS pathfinding on movement graph (rebuilt when `LevelGrid` changes)
- Guard movement (walk, climb, bar, fall) — same interpolation as player
- Guard falling into holes, `StuckInHole` state with timer, escape
- Guard-player collision (player death) including pass-through detection
- Guard picking up and dropping gold
- Guard death on hole regeneration, respawn after delay
- AI differentiation (timer jitter, random tie-breaking, reluctance zones)
- **Milestone:** Fully playable single level with enemies.

### Phase 6 — Game Loop & UI
- `ui.rs` completion, state machine wiring
- Start screen with title bloom and key prompt
- Game over screen with final score
- Level complete transition (auto-despawn via `DespawnOnExit`, particle shower, next level load)
- Lives system, death sequence (`PlayState::Dying` -> respawn or game over)
- Pause functionality (`PlayState::Paused` with overlay)
- Multiple levels with progression (`GameState.current_level`)
- HUD polish (reactive updates via `Changed<GameState>`)
- **Milestone:** Full game loop from start to game over.

### Phase 7 — Polish & Effects
- `effects.rs` completion
- Screen shake (death, guard kill, level complete)
- Death shatter animation (body parts become particles)
- Level complete gold particle shower
- Player trail effect (optional)
- Visual tuning (colors, bloom intensity, particle counts, animation timing)
- Additional hand-crafted levels (target: 5 total)
- Balance pass on speeds, timers, scoring
- **Milestone:** Polished, shippable game.

---

## 15. Level Designs (Initial 5)

All levels are 28 wide x 16 tall. Bottom row is concrete. Hidden ladders on the right edge for escape.

### Level 1 — Tutorial
- Simple flat terrain, few bricks, 3 gold, 0 guards.
- Open layout with one ladder and one bar.
- Teaches: movement, gold collection, climbing, escape ladder.

### Level 2 — First Dig
- Brick barriers blocking gold. 5 gold, 1 guard.
- Gold placed behind single-brick walls that must be dug through.
- Teaches: hole digging to reach gold, trapping a single enemy.

### Level 3 — Vertical Challenge
- Multi-level layout with many ladders and bars. 8 gold, 2 guards.
- Gold spread across 3-4 vertical tiers.
- Teaches: vertical navigation, bar traversal, managing multiple guard paths.

### Level 4 — Trap Gauntlet
- Dense brick layout requiring chain digs. 10 gold, 3 guards.
- Sections that can only be reached by digging 2-3 holes in sequence before they fill.
- Teaches: timing multiple holes, using holes offensively against guards.

### Level 5 — Full Challenge
- Complex layout with all tile types. 12 gold, 4 guards.
- Multiple viable paths, some more risky than others.
- Requires both offensive digging (trapping guards) and utility digging (creating paths).
- Tests: all mechanics working together under pressure.
