# Joust - Design And Implementation Spec (Bevy 0.18.1)

## 1. Purpose

This document describes the target game design and implementation approach for a modernized 2D recreation of Williams Electronics' Joust (1982) using Bevy 0.18.1.

Core creative constraint:

- Gameplay actors are rendered from primitive meshes and Bevy text.
- No sprite or texture art is required for the core loop.
- Existing assets in `assets/` may still be used later for audio, UI polish, or experiments, but they are not required for v1 gameplay.

Current repo status:

- The actual project is still a minimal Bevy starter.
- `src/main.rs` currently spawns a `Camera2d` and centered `Hello, World!` UI text.
- This spec describes the intended destination, not the current implementation.

Project-level constraints:

- Rust edition `2024`
- Bevy `0.18.1`
- `bevy` uses the `dynamic_linking` feature
- Validate most code changes with `cargo check`
- Use `cargo clippy` when API usage or architecture changes broadly

## 2. Bevy Guidance And Best Practices

This section is the main correction to the previous draft. The original spec had good gameplay ideas, but some of the Bevy advice was too rigid or mixed together patterns that should stay separate.

### 2.1 Grow The Codebase In Stages

Do not scaffold a large module tree on day one.

Preferred progression:

1. Keep the first playable loop in `src/main.rs`.
2. Extract `constants.rs`, `components.rs`, `resources.rs`, and `states.rs` once they become crowded or reused.
3. Split domain plugins such as `player.rs`, `enemy.rs`, `combat.rs`, and `ui.rs` only after each domain has multiple systems and clear ownership.

This matches the repo guidance to make the smallest coherent change and avoid cleaning up structure before the structure exists.

### 2.2 Use States For High-Level Flow

Use Bevy states for menu flow and wave flow.

- `AppState`: `StartScreen`, `Playing`, `GameOver`
- `PlayState` as a sub-state of `AppState::Playing`: `WaveIntro`, `WaveActive`, `WaveClear`

Use:

- `OnEnter(...)` for setup
- `OnExit(...)` for teardown and resource resets
- `run_if(in_state(...))` to gate gameplay systems
- `DespawnOnExit<S>` for entities that should disappear automatically when leaving a state

Do not rely on manual cleanup systems when state-scoped entity lifetimes are enough.

### 2.3 Prefer Buffered Messages For Gameplay Notifications

For most gameplay communication, prefer buffered messages over observers.

Use messages for things like:

- score changes
- kill notifications
- egg collection
- wave clear notifications
- particle spawn requests

In Bevy 0.18, `EventWriter` is effectively an alias over the message system, but it is clearer in a new codebase to think in terms of:

- `#[derive(Message)]` for scheduled, buffered communication between systems
- `#[derive(Event)]` plus observers only for immediate reactive flows that truly benefit from triggers

Good rule of thumb:

- If the reaction can happen later in the frame in a normal schedule, use a message.
- If the reaction must happen immediately when triggered, use an observer-backed event.

Avoid designing core gameplay around observers unless there is a clear win. The earlier draft used an observer on enemy removal to spawn eggs. That is not the best fit here. Egg spawning should happen in combat resolution while the defeated entity data is still in hand.

### 2.4 Physics Scheduling

For a game where feel matters, prefer:

- input collection in `Update`
- physics and gameplay resolution in `FixedUpdate`
- animation, UI, and visual-only effects in `Update`

If the first playable version keeps everything in `Update` for simplicity, all motion must still be delta-time scaled and collision must use previous and current positions to avoid tunneling.

### 2.5 Keep Resources And Components Focused

Use:

- components for per-entity state
- resources for truly shared mutable state
- constants for tunable numeric values

Examples:

- `Velocity`, `FacingDirection`, `Grounded`, `EnemyTier`, `Egg`, `Invincible`: components
- `WaveState`, `HighScore`, `PlatformLayout`, `ScreenShake`: resources
- gravity, speeds, radii, timers: constants

### 2.6 Avoid Premature Command Flushes

Do not insert `ApplyDeferred` by default just because commands exist.

Prefer this pattern:

1. gameplay systems write messages describing outcomes
2. presentation systems react to messages
3. despawns and spawns happen through normal commands

Only add `ApplyDeferred` if a later system in the same schedule must read command results immediately.

## 3. Technical Baseline

### 3.1 Window And World Coordinates

- Fixed game window: `1200 x 900`
- No camera scrolling
- Single-screen arena
- Origin at screen center
- Positive Y is up

Recommended primary window settings:

- title set to `Joust`
- fixed resolution
- non-resizable for v1

Store gameplay dimensions as `f32` constants for physics math and cast to `u32` only when constructing `WindowResolution::new(...)`.

### 3.2 Arena Bounds

Suggested initial world bounds:

```text
ARENA_LEFT   = -600.0
ARENA_RIGHT  =  600.0
ARENA_TOP    =  450.0
ARENA_BOTTOM = -450.0
```

The playable region can reserve a top strip for HUD spacing and a bottom strip for lava visuals.

### 3.3 Camera

Spawn a `Camera2d` at startup. If bloom is enabled, follow current Bevy 0.18 examples:

```rust
commands.spawn((
    Camera2d,
    Camera {
        clear_color: ClearColorConfig::Custom(Color::srgb(0.04, 0.04, 0.07)),
        ..default()
    },
    Tonemapping::TonyMcMapface,
    Bloom::default(),
    DebandDither::Enabled,
));
```

Imports:

```rust
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
};
```

Do not rely on undocumented preset constants for bloom configuration.

## 4. Game States

### 4.1 State Definitions

```rust
#[derive(States, Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
enum AppState {
    #[default]
    StartScreen,
    Playing,
    GameOver,
}

#[derive(SubStates, Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
enum PlayState {
    #[default]
    WaveIntro,
    WaveActive,
    WaveClear,
}
```

Register with:

```rust
app.init_state::<AppState>()
    .add_sub_state::<PlayState>();
```

### 4.2 State Responsibilities

| State | Responsibility |
|---|---|
| `StartScreen` | Title, mode selection, high score, optional attract/demo mode |
| `Playing` | Owns gameplay entities and HUD |
| `WaveIntro` | Short pause, wave announcement, spawn setup |
| `WaveActive` | Main gameplay loop |
| `WaveClear` | Short transition and wave increment |
| `GameOver` | Final score, restart prompt, optional frozen arena backdrop |

### 4.3 Entity Lifetime

Attach `DespawnOnExit<S>` to state-owned entities such as:

- start screen UI
- gameplay arena entities
- wave announcement text
- game over overlay

Use `OnExit(...)` for:

- resetting resources
- clearing timers
- persisting high score
- changing to the next state

## 5. Core Gameplay

### 5.1 Flight Physics

The flight model is the heart of the game.

Rules:

- no jump button
- no physics engine
- gravity always pulls downward while airborne
- each flap adds upward impulse
- horizontal movement is acceleration-based and slippery

Suggested components:

- `Velocity(Vec2)`
- `FacingDirection`
- `Grounded`
- `FlapBuffer`
- `Invincible`

Suggested tuning targets:

- about 4 to 5 flaps per second to maintain altitude
- visible bob between flaps
- noticeable slide when changing horizontal direction

Suggested constants:

| Constant | Meaning |
|---|---|
| `GRAVITY` | Downward acceleration |
| `FLAP_IMPULSE` | Upward velocity impulse per flap |
| `MAX_RISE_SPEED` | Clamp on upward velocity |
| `MAX_FALL_SPEED` | Terminal downward speed |
| `AIR_ACCEL` | Airborne horizontal acceleration |
| `GROUND_ACCEL` | Ground movement acceleration |
| `AIR_DRAG` | Horizontal drag in air |
| `GROUND_DRAG` | Horizontal drag while grounded |
| `MAX_AIR_SPEED` | Horizontal speed cap in air |
| `MAX_GROUND_SPEED` | Horizontal speed cap on platforms |

Implementation notes:

- Input should request a flap.
- Physics should consume that request and update velocity.
- Clamp velocity after gravity and flap impulses.
- Use `time.delta_secs()` in `Update` or fixed timestep delta in `FixedUpdate`.

### 5.2 Platforms And Grounded State

Platforms are one-way:

- actors may pass upward through them
- actors may land on top from above

Store static platform definitions in a resource:

```rust
struct PlatformDef {
    center: Vec2,
    width: f32,
    wraps: bool,
}
```

With only a handful of platforms, brute-force collision is fine.

Important correction from the previous draft:

Do not test landing using only the current frame position and a snap distance. Use a swept check based on previous and current bottom edges.

Landing rule:

1. previous bottom is above the platform top
2. current bottom is at or below the platform top plus a small snap allowance
3. vertical velocity is downward
4. horizontal position is inside the platform span

When landing:

- snap the entity to platform top
- zero vertical velocity
- add `Grounded`

When leaving a platform:

- remove `Grounded`

Ground-level walkway:

- spans the full arena width
- wraps at the left and right edges

Upper platforms:

- do not wrap

### 5.3 Screen Wrap

Horizontal wrap is a gameplay rule, not just a render trick.

Required helper behavior:

- wrap X position when leaving arena bounds
- compute horizontal deltas using wrapped distance
- use wrapped distance in combat, AI targeting, egg pickup, and lava-hand targeting

Suggested helpers:

```text
wrapped_dx(a, b, arena_width)
wrapped_distance(a, b, arena_width)
wrap_x(x, left, right, margin)
```

Visual duplication near screen edges is optional but useful. If enabled:

- spawn or update a render-only ghost entity when an actor overlaps an edge
- ghost entities must not carry gameplay components
- collision and AI should use wrapped math, never ghost entities

### 5.4 Joust Combat

Combat is resolved in two steps:

1. detect body overlap
2. compare joust height

Use a dedicated combat point rather than vague wording like "top of rider". For each rider, define one vertical comparison point:

```text
joust_y = transform.translation.y + JOUST_POINT_Y
```

Resolution:

- if `joust_y_a` is meaningfully above `joust_y_b`, A wins
- if `joust_y_b` is meaningfully above `joust_y_a`, B wins
- otherwise both bounce

Use:

- a small dead zone, such as `JOUST_DEAD_ZONE`
- short invincibility after bounce

Recommended collision approach:

- circle-vs-circle for rider bodies
- separate tiny hit test for pterodactyl mouth vs lance tip

Important determinism rule:

- collect collision pairs first
- decide winners and losers second
- despawn and spawn follow-up entities third

An entity that loses any joust in the frame dies even if it won another pair that same frame.

### 5.5 Eggs

When an enemy dies:

1. combat resolution determines the loser
2. an egg is spawned immediately at the defeat position
3. the defeated enemy is despawned

Do not make egg spawning depend on an "entity removed" observer.

Egg rules:

- eggs fall with gravity
- eggs can land on platforms
- eggs hatch after a timer if uncollected
- eggs that touch lava are destroyed
- players may collect eggs for points

Hatch behavior:

- Bounder egg -> Hunter
- Hunter egg -> Shadow Lord
- Shadow Lord egg -> Shadow Lord or top-tier replacement, depending on tuning

### 5.6 Lava

Lava occupies the bottom of the arena.

Rules:

- touching lava kills riders, enemies, and eggs
- lava is visually animated but collision should use a simple threshold

For v1, use a single `LAVA_Y` kill line and keep the animated surface cosmetic.

Lava hand:

- good fit for a later milestone
- should not block the first playable game loop

### 5.7 Enemies

Enemy tiers:

| Tier | Role |
|---|---|
| `Bounder` | slow, weak, basic wander |
| `Hunter` | more direct pursuit |
| `ShadowLord` | fast and aggressive |

Recommended v1 AI state machine:

- `Wander`
- `Pursue`
- `Recover` or short `Land` state

The previous draft included more states than are needed for a first implementation. Start simpler and add `Evade` only if playtesting proves it necessary.

AI requirements:

- use wrapped distance for target selection
- flap on a timer with small randomness
- respect the same movement rules as players
- use spawn invincibility briefly to avoid unfair top-spawn collisions

Dependency note:

- add `rand` only when randomness is actually introduced

### 5.8 Pterodactyl

Late-wave threat:

- appears if the wave drags on too long
- ignores platforms
- kills riders on body contact
- only dies to a precise lance-tip hit on the mouth or head hitbox

Treat this as a later milestone after the normal wave loop is solid.

### 5.9 Waves

Wave data belongs in a resource, for example:

```rust
struct WaveDef {
    bounders: u32,
    hunters: u32,
    shadow_lords: u32,
    egg_hatch_time: f32,
    pterodactyl_after: Option<f32>,
}
```

Wave clear condition:

- zero living enemies
- zero eggs

Between waves:

- short pause
- wave announcement
- spawn the next set of enemies

### 5.10 Player Death, Lives, And Respawn

On player death:

1. decrement lives
2. emit a buffered message for UI and effects
3. play death effect
4. respawn after a timer if lives remain

Respawn rules:

- pick a safe spawn location
- grant brief invincibility
- do not respawn directly into an enemy

Two-player rules:

- players can kill each other
- game over only when all players are out of lives

## 6. Rendering And Effects

### 6.1 Primitive-Only Actors

Gameplay actors are hierarchical entities built from child meshes:

- body rectangles
- head circles
- wing ellipses or capsules
- lance rectangles and tip shapes
- simple leg shapes

Use:

- `Mesh2d`
- `MeshMaterial2d<ColorMaterial>`
- parent-child transforms

Rendering goals:

- readable silhouettes first
- fancy effects second

### 6.2 Mesh And Material Reuse

Pre-create and cache reusable handles in a resource once repeated spawning begins.

Examples:

- one rectangle mesh reused across body parts, platforms, and lava strips
- one circle mesh reused across heads, particles, and eggs
- a small set of shared materials by faction and effect color

Avoid creating a fresh mesh or material handle every spawn when reuse is enough.

### 6.3 Animation

Animation should be transform-driven, not sprite-sheet-driven.

Examples:

- wing flaps rotate child wing transforms
- walking alternates leg offsets while grounded
- egg pulsing uses scale and color interpolation
- death bursts spawn temporary primitive particles

### 6.4 World And UI Text

Use:

- `Text2d` for in-world announcements such as `WAVE 3`
- UI nodes for HUD, title screen, and game over overlays

### 6.5 Z Layers

Suggested draw ordering:

| Z | Layer |
|---|---|
| 0 | background |
| 1 | lava back |
| 2 | platforms |
| 3 | eggs |
| 4 | enemies |
| 5 | players |
| 6 | trails and particles |
| 7 | lava front |
| 8 | world-space wave text |

UI renders separately and does not need to share world Z values.

### 6.6 Effects

Effects that support readability:

- bloom on very bright elements only
- short hit flash on bounce
- small death particle burst
- modest screen shake on major kills
- subtle background stars

Effects that are safe to defer:

- lava hand visuals
- elaborate pterodactyl outline treatment
- heavy trail systems

## 7. Input, Scoring, UI, And Audio

### 7.1 Keyboard Layout

Recommended controls:

| Action | Player 1 | Player 2 |
|---|---|---|
| Move left | `A` | `J` |
| Move right | `D` | `L` |
| Flap | `W` or `Space` | `I` |

Menu controls:

- `Space` starts the game in `StartScreen`
- `2` enables two-player mode before starting

To avoid conflicts:

- in single-player, arrow keys can optionally mirror player 1
- in two-player mode, keep to the split keyboard layout above

### 7.2 Flap Buffer

Use a short flap buffer so a press just before landing still produces a takeoff on contact.

This is a worthwhile quality-of-life feature and makes the game feel less sticky.

### 7.3 Scoring

Suggested values:

| Action | Points |
|---|---|
| kill Bounder | 500 |
| kill Hunter | 750 |
| kill Shadow Lord | 1000 |
| collect egg | 250 |
| kill pterodactyl | 2000 |

Extra life:

- every `10_000` points
- cap total lives with `MAX_LIVES`

### 7.4 High Score Persistence

Do not store save data under `assets/`.

Preferred order:

1. platform-appropriate data directory
2. fallback file in working directory if needed

This can be a later milestone.

### 7.5 Audio

Audio is optional for the first playable version.

Design the gameplay loop so audio can subscribe to the same buffered messages later:

- flap
- kill
- bounce
- egg collect
- hatch
- wave start
- game over

## 8. Scheduling And Data Flow

### 8.1 Suggested System Sets

Use set-level ordering once the project has enough systems to justify it.

Suggested fixed-step gameplay order:

```text
InputIntent -> AiIntent -> Physics -> Combat -> Progression
```

Suggested frame presentation order:

```text
Animation -> Effects -> UI
```

### 8.2 Message Types

Recommended buffered messages:

- `FlapMessage`
- `JoustKillMessage`
- `JoustBounceMessage`
- `EggCollectedMessage`
- `EggHatchedMessage`
- `PlayerDiedMessage`
- `ScoreMessage`
- `WaveClearedMessage`

Use them to decouple:

- gameplay
- scoring
- UI
- effects

### 8.3 Observer Use Cases

Observers are still useful, but keep them narrow.

Good uses:

- immediate reaction to a manually triggered one-shot event
- entity-targeted reaction that benefits from trigger semantics

Less good uses in this project:

- ordinary score updates
- ordinary effect spawning
- normal wave progression

## 9. File Layout

### 9.1 First Playable Milestone

Start with:

- `src/main.rs`

Optionally extract only if needed:

- `src/constants.rs`
- `src/states.rs`

### 9.2 Expected Medium-Term Layout

Once the codebase grows, this structure is a good target:

```text
src/
  main.rs
  constants.rs
  components.rs
  resources.rs
  states.rs
  player.rs
  enemy.rs
  combat.rs
  physics.rs
  waves.rs
  ui.rs
  rendering.rs
  effects.rs
```

This is a target layout, not a requirement to create immediately.

## 10. Initial Constants

Keep tuning values in one place once `constants.rs` exists.

```text
WINDOW_WIDTH            = 1200.0
WINDOW_HEIGHT           = 900.0

ARENA_LEFT              = -600.0
ARENA_RIGHT             =  600.0
ARENA_TOP               =  450.0
ARENA_BOTTOM            = -450.0

GRAVITY                 = 980.0
FLAP_IMPULSE            = 320.0
MAX_RISE_SPEED          = 400.0
MAX_FALL_SPEED          = 600.0
AIR_ACCEL               = 800.0
GROUND_ACCEL            = 400.0
AIR_DRAG                = 350.0
GROUND_DRAG             = 500.0
MAX_AIR_SPEED           = 300.0
MAX_GROUND_SPEED        = 150.0

PLATFORM_SNAP_DISTANCE  = 3.0
PLATFORM_THICKNESS      = 12.0

JOUST_POINT_Y           = 18.0
JOUST_DEAD_ZONE         = 8.0
BOUNCE_HORIZONTAL       = 250.0
BOUNCE_VERTICAL         = 150.0

RIDER_RADIUS            = 18.0
EGG_RADIUS              = 10.0
PTERO_BODY_RADIUS       = 25.0
PTERO_HEAD_RADIUS       = 8.0
LANCE_TIP_RADIUS        = 6.0

EGG_HATCH_TIME_BASE     = 15.0
LAVA_Y                  = -410.0

PLAYER_RESPAWN_DELAY    = 1.5
BOUNCE_INVINCIBILITY    = 0.3
RESPAWN_INVINCIBILITY   = 2.0

MAX_LIVES               = 5
EXTRA_LIFE_INTERVAL     = 10_000

WRAP_MARGIN             = 20.0
```

Type guidance:

- most gameplay constants: `f32`
- counts: `u32` or `usize`

## 11. Milestones

Recommended implementation order:

1. Replace `Hello, World!` with window setup, camera, and app states.
2. Build one controllable rider with flap, gravity, and horizontal movement.
3. Add platforms, grounded state, and walk-off behavior.
4. Add wrap helpers and wrapped collision math.
5. Add one enemy tier and joust combat.
6. Add eggs, scoring, and wave clear logic.
7. Add wave intro, HUD, lives, death, and respawn.
8. Add second enemy tiers, pterodactyl, and polish effects.

This order keeps the game playable early and matches the repo guidance to make targeted, testable changes.

## 12. Acceptance Criteria For A Good V1

The first good version should support all of the following:

- fixed single-screen arena
- one player can move, flap, land, and wrap
- at least one enemy type can spawn and fight correctly
- kills resolve by vertical advantage, not raw collision order
- eggs spawn, can be collected, and can hatch
- wave clear requires no enemies and no eggs
- player death, lives, and respawn work
- HUD shows score, wave, and lives
- code validates with `cargo check`

Stretch goals after that:

- two-player mode
- lava hand
- pterodactyl
- high score persistence
- audio

## 13. Dependency Notes

Current dependency baseline:

```toml
[dependencies]
bevy = { version = "0.18.1", features = ["dynamic_linking"] }
```

Expected additions:

- `rand = "0.9"` when enemy randomness or particle spread is introduced
- optional save-path helper crate later if high score persistence needs a platform-specific location
