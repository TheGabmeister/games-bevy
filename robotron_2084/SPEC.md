# Robotron 2084 Technical Specification

## 0. Document Status

This document describes the target architecture and gameplay plan for `robotron_2084`.

Current repository status on March 29, 2026:

- The game is still a Bevy starter app.
- `src/main.rs` creates an `App`, adds `DefaultPlugins`, spawns `Camera2d`, and shows centered `Hello, World!` UI text.
- No gameplay state machine, asset pipeline, enemy logic, wave system, or audio system exists yet.

This spec is intentionally written to:

- align with the current repo instead of assuming systems already exist
- follow Bevy 0.18.1 patterns where they are stable and useful
- stay small-project friendly so implementation can begin in `main.rs` and split into modules only when justified

## 1. Project Goals

Create a modernized single-screen Robotron-inspired twin-stick shooter in Bevy 0.18.1 using:

- primitive 2D meshes instead of character sprites
- bright HDR-style colors with bloom
- custom particles and screen feedback
- keyboard-first controls with gamepad support

### Core Loop

1. Start a wave.
2. Move, shoot, survive, and rescue humans when possible.
3. Clear killable enemies.
4. Advance to a harder wave.
5. Lose when all lives are gone.

### Non-Goals For Initial Implementation

- online multiplayer
- deterministic replays
- save system beyond optional high-score persistence
- full physics engine integration
- content pipelines or data-driven mod support

## 2. Bevy Best-Practice Guardrails

The previous version of this spec was close in spirit, but it needed a few important corrections.

### Follow

- Use `OnEnter` and `OnExit` for spawn and cleanup symmetry when state boundaries are clear.
- Use marker components for categories such as `Player`, `Enemy`, `Human`, and `WaveEntity`.
- Use resources for persistent shared game state such as score, lives, current wave, and loaded mesh/material handles.
- Use buffered Bevy messages for cross-system notifications such as enemy death, score awards, and player death.
- Use observers sparingly for localized lifecycle reactions, not as the main gameplay communication path.
- Use `DespawnOnExit(AppState::...)` for entities that truly belong to a whole top-level app state.
- Keep gameplay systems in `Update` first; only move simulation to `FixedUpdate` later if testing shows a need.
- Prefer explicit ordering only where outcome depends on it.

### Avoid

- Replacing Bevy messages with ad hoc `Vec`-of-events resources unless data must persist across multiple frames.
- Overusing observers for ordinary gameplay flow.
- Splitting into many modules before the game has enough systems to justify it.
- Deep entity hierarchies for purely decorative child meshes.
- Large cleanup systems that run every frame when a clean state transition can own the cleanup.

### Notes Specific To Bevy 0.18.1

- `despawn()` is recursive by default.
- `ScalingMode` lives in `bevy::camera::ScalingMode`.
- `DespawnOnExit<S>` is the state-scoped despawn marker to use.
- Bloom is enabled with the `Bloom` component on the camera.
- Bevy 0.18 keeps buffered messages (`MessageWriter` / `MessageReader`) and observers as separate tools.

## 3. High-Level Technical Direction

### Rendering Style

- 2D top-down arena
- `Mesh2d` plus `MeshMaterial2d<ColorMaterial>` for gameplay entities
- bright color values for bloom-friendly presentation
- dark background and strong silhouette readability

### Camera

- one fixed `Camera2d`
- fixed logical vertical space using `ScalingMode::FixedVertical { viewport_height: 720.0 }`
- no scrolling
- a small gameplay margin around the arena

Recommended camera setup:

```rust
commands.spawn((
    Camera2d,
    Camera {
        clear_color: ClearColorConfig::Custom(Color::BLACK),
        ..default()
    },
    Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: 720.0,
        },
        ..OrthographicProjection::default_2d()
    }),
    Tonemapping::TonyMcMapface,
    Bloom::NATURAL,
    DebandDither::Enabled,
));
```

### Window

- default window size: `960 x 720`
- resizable
- gameplay coordinates expressed in logical units, not physical pixels

### Collision Philosophy

- manual 2D collision
- circular collision volumes for all gameplay entities
- no external physics dependency in v1

This is the right tradeoff for an arcade game with simple overlap checks and authored movement.

## 4. Architecture Plan

Start small, then split by ownership once systems stop fitting comfortably in `main.rs`.

### Recommended Growth Path

Phase 1:

- keep everything in `src/main.rs`
- add states, player movement, basic bullets, and one enemy type

Phase 2:

- extract `states.rs`, `components.rs`, `resources.rs`, and `constants.rs`
- split into small domain plugins

Phase 3:

- add `waves.rs`, `enemy.rs`, `combat.rs`, `effects.rs`, and `ui.rs`

### Target Module Layout Once The Game Grows

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
  combat.rs
  human.rs
  waves.rs
  effects.rs
  ui.rs
  audio.rs
```

### Plugin Boundaries

- `ArenaPlugin`: arena geometry, confinement helpers
- `PlayerPlugin`: input, movement, aim, shooting, respawn
- `EnemyPlugin`: enemy AI, spawn helpers, projectile fire
- `CombatPlugin`: collision detection and damage resolution
- `WavePlugin`: wave progression and spawn orchestration
- `EffectsPlugin`: particles, popups, flashes, shake
- `UiPlugin`: HUD, menus, overlays
- `AudioPlugin`: reserved for later

## 5. State Model

Use a small top-level state and one play sub-state.

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
    PlayerDeath,
    Paused,
}
```

### State Responsibilities

- `StartScreen`: title, prompt, optional attract-mode visuals
- `WaveIntro`: spawn wave, show overlay, freeze hostile behavior briefly
- `WaveActive`: normal gameplay
- `WaveClear`: short pause and cleanup between waves
- `PlayerDeath`: death effect and respawn decision
- `Paused`: gameplay frozen, UI still responsive
- `GameOver`: final score and restart prompt

### Entity Lifetime Rules

- Gameplay entities should carry `DespawnOnExit(AppState::Playing)`.
- Wave-owned entities should also carry `WaveEntity`.
- Wave cleanup should happen explicitly when starting the next wave, not by leaving `WaveActive`, because `Paused` and `PlayerDeath` should preserve the arena.
- Menu UI should use `DespawnOnExit` for its owning top-level state.

## 6. Scheduling Strategy

Use sets only where they improve readability or guarantee outcome.

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet {
    Input,
    Movement,
    Confinement,
    Combat,
    Resolution,
    Effects,
    UiSync,
}
```

Recommended ordering during `PlayState::WaveActive`:

1. `Input`
2. `Movement`
3. `Confinement`
4. `Combat`
5. `Resolution`
6. `Effects`
7. `UiSync`

Only use `.after(...)` or `.chain()` where order changes behavior. Everything else can stay parallel.

## 7. Messaging, Resources, And Components

### Prefer Messages For Cross-System Gameplay Signals

Examples:

- `EnemyKilled`
- `HumanRescued`
- `PlayerKilled`
- `WaveCleared`
- `ScoreAwarded`

Recommended pattern:

```rust
#[derive(Message)]
struct EnemyKilled {
    position: Vec2,
    points: u32,
    enemy_type: EnemyKind,
}
```

Why this is preferred:

- clearer intent than a custom `Vec` resource
- automatic frame-based buffering
- easier to test and reason about

### Use Resources For Persistent Shared State

```rust
#[derive(Resource)]
struct GameState {
    score: u32,
    high_score: u32,
    lives: u32,
    current_wave: u32,
    rescue_count_this_wave: u32,
    next_extra_life_score: u32,
}

#[derive(Resource)]
struct WaveState {
    clear_requested: bool,
    intro_timer: Timer,
    clear_timer: Timer,
    death_timer: Timer,
}

#[derive(Resource, Default)]
struct ScreenShake {
    trauma: f32,
}
```

### Core Components

```rust
#[derive(Component)] struct Player;
#[derive(Component)] struct Enemy;
#[derive(Component)] struct Human;
#[derive(Component)] struct Electrode;
#[derive(Component)] struct Hulk;
#[derive(Component)] struct Brain;
#[derive(Component)] struct Spawner;
#[derive(Component)] struct PlayerBullet;
#[derive(Component)] struct EnemyProjectile;
#[derive(Component)] struct WaveEntity;
#[derive(Component)] struct Confined;

#[derive(Component)] struct Velocity(Vec2);
#[derive(Component)] struct Facing(Vec2);
#[derive(Component)] struct CollisionRadius(f32);
#[derive(Component)] struct PointValue(u32);
#[derive(Component)] struct FireCooldown(Timer);
#[derive(Component)] struct Lifetime(Timer);
#[derive(Component)] struct Invincible(Timer);
#[derive(Component)] struct WanderTarget(Vec2);
#[derive(Component)] struct WanderTimer(Timer);

#[derive(Component)]
struct SpawnerState {
    children_spawned: u32,
    max_children: u32,
    cooldown: Timer,
}

#[derive(Component)]
struct Knockback {
    velocity: Vec2,
}

#[derive(Component)]
struct HomingMissile {
    turn_rate: f32,
}

#[derive(Component)]
struct BouncesRemaining(u32);
```

Deliberately omitted:

- a placeholder `Health` unit component

Reason:

- this game mostly uses one-hit kill or despawn-on-contact rules
- hit points should only exist if an entity truly needs multiple health states later

## 8. Input Design

### Supported Schemes

Keyboard:

- move: `WASD`
- aim and fire: arrow keys
- pause: `Escape`
- start and confirm: `Space`, `Enter`, or any directional input

Gamepad:

- move: left stick
- aim and fire: right stick
- pause: `Start`

### Input Rules

- movement vector is normalized to avoid faster diagonal speed
- keyboard aim is 8-direction only
- gamepad aim is analog with a deadzone
- firing is automatic while a valid aim direction is held

Recommended constants:

```rust
pub const PLAYER_FIRE_COOLDOWN: f32 = 0.08;
pub const GAMEPAD_DEADZONE: f32 = 0.2;
```

### Ghosting

Keyboard ghosting is a real hardware risk for twin-stick keyboard play. The game should:

- document `WASD + Arrows` as the default layout
- allow remapping later if input reliability becomes a problem
- avoid promising perfect keyboard compatibility across all devices

## 9. Arena And Camera Space

### Arena Rules

- single-screen rectangular arena
- no internal walls
- no scrolling
- outer electric fence acts as the hard gameplay boundary

### Recommended Arena Constants

```rust
pub const WINDOW_WIDTH: u32 = 960;
pub const WINDOW_HEIGHT: u32 = 720;

pub const ARENA_HALF_WIDTH: f32 = 440.0;
pub const ARENA_HALF_HEIGHT: f32 = 320.0;
pub const ARENA_BORDER_THICKNESS: f32 = 3.0;
```

### Boundary Rendering

Represent the border as four thin capsules or quads. Capsules are acceptable because `Mesh2d` does not provide a dedicated line primitive.

### Boundary Enforcement

All entities with `Confined` should be clamped after movement:

- player
- enemies
- humans
- projectiles that bounce

Straight-line player bullets should usually despawn when leaving the arena rather than clamp.

## 10. Entity Catalog

All gameplay entities are rendered with primitive meshes and shared materials loaded once into a resource.

### Visual Design Rules

- each entity must be readable by shape first, color second
- colors should stay distinct even when bloom is strong
- composite shapes should use shallow parent-child hierarchies
- decorative child meshes should not carry gameplay components

### Gameplay Entities

| Entity | Shape | Role | Notes |
|---|---|---|---|
| Player | rotated square / diamond | move, fire, rescue | faces aim direction |
| Grunt | square | basic chaser | bread-and-butter enemy |
| Hulk | large square with inner accent | slow indestructible threat | bullets push, do not kill |
| Brain | circle with inner accent | seeks humans, spawns missiles | converts humans to Progs |
| Prog | short capsule | fast chaser | created from converted humans |
| Spheroid | pulsing circle | spawner | creates Enforcers |
| Enforcer | triangle | ranged pressure | fires aimed sparks |
| Quark | rotating hexagon | spawner | creates Tanks |
| Tank | pentagon | slow ranged enemy | fires bouncing shells |
| Electrode | cross | static hazard | hurts player and most enemies |
| Human family | small capsule/circle variants | rescue targets | three cosmetic variants only |

### Projectiles

| Projectile | Source | Behavior |
|---|---|---|
| Player bullet | player | fast straight shot |
| Cruise missile | brain | homing with limited turn rate |
| Spark | enforcer | aimed shot, no homing |
| Bounce shell | tank | wall reflection, fixed bounce count |

## 11. Movement And Simulation

### Player

- no inertia
- immediate stop when input ends
- constant move speed
- aim direction stored separately from move direction

Recommended constant:

```rust
pub const PLAYER_SPEED: f32 = 300.0;
```

### Enemies

Use authored motion, not physics bodies.

- Grunts: direct chase with small per-entity steering offset
- Hulks: slow biased wander plus knockback response
- Brains: seek nearest human, or player if none remain
- Progs: faster direct chase than Grunts
- Spheroids and Quarks: wander toward chosen target points
- Enforcers and Tanks: slow drift plus ranged attack timers

### Separation

A light anti-stacking force is acceptable for readability, but keep it subtle and optional.

Recommendation:

- implement only if overlapping primitive shapes become a real readability problem
- keep it off until playtesting proves the need

This is more conservative than the previous spec and matches the repo rule to make the smallest coherent change first.

## 12. Collision Design

### Collision Shape

Use circle-circle overlap for gameplay entities.

```rust
distance_squared < (r1 + r2) * (r1 + r2)
```

### Why This Is Correct For This Project

- cheap
- easy to debug
- good enough for abstract shapes
- no need for Rapier in v1

### Collision Pairs

Keep collision responsibilities split by meaning:

- player vs enemies
- player vs enemy projectiles
- player vs humans
- player bullets vs killable enemies
- player bullets vs hulks
- player bullets vs electrodes
- hulk vs humans
- brain vs humans
- enemy vs electrodes

### Ordering Rules

- rescue should resolve before brain conversion if both happen in the same frame
- player death resolution should happen once even if multiple collisions occur
- score awards and effects should happen after collision outcomes are decided

### Bullet Tunneling

This was underspecified before. Best practice for v1:

- keep bullets reasonably fast but not extreme
- use slightly generous collision radii
- if testing shows visible misses, add swept tests for bullets only

Do not add full continuous collision detection globally.

## 13. Wave System

### Wave Definition

```rust
struct WaveDefinition {
    grunts: u32,
    hulks: u32,
    brains: u32,
    spheroids: u32,
    quarks: u32,
    electrodes: u32,
    humans: u32,
    speed_mult: f32,
}
```

Store wave definitions in code, not an external data file.

Reason:

- simpler iteration
- compile-time checking
- no parser or asset-management overhead for a small game

### Intro Sequence

1. Clear old `WaveEntity` entities.
2. Spawn the new field.
3. Show `WAVE N` overlay.
4. Hold enemies inactive for a short intro timer.
5. Transition to `WaveActive`.

### Completion Rule

The wave clears when all killable enemies are gone.

Implications:

- Hulks do not block wave completion.
- Spawned children do count.
- Spawners matter strategically because ignoring them increases pressure.

### Spawn Rules

- player starts in the arena center
- enemies spawn on edges
- humans and electrodes spawn inside the arena
- avoid center spawn overlap and obvious stacking

Recommended constants:

```rust
pub const SPAWN_EXCLUSION_RADIUS: f32 = 100.0;
pub const SPAWN_MIN_SEPARATION: f32 = 20.0;
pub const MAX_TOTAL_ENEMIES: u32 = 150;
```

### Spawner Limits

- each Spheroid has a child cap
- each Quark has a child cap
- also enforce a global active enemy cap

This keeps the game difficult without letting one neglected spawner create runaway entity counts.

## 14. Scoring And Progression

### Points

| Entity | Points |
|---|---:|
| Grunt | 100 |
| Prog | 100 |
| Enforcer | 150 |
| Tank | 200 |
| Electrode | 25 |
| Brain | 500 |
| Spheroid | 1000 |
| Quark | 1000 |
| Hulk | 0 |

### Human Rescue Bonus

Per-wave escalating rescue bonus:

- 1st: 1000
- 2nd: 2000
- 3rd: 3000
- 4th: 4000
- 5th and later: 5000

Reset the rescue count each wave.

### Extra Lives

- award one extra life every `25_000` points
- track the next threshold in `GameState`

### High Score

Initial implementation:

- track high score for the current app session

Optional later enhancement:

- persist to disk with a small local file

## 15. Rendering, Effects, And UI

### Shared Asset Strategy

Load reusable meshes and materials once and store them in a resource.

```rust
#[derive(Resource)]
struct GameAssets {
    player_mesh: Handle<Mesh>,
    grunt_mesh: Handle<Mesh>,
    bullet_mesh: Handle<Mesh>,
    player_material: Handle<ColorMaterial>,
    grunt_material: Handle<ColorMaterial>,
    bullet_material: Handle<ColorMaterial>,
}
```

This avoids repeated asset creation and helps Bevy batch identical mesh/material combinations.

### Bloom And Color

- use bright color values for bloom
- keep the background dark
- do not rely on bloom for readability

Bloom should enhance silhouettes, not become the only way an entity is readable.

### Particle System

Entity-per-particle is acceptable for this project if the budget is capped.

Recommended rules:

- same component layout for all particles
- shared meshes and materials
- lifetime-based despawn
- optional drag and scale fade
- global particle cap

Suggested constants:

```rust
pub const MAX_PARTICLES: u32 = 200;
pub const EXPLOSION_PARTICLE_COUNT: u32 = 16;
pub const RESCUE_PARTICLE_COUNT: u32 = 10;
pub const DEATH_PARTICLE_COUNT: u32 = 30;
```

### Screen Shake

Use a trauma-style resource:

- add trauma on major events
- decay to zero over time
- sample random camera offset from trauma

Shake triggers:

- player death
- wave clear
- hulk knockback impact

### UI Layout

Use Bevy UI nodes for screen-space HUD and menus.

HUD:

- score top-left
- wave number top-center
- lives top-right
- high score bottom-left or top-left under score

Overlays:

- title screen
- wave intro text
- pause overlay
- game over overlay

### UI Best-Practice Notes

- use Bevy UI for fixed overlays
- use `Text2d` only for world-space elements such as score popups
- represent lives with simple colored nodes or mesh icons, not loaded images

### Audio

Keep an `AudioPlugin` placeholder in the architecture, but do not block gameplay implementation on audio.

## 16. Edge Cases To Handle Explicitly

### Player Spawn Safety

- player receives short invincibility on spawn and respawn
- visible blink or flicker indicates the protected state

### Wave Clear During Player Death

If the final enemy dies while the player is already in `PlayerDeath`:

- preserve the clear result in `WaveState`
- do not respawn the same wave
- proceed to the next intro after death handling finishes

### Brain Rescue Race

If player rescue and brain conversion target the same human in one frame:

- player rescue wins

### Bounce Shell Lifetime

- cap wall bounces
- also give the shell a timed lifetime

### Shared Material Mutation

Do not mutate a shared material asset for one entity-specific flash.

Instead:

- swap to a separate temporary flash material
- or change a per-entity component such as `Visibility` or transform scale

## 17. Constants Catalog

Final values should live in `constants.rs` once that file exists.

Suggested initial set:

```rust
pub const WINDOW_WIDTH: u32 = 960;
pub const WINDOW_HEIGHT: u32 = 720;

pub const ARENA_HALF_WIDTH: f32 = 440.0;
pub const ARENA_HALF_HEIGHT: f32 = 320.0;
pub const ARENA_BORDER_THICKNESS: f32 = 3.0;

pub const PLAYER_SPEED: f32 = 300.0;
pub const PLAYER_RADIUS: f32 = 8.0;
pub const PLAYER_FIRE_COOLDOWN: f32 = 0.08;
pub const PLAYER_INVINCIBILITY_DURATION: f32 = 2.0;
pub const PLAYER_BLINK_INTERVAL: f32 = 0.1;
pub const STARTING_LIVES: u32 = 3;
pub const EXTRA_LIFE_EVERY: u32 = 25_000;

pub const BULLET_SPEED: f32 = 800.0;
pub const BULLET_RADIUS: f32 = 3.0;
pub const MAX_PLAYER_BULLETS: u32 = 15;

pub const GRUNT_BASE_SPEED: f32 = 120.0;
pub const HULK_SPEED: f32 = 60.0;
pub const BRAIN_SPEED: f32 = 100.0;
pub const PROG_SPEED: f32 = 160.0;
pub const SPHEROID_SPEED: f32 = 80.0;
pub const ENFORCER_SPEED: f32 = 90.0;
pub const QUARK_SPEED: f32 = 70.0;
pub const TANK_SPEED: f32 = 50.0;
pub const HUMAN_SPEED: f32 = 40.0;

pub const GAMEPAD_DEADZONE: f32 = 0.2;
pub const WAVE_INTRO_DURATION: f32 = 1.5;
pub const WAVE_CLEAR_DURATION: f32 = 1.5;
pub const DEATH_PAUSE_DURATION: f32 = 1.5;
pub const GAME_OVER_INPUT_DELAY: f32 = 2.0;

pub const SPAWN_EXCLUSION_RADIUS: f32 = 100.0;
pub const SPAWN_MIN_SEPARATION: f32 = 20.0;
pub const MAX_TOTAL_ENEMIES: u32 = 150;

pub const MAX_PARTICLES: u32 = 200;
pub const SCREEN_SHAKE_MAX_OFFSET: f32 = 8.0;
pub const SCREEN_SHAKE_DECAY: f32 = 3.0;
```

## 18. Dependencies

Required:

```toml
[dependencies]
bevy = { version = "0.18.1", features = ["dynamic_linking"] }
```

Optional but reasonable:

```toml
rand = "0.9"
```

No physics crate is needed for the planned v1.

## 19. Validation Expectations

When implementation starts, validate from the project root with:

```powershell
cargo check
```

Also run:

```powershell
cargo clippy
```

when changing architecture patterns, state setup, or Bevy API usage broadly.

## 20. Recommended Implementation Order

1. Replace the starter setup with window config, camera config, and app states.
2. Add arena constants and render the playfield boundary.
3. Add player movement and confinement.
4. Add player aiming and bullet firing.
5. Add one enemy type: Grunts.
6. Add manual collision and scoring.
7. Add basic wave spawning for a single wave.
8. Add HUD and wave intro overlay.
9. Add humans and rescue scoring.
10. Add Hulks and player respawn flow.
11. Add Brains, Progs, and homing missiles.
12. Add Spheroids, Enforcers, Quarks, and Tanks.
13. Add particles, screen shake, and score popups.
14. Add start screen, pause, and game over.
15. Add optional audio and persistence polish.

## 21. Summary Of Improvements Over The Previous Spec

This revision fixes the main weaknesses in the earlier document:

- aligns the plan with the actual starter repo
- removes stale references to `CLAUDE.md`
- updates Bevy guidance toward messages/resources/observers used appropriately
- avoids overengineering the first implementation pass
- removes duplicated edge-case guidance
- tightens state ownership and cleanup rules
- keeps the architecture modular without pretending those modules already exist
