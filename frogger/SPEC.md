# Frogger — Technical Specification

A modernized 2D Frogger built with Bevy 0.18.1 using only primitive shapes. No sprites, textures, or audio.

---

## 1. Game Overview

Recreate the 1981 Frogger arcade game with modern visual polish. The player guides a frog from the bottom of the screen, across five lanes of traffic, over a safe median, across five lanes of river (riding logs and turtles), and into one of five home bays at the top. Fill all five bays to complete a level. Lose all lives and the game ends.

### What "Modernized" Means

- Smooth interpolated hop animation (not instant grid-snap teleportation)
- Visual juice: bloom/glow, particle-like burst effects, screen shake on death, pulsing UI elements
- Fluid water rendering with animated layered shapes
- Trail effects behind the frog during jumps
- Progressive difficulty with visual escalation (faster traffic, colors shift warmer/more intense)
- Clean, readable UI with animated score popups

### What Stays Faithful

- Lane-based grid movement (frog hops one cell at a time)
- Core hazard model: cars kill, water kills, logs/turtles are rideable platforms
- Five home bays, time pressure, lives system
- Level progression when all five bays are filled

---

## 2. Game World Layout

### Coordinate System

Bevy uses center-origin (0,0 at screen center). The world is divided into 13 horizontal rows plus the home bay structure at the top.

### Window & Grid

| Property | Value | Rationale |
|---|---|---|
| Window size | 720 x 960 | Portrait orientation matches Frogger's vertical gameplay. 720px wide gives room for 13 columns. |
| Cell size | ~48 x 48 | 720 / 15 = 48. Gives 15 columns across and 13 playable rows plus the home row. |
| Playable columns | 13 (centered) | 1 column of padding on each side for visual framing. |

### Row Layout (bottom to top)

| Row | Y Index | Content | Zone |
|---|---|---|---|
| 0 | Bottom | Start zone (safe ground) | Safe |
| 1 | | Traffic lane 1 — slow cars, right | Road |
| 2 | | Traffic lane 2 — medium trucks, left | Road |
| 3 | | Traffic lane 3 — fast cars, right | Road |
| 4 | | Traffic lane 4 — slow long trucks, left | Road |
| 5 | | Traffic lane 5 — fastest cars, right | Road |
| 6 | | Median strip (safe ground) | Safe |
| 7 | | River lane 1 — medium logs, right | River |
| 8 | | River lane 2 — turtles (groups of 3), left | River |
| 9 | | River lane 3 — long logs, right | River |
| 10 | | River lane 4 — turtles (groups of 2), left | River |
| 11 | | River lane 5 — short logs, right | River |
| 12 | | Home row — 5 bays with walls between them | Home |

The Y coordinate for each row: `WORLD_BOTTOM + row_index * CELL_SIZE`.

### Home Bay Geometry

The home row consists of 5 evenly spaced rectangular openings separated by wall segments. Each bay is ~2 cells wide. Walls are ~1 cell wide. The row above the bays is a solid wall (top boundary).

```
 ┌──┐  ┌──┐  ┌──┐  ┌──┐  ┌──┐
 │  │  │  │  │  │  │  │  │  │   <-- bays (openings)
─┘  └──┘  └──┘  └──┘  └──┘  └─  <-- wall segments between bays
```

---

## 3. Entity Catalog

### 3.1 Frog (Player)

**Shape composition:**
- Body: rounded rectangle (Capsule2d), bright green, ~40x36px
- Eyes: two small white circles on top edge, with smaller black circle pupils
- Legs (optional detail): four small rectangles that animate during hops

**States:**
- Idle — sitting still, slight breathing scale pulse
- Hopping — smooth interpolation from current cell to target cell (~150ms duration)
- Riding — idle but inheriting platform velocity
- Dying — death animation plays, then respawn
- Arriving — reached a home bay, brief celebration effect

**Rendering:**
- Emissive green material (glows with bloom)
- Faint green glow circle behind frog (larger, low-alpha)
- Hop trail: 3-4 fading afterimage circles left behind during jumps

### 3.2 Vehicles (Road Hazards)

All vehicles are rectangles of varying size and color. They move horizontally at constant speed and wrap around screen edges.

| Type | Shape | Size (cells) | Color | Speed | Direction |
|---|---|---|---|---|---|
| Car (small) | Rectangle | 1 x 0.8 | Yellow | Slow | Right |
| Truck (medium) | Rectangle | 2 x 0.8 | Orange-red | Medium | Left |
| Race car | Rectangle | 1 x 0.8 | Hot pink | Fast | Right |
| Long truck | Rectangle | 3 x 0.8 | Dark red | Slow | Left |
| Sports car | Rectangle | 1 x 0.8 | Cyan | Fastest | Right |

**Details per vehicle:**
- Two small white/yellow circles at front = headlights (emissive, bloom)
- Two small red circles at rear = tail lights
- Thin darker rectangle inset = windshield

### 3.3 Logs (River Platforms)

Brown/dark-brown rounded rectangles. The frog rides on them.

| Type | Size (cells) | Speed |
|---|---|---|
| Medium log | 3 x 0.8 | Medium |
| Long log | 4 x 0.8 | Slow |
| Short log | 2 x 0.8 | Fast |

**Details:**
- Slightly darker "bark" rectangles overlaid at intervals for texture
- Subtle vertical line details (thin dark rectangles)

### 3.4 Turtles (River Platforms, Diving)

Groups of 2 or 3 turtles moving together as a single platform entity.

**Individual turtle shape:**
- Shell: circle (~0.8 cell diameter), dark green
- Shell pattern: smaller lighter-green circle inside
- Head: small circle protruding on the leading edge

**Group composition:**
- Group of 3: three turtles spaced ~1 cell apart, total platform width ~3 cells
- Group of 2: two turtles spaced ~1 cell apart, total platform width ~2 cells

**Dive cycle (per group, not global):**

| Phase | Duration | Visual | Rideable? |
|---|---|---|---|
| Surfaced | 3-5s (random) | Full opacity, normal size | Yes |
| Warning | 1s | Pulsing alpha 1.0 → 0.5, slight shrink | Yes |
| Submerged | 2-3s (random) | Alpha ≈ 0.15, scaled down 60% | No |
| Surfacing | 0.5s | Alpha 0.15 → 1.0, scale back to full | Yes (once > 50% surfaced) |

Each turtle group has an independent random phase offset so they don't all dive at once.

### 3.5 Home Bays

- Wall segments: dark gray/brown rectangles
- Bay openings: invisible sensor zones (collision-only)
- Filled bay: frog silhouette (green shape matching frog, static) + glow ring
- Empty bay: subtle pulsing glow ring (dim, inviting)

### 3.6 Bonus Entities

| Entity | Appearance | Spawn | Behavior | Score |
|---|---|---|---|---|
| Fly | Small buzzing dot (circle with orbiting smaller circles) | Random empty home bay, timed | Appears for ~5s then disappears. Landing in this bay scores bonus. | +200 |
| Lady frog | Pink version of frog shape | On a random log, mid-river, rare | Ride the log to the frog, then escort to a home bay (frog moves slower while escorting). | +200 |

---

## 4. Core Mechanics

### 4.1 Movement

**Grid-locked hopping:**
- The frog's position is tracked as a grid coordinate `(col, row)`.
- On input, the target cell updates and the visual position interpolates smoothly.
- Hop duration: ~120-150ms (tunable constant).
- During a hop, further input is buffered (max 1 buffered hop). The buffered hop fires immediately when the current hop completes.
- Frog rotates to face the hop direction (0° = up, 90° = right, etc.). Rotation interpolates with the hop for a smooth turn.

**Input mapping:**
- Arrow keys or WASD: hop in that direction
- Each key press = exactly one hop. Holding does NOT auto-repeat (debounced). This preserves the deliberate, committed feel of original Frogger.

**Boundary constraints:**
- Frog cannot hop past the left/right screen edges (input is ignored).
- Frog cannot hop below row 0 (start zone) or above row 12 (home row — handled by bay collision).

### 4.2 Riding Platforms

When the frog is in a river lane (rows 7-11) and overlapping a log or surfaced turtle group:

1. Mark the frog as `Riding(entity_id)`.
2. Each frame, apply the platform's horizontal velocity to the frog's world position.
3. Snap the frog's grid column to the nearest column based on its new world position (so future hops are grid-aligned relative to current position).
4. If the platform carries the frog past the screen edge → death.

When the frog hops off a platform to another row:
- Clear the `Riding` state immediately at hop start.
- If the target row is also a river lane, a new overlap check happens when the hop completes.

### 4.3 Water Death

Every frame, for a frog in a river lane (rows 7-11):
- If not overlapping any rideable platform → death.
- Turtle platforms are only rideable when NOT in the "Submerged" phase.

This check must run AFTER platform movement and AFTER riding velocity is applied, to avoid false positives where the log moved out from under the frog for one frame.

### 4.4 Vehicle Collision

AABB overlap check: if frog's bounding box overlaps any vehicle's bounding box → death.

This runs every frame during gameplay. It must also account for the frog mid-hop (interpolated position), not just the grid-snapped position — otherwise fast cars can pass through a hopping frog.

### 4.5 Home Bay Landing

When the frog hops into row 12:
- Check which bay (if any) the frog's center X falls within.
- If in a bay opening AND bay is empty → score, place frog in bay, spawn next frog.
- If in a bay opening AND bay is already filled → death.
- If NOT in any bay opening (hitting a wall segment) → death.

Tolerance: the frog's center must be within the bay opening with ±4px margin. Not too generous (trivializes aiming) nor too strict (feels unfair).

### 4.6 Time Limit

- Each frog has a timer (default: 30 seconds per frog).
- Visual timer bar at the bottom of the screen depletes left-to-right.
- At 10s remaining: bar turns from green → yellow.
- At 5s remaining: bar turns yellow → red, begins pulsing.
- At 0s: death (time out).
- Timer resets on each new frog (after death or successful bay landing).
- Timer does NOT pause during death animations.

### 4.7 Death & Respawn

**Death sequence:**
1. Frog enters "Dying" state. All input is ignored.
2. Death animation plays (~0.8s):
   - Vehicle collision: frog flattens (scale Y → 0) and turns red, burst of red particles.
   - Water death: frog sinks (scale → 0, alpha → 0) with expanding blue ripple circles.
   - Time out: frog flickers rapidly and fades out.
   - Wall collision: same as vehicle collision.
3. Lives decrement by 1.
4. If lives > 0: brief pause (0.3s), new frog spawns at start zone row 0, center column. All lane objects continue moving during respawn pause.
5. If lives = 0: transition to GameOver state.

### 4.8 Level Completion

When all 5 home bays are filled:
1. Brief celebration (1.5s): all filled bays pulse brightly, score popup.
2. Bonus score: remaining time × 10 points.
3. All bays clear. Level counter increments.
4. Lane speeds increase by a configurable multiplier (e.g., ×1.1 per level).
5. New frog spawns at start zone.
6. Difficulty caps at a maximum speed multiplier (e.g., level 10 = ×2.0 max).

---

## 5. Scoring

| Event | Points |
|---|---|
| Forward hop | +10 |
| Frog reaches home bay | +50 |
| Each remaining second on timer (on bay landing) | +10 per second |
| Bonus fly in home bay | +200 |
| Escorting lady frog home | +200 |
| All 5 bays filled (level clear) | +1000 |

**Important:** Forward hop score only awards for the *furthest row reached* on this life. Hopping backward and forward again doesn't re-award. Track a `max_row_this_life` value.

**Extra life:** at 10,000 points (one time only), award an extra life. Visual + brief text popup.

---

## 6. State Machine

```
StartScreen → Playing → GameOver → StartScreen
                 ↑                      │
                 └──────────────────────┘
```

### States

| State | Enter | Active | Exit |
|---|---|---|---|
| `StartScreen` | Spawn title UI, background animation | Wait for Space/Enter | Despawn title UI |
| `Playing` | Spawn lanes, vehicles, logs, turtles, frog, HUD, timer. Init lives=3, score=0, level=1 | Game loop runs | Despawn all gameplay entities |
| `LevelComplete` | (sub-state or timed event within Playing) Celebration animation, bonus scoring | 1.5s timer | Clear bays, increase difficulty, respawn frog |
| `GameOver` | Spawn game-over UI, show final score + level reached | Wait for Space/Enter | Despawn game-over UI, reset all game data |

`LevelComplete` is handled as a timed event within the `Playing` state using a `LevelCompleteTimer` resource, not a separate top-level state. This avoids despawning/respawning all lane entities on every level.

---

## 7. Visual Design with Primitives

### 7.1 Color Palette

| Element | Color (approx.) | Notes |
|---|---|---|
| Frog body | `#44FF44` (bright green) | Emissive for bloom |
| Road surface | `#2A2A2A` (dark gray) | |
| Road lane markings | `#FFFFFF` at 30% alpha | Dashed white lines between lanes |
| Median / Start zone | `#336633` (dark green) | Grass-like safe zone |
| Water base | `#1A3A5C` (deep blue) | |
| Water highlights | `#3A6A9C` at varying alpha | Animated overlays |
| Log | `#6B3A1F` (brown) | |
| Log bark detail | `#4A2810` (darker brown) | |
| Turtle shell | `#2D6B2D` (forest green) | |
| Home bay walls | `#4A3A2A` (brown-gray) | |
| Home bay glow (empty) | `#FFFF88` at 15% alpha | Subtle beacon |
| Home bay glow (filled) | `#44FF44` at 50% alpha | Bright confirmation |
| Background | `#0A0A12` (near-black blue) | Dark backdrop |
| Timer bar full | `#44FF44` | |
| Timer bar warning | `#FFCC00` | |
| Timer bar critical | `#FF3333` | Pulsing |

### 7.2 Water Rendering

Water is the most visually complex zone since we have no textures.

**Layer stack (bottom to top):**
1. Base: solid deep-blue rectangle spanning all river lanes.
2. Wave layer 1: ~10 horizontal capsule/ellipse shapes per lane, semi-transparent lighter blue, oscillating vertically with a sine wave (different phase per shape). Amplitude: ±3px, period: ~2s.
3. Wave layer 2: same as layer 1 but offset phase and slightly different color/alpha. Creates parallax-like depth.
4. Sparkle layer: tiny white circles that randomly appear and fade (simulating light glinting off water). ~5-8 active at any time across all river lanes. Each lives ~0.3s, fades alpha from 0.5 → 0.

### 7.3 Road Rendering

1. Base: solid dark-gray rectangle spanning all road lanes.
2. Lane dividers: dashed white lines between lanes. Each dash is a small rectangle (~20x3px) with ~20px gaps. These are static (no animation needed).
3. Sidewalk edges (top and bottom of road zone): slightly lighter gray rectangles, 4px tall.

### 7.4 Safe Zones (Start + Median)

- Dark green rectangles.
- Optional: tiny lighter-green circles scattered randomly (like grass/clover). Static decorative elements, spawned once.

### 7.5 Vehicle Details

Each vehicle is a parent entity with child shapes:
- Main body rectangle.
- Headlights: 2 small emissive circles at front (yellow/white glow — blooms).
- Tail lights: 2 small red circles at rear.
- Windshield: thin slightly-lighter rectangle inset from front edge.
- Wheels: 4 small dark circles at corners (below body, partially occluded).

### 7.6 Home Bay Rendering

- Wall segments: dark rectangles, slightly lighter at the top edge for a "lip" effect.
- Each empty bay has a dim glow circle that pulses slowly (scale oscillation 0.95-1.05, alpha oscillation 0.1-0.2).
- When a frog fills a bay, the glow brightens and a static frog silhouette shape is placed.

---

## 8. Visual Effects & Animations

### 8.1 Bloom

Enable Bevy's `Bloom` on the `Camera2d`. Settings:
- `intensity`: ~0.3 (subtle, not overwhelming)
- `low_frequency_boost`: ~0.5
- `threshold`: ~0.8

Elements with emissive materials (color values > 1.0 via `Color::linear_rgb`):
- Frog body
- Vehicle headlights
- Home bay glow rings
- Score popup text
- Timer bar when critical

### 8.2 Hop Animation

When the frog hops:
1. Scale squash at start: (1.2, 0.8) over first 20% of hop duration.
2. Translate smoothly (ease-in-out interpolation) from source cell to target cell.
3. Scale stretch at peak: (0.85, 1.15) at 50% of hop.
4. Scale settle at landing: back to (1.0, 1.0) with slight overshoot bounce.
5. Trail: spawn 3 small green circles at the start position. Each fades alpha from 0.4 → 0 over 0.2s, and slightly shrinks. Stagger spawn by 30ms.

### 8.3 Death Animations

**Vehicle hit:**
- Frog squashes (scale Y → 0.1 over 0.3s) while turning red.
- Burst: 8-12 small circles in random directions from frog center, red/orange, fade + shrink over 0.4s. Each has a random initial velocity that decelerates.

**Water death:**
- Frog shrinks (scale → 0) and fades (alpha → 0) over 0.5s.
- 3 concentric expanding blue-white circle outlines (ring shapes) from the frog's position, each spawned 0.1s apart. Each ring expands radius over 0.5s and fades to 0.

**Timeout death:**
- Frog flickers (alpha toggling 0/1 every 50ms) for 0.4s, then fades to 0 over 0.2s.

### 8.4 Score Popups

When the frog scores (bay landing, forward hop bonus, etc.):
- Spawn text entity at the event location.
- Text drifts upward (~50px/s) while fading out over 0.8s.
- Text is emissive yellow-white (blooms slightly).

### 8.5 Level Complete Celebration

1. All 5 filled bays pulse brightly (scale 1.0 → 1.3 → 1.0) in sequence, left-to-right, 0.15s each.
2. Large "+1000" popup floats up from center.
3. Brief white flash overlay (full-screen rectangle, alpha 0 → 0.3 → 0 over 0.5s).

### 8.6 Screen Shake

On death: translate the camera randomly ±3-5px for 0.3s, decaying amplitude. Implemented by temporarily offsetting the camera transform each frame, not by moving all entities.

### 8.7 Background Ambient Animation

- Faint vertical "scan lines" (very low alpha horizontal lines scrolling slowly upward) — a nod to the CRT arcade aesthetic. Optional, can be disabled if it looks bad.
- Subtle vignette: dark overlay rectangles along screen edges with gradient alpha (darkest at corners). Implemented as 4 large semi-transparent black rectangles at the screen edges on a high z-layer.

---

## 9. UI/UX

### 9.1 Start Screen

- Title: "FROGGER" in large text, centered vertically at ~60% up the screen.
- Title animation: letters pulse in sequence (scale oscillation), emissive green glow.
- Subtitle: "Press SPACE to Start" — fades in/out slowly (alpha oscillation, period ~2s).
- Background: the game world is visible but all lanes are running (cars driving, logs floating, water animated). No frog. This provides visual interest and teaches the player what to expect.
- High score display (if > 0): "HIGH SCORE: {n}" below subtitle.

### 9.2 HUD (During Gameplay)

Positioned as screen-space UI overlays, not world-space entities.

| Element | Position | Content |
|---|---|---|
| Score | Top-left | "SCORE: {n}" |
| High Score | Top-center | "HI: {n}" |
| Level | Top-right | "LEVEL {n}" |
| Lives | Bottom-left | Small frog icons (green circles) × remaining lives |
| Timer bar | Bottom-center | Horizontal bar that depletes. Width = (time_remaining / max_time) × bar_max_width |

Timer bar sits in a thin dark container rectangle. The fill rectangle shrinks from the right. Color transitions: green → yellow → red (see Section 4.6).

### 9.3 Game Over Screen

- "GAME OVER" — large red text, center screen.
- Appears with a scale-in animation (0 → 1.0 with overshoot bounce, 0.4s).
- Final score, level reached, and high score below.
- "Press SPACE to Play Again" — fade-in/out pulse, appears after 1s delay (prevents accidental restart).
- Background: game world frozen (all entities stopped but visible).

### 9.4 Pause (Optional Stretch Goal)

- Press Escape to toggle pause.
- Semi-transparent dark overlay with "PAUSED" text.
- All systems gated by a `Paused` resource or sub-state.

---

## 10. Technical Architecture

### 10.1 Module Structure

```
src/
├── main.rs          — App setup, plugins, camera + bloom config
├── states.rs        — AppState enum
├── constants.rs     — All tunable numbers
├── components.rs    — All ECS components
├── resources.rs     — GameData, LevelData, timers
├── lanes.rs         — Lane definition data, spawning lane entities + objects
├── player.rs        — Frog spawn, input, hop logic, riding, boundary checks
├── hazards.rs       — Vehicle movement, wrapping, log/turtle movement, turtle diving
├── collision.rs     — Vehicle collision, water check, home bay check, death trigger
├── scoring.rs       — Score awards, forward-hop tracking, extra life, level completion
├── effects.rs       — Particle bursts, screen shake, trail spawning, animations
├── ui.rs            — Start screen, HUD, game over, score popups
├── water.rs         — Water rendering: wave layers, sparkle generation
```

### 10.2 Key Components

```rust
// Entity markers
struct Frog;
struct Vehicle;
struct Log;
struct TurtleGroup;
struct HomeBay { index: u8 }
struct HomeWall;
struct WaterZone;

// Frog state
struct FrogGridPos { col: i32, row: i32 }
struct HopAnimation { from: Vec2, to: Vec2, timer: Timer, buffered_direction: Option<Direction> }
enum FrogState { Idle, Hopping, Riding(Entity), Dying }

// Lane objects
struct LaneMovement { speed: f32, direction: f32 } // direction: 1.0 or -1.0
struct Wrapping; // marker: this entity wraps at screen edges

// Turtle specific
struct DiveCycle { phase: DivePhase, timer: Timer }
enum DivePhase { Surfaced, Warning, Submerged, Surfacing }

// Visuals
struct FadeOut { timer: Timer }
struct ScaleAnimation { from: Vec2, to: Vec2, timer: Timer }
struct Particle { velocity: Vec2, lifetime: Timer }
struct ScreenShake { amplitude: f32, timer: Timer }
struct TrailDot;

// Home bays
struct BayFilled;

// UI
struct ScorePopup;
struct TimerBar;
```

### 10.3 Key Resources

```rust
struct GameData {
    score: u32,
    high_score: u32,
    lives: u32,
    level: u32,
    frogs_home: u8,       // bitfield or count of filled bays (0-5)
    max_row_this_life: i32, // for forward-hop scoring
}

struct FrogTimer {
    timer: Timer, // 30s countdown
}

struct LevelCompleteTimer {
    timer: Option<Timer>, // Some during celebration, None otherwise
}

struct SpeedMultiplier(f32); // increases each level
```

### 10.4 System Execution Order

Order within the `Update` schedule matters significantly for this game. Here's the required ordering:

```
1. input_system              — reads keyboard, sets hop target
2. hop_animation_system      — interpolates frog position during hop
3. lane_movement_system      — moves all vehicles, logs, turtles
4. turtle_dive_system        — updates dive cycles, changes phases
5. wrapping_system           — wraps lane objects at screen edges
6. riding_system             — if frog is riding, apply platform velocity
7. frog_boundary_system      — clamp frog to screen, kill if pushed off by platform
8. collision_system          — check vehicle overlap, water death, home bay
9. death_system              — process death events, spawn effects, decrement lives
10. scoring_system           — award points for forward hops, bay landings
11. level_check_system       — check if all 5 bays filled, trigger level complete
12. timer_system             — tick frog timer, trigger death on timeout
13. effect_systems           — update particles, trails, screen shake, popups, animations
14. ui_update_systems        — refresh HUD text, timer bar width/color
```

All gameplay systems run gated on `in_state(AppState::Playing)`.

Systems 1-2 are ordered with `.after()` chains. Systems 3-5 can run in parallel (no data conflicts). Systems 6-8 must be sequential. Systems 9-12 must follow collision. Systems 13-14 can run in parallel with each other but after everything else.

### 10.5 Entity Wrapping

Vehicles and logs that leave the screen must reappear on the opposite side.

**Approach:** When the entity's leading edge passes the wrap boundary (screen edge + half entity width as buffer so it's fully off-screen), teleport it to the opposite side.

- Right-moving: when `x > screen_right + half_width`, set `x = screen_left - half_width`.
- Left-moving: when `x < screen_left - half_width`, set `x = screen_right + half_width`.

This is simpler than despawn/respawn and preserves entity state (e.g., turtle dive timers).

### 10.6 Lane Spawning

On entering `Playing` state:

For each lane definition (stored as const data or a config resource):
1. Calculate how many objects fit with desired spacing.
2. Spawn them evenly distributed across the lane width (some off-screen).
3. Each object gets: `LaneMovement`, `Wrapping`, and its type-specific components.

Spacing must ensure the frog always has a gap to navigate through. This is tuned per lane with a `gap_size` parameter (minimum empty space between objects in cells).

**Critical tuning concern:** If gaps are random, some configurations become impossible. Use fixed patterns with slight random variation (±0.5 cells) to guarantee solvability.

---

## 11. Edge Cases & Tricky Scenarios

### 11.1 Frog Between Grid Cells During Collision

During a hop animation, the frog's visual position is between cells. A fast vehicle could collide with the frog mid-hop but miss if we only check grid position.

**Solution:** Collision checks use the frog's interpolated world position (Transform), not the grid position.

### 11.2 Log Carries Frog to Fractional Grid Position

When riding a log, the frog drifts to positions not aligned to the grid. If the player then hops, the target cell calculation needs to use the frog's actual world X, snapped to nearest column.

**Solution:** Before processing a hop on a riding frog, snap `FrogGridPos.col` to `round(world_x / CELL_SIZE)`. The hop target is then this snapped column ± 1.

### 11.3 Frog Hops Between Two Close Logs Moving Opposite Directions

If the frog is on a rightward log and hops up to a leftward log, there's one frame where the frog is in the river with no platform. The water death check could trigger incorrectly.

**Solution:** Only check water death when `FrogState` is `Idle` or `Riding`, never during `Hopping`. The hop completion handler checks for a platform at the landing position before the water check runs.

### 11.4 Two Turtle Groups Diving at the Same Time

If two adjacent turtle groups dive simultaneously, the frog has no platform to ride in that section of the river. This can create impossible situations.

**Solution:** Guarantee at least one turtle group per lane is surfaced at all times. Track dive schedules per lane and reject a dive if it would leave zero surfaced groups in that lane.

### 11.5 Frog Pushed Into Home Bay Wall by Log/Turtle

A platform moving horizontally could push the frog into a wall segment of the home bay row.

**Solution:** When the frog is in row 11 (adjacent to home), riding velocity cannot push the frog into row 12. Riding velocity only applies horizontally. The frog enters row 12 exclusively through an explicit upward hop. If riding velocity pushes the frog's X into a position that would overlap a wall at the current row, clamp the frog to the platform edge and do not kill.

### 11.6 Input During Death Animation

If the player presses keys during the death animation, those inputs should be discarded entirely (not buffered for after respawn).

**Solution:** Input system early-returns if `FrogState::Dying`.

### 11.7 Level Speed Escalation Making Lanes Impossible

At high levels, if speeds scale unchecked, gaps between vehicles may become too short to cross.

**Solution:** Speed multiplier caps at 2.0×. Additionally, gap sizes (minimum empty space between objects) are enforced as an absolute minimum regardless of speed. This may mean spawning fewer objects at high speeds.

### 11.8 Simultaneous Bay + Timer Events

The frog lands in a bay on the exact frame the timer expires.

**Solution:** Bay landing check runs before timer death check in system ordering. If the frog lands safely, the timer is irrelevant. Score the landing.

### 11.9 Score Overflow / Display

Score can technically grow indefinitely across many levels.

**Solution:** Display up to 7 digits. If score exceeds 9,999,999 (extremely unlikely), just cap the display. Store as `u32` internally (max ~4 billion).

### 11.10 Multiple Rapid Deaths

If something causes two death triggers in the same frame (e.g., frog pushed into water AND off-screen simultaneously), only process one death.

**Solution:** Death is triggered via a flag/event. The death system consumes the event and sets `FrogState::Dying`. Subsequent collision checks skip entities in `Dying` state.

### 11.11 Hop Landing Exactly on Entity Boundary

The frog lands at a position that is pixel-perfectly on the edge of a log or vehicle.

**Solution:** Collision uses inclusive overlap checks with a small tolerance (2px). For platforms (logs/turtles), the overlap check is slightly generous (frog center within platform bounds + 4px margin). For hazards (vehicles), the overlap check is slightly conservative (frog bounding box must overlap vehicle bounding box by at least 4px).

---

## 12. Concerns & Risks

### 12.1 Entity Count

Each vehicle has 5+ child entities (body, headlights, tail lights, windshield, wheels). With 5 road lanes × ~4 vehicles each = ~100 entities just for vehicles. Add logs, turtles, water decorations, particles, and UI — total could reach 300-500 entities.

**Risk level:** Low. Bevy handles thousands of 2D entities fine. But keep particle spawning bounded (max ~50 concurrent particles).

### 12.2 Bloom Performance

Bloom is a post-processing effect that adds GPU cost. On integrated graphics, it may cause frame drops.

**Mitigation:** Make bloom toggleable. If we detect low FPS on first few frames, disable it. Or provide a settings toggle.

### 12.3 Frame-Rate Independence

All movement and animation must use `Time::delta_secs()`. Fixed frame rates cannot be assumed. A frame spike could cause a vehicle to skip past the frog without collision being detected.

**Mitigation:** For vehicles, collision detection uses swept AABB (check the frog's bounding box against the vehicle's path over the frame delta, not just its current position). Or: clamp delta to a maximum (e.g., 0.05s = 20fps floor) and run the physics loop multiple sub-steps if delta is large.

**Simpler alternative:** Since speeds are not extreme, standard per-frame AABB overlap is likely sufficient. The swept approach is a stretch goal if tunneling is observed.

### 12.4 Determinism and Randomness

Turtle dive timers and bonus spawns use randomness. Without seeding, each run is different. This is fine for gameplay but could complicate debugging.

**Approach:** Use `rand` crate (or Bevy's built-in random if available). Optionally seed for deterministic testing.

### 12.5 State Cleanup Bugs

Missing cleanup on state exit is a common Bevy bug source. Every entity spawned in `OnEnter(Playing)` must have a marker component so `OnExit(Playing)` can query and despawn them all.

**Approach:** Add a `GameplayEntity` marker to everything spawned during play. `OnExit(Playing)` despawns all entities with this marker. Individual sub-markers (Vehicle, Log, etc.) are used for gameplay logic, `GameplayEntity` for bulk cleanup only.

---

## 13. Tradeoffs & Decisions

### 13.1 Grid Hop vs. Free Movement

| | Grid Hop | Free Movement |
|---|---|---|
| Collision | Simpler (grid-aligned checks) | Continuous AABB every frame |
| Feel | Authentic retro, deliberate | Modern, fluid |
| Riding logs | Snap to nearest cell | Precise sub-pixel position |
| Implementation | Moderate | Moderate-high |

**Decision:** Grid hop with smooth interpolated animation. This is the hybrid: movement is committed and grid-locked (preserving Frogger's strategic DNA), but the visual presentation is smooth. The frog's effective collision position during a hop uses the interpolated world position, not the target cell.

### 13.2 Entity Wrapping vs. Despawn/Respawn

| | Wrapping | Despawn/Respawn |
|---|---|---|
| Simplicity | Simple teleport | Must manage spawn timing |
| State preservation | Keeps dive timers, etc. | Must reinitialize |
| Visual glitches | Possible pop-in at edge | Same issue |

**Decision:** Wrapping. Simpler, and turtle dive state is preserved across wraps.

### 13.3 Bevy UI vs. World-Space Text

| | Bevy UI | World-Space Text |
|---|---|---|
| Positioning | Screen-relative, auto-layout | Manual transform math |
| Scaling | Handles resolution changes | Must scale manually |
| Score popups | Awkward (UI is screen space) | Natural (spawn at world position) |

**Decision:** Bevy UI for HUD (score, lives, timer). World-space `Text2d` for score popups and title effects.

### 13.4 Child Entities vs. Flat Hierarchy for Visuals

Vehicle headlights, turtle shell patterns, etc. could be child entities (move with parent automatically) or separate entities with manual position sync.

**Decision:** Child entities. Bevy's Transform propagation handles movement automatically. This is exactly what the parent-child hierarchy is for.

### 13.5 Event-Driven Death vs. Flag-Based Death

| | Events | Flags |
|---|---|---|
| Multiple triggers | Must drain to avoid duplicates | Single flag, natural dedup |
| System ordering | Event must be read in same or next frame | Flag persists |
| Code clarity | Decoupled systems | More explicit |

**Decision:** Hybrid. Use a `DeathEvent` Bevy event for triggering. The death system reads the event, sets `FrogState::Dying`, and subsequent systems check the flag. Events are drained each frame automatically.

### 13.6 Bonus Features Priority

Lady frog and fly bonuses add complexity. If time is limited:
- **Must have:** Core gameplay (all 5 lanes of road + river, home bays, scoring, lives, timer, level progression).
- **Should have:** Visual effects (bloom, particles, screen shake, hop trail).
- **Nice to have:** Fly bonus, lady frog, background ambient animations (scan lines, vignette).

### 13.7 rand Crate Dependency

Turtle dive timing and sparkle positions need randomness. Bevy doesn't ship a random API.

**Options:**
- Add `rand` crate — standard, well-supported, small compile cost.
- Use `fastrand` — lighter weight, no dependencies.
- Use frame count / time-based pseudo-random (hacky, bad distribution).

**Decision:** Add `rand` crate. It's the idiomatic Rust choice and the compile overhead is negligible.

---

## 14. Constants Catalog

All tunable values live in `constants.rs`. Key values:

```
// Window
WINDOW_WIDTH: 720.0
WINDOW_HEIGHT: 960.0
WINDOW_TITLE: "Frogger"

// Grid
CELL_SIZE: 48.0
GRID_COLS: 13
GRID_ROWS: 13
GRID_OFFSET_X: -(GRID_COLS as f32 * CELL_SIZE) / 2.0
GRID_OFFSET_Y: -(GRID_ROWS as f32 * CELL_SIZE) / 2.0

// Frog
FROG_SIZE: 40.0
HOP_DURATION: 0.12 (seconds)
FROG_LIVES: 3

// Timer
FROG_TIME_LIMIT: 30.0 (seconds)
TIMER_WARNING_THRESHOLD: 10.0
TIMER_CRITICAL_THRESHOLD: 5.0

// Speeds (pixels per second, base values before level multiplier)
SPEED_SLOW: 60.0
SPEED_MEDIUM: 100.0
SPEED_FAST: 150.0
SPEED_FASTEST: 200.0
SPEED_MULTIPLIER_PER_LEVEL: 1.1
SPEED_MULTIPLIER_CAP: 2.0

// Scoring
SCORE_HOP_FORWARD: 10
SCORE_BAY_LANDING: 50
SCORE_TIME_BONUS_PER_SEC: 10
SCORE_FLY_BONUS: 200
SCORE_LADY_FROG_BONUS: 200
SCORE_LEVEL_CLEAR: 1000
SCORE_EXTRA_LIFE_THRESHOLD: 10000

// Turtle diving
DIVE_SURFACED_MIN: 3.0
DIVE_SURFACED_MAX: 5.0
DIVE_WARNING_DURATION: 1.0
DIVE_SUBMERGED_MIN: 2.0
DIVE_SUBMERGED_MAX: 3.0
DIVE_SURFACING_DURATION: 0.5

// Effects
SCREEN_SHAKE_DURATION: 0.3
SCREEN_SHAKE_AMPLITUDE: 4.0
PARTICLE_BURST_COUNT: 10
TRAIL_DOT_COUNT: 3
TRAIL_DOT_LIFETIME: 0.2
SCORE_POPUP_SPEED: 50.0
SCORE_POPUP_LIFETIME: 0.8

// Bloom
BLOOM_INTENSITY: 0.3
BLOOM_THRESHOLD: 0.8

// Death animation
DEATH_SQUASH_DURATION: 0.3
DEATH_SINK_DURATION: 0.5
DEATH_FLICKER_DURATION: 0.4
RESPAWN_DELAY: 0.3
LEVEL_COMPLETE_CELEBRATION: 1.5
```

---

## 15. Input Handling Detail

### Debouncing

Each direction key tracks its previous state. A hop triggers only on the frame the key transitions from "not pressed" to "pressed" (`just_pressed`). Holding the key does nothing after the initial hop.

### Input Buffer

While the frog is mid-hop, one additional input can be buffered:
- If a direction key is pressed during a hop, store it as `buffered_direction`.
- When the current hop completes, immediately start a new hop in the buffered direction (if valid).
- If multiple directions are pressed during the buffer window, the last one wins.
- The buffer is cleared on hop completion if no new input arrived.

### Simultaneous Keys

If two directional keys are pressed on the same frame (e.g., Up + Right), prioritize Up. Priority order: Up > Left > Right > Down. This matches the original game's bias toward forward progress.

---

## 16. Lane Configuration Data

Each lane is defined by a struct containing:

```rust
struct LaneConfig {
    row: i32,
    zone: Zone,           // Road, River, Safe
    object_type: LaneObjectType,
    speed: f32,           // base speed
    direction: f32,       // 1.0 = right, -1.0 = left
    object_count: u32,    // number of objects in lane
    object_width: f32,    // width in cells
    min_gap: f32,         // minimum gap between objects in cells
}

enum Zone { Safe, Road, River }
enum LaneObjectType { None, Car, Truck, RaceCar, LongTruck, SportsCar, MediumLog, LongLog, ShortLog, TurtleGroup2, TurtleGroup3 }
```

Lane configs are defined as a const array in `constants.rs` or `lanes.rs`.

---

## 17. Testing Strategy

### Manual Testing Checklist

- [ ] Frog hops correctly in all 4 directions
- [ ] Frog cannot move off-screen
- [ ] Vehicles wrap correctly at both edges
- [ ] Frog dies on vehicle contact
- [ ] Frog dies in water without platform
- [ ] Frog rides logs correctly (moves with log)
- [ ] Frog rides turtles correctly
- [ ] Frog dies on submerged turtle
- [ ] Turtles cycle through all 4 dive phases
- [ ] Home bay landing scores correctly
- [ ] Filled bay re-entry causes death
- [ ] Wall contact causes death
- [ ] All 5 bays filled triggers level complete
- [ ] Timer depletes and kills frog at 0
- [ ] Lives decrement on death, game over at 0
- [ ] Score popup appears and fades
- [ ] Bloom visible on frog, headlights, bays
- [ ] Screen shake on death
- [ ] Hop trail visible during jumps
- [ ] Level speed increase is noticeable but fair
- [ ] Start screen transitions to playing
- [ ] Game over screen shows score, transitions to start

### Automated Testing (Stretch)

Unit tests for pure logic functions:
- Grid-to-world / world-to-grid coordinate conversion
- Bay overlap detection
- Score calculation
- Speed scaling per level
