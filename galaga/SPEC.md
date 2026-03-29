# Galaga — Modernized 2D Clone Specification

A faithful-in-spirit 2D reimagining of the 1981 arcade classic, built with Bevy 0.18.1 using only primitive shapes and post-processing effects. No sprites, no textures, no audio.

---

## 1. Visual Design (Primitive Shapes Only)

### 1.1 Entity Shapes

| Entity | Shape | Color (HDR for bloom) | Notes |
|---|---|---|---|
| **Player ship** | Chevron (two triangles forming an arrow pointing up) | Cyan glow `linear_rgb(0.0, 3.0, 3.0)` | ~30px wide, ~36px tall |
| **Bee (basic enemy)** | Hexagon | Yellow glow `linear_rgb(3.0, 3.0, 0.0)` | 6-sided, ~24px radius |
| **Butterfly (mid enemy)** | Diamond (rotated square) with two small triangles as "wings" | Red-pink glow `linear_rgb(3.0, 0.5, 0.5)` | Parent entity + 2 wing children |
| **Boss Galaga** | Large octagon with inner circle | Green glow `linear_rgb(0.5, 3.0, 0.5)` | ~36px radius, 2-hit HP |
| **Player bullet** | Tall narrow rectangle | White-cyan `linear_rgb(2.0, 4.0, 4.0)` | 4px wide, 14px tall |
| **Enemy bullet** | Small circle | Orange-red `linear_rgb(4.0, 1.0, 0.2)` | 5px radius |
| **Tractor beam** | Trapezoid (narrow at top, wide at bottom) with alpha pulse | Green-white `linear_rgba(0.5, 3.0, 0.5, 0.4)` | Semi-transparent, oscillating alpha |
| **Captured ship** | Same chevron as player | Dimmed red tint | Sits above boss in formation |
| **Explosion particles** | Small squares and circles | Match destroyed entity's color, fading | 10-16 per explosion |
| **Star (background)** | 1-3px circles | White at various dim intensities | 3 parallax layers |
| **Exhaust flame** | Small triangle below player | Orange-yellow, flicker via scale oscillation | Cosmetic child entity |

### 1.2 Post-Processing & Effects

- **Bloom**: `BloomSettings` on the camera with HDR enabled. This is the single highest-impact visual. All gameplay entities use HDR color values > 1.0 so they glow against the black background.
- **Particle explosions**: On entity death, spawn 10-16 small shape entities with randomized outward velocity, rotation, shrinking scale, and alpha fade over ~0.5s, then despawn.
- **Engine exhaust**: Small flickering triangle beneath the player ship. Oscillate scale Y between 0.6-1.0 using a sine wave on elapsed time.
- **Bullet trails**: Each bullet spawns fading afterimage entities every 2-3 frames that shrink and fade over ~0.15s.
- **Screen shake**: On player death, offset the camera by random small amounts (+-4px) for ~0.3s, decaying to zero.
- **Formation breathing glow**: Enemies in formation subtly pulse brightness via a shared sine wave modulating their color intensity.
- **Starfield parallax**: Three layers of randomly placed dots scrolling downward at different speeds (slow/medium/fast). Wrap around when off-screen.
- **Tractor beam animation**: Trapezoid shape with alpha oscillating via sine wave. Expands downward over ~0.5s on activation.

### 1.3 Z-Ordering (back to front)

| Layer | Z |
|---|---|
| Background stars | 0.0 |
| Tractor beam | 5.0 |
| Enemies | 10.0 |
| Captured ship | 10.0 |
| Player | 15.0 |
| Bullets (player & enemy) | 20.0 |
| Explosion particles | 25.0 |
| UI | N/A (Bevy UI overlay) |

---

## 2. Game Mechanics

### 2.1 Player

- **Movement**: Horizontal only (left/right). Locked to bottom ~60px of the screen. Arrow keys and A/D.
- **Shooting**: Space bar fires a bullet upward from ship's nose. **Maximum 2 player bullets on screen at once.** Pressing Space while 2 bullets exist does nothing. This is critical to authentic Galaga feel — it forces the player to aim rather than spam.
- **Lives**: Start with 3 lives. Displayed as small ship icons in the bottom-left HUD.
- **Respawn**: On death, 1.5s delay, then new ship fades in at center-bottom with 2s of invincibility (ship blinks).
- **Dual fighter**: If the player rescues a captured ship (see 2.4), two ships fly side-by-side with synchronized movement. Doubles the width (~60px total). Each Space press fires 2 bullets (1 from each ship), but the 2-bullet-on-screen limit still applies per salvo (so 2 shots = 4 bullets briefly, then must wait). When hit in dual mode, lose one ship (revert to single), no life lost.

### 2.2 Enemy Types

| Type | HP | Formation Score | Dive Score | Behavior |
|---|---|---|---|---|
| **Bee** | 1 | 50 | 100 | Basic dive attacks |
| **Butterfly** | 1 | 80 | 160 | Wider, looping dive patterns |
| **Boss Galaga** | 2 | 150 | 400 | Can perform tractor beam attack; escorts 2 bees on dive |

- **Hit feedback**: On first hit of a 2-HP boss, flash the entity white for 1 frame and change its inner circle color to indicate damage.

### 2.3 Enemy Formation

- **Grid layout**: 5 columns x 4 rows at the top of the screen (narrower than the full window to leave dodge room). Row composition from top:
  - Row 0: 4 Boss Galaga (centered, skip edge columns)
  - Row 1: 8 Butterflies
  - Row 2: 8 Bees
  - Row 3: 8 Bees (not present in wave 1; added from wave 2+)
- **Breathing**: The entire formation oscillates horizontally using a sine wave. Amplitude: ~40px, period: ~3s. All in-formation enemies move with the grid origin.
- **Slot tracking**: Each enemy has an assigned `FormationSlot { row: u8, col: u8 }`. The world position of a slot = formation_origin + slot_offset. When an enemy is diving, its slot remains reserved. When it returns, it interpolates back to the slot's current world position.

### 2.4 Tractor Beam & Ship Capture

This is the signature Galaga mechanic and the hardest to implement correctly.

**Sequence:**
1. A Boss Galaga initiates a tractor beam dive — it flies down toward the player.
2. When it reaches a Y position ~120px above the player, it stops and activates a trapezoid tractor beam extending downward.
3. If the player's ship overlaps the beam for ~1.5s (or is within the beam's X range and below a Y threshold), the ship is "captured": the player loses control, the ship floats upward and docks above the boss.
4. The boss returns to formation with the captured ship displayed directly above it.
5. The player loses a life and respawns. On the next encounter, if the player destroys that specific boss, the captured ship is released and flies down to join the player, forming a dual fighter.

**Edge cases:**
- If the captured ship was the player's last life: game over. No rescue opportunity.
- If the boss is destroyed by someone else (future multiplayer?) or during the tractor beam animation before capture completes: beam cancels, no capture.
- If the boss dives with a captured ship and both the boss AND the captured ship are hit: the captured ship is destroyed (no rescue), player gets score for boss.
- If the player is already in dual-fighter mode when a boss tries to tractor beam: the beam can still capture one of the two ships, reverting to single fighter. The captured ship docks above the boss as normal.

**Simplification decision**: Implement the full mechanic. It's the soul of Galaga. A Galaga without tractor beam is just a generic space shooter.

### 2.5 Enemy Attack Patterns (Dive Bombing)

This is the second-hardest technical challenge.

- **Dive selection**: A timer fires every ~2-4s (decreasing with wave number). It selects 1-2 enemies from the formation to begin a dive. Boss Galaga dives with 2 bee escorts.
- **Dive paths**: Enemies follow **cubic Bezier curves** from their formation slot downward, curving left or right, then looping below the screen and re-entering from the top to return to their slot.
  - Define 6-8 predefined dive path templates (mirrored left/right variants).
  - Each path is a sequence of 2-3 cubic Bezier segments.
  - The enemy advances along the path using a parametric `t` value incremented by `speed * delta_time / arc_length_estimate`.
- **Shooting during dive**: Enemies fire 1-2 bullets during their dive, aimed toward the player's current X position (with slight inaccuracy for fairness).
- **Off-screen wrapping**: If an enemy goes below the bottom of the screen during a dive, it re-enters from the top and curves back toward its formation slot.
- **Return to formation**: After completing the dive path, the enemy interpolates back to its formation slot position over ~0.5s.

### 2.6 Wave Entry Choreography

At the start of each wave, enemies don't just appear — they **fly in** along scripted paths.

- **Entry groups**: Enemies enter in 4-6 groups of 4-8, with ~1s gaps between groups.
- **Entry paths**: Each group follows a curved entry path from off-screen (top, top-left, top-right) and peels off one-by-one into their formation slots.
- **Implementation**: A `WaveScript` resource holds a timeline of `(delay, group, entry_path)` tuples. A system advances the timeline and spawns groups at the right time. Each enemy in a group has a staggered delay (0.1s apart) before following the same path.
- **Wave doesn't "start" until all enemies are in formation**: No dive attacks during entry choreography.

### 2.7 Challenge Stages

Every 3rd wave (waves 3, 6, 9, ...) is a **challenge stage**:
- Enemies fly across the screen in decorative patterns but **do not attack or shoot**.
- The player **cannot die** during challenge stages.
- Scoring: 100 points per enemy hit. Bonus at the end: "PERFECT" bonus of 10,000 if all enemies destroyed, otherwise "Number Hit: X" with no bonus.
- After the stage, proceed to the next normal wave.

### 2.8 Wave Progression & Difficulty Scaling

| Parameter | Wave 1 | Wave 5 | Wave 10+ |
|---|---|---|---|
| Dive interval | 3.5s | 2.0s | 1.0s |
| Enemy bullet speed | 200 | 280 | 380 |
| Simultaneous divers | 1 | 2 | 3 |
| Boss tractor beam chance | 10% | 20% | 30% |
| Enemy bullets per dive | 1 | 1-2 | 2-3 |
| Formation rows | 3 | 4 | 4 |

Scaling is linear interpolation between these anchor points, clamped at wave 10.

---

## 3. State Machine

```
StartScreen --> Playing --> GameOver --> StartScreen
                  |             ^
                  |             |
                  +--> Paused --+  (optional, stretch)
                  |
                  +--> WaveIntro --> Playing (sub-state for entry choreography)
```

### 3.1 AppState Enum

```rust
enum AppState {
    StartScreen,
    Playing,       // Active gameplay (includes WaveIntro as a sub-phase via resource, not a separate state)
    GameOver,
}
```

Wave intro vs. active combat is tracked via a `WavePhase` resource (`Entering` | `Combat`), not a separate AppState. This avoids complex state transition cleanup — enemies and the player all exist during both phases.

### 3.2 GameData Resource

```rust
struct GameData {
    score: u32,
    high_score: u32,
    lives: u32,
    wave: u32,
    wave_phase: WavePhase,
    respawn_timer: Option<Timer>,
    invincibility_timer: Option<Timer>,
    screen_shake: Option<ScreenShake>,
}
```

---

## 4. Module Architecture

```
src/
  main.rs           — App builder, plugin registration, camera setup (with BloomSettings)
  components.rs     — All marker + data components
  constants.rs      — All tuning constants
  resources.rs      — GameData, WaveScript, FormationState, DivePathLibrary
  states.rs         — AppState enum, WavePhase enum
  player.rs         — PlayerPlugin: movement, shooting, respawn, invincibility blink, dual fighter sync
  formation.rs      — FormationPlugin: grid management, breathing oscillation, slot tracking
  enemy.rs          — EnemyPlugin: enemy types, dive AI, entry choreography, tractor beam
  combat.rs         — CombatPlugin: bullet movement, collision detection, scoring, win/loss checks
  effects.rs        — EffectsPlugin: explosions, screen shake, bullet trails, exhaust flame
  background.rs     — BackgroundPlugin: starfield parallax layers
  ui.rs             — UiPlugin: start screen, HUD (score, high score, lives, wave), game over, challenge stage results
  paths.rs          — Cubic Bezier path definitions, path-following utility, dive path library
```

---

## 5. Technical Implementation Details

### 5.1 Shape Rendering with Mesh2d

Every gameplay entity uses `Mesh2d` + `MeshMaterial2d<ColorMaterial>`. Mesh handles must be **created once and shared** to avoid duplicating GPU resources.

```rust
// In a setup system or resource initialization:
#[derive(Resource)]
struct GameMeshes {
    player_ship: Mesh2d,      // Custom triangle mesh
    bee: Mesh2d,              // RegularPolygon(6, 24.0)
    butterfly_body: Mesh2d,   // Rotated square
    butterfly_wing: Mesh2d,   // Small triangle
    boss: Mesh2d,             // RegularPolygon(8, 36.0)
    boss_inner: Mesh2d,       // Circle(18.0)
    player_bullet: Mesh2d,    // Rectangle(4.0, 14.0)
    enemy_bullet: Mesh2d,     // Circle(5.0)
    particle: Mesh2d,         // Circle(3.0) or Rectangle(3.0, 3.0)
    star: Mesh2d,             // Circle(1.5)
    exhaust: Mesh2d,          // Small triangle
}
```

**Concern**: Bevy's `Mesh2d` requires a `Handle<Mesh>` added to the asset server. The `GameMeshes` resource should store `Handle<Mesh>` values created via `meshes.add(...)` in a startup system that runs before any entity spawning.

**Tradeoff**: Could use `Sprite` with a 1x1 white pixel and scaling, which is simpler but doesn't give us actual geometric shapes (everything would be rectangles). Mesh2d is the correct choice for geometric primitives.

### 5.2 Bloom Setup

```rust
commands.spawn((
    Camera2d,
    Camera {
        hdr: true,
        ..default()
    },
    Tonemapping::TonyMcMapface,
    BloomSettings {
        intensity: 0.3,
        low_frequency_boost: 0.6,
        ..default()
    },
));
```

**Concern**: Bloom affects ALL rendered entities including UI text. UI text with HDR values will glow. Solution: keep UI text colors at SDR values (0.0-1.0 range) so bloom doesn't affect them significantly, or accept the slight glow as an aesthetic choice.

### 5.3 Cubic Bezier Path Following

```rust
struct BezierSegment {
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,
}

struct DivePath {
    segments: Vec<BezierSegment>,
    total_arc_length: f32,  // Pre-computed approximate arc length
}

#[derive(Component)]
struct PathFollower {
    path_index: usize,       // Which DivePath from the library
    segment_index: usize,    // Current segment within the path
    t: f32,                  // 0.0..1.0 within current segment
    speed: f32,              // World units per second
}
```

**Hard part**: Uniform-speed traversal along a Bezier curve. Naive `t += speed * dt` produces non-uniform speed because Bezier parameterization is not arc-length. Solutions:
1. **Pre-compute a lookup table** of arc-length-to-t mappings per segment (sample ~20 points, build cumulative distance table, binary search at runtime). Best balance of accuracy and performance.
2. **Approximate with velocity magnitude correction**: Compute `|dB/dt|` at current t, adjust dt. Cheaper but less accurate.
3. **Just use naive t advancement**: Accept non-uniform speed. Enemies will accelerate through tight curves and slow on straight sections. Actually looks somewhat natural for dive-bombing.

**Decision**: Use option 3 (naive t) for initial implementation. The non-uniform speed actually creates a pleasing acceleration/deceleration effect on curves. If it looks wrong, upgrade to option 1.

### 5.4 Formation System

```rust
#[derive(Resource)]
struct FormationState {
    origin: Vec2,            // Center of formation, oscillates horizontally
    phase: f32,              // Sine wave phase for breathing
    slots: HashMap<(u8, u8), Option<Entity>>,  // (row, col) -> entity or empty
}
```

**Hard part**: An enemy's "home position" is **moving** (because the formation breathes). When an enemy finishes its dive and returns, it must interpolate toward a moving target. Solution: each frame, compute the slot's current world position and lerp toward it. Don't cache the target.

**Edge case**: If all enemies in a row are diving simultaneously, the formation still breathes. When they return, they might collide if the formation has shifted significantly. In practice, the oscillation amplitude is small enough this isn't an issue.

### 5.5 Collision Detection

All collision uses **circle-circle** intersection (distance < sum of radii). Even for non-circular shapes, the gameplay hitbox is circular for fairness.

| Collision pair | Result |
|---|---|
| Player bullet vs Enemy | Enemy takes 1 damage, bullet despawned |
| Enemy bullet vs Player | Player dies (unless invincible), bullet despawned |
| Diving enemy vs Player | Both die |
| Tractor beam vs Player | Begin capture sequence |
| Player bullet vs Captured ship | Ship destroyed (no rescue) |

**Concern — collision ordering**: Multiple collisions in the same frame (e.g., bullet hits boss AND captured ship). Process in this order:
1. Player bullets vs enemies (check boss before captured ship — if boss dies, captured ship is freed)
2. Enemy bullets vs player
3. Body-to-body (diving enemy vs player)
4. Tractor beam vs player

Use `HashSet<Entity>` to track already-processed entities within a frame to prevent double-processing (already in template).

**Concern — deferred despawns**: `commands.entity(e).despawn()` is deferred to end-of-stage. If system A despawns an entity and system B runs after it in the same frame, the entity still exists in queries. This is fine as long as we use the `HashSet` approach to skip already-hit entities. But it means an entity could be "hit" twice if two systems both check it. Solution: run all collision logic in **one system** with explicit ordering of checks within that system, not split across systems.

### 5.6 Particle System (Lightweight)

No Bevy built-in particle system. Roll our own:

```rust
#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime: Timer,
    initial_scale: f32,
}
```

A single `update_particles` system: advance timer, move by velocity, shrink scale linearly toward 0, reduce alpha. Despawn when timer finishes.

Spawn function: takes position, base color, count. For each particle, randomize direction (360 degrees), speed (50-150), lifetime (0.3-0.6s), and slight color variation.

**Performance concern**: At peak, maybe 50-100 particle entities exist simultaneously (a few explosions overlapping). This is trivially fine for Bevy's ECS.

### 5.7 Bullet Limiting

```rust
fn player_shoot(
    bullet_query: Query<&PlayerBullet>,
    // ...
) {
    if bullet_query.iter().count() >= MAX_PLAYER_BULLETS {
        return;
    }
    // spawn bullet
}
```

`MAX_PLAYER_BULLETS = 2`. Simple query count check. In dual-fighter mode, each press spawns 2 bullets (one per ship), and both count toward the limit. So effectively: single fighter gets 2 shots, dual fighter gets 1 salvo of 2 (which is 2 bullets = the limit). The player must wait for both to hit/exit before firing again. This matches original Galaga.

### 5.8 Screen Shake

```rust
struct ScreenShake {
    timer: Timer,
    intensity: f32,  // Max pixel offset, decays over duration
}
```

System: if `screen_shake` is Some, offset the camera's `Transform.translation` by `(random_x, random_y) * intensity * (1.0 - timer.fraction())`. When timer finishes, reset camera to (0, 0) and clear the resource.

**Concern**: Camera offset affects ALL rendering including UI. Bevy UI is screen-space and unaffected by Camera2d transform, so this is fine — only gameplay entities shake, UI stays stable.

---

## 6. UI/UX Design

### 6.1 Start Screen

```
         ╔═══════════════════════╗
         ║       G A L A G A     ║   (large, glowing cyan text)
         ╚═══════════════════════╝

             ▲  (player ship shape, animated bob)

          Press SPACE to Start       (pulsing opacity)

           High Score: 12500         (persisted in resource, not to disk)
```

- Title text uses large font with HDR color for bloom glow.
- A decorative player ship shape bobs up and down below the title.
- "Press SPACE" text pulses opacity via sine wave.
- Starfield background is active on all screens.

### 6.2 In-Game HUD

```
  Score: 1250          HIGH SCORE: 12500          Wave 3

  [bottom-left: ▲ ▲ ▲ life icons]
```

- Top bar with score (left), high score (center), wave number (right).
- Bottom-left: remaining lives as small ship silhouettes (not counting the active ship).
- All HUD text in SDR white to avoid excessive bloom.

### 6.3 Game Over Screen

```
             GAME OVER                (red glow)

          Final Score: 8500
          High Score: 12500

        Press SPACE to Continue       (pulsing)
```

### 6.4 Challenge Stage Results (shown after challenge stage ends)

```
          CHALLENGING STAGE

          Number of hits: 36

             PERFECT!               (if 40/40, gold glow)
             Bonus: 10000

        (auto-advances after 3s)
```

### 6.5 Wave Intro Banner

When a new wave starts, briefly display "STAGE X" centered on screen for 2s, then fade out. Wave entry choreography begins after 1s of the banner.

---

## 7. Edge Cases & Hard Problems

### 7.1 Entity Lifecycle Conflicts

- **Boss destroyed while tractor beam is active**: Cancel beam animation, free player if capture was in progress (player regains control at their current dragged position).
- **Boss destroyed while holding captured ship**: The captured ship should be released — it flies down and joins the player as a dual fighter, even though the boss was killed during a dive (not necessarily a tractor beam dive).
- **Player killed during capture animation**: If the player still has lives, the capture completes (ship is lost) AND a life is consumed. New ship respawns. If it was the last life, game over.
- **All enemies destroyed during entry choreography**: Shouldn't happen (enemies are invulnerable during entry? No — in original Galaga, you CAN shoot them during entry). If all are killed, immediately trigger wave complete and advance.

### 7.2 Dual Fighter Edge Cases

- **Dual fighter width vs window bounds**: Two ships side-by-side are ~60px wide. Clamp logic must use the combined width, not single ship width. The center of the pair is the controlled position.
- **Which ship fires?**: Both fire simultaneously, one bullet from each ship's nose position.
- **Hit detection on dual fighter**: Each ship has its own collision circle. A single enemy bullet can only hit one.
- **Losing the dual fighter**: The hit ship explodes, the remaining ship slides to center position. No life lost — the destroyed ship was the "bonus" captured ship. If the ORIGINAL ship is hit (left ship by convention), you lose a life and the captured ship becomes your new active ship. Actually, in original Galaga, either ship destroyed just reverts to single — no life lost. Go with this simpler model.
- **Can a dual fighter be captured again?**: Yes. Tractor beam captures one ship, reverting to single. The captured ship can later be rescued for a second dual fighter.

### 7.3 Simultaneous Events

- **Player bullet hits boss at the same frame player collides with diving enemy**: Process bullet collisions first. Boss takes damage. Then body collision kills player. Both happen. Player gets score, then dies.
- **Two player bullets hit the same 2-HP boss in one frame**: Both should register. Boss takes 2 damage and dies. This is correct — both bullets were in flight.
- **Player dies and kills last enemy in the same frame**: Player death takes priority. Don't trigger wave complete. The dead enemy is gone, but the player must respawn (if lives remain) and then the wave-complete check runs on next frame and finds zero enemies.

### 7.4 Performance Considerations

- **Entity count**: At peak — ~40 formation enemies + player + ~10 bullets + ~50 particles + ~200 stars = ~300 entities. Trivial for Bevy.
- **Mesh handle sharing**: If each entity gets its own mesh, that's 300 mesh allocations. Use shared handles from `GameMeshes` resource.
- **Particle cleanup**: Ensure particle despawn system runs every frame. Leaked particles accumulate.

---

## 8. Input Mapping

| Key | Action | Context |
|---|---|---|
| Left / A | Move left | Playing |
| Right / D | Move right | Playing |
| Space | Shoot | Playing |
| Space | Start game | StartScreen |
| Space | Restart | GameOver |
| Escape | Quit game (optional) | Any |

**Concern**: Player movement is horizontal-only in Galaga (unlike the template which allows 4-directional). Must remove up/down movement.

---

## 9. Tradeoffs & Decisions

### 9.1 Bezier Paths vs. Simpler Movement

**Decision**: Use cubic Bezier curves.

Galaga's identity IS the swooping enemy patterns. Sine-wave approximations look wrong. The implementation cost is ~100 lines for the path system + ~50 lines of path data definitions. Worth it.

### 9.2 Tractor Beam: Full Mechanic vs. Simplified Boss

**Decision**: Implement full tractor beam + dual fighter.

Without it, this is just "Space Invaders with curves." The tractor beam creates the central risk/reward tension that defines Galaga: do you intentionally sacrifice a ship to get the dual fighter?

### 9.3 Challenge Stages: Include or Defer

**Decision**: Include in initial build.

They're mechanically simple (enemies follow fixed paths, no combat logic, just scoring). The only cost is defining 2-3 decorative flight patterns. They provide pacing variety between intense combat waves.

### 9.4 High Score Persistence

**Decision**: In-memory only (resets on app close).

File I/O adds platform-specific complexity and error handling for minimal benefit in a demo game. High score persists across restarts within a session via the `GameData` resource.

### 9.5 Collision Shapes: Circle vs. Polygon

**Decision**: All circle collisions.

The visual shapes are polygons, but gameplay hitboxes are circles. This is standard for arcade games — circle collision is cheap (`distance < r1 + r2`) and feels fair. Polygon-accurate collision would create "unfair" deaths on corners that the player can't visually predict.

### 9.6 Separate Collision System vs. Split Across Plugins

**Decision**: Single unified collision system in `combat.rs`.

Splitting collision across `player.rs`, `enemy.rs`, etc. creates ordering nightmares (entity despawned in one system but queried in another). One system with explicit check ordering is safer.

### 9.7 Wave Script: Data-Driven vs. Hardcoded

**Decision**: Hardcoded wave scripts for waves 1-3 entry patterns, then procedurally vary for later waves.

Full data-driven scripting (JSON/RON files) is overengineered for this scope. 3-4 handcrafted entry patterns, mirrored and shuffled for variety, is sufficient. Later waves reuse patterns with increased enemy counts and speed.

---

## 10. Implementation Order (Suggested Phases)

### Phase 1: Core Rendering & Movement
1. Camera with bloom, starfield background
2. Player ship as mesh, horizontal movement, window clamping
3. Mesh resource system (`GameMeshes`)
4. Basic enemy formation (static grid, no breathing yet)

### Phase 2: Combat Basics
5. Player shooting with 2-bullet limit
6. Bullet-enemy collision, scoring, enemy despawn
7. Explosion particle effects
8. Enemy despawn → wave complete check → next wave

### Phase 3: Enemy AI
9. Formation breathing oscillation
10. Dive path system (Bezier curves, path library)
11. Enemy dive selection timer, dive execution, return-to-formation
12. Enemy shooting during dives

### Phase 4: Full Galaga Mechanics
13. Boss Galaga (2 HP, escorts on dive)
14. Tractor beam attack + ship capture
15. Dual fighter mechanic
16. Wave entry choreography

### Phase 5: Polish & Game Loop
17. Lives system, respawn with invincibility blink
18. Challenge stages
19. Wave progression, difficulty scaling
20. Screen shake, bullet trails, exhaust flame
21. UI: complete HUD, wave banner, challenge results
22. Start screen and game over screen polish

---

## 11. Constants Reference

```
WINDOW_WIDTH:            800
WINDOW_HEIGHT:           1000    (portrait orientation, like original arcade)
WINDOW_TITLE:            "Galaga"

PLAYER_SPEED:            350.0
PLAYER_Y:               -420.0   (near bottom)
PLAYER_HITBOX_RADIUS:    14.0
MAX_PLAYER_BULLETS:      2

FORMATION_CENTER_Y:      280.0
FORMATION_COL_SPACING:   48.0
FORMATION_ROW_SPACING:   44.0
FORMATION_BREATHE_AMP:   40.0
FORMATION_BREATHE_PERIOD: 3.0

BEE_HITBOX_RADIUS:       12.0
BUTTERFLY_HITBOX_RADIUS:  14.0
BOSS_HITBOX_RADIUS:       18.0

PLAYER_BULLET_SPEED:     600.0
ENEMY_BULLET_SPEED_BASE: 200.0

DIVE_INTERVAL_BASE:      3.5     (seconds, wave 1)
DIVE_INTERVAL_MIN:       1.0     (seconds, wave 10+)

RESPAWN_DELAY:           1.5     (seconds)
INVINCIBILITY_DURATION:  2.0     (seconds)

PARTICLE_COUNT:          12
PARTICLE_LIFETIME:       0.5     (seconds)

STAR_COUNT_LAYER_1:      60
STAR_COUNT_LAYER_2:      40
STAR_COUNT_LAYER_3:      20
STAR_SCROLL_SPEED_1:     15.0
STAR_SCROLL_SPEED_2:     30.0
STAR_SCROLL_SPEED_3:     50.0

BLOOM_INTENSITY:         0.3
```

**Note on window dimensions**: Original Galaga is portrait (taller than wide). 800x1000 preserves this feel and gives vertical room for formation + dive space + player area. This is a departure from the template's 1280x720.
