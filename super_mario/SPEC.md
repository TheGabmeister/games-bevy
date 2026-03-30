# Super Mario Bros — Primitive Shapes Edition

A 2D side-scrolling platformer inspired by Super Mario Bros, built with Bevy 0.18.1 using only primitive shapes (rectangles, circles, polygons). No sprites, textures, or audio.

---

## Visual Style

Everything is drawn with colored primitive shapes:

| Entity | Shape | Color |
|---|---|---|
| Mario (small) | Rectangle 14×16 | Red body, tan face square |
| Mario (big) | Rectangle 14×32 | Red body, tan face square |
| Mario (fire) | Rectangle 14×32 | White body, red overalls, tan face |
| Goomba | Ellipse + rectangle feet | Brown |
| Koopa | Rectangle + circle head | Green |
| Ground tile | Rectangle | Brown (`#8B4513`) |
| Brick block | Rectangle with grid lines | Dark brown |
| `?` block | Rectangle + `?` text | Yellow/gold |
| Pipe | Two rectangles (lip + body) | Green |
| Coin | Small circle | Gold/yellow |
| Mushroom | Half-circle + rectangle stem | Red cap, tan stem |
| Fire Flower | Circle + rectangle stem | Orange/red petals |
| Starman | Regular polygon (5-point) | Yellow, flashing |
| Fireball | Small circle | Orange |
| Flagpole | Thin rectangle + triangle flag | Gray pole, green flag |
| Castle | Rectangles + triangle roof | Gray |
| Cloud | Overlapping circles | White |
| Bush | Overlapping circles | Green |
| Hill | Large ellipse | Dark green |

---

## Window & Camera

- **Window size**: 800×600
- **Camera**: Follows Mario horizontally with smooth lerp. Camera never scrolls left (one-way scrolling like the original). Camera has a dead zone in the center — only scrolls when Mario approaches the right third.
- **Camera zoom**: The camera uses a projection scale so that the visible area is ~267×200 world units (roughly matching the NES 256×240 resolution). This makes 16×16 tiles appear large enough on screen (~48 rendered pixels per tile).
- **Coordinate system**: World units map to tile units. Origin at bottom-left of the level.

---

## Grid & Level Layout

- **Tile size**: 16×16 world units
- **Level dimensions**: ~210 tiles wide × 15 tiles tall (Level 1-1). Height includes 2 tiles below ground for visual depth.
- Levels are defined as tile grids in code (array of rows, each row is a string of tile characters). Row 0 is the top row; the bottom rows are ground.
- Tile legend for level data:

| Char | Meaning |
|---|---|
| `.` | Empty / air |
| `#` | Ground block |
| `B` | Brick block |
| `?` | Question block (coin) |
| `M` | Question block (mushroom) |
| `[` | Pipe top-left (left half of pipe lip) |
| `]` | Pipe top-right (right half of pipe lip) |
| `{` | Pipe body-left (left half of pipe shaft) |
| `}` | Pipe body-right (right half of pipe shaft) |
| `C` | Coin (floating) |
| `G` | Goomba spawn (on ground) |
| `K` | Koopa spawn (on ground) |
| `S` | Mario spawn point |
| `F` | Flagpole base (pole extends upward 10 tiles) |
| `X` | Castle position |

---

## Core Physics

- **Gravity**: Constant downward acceleration (~980 units/s²)
- **Terminal velocity**: ~500 units/s (capped fall speed)
- **Grounded detection**: Overlap check — a narrow rect just below the entity's feet, tested against solid tiles
- **Collision**: AABB (axis-aligned bounding box) for all entities. All solid tiles block from all four sides (no one-way platforms).
- **Collision resolution**: On overlap, push the entity out by the smallest penetration axis. Horizontal and vertical axes are resolved independently — resolve vertical first (landing/ceiling), then horizontal (walls). This prevents corner-catching.
- **FacingDirection**: All moving entities track which direction they face (Left/Right). Used for fireball direction, shell kick direction, and visual orientation.

---

## Player (Mario)

### States
- **Small Mario**: 14×16, dies on enemy contact
- **Big Mario**: 14×32, shrinks to Small on enemy contact (with brief invincibility)
- **Fire Mario**: Big Mario with fireball ability, shrinks to Small on hit

### Movement
- **Run**: Left/right with acceleration and deceleration (not instant stop)
- **Max walk speed**: ~130 units/s
- **Max run speed**: ~200 units/s (hold Shift/run button)
- **Jump**: Variable-height — holding jump goes higher, releasing cuts it short
- **Jump height**: ~64 units (4 tiles) for full jump
- **Air control**: Reduced but present — can steer mid-air
- **Skid**: When reversing direction while moving, brief skid animation (color flash)
- **No double jump**: Only jump when grounded
- **Ducking**: Down arrow while Big Mario — reduces hitbox height, cannot move while ducking

### Death
- Mario plays a brief "death" animation (bounces up then falls off screen)
- Lose one life
- Respawn at level start (or checkpoint)
- Game Over when lives reach 0
- **Starting lives**: 3

---

## Enemies

### Goomba
- Walks in one direction, reverses on wall collision
- Defeated by: stomp (jump on top) — squishes flat, then disappears
- Kills Mario on side/bottom contact (unless invincible)
- Falls off edges (doesn't turn at ledges)

### Koopa Troopa
- Walks in one direction, reverses on wall collision
- **Stomp**: Retreats into shell (becomes a static green rectangle)
- **Kick shell**: Touch shell while it's stationary — it slides fast in that direction
- **Shell kills**: Moving shell defeats other enemies on contact
- **Shell danger**: Moving shell kills Mario on contact too
- Shell bounces off walls

---

## Blocks & Tiles

### Ground Block
- Solid, indestructible
- Brown rectangle

### Brick Block
- Solid from all sides
- **Small Mario**: Bumps from below (block bounces, enemies on top get killed)
- **Big Mario**: Breaks the block (block disappears with particle effect — small rectangles scatter)

### Question Block (`?`)
- Hit from below to release contents
- After hit, becomes an empty block (gray, darker)
- Contents: Coin (adds to coin counter) or Power-up (mushroom/fire flower depending on Mario state)

### Empty Block
- Solid, no interaction — visual indicator that a `?` block was already used

---

## Items & Power-ups

### Coin
- Floating coins in the level: collected on contact, +1 coin counter
- **100 coins = 1 extra life**
- Block coins: Pop up from `?` block with a small arc animation, then disappear

### Super Mushroom
- Emerges from `?` block, slides along the ground
- Falls off edges, bounces off walls
- On contact with Mario: Small → Big (grow animation)

### Fire Flower
- Emerges from `?` block (only when Mario is already Big)
- On contact: Big → Fire Mario (color change to white/red)
- **Fireball**: Press fire button to shoot. Max 2 on screen. Fireballs travel horizontally, bounce once on ground, then disappear. Kill most enemies on contact.

### Starman (Stretch Goal)
- Bounces around the level
- On contact: Mario becomes invincible for ~10 seconds
- Invincible Mario: Flashing colors, kills enemies on any contact, immune to damage
- Still dies from pits

---

## Level Elements

### Pipes
- 2-tile wide, variable height
- The lip (top) is slightly wider than the body (rendered as a wider rectangle on top of a narrower one)
- Solid — Mario walks over them and collides from all sides
- Some pipes are entry points (down arrow on top to enter) — warp to another location (stretch goal)

### Pits / Gaps
- Gaps in the ground — falling in = instant death regardless of power-up state

### Flagpole (End of Level)
- Tall pole at the end of the level
- Mario touches it → slides down → walks to castle → level complete
- Score bonus based on height of contact (higher = more points)

### Castle
- Small decorative castle shape at the end of the level (after flagpole)
- Mario walks into it to complete the level

---

## Render Ordering (Z-layers)

Entities are drawn in this order (back to front):

| Z | Layer |
|---|---|
| 0 | Background (sky color via `ClearColor`) |
| 1 | Decorations (hills, clouds, bushes) |
| 2 | Pipes (behind entities but in front of decorations) |
| 3 | Tiles (ground, bricks, `?` blocks) |
| 4 | Items (coins, mushrooms, fire flowers) |
| 5 | Enemies |
| 6 | Player (Mario) |
| 7 | Particles (brick debris, coin sparkle) |
| 8 | Score popups (floating +100 text) |

HUD is rendered as UI nodes (separate from the world), always on top.

---

## Enemy Spawn Behavior

- Enemies are placed in level data at their spawn positions
- They are spawned as entities when the level loads, but **inactive** (no AI, no physics) until the camera approaches within ~2 tiles of the screen edge
- Once activated, they stay active even if they leave the screen
- Enemies that fall below the level are despawned

---

## HUD / UI

### In-Game HUD (top of screen)
- **Score**: "MARIO — 000000" (top-left)
- **Coins**: Coin icon (yellow circle) + "×00" (top-center-left)
- **World**: "WORLD 1-1" (top-center)
- **Time**: Countdown timer from 400, ticking down ~1 per 0.4s (top-right)
- Running out of time = death

### Start Screen
- Game title text
- "PRESS ENTER TO START"
- Blinking prompt

### Game Over Screen
- "GAME OVER" centered
- "PRESS ENTER TO RESTART"

### Level Complete Screen
- Brief score tally (time bonus)
- Auto-advance to next level (or loop back to level 1)

---

## Scoring

| Action | Points |
|---|---|
| Stomp Goomba | 100 |
| Stomp Koopa | 100 |
| Kick shell kills enemy | 200 (doubles per enemy in chain) |
| Coin | 200 |
| Mushroom | 1000 |
| Fire Flower | 1000 |
| Starman | 1000 |
| Brick block break | 50 |
| Flagpole (bottom) | 100 |
| Flagpole (top) | 5000 |
| Time bonus | Remaining time × 50 |

**Combo stomp**: Consecutive stomps without landing give increasing points (200, 400, 800, 1000, 2000, 4000, 8000, 1-Up).

---

## Controls

| Action | Key |
|---|---|
| Move left | A / Left Arrow |
| Move right | D / Right Arrow |
| Jump | Space / W / Up Arrow |
| Run | Left Shift |
| Duck | S / Down Arrow |
| Fireball | J / E |
| Pause | Escape |

---

## Game States

```
StartScreen → Playing → LevelComplete → Playing (next level)
                ↓                            ↓
            GameOver  ←──────────────────  GameOver
                ↓
          StartScreen
```

### Sub-states of Playing
- `Running` — normal gameplay
- `Paused` — freeze all systems, show "PAUSED" overlay
- `Dying` — Mario death animation playing, then respawn or game over
- `LevelComplete` — flagpole sequence playing

---

## Levels

### Level 1-1 (minimum viable level)
- Flat ground with a few gaps
- 3-4 Goombas
- 2 Koopas
- Several `?` blocks (coins + 1 mushroom)
- Brick blocks in clusters
- 2 pipes
- Flagpole + castle at the end

### Level 1-2 (stretch goal)
- Underground theme (dark background, different block colors)
- More platforming challenges
- Denser enemy placement

---

## Stretch Goals (not required for MVP)

1. **Warp pipes** — Enter pipes to warp to different sections
2. **Underground/bonus areas** — Sub-areas accessed via pipes
3. **Moving platforms** — Horizontally or vertically moving platforms
4. **Piranha Plant** — Enemy that pops in and out of pipes
5. **Level editor** — Simple level definition format for custom levels
6. **Screen shake** — On block break and enemy stomp
7. **Particle effects** — Coin sparkles, brick debris, death poof
8. **Multiple worlds** — 1-1 through 1-4 with increasing difficulty
9. **Starman power-up** — Full invincibility implementation
10. **1-Up mushroom** — Green mushroom that grants an extra life
