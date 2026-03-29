# Joust — Modernized 2D (Bevy 0.18.1)

A modernized 2D recreation of Williams Electronics' Joust (1982) using Bevy with primitive shapes only — no sprites or textures. Custom animations and visual effects make up for the lack of art assets.

---

## 1. Game Overview

Single-screen arena. One or two players ride lance-wielding birds. Flight is controlled by repeated flapping (button mashing for altitude). Combat resolves by height — the higher lance wins. Defeated enemies drop eggs that hatch into stronger foes if not collected. Waves of increasing difficulty, culminating in survival rounds with an unkillable pterodactyl.

**Window**: 1200 x 900, fixed. No camera zoom or scroll. Constants stored as `f32` for physics math; cast to `u32` when passed to `WindowResolution::new()`.

---

## 2. Core Mechanics

### 2.1 Flight Physics

This is the single most important system. If flapping doesn't feel right, nothing works.

**Model**: Custom physics, no physics engine. Gravity is a global constant, not per-entity. Each flapping entity has a `Velocity(Vec2)` component. Shared physics constants live in `constants.rs`:

| Constant | Description |
|---|---|
| `GRAVITY` | Downward acceleration (~980 px/s^2, tunable) |
| `FLAP_IMPULSE` | Upward velocity added per flap (~320 px/s) |
| `MAX_RISE_SPEED` | Clamp on upward velocity to prevent rocket launches |
| `MAX_FALL_SPEED` | Terminal velocity downward |
| `HORIZONTAL_ACCEL` | Left/right acceleration while airborne |
| `WALK_ACCEL` | Left/right acceleration while grounded (slower than airborne) |
| `HORIZONTAL_DRAG` | Deceleration when no input (creates the signature slide) |
| `MAX_HORIZONTAL_SPEED` | Horizontal speed cap |
| `WALK_MAX_SPEED` | Horizontal speed cap while grounded (slower) |

**Flap behavior**: Each flap press adds `FLAP_IMPULSE` to vertical velocity (clamped by `MAX_RISE_SPEED`). Gravity applies every frame via `velocity.y -= GRAVITY * time.delta_secs()`. Rapid tapping = hover. Single tap = small hop. No tap = fall. There is NO jump — only accumulated flap impulses against gravity.

**Horizontal movement**: Acceleration-based with drag. The player cannot stop instantly — they slide. This momentum is core to the jousting feel. Direction input accelerates; releasing input applies drag toward zero.

**Ground state**: When on a platform, vertical velocity is zeroed, gravity stops, and the entity can walk left/right using `WALK_ACCEL` / `WALK_MAX_SPEED` (slower than airborne). A flap while grounded launches upward. Track ground state with a `Grounded` marker component — add it on landing, remove it on leaving the platform or flapping.

**Delta-time**: All physics math must multiply by `time.delta_secs()`. Never assume fixed frame rate.

**Tuning concern**: The flap-to-gravity ratio determines the entire feel. Too floaty = no tension. Too heavy = frustrating. The original Joust used roughly 4-5 flaps per second to maintain altitude. Target: ~5 flaps/sec to hover, with visible bob between flaps.

### 2.2 Screen Wrap

Entities wrap horizontally. Exit the left edge, appear on the right, and vice versa.

**Implementation**: When `entity.x > ARENA_RIGHT + WRAP_MARGIN`, teleport to `ARENA_LEFT - WRAP_MARGIN` (and inverse). `WRAP_MARGIN` is half the entity's visual width so the entity fully exits before reappearing. Arena bounds are derived from window size and any HUD inset.

**Collision across the wrap boundary**: Two entities near opposite edges are actually adjacent. Use modular distance for ALL distance/collision checks: `dx = min(|x1 - x2|, ARENA_WIDTH - |x1 - x2|)`. This covers combat, egg collection, AI targeting, and lava hand detection.

**Split rendering**: Separate concern from collision. When an entity's visual bounds overlap a screen edge, spawn a **render-only ghost** — a child entity with the same mesh/material parented to a separate transform at the mirrored position. Ghost entities carry a `WrapGhost` marker component and are updated (or despawned) each frame by a dedicated system in `RenderSet`. Ghosts have no collision components and do not participate in gameplay logic.

**Platform wrap**: The bottom ground-level platforms span the full arena width and connect at the screen edges. Entities walking off one end appear on the other. Mid-level and upper platforms do NOT wrap — they have defined left/right edges.

### 2.3 Platforms

Platforms are one-way: entities can fly up through them but land on top.

**Collision rule**: Only apply platform collision when:
1. Entity's bottom edge is within `PLATFORM_SNAP_DISTANCE` (3px) of the platform's top edge, AND
2. Entity's vertical velocity is downward (or zero)

This means you can flap up through a platform from below, but land on it from above. Walking off a platform edge causes falling (remove `Grounded` component).

**Data structure**: Platforms are stored as a `Vec<PlatformDef>` resource (each: `x, y, width, wraps: bool`). With ~10 platforms, brute-force collision against all is fine — no spatial index needed.

**Platform layout**: Static per wave. The classic layout:
- Two long platforms mid-height on left and right
- One short platform top-center
- Two small ledges at upper-left and upper-right
- Ground-level walkway along the very bottom (above lava), `wraps: true`
- All platforms have small lips/edges rendered as rectangles

**Edge case — landing precision**: Entities snap to platform top when within `PLATFORM_SNAP_DISTANCE` and moving downward. The `Grounded` marker component tracks state; add it on landing, remove it when the entity flaps or walks off the edge.

### 2.4 Combat (Height-Based Jousting)

The core combat rule: when two riders collide, compare the Y position of their lances (top of the rider). Higher lance wins. Loser is destroyed and drops an egg.

**Height comparison**:
- `lance_y = entity.y + LANCE_Y_OFFSET` (top of the rider shape)
- If `lance_y_a - lance_y_b > JOUST_THRESHOLD`: A wins
- If `lance_y_b - lance_y_a > JOUST_THRESHOLD`: B wins
- If `abs(lance_y_a - lance_y_b) <= JOUST_THRESHOLD`: Both bounce off (no kill)

`JOUST_THRESHOLD` (~8px): The "equal height" dead zone. Too small = feels random. Too large = hard to get kills.

**Bounce on equal height**: Both entities receive opposing horizontal impulse and small vertical impulse. Apply an `Invincible` marker component with a `Timer` (0.3s) to prevent immediate re-collision. Systems skip collision checks for entities with `Invincible`.

**Collision shapes and radii**:

| Entity | Collision Radius |
|---|---|
| Rider (player/enemy) | 18px |
| Egg | 10px |
| Pterodactyl body | 25px |
| Pterodactyl head (kill zone) | 8px |
| Lance tip (for pterodactyl kill) | 6px |

Use circle-circle collision. Compute modular distance (for screen wrap), check if < sum of radii.

**Edge cases**:
- **Simultaneous multi-collision**: Collect all collision pairs in one pass. In the second pass, mark all losers. An entity that loses ANY joust is dead — even if it also won a different joust in the same frame. Third pass: despawn dead entities and spawn eggs. This prevents order-of-processing from affecting outcomes.
- **Player vs player (2P mode)**: Same rules apply. Friendly fire is ON (faithful to original). Both players bounce on equal height.

### 2.5 Egg System

Defeated enemies become eggs. Eggs are collectible for bonus points. Uncollected eggs hatch after a timer into a stronger enemy type.

**Lifecycle**:
1. Enemy dies -> egg spawns at death position with enemy's velocity (use an `Observer` on entity removal — see section 10.2)
2. Egg falls (has gravity, no flapping), lands on platforms
3. Egg sits for `EGG_HATCH_TIME` seconds (stored as a `Timer` component, decreases in later waves)
4. If collected (player touches egg): emit `ScoreEvent`, despawn egg
5. If timer expires: egg hatches -> spawn enemy one tier higher (Bounder->Hunter->Shadow Lord)

**Egg physics**: Gravity + platform landing only. No horizontal movement once landed (apply full drag). Eggs can be nudged off platforms by a player walking into them — apply a small horizontal impulse, remove `Grounded`.

**Visual**: Egg shape pulses (scale oscillation driven by `Timer`) as hatch timer approaches zero. Color lerps from white toward the enemy tier color based on `timer.fraction()`.

**Edge case**: Egg falls into lava -> destroyed, no hatch, no points.

### 2.6 Lava

The bottom of the screen is lava. Anything that touches it is destroyed.

**Lava surface**: Animated sine wave along the bottom ~60px tall. Entities below `LAVA_Y` are destroyed on contact.

**Lava Troll (hand)**: When an entity is near lava level, a hand rises and grabs. Uses modular distance for targeting (respects screen wrap).

**Hand behavior**:
1. Track nearest entity within `HAND_DETECT_RANGE` of hand's X position, below `HAND_DETECT_Y` threshold
2. Rise at `HAND_RISE_SPEED` toward target
3. If target escapes range, hand retracts
4. If hand contacts target, target is destroyed
5. Cooldown `Timer` after grab attempt: `HAND_COOLDOWN` seconds
6. Hand does not clip through platforms — clamp max Y to the bottom of the lowest platform above it

### 2.7 Enemies

Three tiers of increasing difficulty:

| Type | Color | Speed | AI Aggression | Flap Rate |
|---|---|---|---|---|
| Bounder | Red tones | Slow | Wanders, avoids player | Low |
| Hunter | White/silver | Medium | Seeks player | Medium |
| Shadow Lord | Dark blue | Fast | Aggressive pursuit | High |

**AI behavior** — each enemy runs a simple state machine stored as an `AiState` enum component:

1. **Wander**: Pick random direction, flap occasionally, change direction at edges or after a `Timer`. Transition to Pursue when player is within detection range.
2. **Pursue**: Fly toward player using modular distance (screen wrap-aware). Higher aggression = more direct path. Transition to Evade if player is above and closing.
3. **Evade**: If player is above and approaching, gain altitude or dodge horizontally. Transition back to Pursue after evade `Timer` expires.
4. **Land**: If on platform, walk briefly (ground `Timer`), then take off. Max ground time: Bounder 4s, Hunter 2s, Shadow Lord 1s.

**AI flapping**: Enemies flap at their tier's rate using a per-entity `FlapCooldown` timer. Add small random jitter (0-0.1s) to the cooldown to prevent robotic synchronized flapping.

**Spawn pattern**: Enemies spawn from the top of the screen. Brief materialization animation (circles expanding from a point). Spawning enemies get an `Invincible` marker with a 0.5s `Timer` to prevent spawn-camping. Visual indicator: entity blinks (alpha oscillates) during invincibility.

**AI and screen wrap**: All AI distance calculations use modular distance.

**Random number generation**: Add `rand` crate to `Cargo.toml` for AI jitter, wander direction, and spawn positions.

### 2.8 Pterodactyl

Appears if a wave takes too long (e.g., 60s with <=1 enemy remaining, or 90s total). Nearly unkillable — can only be hit by a perfectly timed lance to the mouth (tiny hitbox).

**Behavior**: Flies in sine-wave patterns, targeting the nearest player. Ignores platforms (flies through them). Very fast. Contact with body = instant death to rider.

**Kill condition**: Player's lance tip hitbox (6px radius circle at lance end) must contact the pterodactyl's head hitbox (8px radius circle at the front of a ~60px body). This is intentionally very hard.

**Visual**: Larger shape than riders (~3x). Distinct angular/triangular wing shapes. Pulsing emissive outline (HDR color for bloom glow).

### 2.9 Wave System

| Wave | Enemies | Types | Egg Hatch Time | Pterodactyl Timer |
|---|---|---|---|---|
| 1 | 3 | Bounders only | 15s | None |
| 2 | 4 | Bounders + 1 Hunter | 13s | None |
| 3 | 5 | Mix | 11s | 90s |
| 4 | 6 | More Hunters | 10s | 80s |
| 5+ | 6-8 | Increasing Shadow Lords | 8s (min) | 60s (min) |

Wave definitions stored as a `Vec<WaveDef>` resource, each containing enemy counts by tier, hatch time, and pterodactyl timer. Waves beyond the table extrapolate by clamping to max difficulty values.

**Wave clear condition**: Zero living enemies AND zero eggs on screen. Eggs that fall into lava count as cleared.

**Between waves**: Brief pause (2s `Timer`), "WAVE X" announcement, remaining eggs are auto-despawned (bonus points for any collected during the wave, none awarded for auto-cleared eggs). Platform layout may change in later waves.

**Survival wave** (every 5th wave): No enemies. Pterodactyl spawns immediately. Survive for 30 seconds. Bonus points for killing it.

### 2.10 Player Death and Respawn

When a player dies with remaining lives:
1. Decrement lives, emit `PlayerDeathEvent`
2. Brief death animation (shape explodes into particles)
3. After 1.5s `Timer`, respawn at a safe location (top-center platform, or highest empty platform)
4. Spawned player gets `Invincible` marker with 2.0s `Timer` (longer than enemy invincibility)
5. Visual indicator: blinking during invincibility

When a player dies with zero lives remaining: that player is permanently dead. In 2P mode, the other player continues. Game over when all players are dead.

---

## 3. Entity Rendering (Primitive Shapes Only)

All entities built from Bevy `Mesh2d` + `MeshMaterial2d<ColorMaterial>`. Each entity is a parent with child shape entities using relative `Transform` offsets.

### 3.1 Player / Enemy Rider

Built from ~6-8 primitive shapes as children of a root entity:

```
    [diamond]         <- lance tip (rotated square, small)
   /
  [circle]            <- head
  [rectangle]         <- body/torso
  [trapezoid/rect]    <- bird body (wider rectangle below torso)
 / \
[ellipse] [ellipse]   <- wings (animated rotation for flapping)
  | |
 [rect][rect]         <- legs/feet (animated when grounded)
```

- **Body**: Rectangle (torso) + larger rectangle (bird body)
- **Head**: Circle on top of torso
- **Lance**: Thin rotated rectangle extending from the head in the facing direction. Tip uses HDR emissive color for bloom glow.
- **Wings**: Two ellipses parented to the bird body, animated via `Transform` rotation
- **Legs**: Two small rectangles below bird body, animated to alternate when grounded and walking

**Facing direction**: Flip all child `Transform` X-offsets and rotations when `FacingDirection` changes. Lance points in movement direction.

**Color scheme**: Player 1 = warm yellow/gold. Player 2 = cyan/teal. Enemies use their tier color for bird body, with consistent grey for rider parts.

### 3.2 Wing Flap Animation

Each flap press triggers a wing animation cycle driven by a `FlapAnimation` component containing a `Timer` and current phase:

1. **Downstroke**: Wings rotate from 0 deg to -60 deg over ~0.08s (ease-out for snappy feel)
2. **Upstroke**: Wings rotate from -60 deg to +20 deg over ~0.15s (ease-in)
3. **Settle**: Wings return to 0 deg over ~0.1s (linear)

Interpolate rotation using `Timer::fraction()` with easing curves. When not flapping (falling), wings rest at a slight upward angle (+15 deg) to suggest gliding.

### 3.3 Platforms

Rectangles with subtle border effect (slightly smaller inner rectangle in a lighter shade). Optionally add small triangular "stalactite" shapes hanging below for visual flair.

### 3.4 Lava

Three layers of overlapping animated shapes:
- Back layer: dark red, slow sine wave
- Mid layer: orange, medium sine wave, phase-offset
- Front layer: bright yellow/white (HDR emissive), fast small sine wave

**Implementation**: Use 20-30 thin vertical rectangles per layer. Each rectangle's Y-scale oscillates via `Transform` driven by `sin(x_position * frequency + time * speed)`. No custom mesh generation needed — standard rectangle `Mesh2d` with `Transform` scale changes.

### 3.5 Eggs

Small ellipse shape. Scale pulses using `sin(timer.fraction() * PI * pulse_count)` — faster pulsing as hatch approaches. Color lerps from white toward the enemy tier color based on `timer.fraction()`.

### 3.6 Pterodactyl

Larger angular shape (~3x rider size):
- Body: elongated diamond/hexagon
- Wings: two large triangles, animated with slow flapping
- Head: small triangle at front with a circle "eye"
- Distinguished by size and HDR emissive outline color for bloom glow

### 3.7 Lava Hand

Built from overlapping circles and rectangles forming a blocky claw. Rises from lava surface. Uses lava color palette (orange/red). HDR emissive fingertips.

---

## 4. Visual Effects (Fancy Graphics)

### 4.1 Bloom / Glow

Enable Bevy's `Bloom` post-processing on the camera. Camera setup:

```rust
commands.spawn((
    Camera2d,
    Camera {
        clear_color: ClearColorConfig::Custom(Color::srgb(0.04, 0.04, 0.07)),
        ..default()
    },
    Tonemapping::TonyMcMapface,
    Bloom {
        intensity: 0.3,
        ..Bloom::OLD_SCHOOL  // 1990s arcade glow aesthetic
    },
    DebandDither::Enabled,
));
```

Import: `use bevy::{core_pipeline::tonemapping::{DebandDither, Tonemapping}, post_process::bloom::Bloom};`

HDR emissive colors use values > 1.0 on `ColorMaterial::color` directly (e.g., `Color::srgb(5.0, 1.0, 0.2)`). `ColorMaterial` has no separate emissive field — the bloom post-process extracts bright regions from any color above the threshold.

**Bloom targets** (entities with HDR colors):
- Lance tips (bright glow matching rider color)
- Lava front layer (orange/yellow glow)
- Death particle bursts
- Egg at near-hatch
- Pterodactyl outline

### 4.2 Particle Effects

Lightweight custom particle system. Each particle is a single `Mesh2d` entity with:
- `Particle` component: `velocity: Vec2`, `lifetime: Timer`
- `MeshMaterial2d<ColorMaterial>` with alpha faded based on `lifetime.fraction_remaining()`

A single `update_particles` system ticks all particle timers, applies velocity, fades alpha, and despawns when `lifetime` finishes. Particles are spawned in response to gameplay events (see section 10.2).

| Event | Particle Count | Behavior |
|---|---|---|
| Flap | 3-5 | Small circles downward from wings, fade quickly |
| Kill | 15-20 | Burst outward from death point, enemy color, HDR |
| Egg collect | 8-10 | Spiral upward, gold |
| Egg hatch | 10-12 | Ring of expanding shapes |
| Lava bubble | 1-2 | Occasional circles rising from lava surface |
| Landing dust | 2-4 | Small shapes at feet on platform landing |
| Joust bounce | 6-8 | Bright sparks at collision point, HDR |

**Performance**: Cap total particle entities at ~200. Query `With<Particle>` count before spawning; if at cap, skip ambient particles (lava bubbles, landing dust) but still spawn gameplay-critical ones (kill burst). Each particle is a single entity — at 200 this is trivial for Bevy.

### 4.3 Screen Shake

On high-impact events (kills, lava death, pterodactyl appearance):
- Store `ScreenShake` resource: `intensity: f32`, `decay_timer: Timer`
- Each frame: apply random offset to camera `Transform` scaled by `intensity * decay_timer.fraction_remaining()`
- Exponential decay via the timer's fraction

### 4.4 Trail Effects

Moving entities leave a brief trail:
- A `TrailTimer` component (repeating `Timer`, ~0.05s) on each rider entity
- When timer fires: spawn a small shape at entity's current position with a `TrailFade` component containing a 0.15s `Timer`
- `TrailFade` system: shrink scale and reduce alpha based on `timer.fraction()`, despawn when finished
- Max 3 trail shapes per entity to avoid clutter

### 4.5 Flash on Hit

When a joust bounce occurs (equal height), both entities get a `FlashTimer` component (0.1s `Timer`). While active, override all child material colors to white. On timer finish, restore original colors and remove the component.

### 4.6 Wave Announcement

Large "WAVE X" text as a world-space `Text2d` entity (not UI node) at center screen, Z-layer 10. Animation driven by a `Timer`:
- 0.0-0.3s: scale from 2.0 to 1.0 (ease-out)
- 0.3-1.5s: hold at scale 1.0
- 1.5-2.0s: fade alpha to 0, then despawn

Use HDR text color for bloom glow on the announcement.

### 4.7 Background

Dark background (clear color ~#0A0A12) with:
- Subtle grid of very dim dots (static entities, Z=0, low alpha)
- Occasional "star" twinkle: a few entities with a repeating `Timer` that briefly increases alpha
- Keeps focus on the bright gameplay elements with bloom

---

## 5. State Machine

```
AppState:
  StartScreen -> Playing -> GameOver -> StartScreen

PlayState (SubState of AppState::Playing):
  WaveIntro -> WaveActive -> WaveClear -> WaveIntro (next wave)
                                      \-> triggers GameOver (if no lives)
```

### 5.1 State Definitions

```rust
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum AppState {
    #[default]
    StartScreen,
    Playing,
    GameOver,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
enum PlayState {
    #[default]
    WaveIntro,
    WaveActive,
    WaveClear,
}
```

Register: `app.init_state::<AppState>().add_sub_state::<PlayState>();`

### 5.2 State Behavior

| State | What Happens |
|---|---|
| `StartScreen` | Title, "Press SPACE to start", "Press 2 for 2-player". High score display. Background demo AI. |
| `Playing` | Active gameplay. `PlayState` sub-state drives wave flow. |
| `WaveIntro` | 2s pause, "WAVE X" announcement, enemies spawn. Transition to `WaveActive`. |
| `WaveActive` | Normal gameplay. Transition to `WaveClear` when wave clear condition met. |
| `WaveClear` | 2s celebration pause, bonus tally. Transition to next `WaveIntro` or `GameOver`. |
| `GameOver` | "GAME OVER" overlay on frozen arena, final score, "Press SPACE to restart". |

### 5.3 Entity Cleanup

Use `DespawnOnExit(state)` component (Bevy 0.18's state-scoped entity system) on all entities tied to a state:
- Gameplay entities (riders, enemies, eggs, particles, platforms): `DespawnOnExit(AppState::Playing)`
- Wave announcement text: `DespawnOnExit(PlayState::WaveIntro)`
- Start screen UI: `DespawnOnExit(AppState::StartScreen)`
- Game over overlay: `DespawnOnExit(AppState::GameOver)`

This eliminates manual cleanup systems. Only use `OnExit` schedules for resetting resources, not despawning entities.

---

## 6. Input

### 6.1 Key Bindings

| Action | Player 1 | Player 2 |
|---|---|---|
| Move Left | A / Left Arrow | J |
| Move Right | D / Right Arrow | L |
| Flap | W / Up Arrow / Space | I |

**Concern**: Space is used for both flap (P1) and start game. Disambiguate by state: Space starts on `StartScreen`, flaps on `Playing`. Gate input systems with `run_if(in_state(...))`.

### 6.2 Input Buffering

Flap inputs should feel responsive. Store the last flap press time in a `FlapBuffer` component with a short `Timer` (0.08s). If the entity lands on a platform while the buffer timer hasn't finished, immediately trigger a take-off flap. This prevents the "I pressed flap but nothing happened" frustration.

### 6.3 Gamepad Support (Optional/Future)

Not in initial scope. Note: Bevy's gamepad API would map well (left stick = move, A button = flap).

---

## 7. Scoring

| Event | Points |
|---|---|
| Kill Bounder | 500 |
| Kill Hunter | 750 |
| Kill Shadow Lord | 1000 |
| Collect egg | 250 |
| Kill Pterodactyl | 2000 |
| Survive survival wave | 1500 |
| Extra life | Every 10,000 points |

Cap lives at `MAX_LIVES` (5) to prevent trivialization at high scores.

**High score persistence**: Save to a file outside `assets/` (which is for read-only game assets). Use a platform-appropriate path — e.g., `dirs::data_local_dir()` via the `dirs` crate, or a simple `highscore.dat` in the working directory as a fallback. Store as a single `u32`. Load on startup, save on game over if beaten.

---

## 8. Audio

**No audio in initial implementation.** The spec is visual-first. Audio can be layered in later. The event system (section 10.2) already emits all gameplay events an audio system would subscribe to — no additional hooks needed.

---

## 9. UI Layout

### 9.1 HUD (During Play)

```
+-------------------------------------------------+
| P1: 12500    WAVE 3    LIVES: *** |   P2: 8300  |
|                                                  |
|              [game arena]                        |
|                                                  |
+-------------------------------------------------+
```

- Top bar: Bevy UI `Node` with `FlexDirection::Row`, `JustifyContent::SpaceBetween`
- Score text: update via `Changed<Score>` query filter (only re-render when score changes)
- Lives: shown as small diamond shapes (rider silhouette icons)
- Use `DespawnOnExit(AppState::Playing)` on the HUD root node

### 9.2 Start Screen

```
         +==================+
         |    J O U S T     |
         +==================+

      [animated demo riders jousting]

        Press SPACE to Start
        Press 2 for 2 Players

         High Score: 25000
```

Background: two AI-controlled riders jousting on the actual game arena. Systems run in `StartScreen` state with AI-only input. Gives a preview of gameplay.

### 9.3 Game Over Screen

Semi-transparent overlay on top of frozen gameplay:
```
         GAME OVER

       Final Score: 15750
       High Score:  25000

      Press SPACE to Restart
```

Freeze: stop running `PhysicsSet` and `CombatSet` systems in `GameOver` state (they already gate on `Playing`). Overlay is a UI node with a dark semi-transparent background.

---

## 10. Architecture (Module Layout)

```
src/
  main.rs        -- App setup, plugin registration, state init, system set ordering
  constants.rs   -- All tunable values
  components.rs  -- ECS components (Velocity, Rider, Enemy, Egg, FacingDirection, etc.)
  resources.rs   -- GameState (scores, lives, wave), ScreenShake, WaveDefs
  states.rs      -- AppState, PlayState enums
  physics.rs     -- Gravity, flap, drag, platform collision, screen wrap
  player.rs      -- Player input, spawn/despawn, respawn
  enemy.rs       -- Enemy AI, spawn logic, tier definitions, pterodactyl
  combat.rs      -- Joust collision detection, height comparison, kill/bounce, egg lifecycle
  rendering.rs   -- Shape builders (build_rider, build_egg, etc.), wrap ghost management
  effects.rs     -- Particles, screen shake, flash, trails, bloom setup, lava animation
  waves.rs       -- Wave definitions, progression, spawn scheduling
  ui.rs          -- HUD, start screen, game over, wave announcements
```

13 modules. Each domain module (player, enemy, combat, waves, effects, rendering, ui) exposes a `Plugin` registered in `main.rs`. Shared types live in `components.rs`, `resources.rs`, `states.rs`, `constants.rs`.

**Why not more modules**: Separating `egg.rs` from `combat.rs`, or `particles.rs` from `effects.rs`, or `pterodactyl.rs` from `enemy.rs` would create modules with only 1-2 systems each. Keep related systems together until a module exceeds ~300 lines, then split.

### 10.1 System Sets and Ordering

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet {
    Input,
    Physics,
    Combat,
    Effects,
    Render,
}
```

Configure in `main.rs`:
```
Input -> Physics -> Combat -> ApplyDeferred -> Effects -> Render
```

`ApplyDeferred` between `Combat` and `Effects` ensures entity despawns from kills are flushed before particle systems try to read dead entities.

**Set contents**:

| Set | Systems |
|---|---|
| `Input` | `player_input`, `ai_decision` |
| `Physics` | `apply_gravity`, `apply_flap`, `apply_movement`, `platform_collision`, `screen_wrap` |
| `Combat` | `detect_jousts`, `resolve_jousts`, `collect_eggs`, `hatch_eggs`, `lava_kill`, `hand_behavior` |
| `Effects` | `update_particles`, `screen_shake`, `flash_update`, `trail_spawn`, `trail_fade`, `lava_animate` |
| `Render` | `update_facing`, `animate_wings`, `animate_walk`, `manage_wrap_ghosts` |

All sets gated with `.run_if(in_state(AppState::Playing))` except `Render` (which also runs in `StartScreen` for the demo).

### 10.2 Events and Observers

**Events** (emitted by gameplay systems, consumed by effects/UI/scoring):

| Event | Payload | Emitted By | Consumed By |
|---|---|---|---|
| `FlapEvent` | `entity, position` | `player_input`, `ai_decision` | `effects` (particles) |
| `JoustKillEvent` | `winner, loser, position, enemy_tier` | `combat` | `effects` (particles, shake), `waves` (count), `ui` (score) |
| `JoustBounceEvent` | `entity_a, entity_b, position` | `combat` | `effects` (sparks, flash) |
| `EggCollectEvent` | `position, tier` | `combat` | `effects` (particles), `ui` (score) |
| `EggHatchEvent` | `position, new_tier` | `combat` | `effects` (particles), `enemy` (spawn) |
| `PlayerDeathEvent` | `player_id, position, lives_remaining` | `combat` | `effects` (particles, shake), `player` (respawn), `ui` (lives) |
| `LavaDeathEvent` | `entity, position` | `combat` | `effects` (particles, shake) |
| `LandingEvent` | `entity, position` | `physics` | `effects` (dust particles) |
| `ScoreEvent` | `player_id, points` | `combat` | `ui` (score update, extra life check) |

**Observers**: Use `Observer` + `Trigger` for one-shot entity lifecycle reactions:
- When an enemy entity is despawned: clean up any references (AI target tracking)
- When a `Grounded` component is added: trigger landing effects
- When a `Grounded` component is removed: reset walk animation

This follows CLAUDE.md guidance to prefer observers over `Added<T>`/`RemovedComponents<T>` query patterns.

---

## 11. Edge Cases and Hard Problems

### 11.1 Multi-Body Collision in Single Frame

Three entities collide simultaneously. Entity A beats B (higher), B beats C, but C beats A.

**Resolution**: Two-pass approach. First pass: collect all collision pairs and determine winner/loser for each pair independently. Second pass: any entity that is a loser in ANY pair is marked dead — regardless of whether it also won a different pair. Dead entities still award kills to their victors. Third pass: despawn dead entities, spawn eggs, emit events.

This means mutual kills are possible: if A kills B and B kills C in the same frame, both B and C die. A only dies if it also lost a separate joust.

### 11.2 Platform Edge Precision

An entity standing on a platform walks to the edge. At what point do they fall off?

**Rule**: The entity's center X must be within the platform's `[left, right]` range. Once center passes the edge, remove `Grounded` and let them fall. No partial-on-platform state.

### 11.3 Flap While Rising

Player flaps again while still rising from a previous flap. Each flap adds `FLAP_IMPULSE` but velocity is clamped by `MAX_RISE_SPEED`. Rapid flapping feels additive up to the cap, then has no additional effect until velocity drops. This is correct — do NOT gate flapping on "is falling."

### 11.4 Enemy Stuck on Platform

AI enemy lands on a platform and never takes off. Each enemy has a `GroundTimer` component (per-tier duration: Bounder 4s, Hunter 2s, Shadow Lord 1s). When timer finishes, force a flap.

### 11.5 Entity Spawn Overlap

New wave enemies spawn at top of screen. If a player is near the top, they could immediately collide.

**Solution**: Spawning enemies get `Invincible` with 0.5s `Timer`. Collision systems skip entities with `Invincible`. Visual: alpha oscillates to indicate invincibility.

### 11.6 All Enemies Become Eggs Simultaneously

Player kills all remaining enemies at once. Wave does NOT clear yet — eggs still exist. Wave clear condition: zero entities with `Enemy` component AND zero entities with `Egg` component.

### 11.7 Two Players Both Die on Same Frame

Process both deaths. If either player has remaining lives, only that player respawns. If both have zero lives, game over. If both have lives, both respawn.

### 11.8 Egg on Lava

An egg reaches `LAVA_Y`. Despawn it — no hatching, no points. This is a valid strategy: knock enemies near lava so their eggs fall in.

### 11.9 Pterodactyl and Screen Wrap

The pterodactyl is large (~60px wide). Its wrap ghost rendering must handle partial visibility on both edges. Its collision uses modular distance like all other entities.

### 11.10 Respawn Safety

Player respawns at "safest" location: scan platforms top-to-bottom, pick the one farthest from any enemy (using modular distance). If all platforms have nearby enemies, use top-center as fallback.

---

## 12. Technical Concerns

### 12.1 Lava Animation Performance

Lava uses ~60-90 thin rectangle entities (3 layers x 20-30 per layer). Each frame, a system updates their `Transform.scale.y` based on a sine function. This is cheap — just Transform writes, no mesh regeneration, no GPU resource allocation.

### 12.2 Z-Ordering

2D rendering order via `Transform.translation.z`:

| Z | Layer |
|---|---|
| 0.0 | Background (stars, grid) |
| 1.0 | Lava back layer |
| 2.0 | Platforms |
| 3.0 | Eggs |
| 4.0 | Lava hand |
| 5.0 | Trails |
| 6.0 | Enemies |
| 7.0 | Players |
| 8.0 | Lava front layer (overlaps entities near bottom for depth) |
| 9.0 | Particles |
| 10.0 | Wave announcement text |

UI nodes render in their own pass and don't need Z-ordering against world entities.

### 12.3 Entity Count Budget

| Category | Count |
|---|---|
| Players (2 x ~8 shapes) | 16 |
| Enemies (8 x ~8 shapes) | 64 |
| Eggs (8 x 1 shape) | 8 |
| Pterodactyl (~6 shapes) | 6 |
| Platforms (~10 shapes) | 10 |
| Lava (3 layers x 25) | 75 |
| Particles (cap) | 200 |
| Trails (~30) | 30 |
| Wrap ghosts (~10) | 10 |
| Background (~20) | 20 |
| UI (~20) | 20 |
| **Total** | **~459** |

Well within Bevy's comfort zone. No optimization needed.

### 12.4 Mesh and Material Reuse

Pre-create shared `Handle<Mesh>` and `Handle<ColorMaterial>` in a startup system, store them in a `GameAssets` resource. Reuse for all entities of the same type. Avoids redundant `meshes.add()` / `materials.add()` calls per spawn.

Example: one `circle_mesh` handle reused for all rider heads, all particles, all egg shapes. One `rect_mesh` for all body parts, platforms, lava strips. Create per-color materials for each entity tier.

### 12.5 Determinism

Not aiming for deterministic simulation. AI randomness and float physics mean replays won't match. Acceptable for an arcade game.

---

## 13. Tradeoffs and Decisions

### 13.1 Custom Physics vs Physics Engine (e.g., Rapier)

**Decision: Custom physics.**

Joust's physics are simple (gravity, impulse, drag, circle collisions) but quirky (one-way platforms, height-based combat, screen wrap modular distance). A physics engine would fight us on all three. Custom code is <300 lines and gives full control over the "feel."

### 13.2 Faithful vs Modernized

**Decision: Modernized aesthetic, faithful mechanics.**

Gameplay rules match the original closely (flapping, height combat, eggs, waves, pterodactyl). Visuals are modernized (bloom, particles, smooth animation). No gameplay changes that would alter the fundamental Joust feel.

### 13.3 Shape Complexity vs Readability

**Decision: 6-8 shapes per entity, distinct silhouettes.**

More shapes = better looking but harder to distinguish at speed. Fewer = clear but ugly. 6-8 is the sweet spot where entities read as "rider on bird" without becoming visual noise. Color does most of the identification work.

### 13.4 Particle Count vs Visual Impact

**Decision: Cap at 200, bias toward fewer-but-brighter particles.**

Fewer HDR/bloom particles are individually more visible than many dim ones. Kill bursts (15-20) are the heaviest single event. Ambient effects (lava, dust) are optional and shed first at cap.

### 13.5 2-Player on Same Keyboard

**Decision: Support it, WASD + IJK split.**

Same-keyboard 2P is authentic to the arcade original. The WASD/IJK split keeps hands separate. Gamepad support is out of scope for v1.

### 13.6 `rand` Dependency

**Decision: Add `rand` crate.**

Needed for AI jitter, wander direction, spawn position variation, particle spread. Bevy does not bundle an RNG. Use `rand::thread_rng()` — no need for deterministic seeded RNG since we don't need replay.

### 13.7 High Score Persistence

**Decision: Simple file I/O in working directory.**

Write a single `u32` to `highscore.dat` in the current working directory. No external crate needed for the fallback approach. Optional: use `dirs` crate for a proper platform path.

---

## 14. Implementation Order

Recommended build sequence. Each step produces a testable artifact.

1. **Module scaffolding + state machine** — Create all source files, `AppState`/`PlayState` enums, plugin stubs, system set ordering in `main.rs`. Verify state transitions with placeholder text.
2. **Window + camera + bloom** — Dark window with `Bloom` enabled, `Tonemapping`, background stars. Verify HDR colors glow.
3. **Platforms + lava rendering** — Static platform rectangles, animated lava strips. Verify visual layout.
4. **Player shape + flight physics** — Flap, gravity, drag, delta-time. Tune until it feels right. **This is the longest step.**
5. **Platform collision + grounded state** — One-way landing, `Grounded` component, walking.
6. **Screen wrap** — Modular distance, teleport, wrap ghost rendering.
7. **Wing/walk animation** — Flap animation cycle, walk leg alternation, facing direction flip.
8. **Enemy shapes + AI** — Basic wander AI, then pursue. Spawn/despawn.
9. **Combat system** — Height comparison, kill, bounce, invincibility.
10. **Egg system** — Spawn on kill, timer, hatch, collect.
11. **Particles + effects** — Flap dust, death burst, screen shake, trails, flash.
12. **Wave system** — Wave definitions, progression, spawning, clear condition.
13. **Lava hand** — Targeting, rise/retract, kill.
14. **Pterodactyl** — Spawn trigger, sine-wave flight, tiny hitbox.
15. **UI** — HUD, start screen (with demo), game over overlay.
16. **Scoring + lives** — Point tracking, extra lives, high score file, respawn.
17. **2-player support** — Second input set, second player entity, per-player scores/lives.
18. **Polish** — Animation easing, difficulty balancing, visual tuning, edge case testing.

---

## 15. Constants (Initial Values)

```
// Window
WINDOW_WIDTH:           1200.0
WINDOW_HEIGHT:          900.0

// Arena (inset from window for HUD)
ARENA_TOP:              410.0
ARENA_BOTTOM:           -410.0   // above lava
ARENA_LEFT:             -600.0
ARENA_RIGHT:            600.0

// Flight physics
GRAVITY:                980.0
FLAP_IMPULSE:           320.0
MAX_RISE_SPEED:         400.0
MAX_FALL_SPEED:         600.0
HORIZONTAL_ACCEL:       800.0
WALK_ACCEL:             400.0
HORIZONTAL_DRAG:        400.0
MAX_HORIZONTAL_SPEED:   300.0
WALK_MAX_SPEED:         150.0

// Platform
PLATFORM_SNAP_DISTANCE: 3.0
PLATFORM_THICKNESS:     12.0

// Combat
JOUST_THRESHOLD:        8.0
BOUNCE_HORIZONTAL:      250.0
BOUNCE_VERTICAL:        150.0
INVINCIBILITY_TIME:     0.3
RESPAWN_INVINCIBILITY:  2.0

// Collision radii
COLLISION_RADIUS_RIDER: 18.0
COLLISION_RADIUS_EGG:   10.0
COLLISION_RADIUS_PTERO: 25.0
COLLISION_RADIUS_PTERO_HEAD: 8.0
COLLISION_RADIUS_LANCE: 6.0

// Eggs
EGG_HATCH_TIME_BASE:    15.0

// Lava
LAVA_Y:                 -410.0
HAND_DETECT_RANGE:      100.0
HAND_DETECT_Y:          -350.0
HAND_RISE_SPEED:        200.0
HAND_COOLDOWN:          5.0

// Particles
PARTICLE_CAP:           200

// Scoring
MAX_LIVES:              5
EXTRA_LIFE_INTERVAL:    10000

// Waves
PTERODACTYL_BASE_TIMER: 90.0
PTERODACTYL_MIN_TIMER:  60.0

// Screen wrap
WRAP_MARGIN:            20.0
```

All values in `constants.rs`. All `f32` except `PARTICLE_CAP` (`usize`), `MAX_LIVES` (`u32`), `EXTRA_LIFE_INTERVAL` (`u32`). Expect heavy iteration on physics values.

---

## 16. Dependencies

```toml
[dependencies]
bevy = { version = "0.18.1", features = ["dynamic_linking"] }
rand = "0.9"
```

Optional (for proper high score path): `dirs = "6"`.
