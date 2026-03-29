# Robotron 2084 — Technical Specification

A modernized 2D clone of Williams Electronics' Robotron: 2084 (1982), built with Bevy 0.18.1 using only primitive geometric shapes, HDR bloom, and custom particle effects.

---

## 1. Game Overview

### Core Loop

Single-screen arena twin-stick shooter. Left input moves the player, right input aims/fires. Waves of enemies spawn; the player must survive and optionally rescue human NPCs for escalating score bonuses. Waves increase in enemy count, type variety, and aggression. The game ends when all lives are lost.

### Arena

- Fixed rectangular playfield, no scrolling
- Visible electric-fence boundary rendered as four thin `Capsule2d` shapes (one per edge) with HDR color and bloom glow; no line primitives exist for `Mesh2d`, so thin capsules serve as the boundary segments
- Pulsing intensity animation on the border color via slow sine wave
- Interior is empty — no walls, no obstacles (except Electrode enemies which act as static hazards)
- Camera is fixed orthographic, framing the entire arena with small margin

### Coordinate System

All game measurements are in **logical units**, not pixels. The camera uses `ScalingMode::FixedVertical(720.0)` so 1 unit ~= 1 pixel at the default 960x720 window, but the arena scales correctly at any resolution. All constants in this spec are expressed in these logical units.

### Win/Lose Conditions

- **Wave clear**: all killable enemies destroyed (Hulks are indestructible and do not count)
- **Life lost**: player contacts any enemy or enemy projectile
- **Game over**: all lives exhausted

---

## 2. Entity Design (Primitive Shapes)

Every entity is built from Bevy `Mesh2d` + `MeshMaterial2d<ColorMaterial>` using primitive shapes. Colors use HDR values (> 1.0) via `Color::srgb(...)` for bloom glow. Each entity type must be instantly distinguishable by shape + color.

**Composite entities** (Hulk, Brain, Electrode) use multiple meshes. Per CLAUDE.md, avoid nested `.with_children` calls — instead, flatten all child meshes as siblings under one parent entity using a single `.with_children(|parent| { ... })` closure.

### Player

| Property | Value |
|----------|-------|
| Shape | Diamond (rotated square) — `RegularPolygon::new(r, 4)` rotated 45 deg |
| Color | `Color::srgb(0.0, 4.0, 5.0)` (cyan HDR) |
| Size | ~16 circumradius |
| Facing | Rotate diamond to point in aim direction |

### Grunts

| Property | Value |
|----------|-------|
| Shape | Small square — `RegularPolygon::new(r, 4)` |
| Color | `Color::srgb(5.0, 0.2, 0.2)` (red HDR) |
| Size | ~10 circumradius |
| Behavior | Chase player directly |
| Points | 100 |
| Markers | `Enemy`, `Killable`, `WaveEntity`, `Confined` |

### Hulks

| Property | Value |
|----------|-------|
| Shape | Large square — `RegularPolygon::new(r, 4)` with inner nested square (rotated 45 deg offset), flattened as sibling children under one parent |
| Color | `Color::srgb(0.3, 4.0, 0.3)` (green HDR) |
| Size | ~18 circumradius |
| Behavior | Wander + drift toward player, indestructible, kill humans on contact |
| Points | N/A (cannot be killed) |
| Special | Player bullets push Hulks backward instead of destroying them. Immune to Electrodes. |
| Markers | `Enemy`, `Hulk`, `WaveEntity`, `Confined` (no `Killable`) |

### Brains

| Property | Value |
|----------|-------|
| Shape | Circle — `Circle::new(r)` with smaller inner circle, flattened as sibling children under one parent |
| Color | `Color::srgb(5.0, 0.3, 4.0)` (magenta HDR) |
| Size | ~12 radius |
| Behavior | Seek nearest human, convert to Prog on contact, fire Cruise Missiles |
| Points | 500 |
| Markers | `Enemy`, `Killable`, `Brain`, `WaveEntity`, `Confined` |

### Progs (Converted Humans)

| Property | Value |
|----------|-------|
| Shape | Capsule — `Capsule2d::new(r, len)` |
| Color | `Color::srgb(4.0, 0.2, 3.0)` (magenta HDR, same family as Brains) |
| Size | ~8 radius, short capsule |
| Behavior | Chase player (faster than Grunts) |
| Points | 100 |
| Markers | `Enemy`, `Killable`, `WaveEntity`, `Confined` |

### Spheroids

| Property | Value |
|----------|-------|
| Shape | Circle — `Circle::new(r)`, pulsing scale animation |
| Color | `Color::srgb(5.0, 2.0, 0.2)` (orange HDR) |
| Size | ~14 radius, oscillates between 10-18 via scale |
| Behavior | Float in smooth random curves, periodically spawn Enforcers |
| Points | 1000 |
| Markers | `Enemy`, `Killable`, `Spawner`, `WaveEntity`, `Confined` |

### Enforcers (Spawned by Spheroids)

| Property | Value |
|----------|-------|
| Shape | Triangle — `RegularPolygon::new(r, 3)` |
| Color | `Color::srgb(3.0, 0.5, 5.0)` (purple HDR) |
| Size | ~10 circumradius |
| Behavior | Drift toward player, fire aimed Sparks |
| Points | 150 |
| Markers | `Enemy`, `Killable`, `WaveEntity`, `Confined` |

### Quarks

| Property | Value |
|----------|-------|
| Shape | Hexagon — `RegularPolygon::new(r, 6)`, rotating animation |
| Color | `Color::srgb(0.3, 4.0, 4.0)` (teal HDR) |
| Size | ~14 circumradius |
| Behavior | Float in smooth random curves, periodically spawn Tanks |
| Points | 1000 |
| Markers | `Enemy`, `Killable`, `Spawner`, `WaveEntity`, `Confined` |

### Tanks (Spawned by Quarks)

| Property | Value |
|----------|-------|
| Shape | Pentagon — `RegularPolygon::new(r, 5)` |
| Color | `Color::srgb(1.0, 4.0, 3.5)` (teal-white HDR) |
| Size | ~12 circumradius |
| Behavior | Slow drift, fire Bounce Shells that reflect off arena walls |
| Points | 200 |
| Markers | `Enemy`, `Killable`, `WaveEntity`, `Confined` |

### Electrodes (Static Hazards)

| Property | Value |
|----------|-------|
| Shape | Plus/cross — two `Capsule2d` shapes at 90 deg, flattened as sibling children under one parent |
| Color | `Color::srgb(5.0, 5.0, 0.3)` (yellow HDR) |
| Size | ~12 arm length |
| Behavior | Static, kill player on contact, kill enemies that touch them (except Hulks), destroyed by player bullets |
| Points | 25 |
| Markers | `Killable`, `Electrode`, `WaveEntity` (no `Enemy` — Electrodes are hazards, not enemies for collision purposes) |

### Human Family (Rescue Targets)

Three visual variants, identical gameplay behavior:

| Variant | Shape | Color |
|---------|-------|-------|
| Daddy | Tall capsule `Capsule2d::new(3.0, 8.0)` | `Color::srgb(0.5, 3.0, 0.5)` (green) |
| Mommy | Medium capsule `Capsule2d::new(3.0, 6.0)` | `Color::srgb(0.5, 0.5, 4.0)` (blue) |
| Mikey | Small circle `Circle::new(5.0)` | `Color::srgb(3.0, 3.0, 3.0)` (white) |

Behavior: Wander randomly with direction changes every 1-3 seconds.

Markers: `Human`, `WaveEntity`, `Confined`

### Projectiles

| Type | Shape | Color | Behavior | Markers |
|------|-------|-------|----------|---------|
| Player Bullet | Small circle `Circle::new(3.0)` | `Color::srgb(5.0, 5.0, 5.0)` (white) | Straight line, fixed speed, despawn on hit or leaving arena | `PlayerBullet`, `WaveEntity` |
| Cruise Missile (Brain) | Small diamond `RegularPolygon::new(4.0, 4)` | `Color::srgb(5.0, 1.0, 2.0)` (pink) | Homes toward player with turning rate limit | `EnemyProjectile`, `WaveEntity` |
| Enforcer Spark | Tiny circle `Circle::new(2.0)` | `Color::srgb(4.0, 0.5, 4.0)` (purple) | Aimed at player position at time of firing, straight line | `EnemyProjectile`, `WaveEntity` |
| Tank Bounce Shell | Small square `RegularPolygon::new(3.0, 4)` | `Color::srgb(1.0, 5.0, 4.0)` (teal) | Reflects off arena walls, limited bounces before despawn | `EnemyProjectile`, `WaveEntity` |

---

## 3. Input System

### Dual-Input Mapping

The game requires simultaneous move + aim inputs. Support two schemes:

**Keyboard (primary)**:
- Move: WASD
- Aim/Fire: Arrow keys (firing is automatic while any arrow key is held)
- Pause: Escape

**Gamepad**:
- Move: Left stick
- Aim/Fire: Right stick (fire when stick is deflected past deadzone threshold)
- Pause: Start button

### Input Processing Details

**Keyboard normalization**: When two movement keys are held (diagonal), normalize the resulting vector to unit length to prevent faster diagonal movement. Same for aim direction.

**Keyboard aim directions**: 8 discrete directions from arrow key combinations (4 cardinal + 4 diagonal). No interpolation — this is authentic and works well with keyboard.

**Gamepad analog aim**: Convert right stick to continuous angle. Apply a deadzone (0.2 recommended) below which no firing occurs. Above deadzone, fire in the stick's direction. This is the "modernized" enhancement — analog aim on gamepad.

**Fire rate**: Cooldown timer between shots (e.g., 0.08s). Store as `Timer` component on the player entity. Tick with `time.delta()`. Bullets spawn at player position traveling in aim direction.

### Hard Problem: Keyboard Ghosting

Some keyboards cannot register certain 3+ key combinations (e.g., W + D + Up + Right). This is a hardware limitation with no software fix. Document recommended key layouts that minimize ghosting risk. WASD + Arrows is generally safe on most keyboards because they're on opposite sides of the matrix.

---

## 4. Movement & Physics

### Player Movement

- Constant speed while any move key is held, zero when released (no momentum/inertia)
- Speed: tunable constant (e.g., `PLAYER_SPEED = 300.0` units/sec)
- Clamped to arena boundaries each frame

### Enemy Movement

All enemy movement uses `velocity * time.delta()` for frame-rate independence. Store velocity as a `Velocity(Vec2)` component on each moving entity.

**Grunts**: Move toward player position each frame. Speed increases with wave number (controlled by the wave's `speed_mult`). Apply a small random offset to target position to prevent perfect convergence — this creates the "swarming" visual where grunts approach from slightly different angles.

**Hulks**: Weighted random walk with player-attraction bias. Each frame: 70% chance to step toward player, 30% random direction. Slower than grunts. Knockback on bullet hit: apply impulse in bullet's travel direction, decaying over ~0.3s via a `Knockback { velocity: Vec2 }` component that decays each frame.

**Brains**: Seek nearest living human. If no humans remain, seek player. Medium speed. Finding the nearest human is a nested query: for each Brain, iterate all `With<Human>` entities and pick the closest by `distance_squared`. With ~10 Brains and ~20 humans, this is ~200 distance calculations — trivial.

**Progs**: Direct chase toward player, faster than Grunts (1.3x Grunt speed).

**Spheroids/Quarks**: Smooth random curves — implement as: pick a random target point in the arena, move toward it with slight sine-wave oscillation perpendicular to travel direction. Pick a new target on arrival. Store target in a `WanderTarget(Vec2)` component.

**Enforcers**: Drift toward player at slow speed. Fire every 1.5-3 seconds (random interval per Enforcer, stored as a `FireCooldown(Timer)` component).

**Tanks**: Very slow drift. Fire every 2-4 seconds (same `FireCooldown(Timer)` pattern).

### Arena Boundary Enforcement

Every entity with a `Confined` marker component is clamped within `[ARENA_MIN, ARENA_MAX]` after position update. Implement as a single system running in the `Confinement` set (after `Movement`), querying `(Entity, &mut Transform, With<Confined>)`.

The `Confined` component is defined in `components.rs` as a unit struct marker.

### Enemy Separation (Anti-Stacking)

When multiple enemies of the same type occupy nearly the same position, they become visually indistinguishable. Apply a soft separation force:
- For each enemy, check distance to other enemies within a small radius (~30 units)
- Apply a gentle repulsion impulse (inverse to distance)
- This runs only between same-type enemies to keep costs low
- Cap the repulsion magnitude so it doesn't overpower chase behavior

**Tradeoff**: This is not in the original game. It improves visual clarity with primitives (where overlapping shapes are harder to distinguish than overlapping sprites) at the cost of slightly less authentic enemy behavior and some CPU cost. Worth it for this rendering style.

---

## 5. Collision Detection

### Approach: Manual Circle-Circle

All collision volumes are circles (even for square/polygon entities). Each collidable entity gets a `CollisionRadius(f32)` component. Detection is `distance_squared < (r1 + r2)^2`.

**Why not bevy_rapier2d**: Adding a full physics engine for simple overlap detection is massive dependency overhead. The game has no physics responses (no bouncing off each other, no friction). Only needs "do these two things overlap?" checks.

**Why circles for square-shaped entities**: Greatly simplifies collision math. The visual polygon shape doesn't need to match the collision shape exactly — the difference is imperceptible at this scale.

### Collision Layers (Separate Systems)

Each collision relationship is its own system to avoid unnecessary pairwise checks:

| System | Query A Filter | Query B Filter | Response |
|--------|---------------|---------------|----------|
| `player_vs_enemy` | `With<Player>` | `With<Enemy>` | Kill player (unless invincible) |
| `player_vs_enemy_projectile` | `With<Player>` | `With<EnemyProjectile>` | Kill player (unless invincible) |
| `bullet_vs_killable` | `With<PlayerBullet>` | `With<Killable>, With<Enemy>` | Destroy both, spawn explosion, add score |
| `bullet_vs_hulk` | `With<PlayerBullet>` | `With<Hulk>` | Destroy bullet, apply knockback to Hulk |
| `bullet_vs_electrode` | `With<PlayerBullet>` | `With<Electrode>` | Destroy both, add score |
| `player_vs_human` | `With<Player>` | `With<Human>` | Rescue human, add escalating score |
| `hulk_vs_human` | `With<Hulk>` | `With<Human>` | Kill human |
| `brain_vs_human` | `With<Brain>` | `With<Human>` | Convert human to Prog |
| `enemy_vs_electrode` | `With<Enemy>, Without<Hulk>` | `With<Electrode>` | Kill the enemy (Hulks are immune) |

Note: `player_vs_enemy` and `player_vs_enemy_projectile` are separate systems so enemy bodies and enemy projectiles (Cruise Missiles, Sparks, Bounce Shells) are both checked. The `EnemyProjectile` marker is on all enemy-fired projectiles.

Progs have the `Enemy` and `Killable` markers, so they are automatically included in `bullet_vs_killable` and `player_vs_enemy`.

### Performance Estimate

Worst case per frame: ~100 enemies, ~30 bullets, ~20 humans, ~15 enemy projectiles, 1 player.
- `player_vs_enemy`: 1 x 100 = 100 checks
- `player_vs_enemy_projectile`: 1 x 15 = 15 checks
- `bullet_vs_killable`: 30 x 90 = 2,700 checks
- `bullet_vs_hulk`: 30 x ~10 = 300 checks
- `bullet_vs_electrode`: 30 x ~15 = 450 checks
- `player_vs_human`: 1 x 20 = 20 checks
- `hulk_vs_human`: 10 x 20 = 200 checks
- `brain_vs_human`: ~10 x 20 = 200 checks
- `enemy_vs_electrode`: ~85 x ~15 = 1,275 checks

Total: ~5,260 distance checks per frame. This is trivially fast — no spatial partitioning needed.

### Hard Problem: Bullet Tunneling

At `BULLET_SPEED = 800 units/s` and 60 FPS, a bullet moves ~13 units per frame. Enemy collision radii are ~10-14 units. This means a bullet can skip over a thin enemy in one frame.

**Mitigation options**:
1. **Swept circle test** (raycast from old to new position): Correct but adds complexity.
2. **Cap bullet speed** so max displacement per frame < smallest collision diameter: At 60fps, need speed < 600 units/s for 10-unit radius enemies. Limiting.
3. **Increase collision radii slightly**: Cheapest fix, barely noticeable.

**Decision**: Use option 3 (generous collision radii) as the primary approach. Bullet speed of ~800 units/s with enemy radii of 12-14 means the gap per frame is ~13 units — still within radius range. Only a problem if we wanted very small enemies with very fast bullets. If issues arise during testing, add swept tests for bullets only.

---

## 6. Wave System

### Wave Data Structure

```rust
struct WaveDefinition {
    grunts: u32,
    hulks: u32,
    brains: u32,
    spheroids: u32,
    quarks: u32,
    electrodes: u32,
    humans: u32,       // total humans (randomly mixed among Daddy/Mommy/Mikey)
    speed_mult: f32,   // multiplier applied to ALL enemy base speeds this wave
}
```

Store as a `const` array in `waves.rs`. No external data file — keeps things simple and compile-time checked.

### Wave Progression

| Waves | Enemies Introduced | Notes |
|-------|-------------------|-------|
| 1-2 | Grunts + Electrodes + Humans | Tutorial difficulty |
| 3-4 | + Hulks | Teach "can't kill everything" |
| 5-7 | + Brains | Human conversion pressure |
| 8-10 | + Spheroids -> Enforcers | Projectile threats |
| 11+ | + Quarks -> Tanks | Bouncing shells, full roster |
| 15+ | All types, counts scaling linearly | Endurance |

After the last defined wave, repeat the final wave template with +10% enemy counts per cycle (capped at a sane maximum like 200 total enemies to prevent frame drops) and +0.1 `speed_mult` per cycle.

### Spawn Placement

- Player always spawns at arena center
- Enemies spawn at random positions along the arena edges (not in the interior) to give the player a brief reaction window
- Humans spawn at random interior positions but not within a `SPAWN_EXCLUSION_RADIUS` zone around the player
- Electrodes spawn at random interior positions (also respecting the exclusion zone)
- Enforce minimum distance between spawn positions (reject and re-roll positions within 20 units of an already-placed entity) to prevent stacking at spawn
- Brief wave intro delay (~1.5s) where enemies are visible but frozen — gives the player time to read the field

### Wave Transition

1. Last killable enemy dies -> `WaveClear` state
2. Short celebration pause (~1.5s) with visual effect (screen flash, score tally)
3. All remaining `WaveEntity` entities despawn (Hulks, particles, projectiles, humans)
4. Wave counter increments
5. New wave spawns -> brief freeze -> `WaveActive`

### Wave Cleanup Strategy

All gameplay entities that should be cleaned up between waves carry the `WaveEntity` marker component. Cleanup is performed explicitly via a system on `OnEnter(PlayState::WaveIntro)` that queries and despawns all `With<WaveEntity>` entities.

**Why not `DespawnOnExit(PlayState::WaveActive)`**: This would also despawn entities when transitioning to `Paused` or `PlayerDeath`, which is wrong. Manual cleanup on `OnEnter(PlayState::WaveIntro)` is the correct trigger because it fires only when actually starting a new (or re-started) wave.

The top-level `DespawnOnExit(AppState::Playing)` is still used on all gameplay entities as a safety net for game-over cleanup.

### Hard Problem: Spawner Enemies (Spheroids/Quarks) and Wave Completion

Spheroids and Quarks continuously spawn children (Enforcers/Tanks). If we require all killable enemies dead to clear a wave, the player must kill spawners before they flood the arena. This is correct — it matches the original design and creates strategic priority (kill spawners first).

**Spawn limits**: Each Spheroid can spawn at most 5 Enforcers. Each Quark can spawn at most 4 Tanks. Track child count in a `SpawnerState { children_spawned: u32, max_children: u32, cooldown: Timer }` component. This prevents infinite spawning if the player ignores spawners.

Also enforce a global cap of `MAX_TOTAL_ENEMIES` (150) — spawners stop producing when the cap is hit.

---

## 7. Scoring & Progression

### Points Table

| Entity | Points |
|--------|--------|
| Grunt | 100 |
| Enforcer | 150 |
| Tank | 200 |
| Electrode | 25 |
| Brain | 500 |
| Prog | 100 |
| Spheroid | 1000 |
| Quark | 1000 |
| Hulk | -- (indestructible) |

### Human Rescue Bonus

Escalating per wave:
- 1st rescue: 1,000
- 2nd: 2,000
- 3rd: 3,000
- 4th: 4,000
- 5th+: 5,000 each

The counter resets each wave. If all humans die in a wave, the player receives no rescue bonuses for the remainder of that wave.

### Extra Lives

Award an extra life at every 25,000 points. No cap. Store lives in a `GameState` resource.

### High Score

- Track the session-best score in a resource (resets on app restart)
- **Optional future**: persist to file using `std::fs` to a local path. Not critical for initial implementation — flag as enhancement.

---

## 8. Visual Effects

### Bloom / HDR Pipeline

Camera setup (matching CLAUDE.md example exactly):
```rust
commands.spawn((
    Camera2d,
    Camera {
        clear_color: ClearColorConfig::Custom(Color::BLACK),
        ..default()
    },
    OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical { viewport_height: 720.0 },
        ..OrthographicProjection::default_2d()
    },
    Tonemapping::TonyMcMapface,
    Bloom::NATURAL,
    DebandDither::Enabled,
));
```

All entity colors use `Color::srgb(...)` with values > 1.0 for glow. The bloom post-process extracts bright regions above its threshold. `ColorMaterial` has no `emissive` field — brightness comes from the color values directly.

### Particle System (Custom)

No Bevy built-in particle system. Implement as entities:

```rust
#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime: Timer,
}
```

Each particle is a `Circle::new(size)` mesh with a **shared** `ColorMaterial` handle (one per particle color family — e.g., red explosions, gold sparkles). Fade effect is achieved via **scale reduction**, not alpha — shrink `transform.scale` toward zero over the particle's lifetime. This avoids needing per-particle materials.

On each frame:
- Move by `velocity * delta`
- Apply drag: `velocity *= PARTICLE_DRAG` (0.95 per frame) for deceleration
- Interpolate `transform.scale` from 1.0 -> 0.0 over lifetime
- Despawn when lifetime expires

All particles use the same component set (same archetype) to minimize ECS fragmentation from rapid spawn/despawn.

**Explosion effect**: Spawn 12-20 particles in radial burst from death position. Initial velocity = random direction * random magnitude (200-500 units/s). Color material = dying entity's color family. Lifetime = 0.3-0.6s. Size = 2-4 units.

**Rescue sparkle**: Spawn 8-12 particles upward with slight spread. Color = gold `Color::srgb(5.0, 4.0, 0.5)`. Slower velocity. Lifetime = 0.5s.

**Player death**: Large explosion (30+ particles), screen shake, brief screen flash (spawn a full-screen quad with `Color::srgb(3.0, 3.0, 3.0)`, scale toward zero over 0.15s).

### Screen Shake

- Store `ScreenShake { trauma: f32 }` as a resource
- On shake trigger: set `trauma` to 1.0 (or add to existing, clamped to 1.0)
- Each frame: `trauma -= SCREEN_SHAKE_DECAY * delta` (linear decay, not multiplicative — avoids never-reaching-zero float issue). Clamp to 0.0.
- Camera offset = `(random_in_range(-1, 1) * trauma^2 * MAX_SHAKE, random_in_range(-1, 1) * trauma^2 * MAX_SHAKE)`
- `MAX_SHAKE = 8.0` units
- Triggers: player death (trauma = 1.0), wave clear (trauma = 0.5), Hulk knockback (trauma = 0.15)

### Entity Animations

- **Spheroid pulse**: Oscillate `transform.scale` between 0.7 and 1.3 using `(time.elapsed_secs() * frequency).sin()`
- **Quark spin**: Rotate `transform.rotation` continuously at ~2 rad/s
- **Electrode flicker**: Randomly toggle `Visibility` between `Visible` and `Hidden` every few frames using a `Timer`
- **Human wander bob**: Small vertical oscillation on Y position using sine wave
- **Brain inner circle rotation**: Rotate inner mesh child opposite to outer
- **Arena border pulse**: Modulate border material color intensity with slow sine wave

### Score Popups

When an enemy is destroyed or human rescued, spawn `Text2d::new(score_string)` at the event position with `TextFont` and `TextColor`. Animate upward (Y += `SCORE_POPUP_RISE_SPEED * delta`) while fading `TextColor` alpha from 1.0 -> 0.0. Despawn when alpha reaches 0. Per-entity `TextColor` mutation is fine here — `TextColor` is a component, not a shared asset.

Tag with `WaveEntity` and `DespawnOnExit(AppState::Playing)`.

### Wave Intro Text

On wave start, display "WAVE {N}" as **Node-based UI text** (not `Text2d`), centered on screen. This ensures it stays fixed during screen shake. Fade in over 0.3s, hold 0.8s, fade out over 0.3s. Use HDR text color for bloom glow.

Spawn on `OnEnter(PlayState::WaveIntro)`, despawn on `OnExit(PlayState::WaveIntro)`.

---

## 9. State Machine

### Top-Level States

```rust
#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum AppState {
    #[default]
    StartScreen,
    Playing,
    GameOver,
}
```

### Play Sub-States

```rust
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Playing)]
enum PlayState {
    #[default]
    WaveIntro,    // brief freeze, show wave number
    WaveActive,   // gameplay running
    WaveClear,    // all enemies dead, celebration pause
    Paused,
    PlayerDeath,  // death animation playing, brief respite before respawn
}
```

Registration: `app.init_state::<AppState>().add_sub_state::<PlayState>();`

### State Transitions

```
StartScreen --[any key/button]--> Playing (WaveIntro)
WaveIntro --[timer expires]--> WaveActive
WaveActive --[all killable enemies dead]--> WaveClear
WaveActive --[player hit]--> PlayerDeath
PlayerDeath --[death anim done, lives > 0]--> WaveIntro (same wave, re-spawn enemies)
PlayerDeath --[death anim done, lives == 0]--> GameOver
WaveClear --[celebration done]--> WaveIntro (next wave)
WaveActive --[pause input]--> Paused
Paused --[pause input]--> WaveActive
GameOver --[any key/button after delay]--> StartScreen
```

### System Gating

- All gameplay movement, AI, and collision systems: `.run_if(in_state(PlayState::WaveActive))`
- Particle/animation/effect update systems: `.run_if(in_state(AppState::Playing))` (run during all play sub-states including Paused for visual continuity — or gate particles differently if pause should freeze effects)
- Wave intro/clear UI systems: gated on their respective `PlayState` via `OnEnter`/`OnExit`
- Input system reads pause key in all `PlayState` variants; reads gameplay input only in `WaveActive`

### Entity Scoping

- All gameplay entities use both `WaveEntity` (for wave-transition cleanup) and `DespawnOnExit(AppState::Playing)` (for game-over cleanup)
- Start screen UI entities use `DespawnOnExit(AppState::StartScreen)`
- Game over UI entities use `DespawnOnExit(AppState::GameOver)`
- Wave intro/clear overlay UI uses `OnEnter`/`OnExit` of the specific `PlayState` variant

### Hard Problem: Wave Clear During Player Death

If a player bullet kills the last enemy during the `PlayerDeath` state, the wave should clear but the player is dead. Solution: track `wave_cleared: bool` on the wave resource. The `WaveActive -> WaveClear` transition checks this flag. After the player respawns into `WaveIntro`, the system sees `wave_cleared == true` and skips to next wave instead of re-spawning enemies. Reset the flag on each wave start.

---

## 10. Module Architecture

```
src/
  main.rs          -- App setup, plugin registration, window config
  constants.rs     -- All tunable game values
  components.rs    -- ECS components (markers + data), see Section 11
  resources.rs     -- Game state resources (score, lives, wave, etc.)
  states.rs        -- AppState, PlayState enums
  player.rs        -- PlayerPlugin: movement, shooting, death/respawn
  enemy.rs         -- EnemyPlugin: spawning, AI behaviors, spawner logic
  combat.rs        -- CombatPlugin: collision detection, damage, knockback
  human.rs         -- HumanPlugin: wandering, rescue, conversion to Prog
  waves.rs         -- WavePlugin: wave definitions, progression, spawn orchestration
  effects.rs       -- EffectsPlugin: particles, screen shake, flashes, popups
  rendering.rs     -- RenderingPlugin: entity mesh/material setup, animations, shared material resource
  ui.rs            -- UiPlugin: HUD (score, lives, wave), start screen, game over screen
  arena.rs         -- ArenaPlugin: boundary rendering, boundary enforcement
  audio.rs         -- AudioPlugin: placeholder/future sound effects
```

### System Ordering

Within `WaveActive`, systems have frame-order dependencies:

```
Input -> Movement -> Confinement -> Collision -> Response -> Effects
```

Use `SystemSet`s (the chain exceeds 8 systems when expanded):

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet {
    Input,
    Movement,
    Confinement,
    Collision,
    Response,
    Effects,
}
```

Configure ordering in `main.rs` plugin setup:
```rust
app.configure_sets(
    Update,
    (
        GameSet::Input,
        GameSet::Movement,
        GameSet::Confinement,
        GameSet::Collision,
        GameSet::Response,
        GameSet::Effects,
    ).chain()  // chain() creates sequential ordering
);
```

Each system is assigned to its set: `.in_set(GameSet::Collision)`. Within a set, systems run in parallel unless further ordered with `.after()`.

---

## 11. Component & Resource Definitions

### Components (`components.rs`)

```rust
// === Marker Components ===

#[derive(Component)] struct Player;
#[derive(Component)] struct Enemy;
#[derive(Component)] struct Killable;        // can be destroyed by player bullets
#[derive(Component)] struct Hulk;            // indestructible enemy subtype
#[derive(Component)] struct Brain;           // enemy that converts humans
#[derive(Component)] struct Electrode;       // static hazard
#[derive(Component)] struct Human;           // rescue target
#[derive(Component)] struct PlayerBullet;
#[derive(Component)] struct EnemyProjectile; // all enemy-fired projectiles
#[derive(Component)] struct WaveEntity;      // despawned on wave transition
#[derive(Component)] struct Confined;        // clamped to arena boundaries

// === Data Components ===

#[derive(Component)] struct Velocity(Vec2);
#[derive(Component)] struct CollisionRadius(f32);
#[derive(Component)] struct PointValue(u32);          // score awarded on kill
#[derive(Component)] struct Health;                    // presence = alive (unit struct)

#[derive(Component)]
struct Knockback {
    velocity: Vec2,  // decays each frame
}

#[derive(Component)]
struct FireCooldown(Timer);  // per-entity fire rate

#[derive(Component)]
struct SpawnerState {
    children_spawned: u32,
    max_children: u32,
    cooldown: Timer,
}

#[derive(Component)]
struct WanderTarget(Vec2);  // for Spheroid/Quark random movement

#[derive(Component)]
struct WanderTimer(Timer);  // for Human direction changes

#[derive(Component)]
struct HomingMissile {
    turn_rate: f32,   // rad/s
}

#[derive(Component)]
struct BouncesRemaining(u32);  // for Tank shells

#[derive(Component)]
struct Lifetime(Timer);        // generic despawn timer

#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime: Timer,
}

#[derive(Component)]
struct Invincible(Timer);  // player spawn protection

#[derive(Component)]
struct ScorePopup {
    rise_speed: f32,
    lifetime: Timer,
}

// === Enum for enemy type identification ===

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnemyType {
    Grunt, Hulk, Brain, Prog, Spheroid, Enforcer, Quark, Tank,
}
```

### Resources (`resources.rs`)

```rust
#[derive(Resource)]
struct GameState {
    score: u32,
    high_score: u32,
    lives: u32,
    current_wave: u32,
    humans_rescued_this_wave: u32,  // for escalating bonus
    next_extra_life_at: u32,        // score threshold for next 1UP
}

#[derive(Resource)]
struct ScreenShake {
    trauma: f32,   // 0.0 = none, 1.0 = max
}

#[derive(Resource)]
struct WaveState {
    wave_cleared: bool,          // true if last enemy died during PlayerDeath
    intro_timer: Timer,
    clear_timer: Timer,
    death_timer: Timer,
}

// Inter-system communication (replaces Bevy events per CLAUDE.md)
#[derive(Resource, Default)]
struct GameEvents {
    enemy_killed: Vec<(Vec2, EnemyType, u32)>,  // position, type, points
    human_rescued: Vec<(Vec2, u32)>,             // position, bonus
    human_died: Vec<Vec2>,                       // position
    player_died: Vec<Vec2>,                      // position
    // Drain vecs each frame after processing in Response/Effects sets
}

// Shared mesh + material handles (loaded once at startup)
#[derive(Resource)]
struct GameAssets {
    // Meshes (one per shape)
    player_mesh: Handle<Mesh>,
    grunt_mesh: Handle<Mesh>,
    hulk_outer_mesh: Handle<Mesh>,
    hulk_inner_mesh: Handle<Mesh>,
    // ... one per entity shape variant

    // Materials (one per color)
    player_material: Handle<ColorMaterial>,
    grunt_material: Handle<ColorMaterial>,
    hulk_material: Handle<ColorMaterial>,
    // ... one per entity color
    // Particle materials: one per color family (red, gold, white, etc.)
}
```

### Observers

Use Bevy `Observer`/`Trigger` for entity-specific one-shot reactions where appropriate:
- **On player entity spawn**: attach invincibility timer, start blink animation
- **On Prog spawn** (human converted): could use an observer on `Added<Enemy>` + `With<Prog>` component combination, or simply handle in the brain-vs-human collision system that spawns the Prog directly

For cross-system data flow (collision -> scoring -> effects), prefer the `GameEvents` resource pattern since multiple systems need to react to the same event.

---

## 12. UI Layout

### HUD (During Play)

```
+------------------------------------------+
|  SCORE: 12500       WAVE 3     LIVES: ***|
|                                          |
|             [gameplay area]              |
|                                          |
|  HIGH: 45000                             |
+------------------------------------------+
```

- Score: top-left, large text, updates in real-time
- Wave number: top-center
- Lives: top-right, shown as small player-colored diamond icons (not number) — spawn N `ImageNode` or colored `Node` boxes
- High score: bottom-left, smaller text
- All text uses **Node-based UI** (not `Text2d`) so it overlays the game world and is unaffected by camera shake

### Start Screen

```
+------------------------------------------+
|                                          |
|           R O B O T R O N               |
|              2 0 8 4                     |
|                                          |
|         [rotating enemy shapes]          |
|                                          |
|        PRESS ANY KEY TO START            |
|                                          |
|          HIGH SCORE: 45000               |
+------------------------------------------+
```

- Title text with bloom glow (HDR color via `TextColor`)
- Animated enemy shape showcase — a few entities drifting across the screen as decoration (world-space `Mesh2d` entities, not UI)
- Pulsing "press any key" prompt (sine-wave on `TextColor` alpha)

### Game Over Screen

```
+------------------------------------------+
|                                          |
|            G A M E   O V E R            |
|                                          |
|           FINAL SCORE: 32100             |
|                                          |
|        PRESS ANY KEY TO CONTINUE         |
|                                          |
+------------------------------------------+
```

- Brief delay (~2s, driven by a `Timer` resource) before accepting input to prevent accidental skip
- If new high score, show "NEW HIGH SCORE!" with extra bloom

---

## 13. Edge Cases & Hard Problems

### 13.1 Player Spawn Safety

**Problem**: On wave start or respawn after death, enemies might be near the center where the player spawns.

**Solution**: Player gets 2 seconds of invincibility on spawn (and on wave start), tracked via `Invincible(Timer)` component. Visual indicator: player entity blinks (toggle `Visibility` every 0.1s using a secondary timer or modular arithmetic on the invincibility timer's elapsed time). During invincibility, collision systems skip the player via `Without<Invincible>` query filter or an explicit `Invincible` presence check.

### 13.2 All Humans Dead

**Problem**: If all humans are killed (by Hulks) or converted (by Brains) mid-wave, the rescue mechanic is gone.

**Solution**: This is intended — it creates tension to protect humans. No special handling needed. The `humans_rescued_this_wave` counter simply stops incrementing. Wave still clears when enemies are dead.

### 13.3 Brain-Human Conversion Race

**Problem**: A Brain reaches a human and converts it to a Prog. But what if the player is also touching the human the same frame?

**Solution**: Run `player_vs_human` (rescue) before `brain_vs_human` (conversion) in system ordering. Both are in `GameSet::Collision`, so add explicit `.before()` within the set. Player rescue takes priority — the human is despawned by rescue before the Brain system sees it.

### 13.4 Hulk Pushing vs. Arena Boundary

**Problem**: A Hulk is knocked back by bullets toward the arena edge. What happens when it hits the boundary?

**Solution**: Arena confinement system clamps Hulk position to within bounds, same as all entities. The knockback velocity is simply zeroed by the clamp. Hulks cannot leave the arena.

### 13.5 Spawner Flood

**Problem**: If the player ignores Spheroids/Quarks, they continuously spawn children, eventually flooding the arena.

**Solution**: Per-spawner child limit (5 Enforcers per Spheroid, 4 Tanks per Quark) via `SpawnerState`. Also a global cap of `MAX_TOTAL_ENEMIES` (150) — spawners stop producing when the cap is hit. Check the cap with a simple `Query<With<Enemy>>.iter().count()` in the spawner system.

### 13.6 Frame Rate Spikes

**Problem**: A large explosion spawns 30 particles + despawns several entities in one frame, causing a hitch.

**Solution**: Particle count budget. Cap total alive particles at `MAX_PARTICLES` (200). When spawning a new batch would exceed the cap, reduce particle count per explosion proportionally. Check with `Query<With<Particle>>.iter().count()` before spawning. Keep all particles in the same archetype (identical component set) to minimize ECS overhead.

### 13.7 Gamepad Hot-Plug

**Problem**: Player unplugs gamepad mid-game.

**Solution**: Bevy detects gamepad connection/disconnection automatically. If the active gamepad disconnects during gameplay, auto-pause and show "Controller Disconnected" overlay. Resume on reconnection. If keyboard was being used, no issue — gamepad events simply produce no input. Track which input device is active via a `Resource` that latches to whichever device last produced input.

### 13.8 Enemy-on-Enemy Stacking at Spawn

**Problem**: Random spawn positions might place two enemies at the exact same location.

**Solution**: Enforce minimum distance between spawn positions at wave start (reject and re-roll positions within 20 units of an already-placed enemy). Use a simple loop with a retry limit (e.g., 10 attempts) — if a valid position isn't found, place anyway; the separation force system handles convergence within a few frames.

### 13.9 Tank Shell Bounce Lifetime

**Problem**: A Tank shell bouncing between two parallel arena walls could bounce forever.

**Solution**: Track remaining bounces in `BouncesRemaining(u32)` component, initialized to `TANK_SHELL_MAX_BOUNCES` (3). Decrement on each wall reflection. Despawn when it reaches 0. Also apply a `Lifetime(Timer)` of 5 seconds as a safety net.

### 13.10 Wave Clear During PlayerDeath

**Problem**: A player bullet kills the last enemy during the player death animation. The wave should clear, but the player is dead.

**Solution**: Track `wave_cleared: bool` on `WaveState` resource. The wave-clear check system runs even during `PlayerDeath`. When the player respawns and `OnEnter(PlayState::WaveIntro)` fires, check this flag: if true, skip enemy re-spawn, increment wave counter, and proceed to next wave's intro. Reset the flag at each wave start.

### 13.11 Score Overflow

**Problem**: Theoretically, score can exceed `u32::MAX` (~4.3 billion) in an extreme session.

**Solution**: Not a realistic concern. Original arcade game capped at 9,999,999. Use `u32`, which is more than sufficient. Optionally clamp display to 9,999,999 for visual consistency.

### 13.12 Cruise Missile Homing Behavior

**Problem**: Homing missiles that perfectly track the player are undodgeable.

**Solution**: Limit the missile's turn rate via `HomingMissile { turn_rate }` (e.g., max 2 rad/s). Each frame, compute the angle to the player, compute the angle delta, clamp to `turn_rate * delta`, apply rotation. This creates a curved trajectory the player can dodge by moving perpendicular. Also attach a `Lifetime(Timer)` (4s) after which the missile despawns.

### 13.13 Particle Drag Decay (Float Precision)

**Problem**: Multiplicative drag (`velocity *= 0.95`) never truly reaches zero, leaving near-zero velocity particles drifting forever.

**Solution**: Particles are despawned by their `lifetime` timer, not by velocity reaching zero. The drag is purely visual (deceleration). No precision issue because lifetime is the authoritative despawn mechanism.

### 13.14 Shared Material Mutation

**Problem**: If any system mutates a shared `ColorMaterial` handle's underlying asset (e.g., to flash an entity white on hit), ALL entities sharing that material change color.

**Solution**: Never mutate shared material assets for per-entity effects. For hit flashes, briefly swap the entity's material handle to a separate "flash white" shared material, then swap back via a `Timer`. For the electrode flicker, toggle `Visibility` instead of changing material color.

---

## 14. Tradeoffs & Decisions

### 14.1 Manual Collision vs. Physics Engine

**Chose**: Manual circle-circle collision.

**Why**: The game needs only overlap detection, not physics simulation. No rigid body responses, no joints, no continuous collision detection. Adding `bevy_rapier2d` (~40k lines, WASM concerns, compile time) for `distance < r1+r2` is not justified.

**Risk**: If we later want more complex collision shapes or physics responses, we'd need to bolt on a library or build more collision math. Acceptable for this scope.

### 14.2 Entity-Per-Particle vs. GPU Instancing

**Chose**: Entity-per-particle (each particle is a Bevy entity with Mesh2d + Material).

**Why**: Simpler to implement, works within ECS naturally, easy to apply per-particle behavior (drag, scale decay). Bevy batches draw calls for identical meshes/materials automatically.

**Risk**: At very high particle counts (500+), entity spawn/despawn churn could pressure the ECS. Mitigated by the 200-particle budget. If performance becomes an issue, future optimization could use a single mesh with instanced rendering or a compute shader particle system.

### 14.3 8-Direction Keyboard Aim vs. Mouse Aim

**Chose**: 8-direction keyboard aim + analog gamepad aim. No mouse aim.

**Why**: Robotron's identity is twin-stick. Mouse aim changes the game feel fundamentally (it becomes a mouse-aim shooter). Keyboard 8-direction is the closest to the original dual-joystick on a keyboard. Gamepad analog aim is the modernized upgrade.

**Risk**: Some players may expect mouse aim. Could be added later as an optional third input mode without architectural changes.

### 14.4 Fixed Logical Resolution with Camera Scaling

**Chose**: Fixed logical resolution of 720 vertical units via `ScalingMode::FixedVertical { viewport_height: 720.0 }`. Window is resizable; the camera scales to fill.

**Why**: Fixed logical units simplify all position math, spawn zones, collision radii, and UI layout. Camera scaling handles different window sizes and monitors. At the default 960x720 window, 1 unit ~= 1 pixel.

**Risk**: Very wide or very tall windows will show more/less horizontal space. The arena boundary clips this visually. Letterboxing could be added later if needed.

### 14.5 Shared Materials with Scale-Based Particle Fade

**Chose**: Shared materials per entity type stored in a `GameAssets` resource. Particle fade uses `transform.scale` reduction, not alpha.

**Why**: One `Handle<ColorMaterial>` per entity type enables Bevy draw call batching. Particles share materials by color family (red, gold, white, etc.). Scale-based fade avoids per-particle material allocation entirely.

**Drawback**: Scale fade looks slightly different from alpha fade (shrinking dot vs. fading dot). Visually acceptable for small fast particles.

### 14.6 Wave Data: Const Array vs. Config File

**Chose**: Const array in `waves.rs`.

**Why**: No runtime I/O, no parsing, no error handling, compile-time validated. Wave tuning happens in code, which is fine for a single-developer project.

**Risk**: Tuning requires recompile. Acceptable given Bevy's fast incremental compile with `dynamic_linking`.

### 14.7 Sound

**Chose**: Stub `AudioPlugin` with no implementation initially.

**Why**: The user's requirements focus on visual presentation (primitive shapes, fancy graphics). Audio is important for game feel but is a separate concern. The plugin stub ensures the architecture supports audio when it's added later.

**Risk**: The game will feel flat without sound. Plan to add procedural/synthesized sounds (Bevy's built-in audio + generated WAV assets) as an enhancement pass.

### 14.8 Randomness: `rand` Crate

**Chose**: Add `rand` crate as a dependency.

**Why**: Bevy does not expose a public game-use RNG API. Numerous systems require randomness: enemy AI decisions, spawn positions, particle burst directions, Hulk random walk, human wander timers, Enforcer/Tank fire intervals. The `rand` crate is lightweight and idiomatic Rust.

Add to `Cargo.toml`: `rand = "0.9"`

---

## 15. Constants Catalog

All tunable values live in `constants.rs`. Initial values:

```rust
// Window
pub const WINDOW_WIDTH: u32 = 960;
pub const WINDOW_HEIGHT: u32 = 720;
pub const WINDOW_TITLE: &str = "Robotron 2084";

// Arena (logical units, centered at origin)
pub const ARENA_HALF_WIDTH: f32 = 440.0;
pub const ARENA_HALF_HEIGHT: f32 = 320.0;
pub const ARENA_BORDER_THICKNESS: f32 = 3.0;

// Player
pub const PLAYER_SPEED: f32 = 300.0;
pub const PLAYER_RADIUS: f32 = 8.0;
pub const PLAYER_FIRE_COOLDOWN: f32 = 0.08;
pub const PLAYER_INVINCIBILITY_DURATION: f32 = 2.0;
pub const PLAYER_BLINK_INTERVAL: f32 = 0.1;
pub const STARTING_LIVES: u32 = 3;
pub const EXTRA_LIFE_EVERY: u32 = 25_000;

// Bullets
pub const BULLET_SPEED: f32 = 800.0;
pub const BULLET_RADIUS: f32 = 3.0;
pub const MAX_PLAYER_BULLETS: u32 = 15; // max simultaneous on screen

// Enemies
pub const GRUNT_BASE_SPEED: f32 = 120.0;
pub const GRUNT_RADIUS: f32 = 10.0;

pub const HULK_SPEED: f32 = 60.0;
pub const HULK_RADIUS: f32 = 16.0;
pub const HULK_KNOCKBACK_FORCE: f32 = 400.0;
pub const HULK_KNOCKBACK_DECAY: f32 = 8.0; // units/sec decay rate (linear)

pub const BRAIN_SPEED: f32 = 100.0;
pub const BRAIN_RADIUS: f32 = 12.0;
pub const BRAIN_FIRE_INTERVAL: f32 = 3.0;
pub const CRUISE_MISSILE_SPEED: f32 = 250.0;
pub const CRUISE_MISSILE_TURN_RATE: f32 = 2.0;  // rad/s
pub const CRUISE_MISSILE_LIFETIME: f32 = 4.0;

pub const PROG_SPEED: f32 = 160.0;
pub const PROG_RADIUS: f32 = 8.0;

pub const SPHEROID_SPEED: f32 = 80.0;
pub const SPHEROID_RADIUS: f32 = 14.0;
pub const SPHEROID_SPAWN_INTERVAL: f32 = 4.0;
pub const SPHEROID_MAX_CHILDREN: u32 = 5;

pub const ENFORCER_SPEED: f32 = 90.0;
pub const ENFORCER_RADIUS: f32 = 10.0;
pub const ENFORCER_FIRE_INTERVAL_MIN: f32 = 1.5;
pub const ENFORCER_FIRE_INTERVAL_MAX: f32 = 3.0;
pub const ENFORCER_SPARK_SPEED: f32 = 300.0;
pub const ENFORCER_SPARK_LIFETIME: f32 = 3.0;

pub const QUARK_SPEED: f32 = 70.0;
pub const QUARK_RADIUS: f32 = 14.0;
pub const QUARK_SPAWN_INTERVAL: f32 = 5.0;
pub const QUARK_MAX_CHILDREN: u32 = 4;

pub const TANK_SPEED: f32 = 50.0;
pub const TANK_RADIUS: f32 = 12.0;
pub const TANK_FIRE_INTERVAL_MIN: f32 = 2.0;
pub const TANK_FIRE_INTERVAL_MAX: f32 = 4.0;
pub const TANK_SHELL_SPEED: f32 = 250.0;
pub const TANK_SHELL_MAX_BOUNCES: u32 = 3;
pub const TANK_SHELL_LIFETIME: f32 = 5.0;

pub const ELECTRODE_RADIUS: f32 = 10.0;

pub const HUMAN_SPEED: f32 = 40.0;
pub const HUMAN_DIRECTION_CHANGE_MIN: f32 = 1.0;
pub const HUMAN_DIRECTION_CHANGE_MAX: f32 = 3.0;

// Scoring
pub const RESCUE_BONUSES: [u32; 5] = [1000, 2000, 3000, 4000, 5000];

// Effects
pub const MAX_PARTICLES: u32 = 200;
pub const EXPLOSION_PARTICLE_COUNT: u32 = 16;
pub const RESCUE_PARTICLE_COUNT: u32 = 10;
pub const DEATH_PARTICLE_COUNT: u32 = 30;
pub const PARTICLE_DRAG: f32 = 0.95;
pub const PARTICLE_BASE_SIZE: f32 = 3.0;
pub const SCREEN_SHAKE_DECAY: f32 = 3.0;  // trauma units/sec (linear)
pub const SCREEN_SHAKE_MAX_OFFSET: f32 = 8.0;
pub const SCORE_POPUP_RISE_SPEED: f32 = 60.0;
pub const SCORE_POPUP_DURATION: f32 = 0.8;

// Timing
pub const WAVE_INTRO_DURATION: f32 = 1.5;
pub const WAVE_CLEAR_DURATION: f32 = 1.5;
pub const DEATH_PAUSE_DURATION: f32 = 1.5;
pub const GAME_OVER_INPUT_DELAY: f32 = 2.0;

// Global limits
pub const MAX_TOTAL_ENEMIES: u32 = 150;
pub const SPAWN_EXCLUSION_RADIUS: f32 = 100.0;
pub const SPAWN_MIN_SEPARATION: f32 = 20.0;
pub const ENEMY_SEPARATION_RADIUS: f32 = 30.0;
pub const ENEMY_SEPARATION_FORCE: f32 = 50.0;
pub const GAMEPAD_DEADZONE: f32 = 0.2;
```

---

## 16. Dependencies

`Cargo.toml` additions beyond the template:

```toml
[dependencies]
bevy = { version = "0.18.1", features = ["dynamic_linking"] }
rand = "0.9"
```

No other external crates needed. Collision, particles, and all game logic are implemented manually.

---

## 17. Implementation Order

Recommended build sequence, each step producing a testable increment:

1. **Scaffold** — Module files, state machine (`AppState` + `PlayState`), empty plugins, constants, window config
2. **Arena + Camera** — Boundary capsule rendering with HDR bloom, `OrthographicProjection` with `ScalingMode::FixedVertical`, `GameAssets` resource with shared meshes/materials
3. **Player** — Diamond mesh, movement (WASD), arena confinement, `Velocity` component
4. **Shooting** — Bullet spawning on arrow keys, `FireCooldown` timer, bullet movement, bullet despawn at boundary, `MAX_PLAYER_BULLETS` cap
5. **Minimal wave system** — Single-wave `WaveDefinition`, spawn Grunts at arena edges, `WaveIntro`/`WaveActive`/`WaveClear` transitions (needed before enemy work)
6. **Grunts** — Chase AI, `bullet_vs_killable` collision, enemy death, scoring, `PointValue`
7. **Effects** — Particle system (spawn, drag, scale-fade, despawn), explosions on enemy kill, screen shake, score popups
8. **HUD** — Node-based UI: score, lives, wave number, high score
9. **Humans** — Wandering AI, `player_vs_human` rescue collision, escalating bonus, rescue sparkle particles
10. **Hulks** — Indestructible, `bullet_vs_hulk` knockback, `hulk_vs_human` kill, Hulk rendering (nested squares), `Without<Killable>` query pattern
11. **Full wave system** — All wave definitions, progression table, `speed_mult`, wave counter, infinite scaling, `WaveEntity` cleanup
12. **Brains & Progs** — Brain seek-human AI, `brain_vs_human` conversion (spawn Prog, despawn Human), Cruise Missile spawning + homing, `HomingMissile` component
13. **Spheroids & Enforcers** — `SpawnerState`, child spawn logic, Enforcer drift + aimed `Spark` shots, `EnemyProjectile` marker, `player_vs_enemy_projectile` collision
14. **Quarks & Tanks** — Same spawner pattern, Tank drift + Bounce Shell, `BouncesRemaining` wall reflection, `enemy_vs_electrode` collision
15. **Electrodes** — Static placement, plus-shape rendering, `bullet_vs_electrode`, `enemy_vs_electrode` (with `Without<Hulk>` filter)
16. **Player death & respawn** — `PlayerDeath` state, death explosion, life decrement, invincibility on respawn, `wave_cleared` flag handling
17. **Start screen & Game over** — Menu states, title rendering, animated shape showcase, input delay, high score display
18. **Gamepad support** — Analog stick input, deadzone, device detection, hot-plug pause
19. **Polish** — Entity animations (pulse, spin, flicker, bob), arena border pulse, hit flash via material swap, difficulty tuning
20. **Audio** — Sound effects (future pass)
