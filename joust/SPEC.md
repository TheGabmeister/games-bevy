# Joust — Modernized 2D (Bevy 0.18.1)

A modernized 2D recreation of Williams Electronics' Joust (1982) using Bevy with primitive shapes only — no sprites or textures. Custom animations and visual effects make up for the lack of art assets.

---

## 1. Game Overview

Single-screen arena. One or two players ride lance-wielding birds. Flight is controlled by repeated flapping (button mashing for altitude). Combat resolves by height — the higher lance wins. Defeated enemies drop eggs that hatch into stronger foes if not collected. Waves of increasing difficulty, culminating in survival rounds with an unkillable pterodactyl.

**Window**: 1200 x 900, fixed. No camera zoom or scroll.

---

## 2. Core Mechanics

### 2.1 Flight Physics

This is the single most important system. If flapping doesn't feel right, nothing works.

**Model**: Custom physics, no physics engine. Each flapping entity has:

| Property | Description |
|---|---|
| `velocity: Vec2` | Current movement vector |
| `gravity: f32` | Constant downward acceleration (~980 px/s^2, tunable) |
| `flap_impulse: f32` | Upward velocity added per flap (~320 px/s, tunable) |
| `max_rise_speed: f32` | Clamp on upward velocity to prevent rocket launches |
| `max_fall_speed: f32` | Terminal velocity downward |
| `horizontal_accel: f32` | Left/right acceleration |
| `horizontal_drag: f32` | Deceleration when no input (creates the signature slide) |
| `max_horizontal_speed: f32` | Horizontal speed cap |

**Flap behavior**: Each flap press adds `flap_impulse` to vertical velocity (clamped by `max_rise_speed`). Gravity applies every frame. Rapid tapping = hover. Single tap = small hop. No tap = fall. There is NO jump — only accumulated flap impulses against gravity.

**Horizontal movement**: Acceleration-based with drag. The player cannot stop instantly — they slide. This momentum is core to the jousting feel. Direction input accelerates; releasing input applies drag toward zero.

**Ground state**: When on a platform, vertical velocity is zeroed, gravity stops, and the entity can walk left/right with the same horizontal acceleration model. A flap while grounded launches upward.

**Tuning concern**: The flap-to-gravity ratio determines the entire feel. Too floaty = no tension. Too heavy = frustrating. The original Joust used roughly 4-5 flaps per second to maintain altitude. Target: ~5 flaps/sec to hover, with visible bob between flaps.

### 2.2 Screen Wrap

Entities wrap horizontally. Exit the left edge, appear on the right, and vice versa.

**Implementation**: When `entity.x > WINDOW_WIDTH / 2 + WRAP_MARGIN`, teleport to `-WINDOW_WIDTH / 2 - WRAP_MARGIN` (and inverse). `WRAP_MARGIN` is half the entity's visual width so the entity fully exits before reappearing.

**Hard problem — collision across the wrap boundary**: Two entities near opposite edges are actually adjacent. Solutions:

- **Ghost entity approach**: When an entity is within `WRAP_MARGIN + COLLISION_RADIUS` of an edge, spawn a temporary ghost at the mirrored position. Run collision checks against ghosts. Despawn ghosts each frame.
- **Chosen approach**: Modular distance — compute horizontal distance as `min(|x1 - x2|, WINDOW_WIDTH - |x1 - x2|)`. This is cheaper than ghost entities and sufficient for a single-screen game. Apply this to ALL distance checks (collision, AI targeting, rendering).

**Hard problem — split rendering**: When an entity is partially off-screen on one side, the other half should be visible on the opposite side. Render the entity at both its actual position and its wrapped position when within `WRAP_MARGIN` of an edge. This means some entities briefly have two visual representations.

### 2.3 Platforms

Platforms are one-way: entities can fly up through them but land on top.

**Collision rule**: Only apply platform collision when:
1. Entity's bottom edge is within a small tolerance of the platform's top edge, AND
2. Entity's vertical velocity is downward (or zero)

This means you can flap up through a platform from below, but land on it from above. Walking off a platform edge causes falling.

**Platform layout**: Static per wave. Defined as arrays of `(x, y, width)`. The classic layout:
- Two long platforms mid-height on left and right
- One short platform top-center
- Two small ledges at upper-left and upper-right
- Ground-level walkway along the very bottom (above lava)
- All platforms have small lips/edges rendered as rectangles

**Edge case — landing precision**: Entities should snap to platform top when close enough (within 2-4px) and moving downward, to avoid jitter. Use a `Grounded` component to track state and avoid re-checking every frame.

### 2.4 Combat (Height-Based Jousting)

The core combat rule: when two riders collide, compare the Y position of their lances (top of the rider). Higher lance wins. Loser is destroyed and drops an egg.

**Height comparison**:
- `lance_y = entity.y + LANCE_Y_OFFSET` (top of the rider shape)
- If `lance_y_a - lance_y_b > JOUST_THRESHOLD`: A wins
- If `lance_y_b - lance_y_a > JOUST_THRESHOLD`: B wins
- If `abs(lance_y_a - lance_y_b) <= JOUST_THRESHOLD`: Both bounce off (no kill)

`JOUST_THRESHOLD` (~6-10px): The "equal height" dead zone. Too small = feels random. Too large = hard to get kills.

**Bounce on equal height**: Both entities receive opposing horizontal impulse and small vertical impulse. Brief invincibility frames (~0.3s) prevent immediate re-collision.

**Collision shape**: Use circle-circle collision for the riders. Each rider has a collision radius. Compute modular distance (for screen wrap), check if < sum of radii.

**Edge cases**:
- **Simultaneous multi-collision**: Process all collision pairs, mark kills, then apply. Don't let order-of-processing affect outcomes.
- **Player vs player (2P mode)**: Same rules apply. Friendly fire is ON (faithful to original). Both players bounce on equal height.
- **Velocity-based tiebreaker**: When heights are equal, the entity with greater downward velocity could lose (optional modernization). Original had pure bounce.

### 2.5 Egg System

Defeated enemies become eggs. Eggs are collectible for bonus points. Uncollected eggs hatch after a timer into a stronger enemy type.

**Lifecycle**:
1. Enemy dies → egg spawns at death position with enemy's velocity
2. Egg falls (has gravity, no flapping), lands on platforms
3. Egg sits for `EGG_HATCH_TIME` seconds (original: ~10-15s, decreases in later waves)
4. If collected (player touches egg): award points, despawn
5. If timer expires: egg hatches → spawn enemy one tier higher (Bounder→Hunter→Shadow Lord)

**Egg physics**: Gravity + platform landing only. No horizontal movement once landed. Eggs on platforms near an edge can fall off if nudged (optional — adds depth but increases complexity).

**Visual**: Egg shape changes/pulses as hatch timer approaches zero. Color shifts from white → enemy color.

**Edge case**: Egg falls into lava → destroyed, no hatch. Egg is on a ledge and gets knocked off by a player walking into it → falls.

### 2.6 Lava

The bottom of the screen is lava. Anything that touches it is destroyed.

**Lava surface**: Animated sine wave along the bottom ~60px tall. Entities below `LAVA_Y` are destroyed on contact.

**Lava Troll (hand)**: When an entity hovers near lava level for too long, a hand rises from the lava and grabs downward. The hand targets the nearest entity within a horizontal range of the hand's position. Contact = instant death.

**Hand behavior**:
1. Track nearest entity within `HAND_DETECT_RANGE` of hand's X, below `HAND_DETECT_Y`
2. Rise at `HAND_RISE_SPEED` toward target
3. If target escapes range, hand retracts
4. If hand contacts target, target is destroyed
5. Cooldown after grab attempt: `HAND_COOLDOWN` seconds

**Edge case**: Two entities near lava — hand targets nearest. Hand should not clip through platforms.

### 2.7 Enemies

Three tiers of increasing difficulty:

| Type | Color | Speed | AI Aggression | Flap Rate |
|---|---|---|---|---|
| Bounder (Red) | Red tones | Slow | Wanders, avoids player | Low |
| Hunter (Silver) | White/silver | Medium | Seeks player | Medium |
| Shadow Lord (Blue) | Dark blue | Fast | Aggressive pursuit | High |

**AI behavior** — each enemy runs a simple state machine:

1. **Wander**: Pick random direction, flap occasionally, change direction at edges or after timer
2. **Pursue**: Fly toward player. Higher aggression = more direct path
3. **Evade**: If player is above and approaching, try to gain altitude or dodge
4. **Land**: If on platform, walk briefly, then take off

**AI flapping**: Enemies flap at their tier's rate. Higher tiers flap more precisely to maintain altitude near the player. AI never flaps perfectly — add small random delays to prevent robotic behavior.

**Spawn pattern**: Enemies spawn from the top of the screen in waves. Brief materialization animation (circles expanding from a point). New enemies have brief invincibility (0.5s) to prevent spawn-camping.

**Edge case — AI and screen wrap**: AI distance calculations must use modular distance, or enemies will fly away from a nearby player who happens to be across the wrap boundary.

### 2.8 Pterodactyl

Appears if a wave takes too long (e.g., 60s with 1 enemy remaining, or 90s total). Nearly unkillable — can only be hit by a perfectly timed lance to the mouth (tiny hitbox).

**Behavior**: Flies in sine-wave patterns, targeting the player. Ignores platforms (flies through them). Very fast. Contact = instant death to rider.

**Kill condition**: Player's lance must contact the pterodactyl's head hitbox (a small circle, ~10px radius at the front of a ~60px body). This is intentionally very hard.

**Visual**: Larger shape than riders. Distinct angular/triangular wing shapes. Flashing/pulsing effect.

### 2.9 Wave System

| Wave | Enemies | Types | Egg Hatch Time | Pterodactyl Timer |
|---|---|---|---|---|
| 1 | 3 | Bounders only | 15s | None |
| 2 | 4 | Bounders + 1 Hunter | 13s | None |
| 3 | 5 | Mix | 11s | 90s |
| 4 | 6 | More Hunters | 10s | 80s |
| 5+ | 6-8 | Increasing Shadow Lords | 8s (min) | 60s (min) |

**Between waves**: Brief pause (2s), "WAVE X" text, all eggs cleared, platform layout may change.

**Survival wave** (every 5th wave): No enemies. Pterodactyl spawns immediately. Survive for 30 seconds. Bonus points for killing it.

---

## 3. Entity Rendering (Primitive Shapes Only)

All entities built from Bevy `Mesh2d` + `MeshMaterial2d` with `ColorMaterial`. Each entity is a parent with child shape entities.

### 3.1 Player / Enemy Rider

Built from ~6-8 primitive shapes:

```
    [diamond]         ← lance tip (rotated square)
   /
  [circle]            ← head
  [rectangle]         ← body/torso
  [trapezoid/rect]    ← bird body (wider rectangle below torso)
 / \
[ellipse] [ellipse]   ← wings (animated rotation for flapping)
  | |
 [rect][rect]         ← legs/feet
```

- **Body**: Rectangle (torso) + larger rectangle (bird body)
- **Head**: Circle on top of torso
- **Lance**: Thin rotated rectangle extending from the head in the facing direction
- **Wings**: Two ellipses parented to the bird body, animated via rotation to simulate flapping
- **Legs**: Two small rectangles below bird body, animated to walk when grounded

**Facing direction**: All child shapes mirror horizontally when direction changes. Lance points in movement direction.

**Color scheme**: Player 1 = warm yellow/gold. Player 2 = cyan/teal. Enemies use their tier color for body, with consistent grey for rider parts.

### 3.2 Wing Flap Animation

Each flap press triggers a wing animation cycle:
1. Wings rotate from 0 degrees (horizontal/resting) to -60 degrees (downstroke) over ~0.08s
2. Wings rotate from -60 degrees back to +20 degrees (upstroke) over ~0.15s
3. Wings settle back to 0 degrees over ~0.1s

Use a `FlapAnimationTimer` component to drive this. Interpolate rotation with easing (ease-out for downstroke snap, ease-in for recovery).

When not flapping (falling), wings rest at a slight upward angle (+15 degrees) to suggest gliding.

### 3.3 Platforms

Rectangles with subtle border effect (slightly smaller inner rectangle in a lighter shade). Optionally add small triangular "stalactite" shapes hanging below for visual flair.

### 3.4 Lava

Multiple overlapping sine-wave shapes (using Bevy mesh generation):
- Back layer: dark red, slow sine wave
- Mid layer: orange, medium sine wave offset
- Front layer: bright yellow/white, fast small sine wave

Generate these as custom meshes each frame or use a few stacked rectangles with vertex displacement. For simplicity: use 20-30 thin vertical rectangles per layer whose heights oscillate with sine functions at different frequencies and phases.

### 3.5 Eggs

Small oval/ellipse shape. Pulses (scale oscillation) as hatch timer progresses. Color shifts from white toward the enemy tier color.

### 3.6 Pterodactyl

Larger angular shape:
- Body: elongated diamond/hexagon
- Wings: two large triangles, animated with slow flapping
- Head: small triangle at front with a circle "eye"
- Distinguished by size (~3x a rider) and distinct dark color with bright outline

### 3.7 Lava Hand

Built from overlapping circles and rectangles to form a blocky claw shape. Rises from lava surface. Uses lava color palette (orange/red).

---

## 4. Visual Effects (Fancy Graphics)

### 4.1 Bloom / Glow

Enable Bevy's `Bloom` post-processing on the camera. Assign emissive materials (HDR colors with values > 1.0) to:
- Lance tips (bright glow matching rider color)
- Lava surface (orange/yellow glow)
- Death explosions
- Egg hatch moments
- Score text

This creates a neon-arcade aesthetic from pure shapes. Key tuning: `BloomSettings { intensity, low_frequency_boost, threshold }`. Start with low intensity (~0.3) to avoid washing out.

### 4.2 Particle Effects

Implement a lightweight custom particle system (no external crate). Each particle is a small shape entity with velocity, lifetime, and color fade.

| Event | Particle Type |
|---|---|
| Flap | 3-5 small circles downward from wings, fade to transparent |
| Kill | 15-20 shapes burst outward from death point, enemy color |
| Egg collect | 8-10 shapes spiral upward, gold |
| Egg hatch | Ring of expanding shapes |
| Lava bubble | Occasional circles rising from lava surface and popping |
| Landing dust | 2-4 small shapes at feet on platform landing |
| Joust bounce | Sparks (bright small shapes) at collision point |

**Implementation**: A `Particle` component with `velocity`, `lifetime`, `max_lifetime`. A single system ticks all particles, applies velocity, fades alpha based on remaining lifetime, despawns at zero. Spawn particles via events.

**Performance concern**: Cap total particles at ~200. Use a ring buffer or despawn oldest when limit is hit. Each particle is a single `Mesh2d` entity — at 200 this is fine for Bevy.

### 4.3 Screen Shake

On high-impact events (kills, lava death, pterodactyl appearance):
- Apply small random offset to camera transform for ~0.2s
- Decay offset exponentially
- Store shake state in a resource, not per-entity

### 4.4 Trail Effects

Moving entities leave a brief trail:
- Spawn small fading shapes at entity's previous position every N frames
- 3-5 trail segments per entity, each fading and shrinking
- Use entity color at reduced alpha

Keep this subtle — 3 trail shapes per entity max, or it clutters the screen.

### 4.5 Flash on Hit

When a joust bounce occurs (equal height), both entities flash white for 2 frames. Set material color to white, then restore. Use a `FlashTimer` component.

### 4.6 Wave Announcement

Large text "WAVE X" with scale animation (start large, settle to normal, then fade out). Use Bevy UI text with transform animation. Emissive color for bloom glow on the text.

### 4.7 Background

Dark background (near-black, #0A0A12) with:
- Subtle grid of very dim dots or lines (parallax optional, but not needed for single-screen)
- Occasional "star" twinkle (small shape that briefly increases alpha)
- Keeps the focus on the bright gameplay elements with bloom

---

## 5. State Machine

```
StartScreen → Playing → GameOver
                ↑          |
                └──────────┘
             (restart)

Playing sub-states:
  WaveIntro → WaveActive → WaveClear → (next wave or GameOver)
```

**States**:

| State | What Happens |
|---|---|
| `StartScreen` | Title text, "Press SPACE to start", "Press 2 for 2-player". High score display. |
| `Playing` | Active gameplay. Sub-states handle wave transitions. |
| `WaveIntro` | Brief pause, "WAVE X" announcement, enemies spawn. |
| `WaveActive` | Normal gameplay until all enemies and eggs are cleared. |
| `WaveClear` | Brief celebration pause, bonus tally. |
| `GameOver` | "GAME OVER" text, final score, "Press SPACE to restart". |

Use Bevy `States` with `SubStates` for the wave sub-states. `StateScoped` entities for cleanup.

---

## 6. Input

### 6.1 Key Bindings

| Action | Player 1 | Player 2 |
|---|---|---|
| Move Left | A / Left Arrow | J |
| Move Right | D / Right Arrow | L |
| Flap | W / Up Arrow / Space | I |

**Concern**: Space is used for both flap (P1) and start game. Disambiguate by state: Space = start on `StartScreen`, Space = flap on `Playing`.

### 6.2 Input Buffering

Flap inputs should feel responsive. If a flap is pressed within 2 frames of landing on a platform, treat it as an immediate take-off flap. This prevents the "I pressed flap but nothing happened" frustration.

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

**High score**: Persist to a local file (`assets/highscore.dat` or similar). Load on startup, save on game over if beaten.

---

## 8. Audio

**No audio in initial implementation.** The spec is visual-first. Audio can be layered in later (flap sound, collision sound, death sound, wave music). Placeholder hooks: emit events (`FlapEvent`, `KillEvent`, etc.) that an audio system can subscribe to.

---

## 9. UI Layout

### 9.1 HUD (During Play)

```
┌─────────────────────────────────────────────┐
│ P1: 12500    WAVE 3    LIVES: ♦♦♦    P2: 0  │
│                                              │
│              [game arena]                    │
│                                              │
└─────────────────────────────────────────────┘
```

- Top bar: score(s), wave number, lives (shown as small rider shape icons)
- Bottom: lava
- Use Bevy UI nodes for HUD, absolutely positioned at top

### 9.2 Start Screen

```
         ╔═══════════════╗
         ║    J O U S T  ║
         ╚═══════════════╝

      [animated riders jousting]

        Press SPACE to Start
        Press 2 for 2 Players

         High Score: 25000
```

Animated demo of two AI riders jousting in the background.

### 9.3 Game Over Screen

Overlay on top of frozen gameplay:
```
         GAME OVER

       Final Score: 15750
       High Score:  25000

      Press SPACE to Restart
```

---

## 10. Architecture (Module Layout)

```
src/
  main.rs          — App setup, plugin registration, state init
  constants.rs     — All tunable values
  components.rs    — ECS components (Velocity, Rider, Enemy, Egg, etc.)
  resources.rs     — GameState (scores, lives, wave), ParticlePool
  states.rs        — AppState + PlayState enums
  physics.rs       — Gravity, flap, drag, platform collision, screen wrap
  player.rs        — Player input handling, spawn/despawn
  enemy.rs         — Enemy AI, spawn logic, tier definitions
  combat.rs        — Joust collision detection, height comparison, kill/bounce
  egg.rs           — Egg lifecycle, hatch timer, collection
  hazards.rs       — Lava rendering, lava hand behavior
  pterodactyl.rs   — Pterodactyl spawn, behavior, tiny hitbox
  rendering.rs     — Entity shape builders (build_rider_shape, build_egg_shape, etc.)
  particles.rs     — Particle spawning, lifecycle, pool management
  effects.rs       — Screen shake, flash, trails, bloom setup
  waves.rs         — Wave definitions, progression, spawn scheduling
  ui.rs            — HUD, start screen, game over screen, wave announcements
  score.rs         — Score tracking, high score persistence
```

### 10.1 System Ordering

```
InputSet → PhysicsSet → CombatSet → EffectsSet → RenderSet

InputSet:   player_input, ai_decision
PhysicsSet: apply_gravity, apply_flap, apply_movement, platform_collision, screen_wrap
CombatSet:  detect_collisions, resolve_jousts, spawn_eggs, collect_eggs, hatch_eggs
EffectsSet: spawn_particles, update_particles, screen_shake, flash_update, trail_update
RenderSet:  update_facing, animate_wings, animate_walk, update_lava
```

`ApplyDeferred` between `CombatSet` and `EffectsSet` to ensure despawns are flushed before effects reference entities.

---

## 11. Edge Cases and Hard Problems

### 11.1 Multi-Body Collision in Single Frame

Three entities collide simultaneously. Entity A beats B (higher), B beats C, but C beats A.

**Resolution**: Process all pairs. If an entity is killed in any pair, it's dead — don't process its "wins." First pass: collect all collision pairs and outcomes. Second pass: mark deaths (an entity dies if it loses ANY joust). Third pass: apply deaths and spawn eggs. This means a dead entity can't also score a kill in the same frame.

### 11.2 Platform Edge Precision

An entity standing on a platform walks to the edge. At what point do they fall off?

**Rule**: The entity's center (X position) must be within the platform's X range. Once center passes the edge, they fall. No partial-on-platform state.

### 11.3 Flap While Rising

Player flaps again while still rising from a previous flap. Each flap adds impulse but is clamped by `max_rise_speed`. Rapid flapping feels additive up to the cap, then does nothing until velocity drops. This is correct — don't gate flapping on "is falling."

### 11.4 Enemy Stuck on Platform

AI enemy lands on a platform and never takes off. Add a max ground time timer per enemy (e.g., 2-4s). After timer, force a flap. Shorter timer for higher tiers.

### 11.5 Entity Spawn Overlap

New wave enemies spawn at top of screen. If a player is near the top, they could immediately collide.

**Solution**: Spawning enemies have 0.5s invincibility (and visual indicator — blinking/pulsing). They phase through other entities during this time.

### 11.6 All Enemies Become Eggs Simultaneously

Player manages to kill all remaining enemies at once. Wave should not clear until all eggs are also collected or hatched-then-killed. Wave clear condition: zero living enemies AND zero eggs.

### 11.7 Two Players Both Die on Same Frame

Game over only triggers when all players are dead. If both P1 and P2 die simultaneously, it's game over. If one dies and the other has lives, only the dead one respawns.

### 11.8 Egg on Lava

An egg drops and reaches lava Y. It's destroyed — no hatching. This is a valid strategy: knock enemies off low platforms so their eggs fall into lava.

### 11.9 Pterodactyl and Screen Wrap

The pterodactyl is large (~60px wide). Its wrap rendering must handle partial visibility on both edges. Its collision must use modular distance.

### 11.10 Score Overflow / Extra Lives

At 10,000-point intervals, extra lives are awarded. With high scores, this could give many lives. Cap lives at 5 (or 9) to prevent trivialization.

---

## 12. Technical Concerns

### 12.1 Custom Mesh Generation for Lava

Lava waves require generating a mesh each frame (or using many small rectangles). Regenerating a mesh every frame:
- Use `Meshes.add()` sparingly — it allocates a new GPU resource each time.
- **Preferred approach**: Pre-allocate the lava mesh, then update vertex positions each frame via direct mesh mutation. Bevy 0.18 supports `Mesh::attribute_mut()` for this.
- Fallback: Use 30 thin rectangles per lava layer whose Y-scale oscillates via Transform. Simpler, no custom mesh code, visually convincing enough.

### 12.2 Z-Ordering

2D rendering order matters. Use `Transform.translation.z` to layer:

| Z | Layer |
|---|---|
| 0 | Background |
| 1 | Lava back layer |
| 2 | Platforms |
| 3 | Eggs |
| 4 | Lava hand |
| 5 | Trails, particles (behind entities) |
| 6 | Enemies |
| 7 | Players |
| 8 | Lava front layer (overlaps entities near bottom) |
| 9 | Particles (front) |
| 10 | UI |

### 12.3 Entity Count Budget

Estimate max entities:
- 2 players x ~8 shapes = 16
- 8 enemies x ~8 shapes = 64
- 8 eggs x 1 shape = 8
- 1 pterodactyl x ~6 shapes = 6
- ~10 platform shapes = 10
- ~60 lava shapes (3 layers x 20) = 60
- ~200 particles = 200
- ~20 trails = 20
- UI entities ~20 = 20
- **Total: ~424 entities**

This is well within Bevy's comfort zone. No optimization needed.

### 12.4 Bloom Configuration

Bloom affects all entities. To control what glows, use HDR colors:
- Entities that should glow: color values > 1.0 (e.g., `Color::linear_rgb(2.0, 0.5, 0.0)`)
- Entities that shouldn't glow: standard 0.0-1.0 range

Need `Camera2d` with `Camera { hdr: true, .. }` and `Bloom` component. Background must be dark or bloom washes out.

### 12.5 Determinism

Not aiming for deterministic simulation. AI randomness and float physics mean replays won't match. This is acceptable for an arcade game. If replays were needed, would need fixed-point math and seeded RNG.

---

## 13. Tradeoffs and Decisions

### 13.1 Custom Physics vs Physics Engine (e.g., Rapier)

**Decision: Custom physics.**

Joust's physics are simple (gravity, impulse, drag, AABB/circle collisions) but quirky (one-way platforms, height-based combat, screen wrap). A physics engine would fight us on one-way platforms, the wrap-around, and the height-comparison combat. Custom physics code is <300 lines and gives full control over the "feel."

### 13.2 Faithful vs Modernized

**Decision: Modernized aesthetic, faithful mechanics.**

Gameplay rules match the original closely (flapping, height combat, eggs, waves, pterodactyl). Visuals are modernized (bloom, particles, smooth animation). No gameplay changes that would alter the fundamental Joust feel (no dashing, no power-ups, no new enemy types).

### 13.3 Shape Complexity vs Readability

**Decision: 6-8 shapes per entity, distinct silhouettes.**

More shapes = better looking but harder to distinguish at speed. Fewer shapes = clear but ugly. 6-8 is the sweet spot where entities read as "rider on bird" without becoming visual noise. Color does most of the identification work.

### 13.4 Particle Count vs Performance vs Visual Impact

**Decision: Cap at 200, bias toward fewer-but-brighter particles.**

More particles look better in screenshots but muddy gameplay. Use emissive/bloom particles (fewer needed since each is more visible). Heavy effects (death bursts) use 15-20 particles; ambient effects (lava bubbles) use 1-2.

### 13.5 2-Player on Same Keyboard

**Decision: Support it, WASD + IJK split.**

Same-keyboard 2P is authentic to the arcade original (which had two sets of controls on one cabinet). The WASD/IJK split keeps hands separate. Gamepad support is out of scope for v1.

### 13.6 High Score Persistence

**Decision: Simple file I/O, not a database.**

Write a single `u32` to a file. Read on startup, write on game over if beaten. No leaderboard, no name entry (those can be v2).

---

## 14. Implementation Order

Recommended build sequence, each step producing a testable artifact:

1. **Window + camera + bloom** — dark window with bloom enabled, lava rendering at bottom
2. **Platforms** — static rectangles, verify visual layout
3. **Player shape + flight physics** — flap, gravity, drag. Tune until it feels right. This is the longest step.
4. **Platform collision** — one-way landing, ground state, walking
5. **Screen wrap** — horizontal wrap with split rendering
6. **Enemy shapes + AI** — basic wander AI, then pursue
7. **Combat system** — height comparison, kill, bounce
8. **Egg system** — spawn, timer, hatch, collect
9. **Wave system** — wave definitions, progression, spawning
10. **Particles + effects** — flap dust, death burst, screen shake, trails
11. **Lava hand** — hazard behavior
12. **Pterodactyl** — spawn trigger, behavior, tiny hitbox
13. **UI** — HUD, start screen, game over
14. **Scoring + lives** — point tracking, extra lives, high score persistence
15. **2-player support** — second input set, second player entity
16. **Polish** — animation easing, visual tuning, difficulty balancing

---

## 15. Constants (Initial Values)

```
WINDOW_WIDTH:       1200
WINDOW_HEIGHT:      900
GRAVITY:            980.0
FLAP_IMPULSE:       320.0
MAX_RISE_SPEED:     400.0
MAX_FALL_SPEED:     600.0
HORIZONTAL_ACCEL:   800.0
HORIZONTAL_DRAG:    400.0
MAX_HORIZONTAL_SPEED: 300.0
JOUST_THRESHOLD:    8.0
COLLISION_RADIUS_RIDER: 18.0
EGG_HATCH_TIME_BASE: 15.0
LAVA_Y:             -410.0
HAND_DETECT_RANGE:  100.0
HAND_RISE_SPEED:    200.0
HAND_COOLDOWN:      5.0
PARTICLE_CAP:       200
SPAWN_INVINCIBILITY: 0.5
MAX_LIVES:          5
EXTRA_LIFE_INTERVAL: 10000
PTERODACTYL_WAVE_TIMER: 60.0
```

All values in `constants.rs`. All tunable. Expect heavy iteration on physics values.
