# Implementation Tasks

Ordered for incremental learning. Each task produces a runnable game with visible progress. Check off tasks as you complete them.

---

## Phase 1: Foundation — Window, Camera, and a Box That Moves

**You'll learn**: Bevy app setup, ECS basics (entities, components, systems), keyboard input, basic rendering with primitives.

- [x] **1.1** Set up window (800×600, title "Super Mario Bros"), spawn a 2D camera, set `ClearColor` to light blue sky
- [x] **1.2** Create `constants.rs` — window size, tile size (16×16), camera scale, ascending gravity (600), descending gravity (980), terminal velocity (500), player speeds (walk 130, run 200), jump impulse, jump cut multiplier (0.4)
- [x] **1.3** Create `components.rs` — `Player` marker, `Velocity` component, `FacingDirection` enum (Left/Right)
- [x] **1.4** Spawn Mario as a red rectangle (14×16) near the bottom of the screen
- [x] **1.5** Add horizontal movement (left/right input → velocity → position update). Track `FacingDirection`.
- [x] **1.6** Add gravity — Mario falls, add a ground line (single rectangle across the bottom)
- [x] **1.7** Add grounded detection and jumping. Variable-height: apply impulse on press, multiply velocity by ~0.4 on release. Use dual gravity: lower while ascending (600), higher while descending (980).
- [x] **1.8** Add acceleration/deceleration to horizontal movement (not instant start/stop)
- [x] **1.9** Add run speed — hold Shift for higher max speed (~200 vs ~130 walk)
- [x] **1.10** Set up camera projection scale so visible area is ~267×200 world units (tiles render large enough)
- [x] **1.11** Enforce system ordering with `.after()` chains: Input → Gravity → Movement → Collision → Grounded check

**Milestone**: A red rectangle that runs and jumps on a flat brown floor with momentum, against a blue sky.

---

## Phase 2: Tile Map — Building a Level

**You'll learn**: Data-driven level design, tile grids, spawning entities from data, component queries.

- [x] **2.1** Create `level.rs` — define Level 1-1 as a 2D array of tile characters (~210 wide × 15 tall). Include a Mario spawn point (`S`).
- [x] **2.2** Create `Tile` marker component and `TileType` enum (Ground, Brick, QuestionBlock, Empty, PipeTopLeft, PipeTopRight, PipeBodyLeft, PipeBodyRight)
- [x] **2.3** Write a system that reads the level data and spawns colored rectangles for each tile. Assign z-layers (see SPEC render ordering).
- [x] **2.4** Spawn Mario at the `S` tile position instead of a hardcoded location
- [x] **2.5** Implement tile collision — AABB overlap detection, push-out by smallest penetration axis (vertical first, then horizontal). Only check ~12 tiles in the entity's neighborhood (convert position to grid coords), not all 3,000+ tiles. Mario stands on tiles, bumps into walls, hits ceilings.
- [x] **2.6** Remove the old single ground rectangle; level is now fully tile-based
- [x] **2.7** Add pipes — parse `[]{}` tile chars, render as green rectangles (lip slightly wider than body), solid and collidable
- [x] **2.8** Add pits/gaps in the ground — verify Mario falls through

**Milestone**: Mario runs and jumps through a real level layout with platforms, pipes, and gaps.

---

## Phase 3: Camera Scrolling

**You'll learn**: Camera systems, coordinate transforms, dead zones, one-way constraints.

- [x] **3.1** Make camera follow Mario horizontally (smooth lerp)
- [x] **3.2** Add dead zone — camera only scrolls when Mario reaches the right third
- [x] **3.3** Prevent camera from scrolling left (one-way scrolling)
- [x] **3.4** Clamp camera to level bounds (don't show empty space past level edges)

**Milestone**: Smooth side-scrolling camera that feels like the original game.

---

## Phase 4: Game States & HUD

**You'll learn**: Bevy states, `OnEnter`/`OnExit`, UI nodes, text rendering, `DespawnOnExit`.

- [x] **4.1** Create `states.rs` — `AppState` enum (StartScreen, Playing, GameOver)
- [x] **4.2** Create `resources.rs` — `GameData` resource (score, coins, lives, world name, timer)
- [x] **4.3** Implement StartScreen state — title text, "Press Enter" prompt
- [x] **4.4** Implement Playing state — spawn level and Mario on enter, cleanup on exit
- [x] **4.5** Implement GameOver state — "Game Over" text, restart on Enter
- [x] **4.6** Add HUD overlay during Playing — score, coins, world, countdown timer
- [x] **4.7** Implement countdown timer (400 → 0, death on timeout)
- [x] **4.8** Gate all gameplay systems with `.run_if(in_state(AppState::Playing))`

**Milestone**: Full game loop — start screen → play → die → game over → restart.

---

## Phase 5: Player Death & Lives

**You'll learn**: Sub-states, animation with timers, entity lifecycle, respawning.

- [x] **5.1** Add `PlayState` sub-state (Running, Dying, Paused, LevelComplete)
- [x] **5.2** Implement pit death — falling below screen triggers Dying state
- [x] **5.3** Add death animation — Mario bounces up, then falls off screen (using a timer)
- [x] **5.4** After death animation: decrement lives, respawn at level start (or GameOver if lives = 0)
- [x] **5.5** Implement pause — Escape toggles Paused sub-state, freeze all systems, show overlay

**Milestone**: Mario dies from pits, has lives, can pause, and transitions cleanly between states.

---

## Phase 6: Enemies — Goomba

**You'll learn**: Enemy AI (simple patrol), collision responses, stomp detection, despawning.

- [x] **6.1** Create `enemy.rs` module — `Goomba` marker, `EnemyWalker` component (speed, direction)
- [x] **6.2** Spawn Goombas from level data at `G` tiles — brown ellipse + rectangle feet
- [x] **6.3** Goomba patrol system — walk in one direction, reverse on wall collision (requires enemy-tile collision)
- [x] **6.4** Goomba gravity + falls off ledges (enemies need the same gravity/ground collision as Mario)
- [x] **6.5** Mario-Goomba collision: stomp detection (Mario above + falling = kill Goomba)
- [x] **6.6** Goomba stomp: squish animation (flatten, brief delay, then despawn), Mario bounces up
- [x] **6.7** Mario-Goomba side/bottom contact: Mario dies (all contact = death until power-ups in Phase 8)
- [x] **6.8** Add score popup on stomp (+100 as floating `Text2d` that rises and fades)
- [x] **6.9** Only activate enemies when they're near the camera (don't simulate off-screen enemies)

**Milestone**: Goombas patrol the level, can be stomped for points, and kill Mario on contact.

---

## Phase 7: Block Interactions

**You'll learn**: Entity-to-entity interaction, message passing, spawn-on-event patterns, component swapping.

- [x] **7.1** Detect Mario hitting a block from below (head collision while jumping). When head overlaps multiple blocks, activate the one whose center is closest to Mario's center. Only one block per jump.
- [x] **7.2** `?` Block hit: block bounces up briefly (visual feedback), then turns into Empty block
- [x] **7.3** `?` Block coin release: spawn a coin that arcs up and disappears, +1 coin counter, +200 score
- [x] **7.4** Brick block + Small Mario: bump animation (block shakes), enemies on top get killed
- [x] **7.5** Brick block + Big Mario: block breaks — spawn 4 small rectangle particles that scatter and fall
- [x] **7.6** Track coin count in HUD — 100 coins = +1 life, reset coin counter
- [x] **7.7** Add floating coins in the level (from `C` tiles) — collected on contact

**Milestone**: Blocks react to being hit — coins pop out, bricks break, `?` blocks empty.

---

## Phase 8: Power-ups — Mushroom & Growth

**You'll learn**: Item physics, state transitions on the player, hitbox changes, invincibility frames.

- [x] **8.1** `?` Blocks marked `M` in level data release a Mushroom instead of a coin
- [x] **8.2** Mushroom emerges from block (rises up), then slides along ground, bounces off walls, falls off edges (mushroom needs tile collision + gravity, same as enemies)
- [x] **8.3** Mario + Mushroom contact: Small → Big (double height to 14×32)
- [x] **8.4** Growth transition: freeze all gameplay for ~1s, flash between small/big sizes 3-4 times. Anchor growth from feet (bottom of sprite stays fixed) to avoid clipping into ceiling tiles.
- [x] **8.5** Big Mario: update collision box, camera offset
- [x] **8.6** Big Mario takes damage → shrink to Small (with ~2s invincibility, flashing sprite)
- [x] **8.7** During invincibility: Mario flashes (toggle visibility each frame), enemies pass through
- [x] **8.8** Retrofit Phase 6/9 enemy contact: Big/Fire Mario takes damage instead of dying
- [x] **8.9** Add ducking — Down key while Big Mario: shrink hitbox to half height, prevent horizontal movement

**Milestone**: Mario collects mushrooms, grows big, shrinks on damage with invincibility frames, and can duck.

---

## Phase 9: Enemies — Koopa Troopa & Shell Mechanics

**You'll learn**: Complex enemy states, entity reuse (Koopa → Shell), chain reactions.

- [x] **9.1** Create Koopa entity — green rectangle body + circle head, patrols like Goomba
- [x] **9.2** Stomp Koopa: transforms into stationary shell (static green rectangle). Shell has 3 states: Walking → Stationary → Moving (and back to Stationary via stomp).
- [x] **9.3** Kick shell: Mario walks into stationary shell → shell launches in Mario's facing direction
- [x] **9.4** Moving shell kills enemies it contacts (Goombas, other Koopas)
- [x] **9.5** Moving shell kills Mario on side contact (stomping a moving shell stops it → returns to Stationary)
- [x] **9.6** Shell bounces off walls
- [x] **9.7** Chain kill scoring — shell kills give 200, 400, 800... per consecutive enemy

**Milestone**: Koopas patrol, become kickable shells, and shells interact with everything.

---

## Phase 10: Fire Flower & Fireballs

**You'll learn**: Projectile systems, max-entity constraints, bouncing physics, conditional power-ups.

- [x] **10.1** When Big/Fire Mario hits a `M` question block: release Fire Flower instead of Mushroom
- [x] **10.2** Fire Flower entity — circle on rectangle stem, orange/red, stationary (doesn't slide)
- [x] **10.3** Mario + Fire Flower: become Fire Mario (color change — white/red palette)
- [x] **10.4** Fire button shoots fireball (small orange circle) in facing direction
- [x] **10.5** Fireball physics: travels horizontally, bounces on ground (one bounce arc), then despawns
- [x] **10.6** Max 2 fireballs on screen at once
- [x] **10.7** Fireball kills enemies on contact (Goomba dies, Koopa goes to shell)
- [x] **10.8** Fire Mario takes damage → Small Mario (skips Big)

**Milestone**: Fire Mario shoots bouncing fireballs that kill enemies.

---

## Phase 11: Level Completion

**You'll learn**: Scripted sequences, disabling player input, score tallying, level transitions.

- [x] **11.1** Add flagpole at the end of the level (tall thin rectangle + triangle flag)
- [x] **11.2** Add castle after flagpole (rectangles + triangle roof)
- [x] **11.3** Mario touches flagpole → enter LevelComplete sub-state, disable all player input. Mario is now driven by scripted movement (direct transform control, not the input→velocity pipeline).
- [x] **11.4** Flagpole sequence: snap Mario to pole x-position, slide down at fixed speed, then walk right to castle entrance at fixed speed
- [x] **11.5** Score based on flagpole contact height (higher = more points, top = 5000)
- [x] **11.6** Time bonus: remaining time × 50, visibly count down on HUD
- [x] **11.7** After tally: transition to next level (or loop to Level 1-1)

**Milestone**: Complete end-of-level sequence with scoring and level transitions.

---

## Phase 12: Decorations & Polish

**You'll learn**: Non-interactive entities, parallax hints, visual polish, background layers.

- [x] **12.1** Add clouds — groups of overlapping white circles, positioned above play area
- [x] **12.2** Add bushes — overlapping green circles at ground level
- [x] **12.3** Add hills — large dark green ellipses behind the ground
- [x] **12.4** Add visual variety to Mario (tan face rectangle on the red body)
- [x] **12.5** Skid visual — Mario flashes or changes color when reversing direction at speed
- [x] **12.6** Polish death animation timing and feel
- [x] **12.7** Add particle burst when breaking bricks (if not done in Phase 7)

**Milestone**: The level looks alive with decorations and polished animations.

---

## Phase 13: Second Level & Progression (Stretch)

- [x] **13.1** Extract level data into RON files — move the char grid out of `level.rs` into `assets/levels/1-1.ron`, load via `AssetServer`
- [x] **13.2** Create Level 1-2 layout as a RON file (underground theme — dark background, different colors)
- [x] **13.3** Level transition system — load different level data based on world progression
- [x] **13.4** Increase difficulty — more enemies, trickier platforming
- [x] **13.5** Add level name display at start ("WORLD 1-2")

---

## Phase 14: Advanced Features (Stretch)

- [x] **14.1** Starman power-up — bouncing star polygon, invincibility + flashing + kill on contact
- [ ] **14.2** Moving platforms — horizontally or vertically moving rectangles Mario can ride
- [ ] **14.3** Warp pipes — press Down on certain pipes to teleport
- [ ] **14.4** Screen shake on block break / big stomp
- [x] **14.5** 1-Up mushroom (green) — grants extra life
- [ ] **14.6** Combo stomp scoring (consecutive stomps without landing)

---

## Learning Concepts by Phase

| Phase | Bevy/Rust Concepts |
|---|---|
| 1 | App, plugins, components, systems, queries, input, transforms, camera projection |
| 2 | Data-driven design, enums, spawning from data, AABB collision, collision resolution |
| 3 | Camera manipulation, lerp, coordinate systems |
| 4 | States, `OnEnter`/`OnExit`, UI nodes, resources, timers |
| 5 | Sub-states, animation timers, entity lifecycle |
| 6 | AI patterns, collision response, despawning, messages |
| 7 | Entity interaction, component swapping, particles |
| 8 | Player state machine, hitbox changes, invincibility |
| 9 | Complex entity states, chain reactions, scoring |
| 10 | Projectiles, entity limits, conditional spawning |
| 11 | Scripted sequences, input locking, score tally |
| 12 | Visual polish, non-gameplay entities, juice |
| 13–14 | Content scaling, advanced mechanics |
