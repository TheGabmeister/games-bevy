# Super Mario Bros - Bevy Clone Spec

A 2D recreation of classic Super Mario Bros (focused on World 1-1) using **Bevy 0.18.1** with primitive-rendered visuals only: no sprite sheets, textures, or audio assets.

The goal is to capture the feel and full gameplay loop of a small, polished Mario-style platformer while keeping the implementation grounded in what this repository actually contains today.

---

## Current Repository Baseline

As of now, the project is still a minimal Bevy starter:

- `src/main.rs` contains the full app
- `DefaultPlugins` are registered with no custom plugins yet
- A `Camera2d` is spawned on startup
- A centered `Hello, World!` UI text node is spawned on startup
- No state machine, world generation, player controller, collisions, or HUD exist yet
- No `assets/` content is currently checked into the repository

This spec describes the intended target, not the current implementation.

---

## Project Goals

- Build a playable single-level Mario-style platformer for World 1-1
- Use primitive geometry and Bevy materials instead of imported art
- Favor clean gameplay structure over exact NES emulation
- Keep the code modular enough to grow beyond a single-file prototype
- Ship a complete gameplay loop: start screen, play, death/lives, level clear, game over

---

## Non-Goals

- Pixel-perfect NES emulation
- Multiple worlds or a save/load system
- Multiplayer, map editor, or online features
- Sprite, texture, music, or sound-effect pipelines
- Faithful reproduction of every original enemy, secret, or scoring edge case

Where the original game and implementation practicality differ, prefer readable code and solid game feel.

---

## Core Constraints

- **2D only** using `Camera2d`
- **Primitive visuals only** using Bevy meshes, text, transforms, and materials
- **No audio**; all feedback is visual
- **No external art dependency** for core gameplay
- **Custom animation** through transforms, color changes, timers, and child entities
- **Stylized polish is allowed**: bloom, particles, screen shake, trails, glow, and camera effects

---

## Technical Decisions

### Engine and update model

- Rust edition `2024`
- Bevy `0.18.1`
- Use `DefaultPlugins`
- Use a fixed-timestep gameplay loop for movement, gravity, collisions, enemy logic, and timer-critical gameplay
- Use frame-based `Update` systems for visuals, UI animation, menu input, and camera smoothing where appropriate

### Rendering approach

- Use `Mesh2d` plus `ColorMaterial` for most world geometry
- Use Bevy UI `Text` for screen-space HUD and menus
- Use `Text2d` only when world-space labels are needed, such as floating score popups or block markers
- Use bloom/HDR only after the base scene remains readable without post-processing

### Coordinate and layout conventions

- Window target: `800x600`
- Tile size: `32x32`
- Positive X moves right; positive Y moves up
- Level data should be authored in tile coordinates, then converted into world positions
- Collision logic should use axis-aligned bounding boxes
- Resolve movement in two passes when needed: horizontal, then vertical

### State model

Recommended app-level flow:

- `StartScreen`
- `Playing`
- `LevelClear`
- `GameOver`

Recommended play sub-state:

- `Running`
- `Paused`
- `Dying`
- `Respawning`
- `Cutscene`

This is more complete than the original draft and better matches death, flagpole, and pause requirements.

### Data-driven gameplay

- Level layout should come from explicit data, not hand-placed one-off entities in setup code
- Question block contents should be data-driven per block
- Enemy spawn positions should be data-driven
- Resettable level state should be reconstructible from source data on respawn

---

## Gameplay Scope

- One playable level: a World 1-1-inspired course
- One player character: Mario
- Core interactables: ground, bricks, question blocks, hard blocks, pipes, gaps, flagpole, castle
- Core enemies: Goomba and Koopa Troopa
- Core items: coin and mushroom
- Optional stretch: star power

---

## Systems and Responsibilities

The implementation should eventually separate into these responsibilities:

- App bootstrap and plugin registration
- States and transitions
- Level data and tile spawning
- Camera follow and bounds
- Player input, physics, collision, and power states
- Block and item interactions
- Enemy AI and collisions
- HUD, menus, and score/timer presentation
- Visual effects and transient entities

Cross-domain communication should use events where practical.

Suggested events:

- `AddScore`
- `SpawnParticles`
- `PlayerDamaged`
- `PlayerDied`
- `BlockHit`
- `EnemyStomped`
- `LevelCompleted`
- `CameraShakeRequested`

---

## Implementation Phases

### Phase 1 - Core Scaffolding

1. **Project structure and states**
   - Create module files: `constants.rs`, `components.rs`, `resources.rs`, `states.rs`, and `events.rs`
   - Define app and play states
   - Replace the starter `Hello, World!` setup with proper bootstrapping
   - Set window title to `"Super Mario Bros"`
   - Set a dark sky-blue clear color
   - Add camera configuration and optional HDR/bloom setup

2. **Constants and shared resources**
   - Window dimensions, tile size, gravity, speed values
   - Player dimensions, jump force, acceleration, max fall speed
   - Color palette values
   - Shared resources for score, coins, lives, timer, and current level status

### Phase 2 - World and Level

3. **Tile system and level data**
   - Define a tile-map data structure
   - Tile types: `Empty`, `Ground`, `Brick`, `QuestionBlock`, `Pipe`, `HardBlock`, `Flagpole`, `Castle`
   - Encode a World 1-1-inspired layout including gaps, platforms, pipes, staircase, and finish area

4. **Tile rendering**
   - Render tiles as primitive geometry with clear silhouettes
   - Ground: solid brown rectangles
   - Brick: orange-brown rectangles with simple line detail
   - Question block: yellow rectangle with `?` overlay and glow-friendly color
   - Pipe: green body with a wider cap
   - Hard block: dark gray solid block
   - Flagpole and castle: simple readable geometric compositions

5. **Camera scrolling**
   - Camera follows Mario horizontally
   - Clamp camera to level bounds
   - Do not allow leftward backtracking
   - Keep vertical framing stable unless a deliberate camera effect is added

### Phase 3 - Player

6. **Mario entity and rendering**
   - Build Mario from primitive child shapes
   - Small Mario should read clearly at gameplay scale
   - Flip facing direction by changing transform scale or child layout

7. **Horizontal movement**
   - Left/Right arrow keys or `A/D`
   - Use acceleration and deceleration, not instant velocity
   - Define a max run speed

8. **Jumping and gravity**
   - `Space`, `Up`, or `W` to jump
   - Variable-height jump
   - Gravity every gameplay tick
   - Max fall speed cap
   - Grounded detection from collision results

9. **Collision with tiles**
   - AABB collision against solid tiles
   - Block wall penetration horizontally
   - Land cleanly on floors
   - Detect head bumps on blocks from below
   - Keep collision handling deterministic and easy to debug

### Phase 4 - Block Interactions

10. **Question blocks**
    - On hit from below, convert to spent state
    - Spawn either a coin or a mushroom depending on block contents
    - Animate block bump
    - Award score

11. **Brick blocks**
    - Small Mario bumps but does not break bricks
    - Big Mario can break bricks
    - Breaking spawns debris fragments and particles

12. **Coin pop animation**
    - Coin rises, arcs, and fades
    - Floating score text rises and despawns
    - Glow should remain readable even if bloom is disabled

### Phase 5 - Enemies

13. **Goomba**
    - Simple primitive shape body
    - Patrol behavior
    - Reverse on wall collision
    - Fall when unsupported

14. **Koopa Troopa**
    - Primitive shape body and shell silhouette
    - Patrol behavior
    - Stomp transitions into shell state
    - Kicked shell moves quickly and damages enemies

15. **Enemy-player collision**
    - Falling onto enemy defeats or alters enemy state
    - Side/bottom contact damages Mario unless invincible
    - Stomp detection depends on downward motion and contact position

16. **Enemy death effects**
    - Goomba squish
    - Koopa shell transition or defeat handling
    - Particles on defeat

### Phase 6 - Power-Ups

17. **Mushroom**
    - Emerges from question block
    - Moves horizontally and obeys simple collision/gravity rules
    - Collecting it upgrades Mario to Big

18. **Mario power states**
    - `Small`
    - `Big`
    - Big Mario can break bricks
    - Taking damage as Big shrinks Mario back to Small
    - Add brief post-hit invulnerability with visible flashing

19. **Star power (optional stretch)**
    - Temporary invincibility
    - Rainbow or palette-cycling visual
    - Contact defeats enemies

### Phase 7 - Scoring and HUD

20. **Score resource and display**
    - Track score, coin count, lives, world label, and time remaining
    - HUD fixed to the screen, not world space
    - Use text formatting close to classic Mario readability

21. **Score events and feedback**
    - Coins and enemy defeats award points
    - Consecutive stomps can escalate score rewards
    - `100` coins should award `1UP`
    - Floating score text appears where actions occur

22. **Timer**
    - Start from `400`
    - Count down during active gameplay only
    - Reaching `0` causes player death
    - Low time may trigger stronger visual warning

### Phase 8 - Player Death and Lives

23. **Mario death**
    - Triggered by lethal damage, timer expiration, or falling into a pit
    - Use a classic death-jump style animation
    - Remove player control during the sequence

24. **Pit and void detection**
    - Falling below a world threshold causes death
    - Gaps are authored in level data

25. **Respawn**
    - Reset Mario to the level start point
    - Rebuild resettable level entities from source data
    - Reset enemies and transient items
    - Keep score, coin count, and remaining lives
    - If lives reach zero, transition to `GameOver`

### Phase 9 - Level Completion

26. **Flagpole**
    - End-of-level collision triggers a completion sequence
    - Score depends on grab height

27. **Flagpole sequence**
    - Mario slides down the pole
    - Flag lowers
    - Player control is suspended during the sequence
    - Mario walks toward the castle after landing

28. **Castle and level end**
    - Mario enters the castle
    - Remaining time converts into score
    - Transition to `LevelClear`
    - From `LevelClear`, `Enter` returns to `StartScreen`

### Phase 10 - Menus and Screens

29. **Start screen**
    - Title text
    - Blinking start prompt
    - Minimal animated character or background motion

30. **Game over screen**
    - Centered final score
    - Restart prompt
    - Return path to a fresh run

31. **Pause**
    - `Escape` toggles pause
    - Pause overlay text
    - Gameplay systems stop while pause-safe UI systems continue

### Phase 11 - Visual Polish and Effects

32. **Particle system**
    - Reusable particle spawner
    - Supports color, count, velocity spread, and lifetime
    - Used for bricks, stomps, coins, dust, and celebratory effects

33. **Screen shake**
    - Small, decaying shake on impactful events
    - Should not interfere with gameplay readability

34. **Background parallax**
    - Hills, clouds, and bushes built from primitive forms
    - Multiple parallax layers for depth

35. **Glow and color polish**
    - Question blocks pulse subtly
    - Coins shimmer
    - Star power cycles color
    - Post-processing stays optional and additive

36. **Animation polish**
    - Mario squash and stretch
    - Enemy walk cycles
    - Flag motion

---

## Missing-From-Draft Details Now Explicitly Defined

These items were implied before, but should be treated as part of the spec:

- The repo starts from a minimal Bevy bootstrap, not an existing platformer framework
- Level state must be rebuildable for respawn, not just enemies alone
- Damage requires temporary invulnerability feedback
- `100` coins award an extra life
- `GameOver` and `LevelClear` need distinct flows from ordinary `Playing`
- Player control must be suspended during death, respawn, and flagpole cutscenes
- The level should be data-driven from the start to avoid hardcoded setup sprawl
- Glow effects should degrade gracefully when bloom is disabled
- World-space score popups and HUD text have different rendering responsibilities

---

## Rendering and Layering

To avoid visual conflicts, use a simple Z-order policy:

- Background decoration
- World tiles
- Items and enemies
- Player
- Particles and floating score text
- HUD and menu UI

Keep this consistent across modules.

---

## Validation Expectations

- Run `cargo check` after implementation milestones
- Run `cargo clippy` when APIs or architectural patterns change substantially
- Keep changes incremental and playable whenever possible

Since this file is a design spec, document-only updates do not require a Rust build by themselves.

---

## File Structure (Target)

```text
src/
|-- main.rs            # App setup, plugin registration
|-- constants.rs       # Tunable values and color palette
|-- components.rs      # ECS components and markers
|-- resources.rs       # Score, lives, timer, level state
|-- states.rs          # AppState, PlayState
|-- events.rs          # Cross-system gameplay events
|-- player/
|   |-- mod.rs         # PlayerPlugin
|   |-- movement.rs    # Input, gravity, collision
|   `-- animation.rs   # Squash/stretch, growth, damage flash
|-- level/
|   |-- mod.rs         # LevelPlugin
|   |-- tiles.rs       # Tile rendering and solid data
|   |-- camera.rs      # Camera follow and bounds
|   `-- data.rs        # Level layout encoding
|-- enemies/
|   |-- mod.rs         # EnemyPlugin
|   |-- goomba.rs      # Goomba behavior
|   `-- koopa.rs       # Koopa and shell behavior
|-- items/
|   |-- mod.rs         # ItemPlugin
|   |-- coin.rs        # Coin pop animation
|   |-- mushroom.rs    # Mushroom behavior
|   `-- blocks.rs      # Question and brick block interactions
|-- ui/
|   |-- mod.rs         # UiPlugin
|   |-- hud.rs         # Score, lives, timer, world label
|   `-- screens.rs     # Start, game over, pause, level-clear screens
`-- effects/
    |-- mod.rs         # EffectsPlugin
    |-- particles.rs   # Particle spawning
    |-- camera_fx.rs   # Screen shake
    `-- background.rs  # Parallax decoration
```

This structure is a target architecture, not a requirement for the very first implementation step.

---

## Controls

| Input | Action |
|---|---|
| Left / A | Move left |
| Right / D | Move right |
| Space / Up / W | Jump |
| Escape | Pause / Unpause |
| Enter | Confirm on menus |

---

## Definition of Done

The project should count as a successful first milestone when:

- The game boots into a start screen
- A full World 1-1-style run is playable from start to finish
- Mario can move, jump, collide, take damage, grow, die, and respawn
- Blocks, coins, enemies, score, timer, and lives all work
- The run can end in either game over or level clear
- The presentation uses only primitive-rendered visuals and text
