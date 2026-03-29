# Galaga Project Specification

## 1. Document Status

This document defines a practical specification for turning this repository from a small Bevy shooter template into a Galaga-inspired fixed-screen arcade game.

It is intentionally grounded in the current codebase:

- Rust edition `2024`
- Bevy `0.18.1`
- State flow: `StartScreen -> Playing -> GameOver -> StartScreen`
- Current architecture: small domain plugins under `src/`
- Current rendering/audio approach: asset-backed sprites and audio loaded through `AssetServer`

This spec replaces the previous draft, which mixed future ideas with implementation details that do not match the repository. In particular, the old draft conflicted with the actual project in these ways:

- It described a primitive-shape, no-texture, no-audio game, while the code uses sprites and audio plugins.
- It referenced modules and resources that do not exist yet as if they were already part of the project.
- It assumed portrait-oriented constants and a different window title than the current repository.
- It treated stretch goals such as tractor beams and challenge stages as baseline requirements for the first implementation pass.

The goal of this version is to be useful during development, not just aspirational.

## 2. Current Baseline

The repository currently implements a simple arcade template rather than Galaga:

- `src/main.rs`: boots Bevy, configures a single `Camera2d`, registers plugins
- `src/states.rs`: defines `AppState::{StartScreen, Playing, GameOver}`
- `src/resources.rs`: contains `GameData { score }`
- `src/player.rs`: spawns one player ship, supports four-direction movement, clamps to the window
- `src/enemy.rs`: spawns three static enemies in a row
- `src/combat.rs`: fires one laser per `Space` press, moves lasers upward, destroys enemies on contact, advances to `GameOver` when all enemies are gone
- `src/ui.rs`: start screen, score HUD, game over screen
- `src/audio.rs`: loops gameplay music while in `Playing`

Current constants in `src/constants.rs`:

- Window: `1280x720`
- Title: `"Bevy 2D Template"`
- Player speed: `300.0`
- Enemy count: `3`
- Enemy score: `100`

Current implementation gaps relative to a Galaga game:

- Player movement is four-directional instead of horizontal-only.
- Enemies are static and never attack.
- There is no lives system, no waves, and no formation logic.
- Reaching zero enemies currently transitions to `GameOver` instead of a new wave or stage-clear flow.
- Asset paths are hardcoded in code, but the repository currently does not include the referenced asset files.

## 3. Product Goal

Build a compact Galaga-inspired arcade game while preserving the repo's simple plugin-based structure.

Primary goals:

- Fixed-screen shooting with a single player ship at the bottom of the screen
- Enemy formation at the top of the playfield
- Waves of enemies that can break formation and dive toward the player
- Score-based progression across repeated waves
- Clear start, gameplay, and game-over screens
- Use of Bevy 0.18.1 idioms already present in the project

Secondary goals:

- Better HUD information such as lives and wave number
- Distinct enemy roles
- Audio feedback for shooting, destruction, and music

Stretch goals:

- Boss capture mechanic
- Dual-fighter recovery
- Challenge stages

## 4. Scope Definition

### 4.1 First Playable Milestone

The first milestone should produce a recognizably Galaga-like loop without overextending the codebase:

- Horizontal-only player movement
- Player bullets with a simple fire-rate or bullet-count limit
- Formation-based enemy layout
- Repeating waves
- Basic enemy dive attacks
- Player lives and respawn flow
- Proper stage clear instead of immediate game over
- HUD showing score, lives, and wave

This milestone should not require tractor beams, captured ships, challenge stages, or a full path-scripting engine.

### 4.2 Deferred Features

These features are valuable but should be deferred until the core loop is solid:

- Tractor beam capture
- Dual fighter mode
- Challenge stages
- Persistent high score saved to disk
- Complex scripted enemy choreography across many wave variants
- Custom rendering with meshes, bloom, or particle-heavy effects

## 5. Gameplay Specification

### 5.1 State Flow

The project should continue using the existing top-level state machine:

`StartScreen -> Playing -> GameOver -> StartScreen`

Within `Playing`, finer-grained gameplay flow should be tracked with a resource rather than a new app state. Recommended phase enum:

```rust
enum WavePhase {
    Spawning,
    Combat,
    Respawning,
    StageClear,
}
```

Reasoning:

- It avoids repeated teardown and respawn of unrelated systems.
- It matches the existing architecture that already uses a single gameplay state.
- It keeps UI and gameplay plugins simpler than adding many new app states.

### 5.2 Player

Required behavior:

- Movement is horizontal-only
- Inputs: `Left/A` and `Right/D`
- Player remains near the bottom of the screen
- `Space` fires upward
- Player begins a run with 3 lives
- On hit, the player loses one life and respawns after a short delay if lives remain
- Temporary invulnerability after respawn is recommended for fairness

Implementation notes:

- Remove vertical movement from `src/player.rs`
- Keep bounds clamping based on the active playable width
- Add lives to `GameData`
- Do not despawn the whole game state on each death; only the player entity should cycle

Recommended initial values:

- `PLAYER_SPEED = 300.0` to `400.0`
- Respawn delay: `1.0` to `1.5` seconds
- Initial invulnerability: about `1.0` second

### 5.3 Player Firing

Required behavior:

- `Space` fires a laser upward from the player's ship
- Laser travels upward and despawns when leaving the screen
- At least one anti-spam control should be present

Recommended approach for milestone 1:

- Limit player bullets on screen to `2`, or
- Add a short cooldown timer between shots

Either approach is acceptable. A two-bullet cap is closer to Galaga and fits the current architecture well.

### 5.4 Enemies

Required behavior:

- Enemies spawn in a formation near the top of the screen
- Formation persists while individual enemies leave and rejoin it
- At intervals, one or more enemies break formation and dive toward the player
- Destroyed enemies award score

Minimum enemy roster for milestone 1:

- One standard enemy type worth `100` points
- Optional second type with more score or a slightly different dive pattern

Stretch roster:

- Standard enemy
- Mid-tier enemy
- Boss enemy with extra health

### 5.5 Formation

Required behavior:

- Enemies should no longer spawn as only three static units
- Formation should represent a grid with deterministic slot positions
- Empty slots remain empty after an enemy is destroyed
- A diving enemy keeps ownership of its slot until it is destroyed

Reasonable starting layout:

- 3 to 5 columns
- 2 to 4 rows
- Top-third of the screen reserved for the formation

Optional polish:

- Gentle horizontal sway of the full formation

### 5.6 Enemy Attacks

Required behavior for the first milestone:

- Diving enemies move on readable curved or angled paths toward the lower part of the screen
- Some dives may fire a bullet downward
- Surviving diving enemies return to formation

Implementation guidance:

- Start with hand-authored path patterns or simple curve math
- Do not introduce a full Bezier library until the simpler approach proves insufficient
- Keep attack selection timer-driven and deterministic enough to debug

### 5.7 Collisions and Damage

Required collision pairs:

- Player laser vs enemy
- Enemy bullet vs player
- Diving enemy body vs player

Expected results:

- Laser hit destroys a standard enemy and awards score
- Enemy bullet hit removes one player life unless invulnerable
- Body collision also costs the player a life and usually destroys the diving enemy

Implementation guidance:

- Circle-distance collision is sufficient for this project
- Keep collision handling centralized in `src/combat.rs` where practical
- Avoid duplicate hit processing within the same frame

### 5.8 Waves and Progression

Required behavior:

- When all active enemies are destroyed, the game should advance to the next wave instead of ending immediately
- Each new wave respawns a new formation
- Difficulty should increase gradually

Recommended scaling knobs:

- More enemies
- Faster dive selection
- Faster bullets
- More simultaneous divers

Wave tracking should live in `GameData`.

### 5.9 Win and Loss Conditions

Loss condition:

- The run ends when the player has no lives remaining and no respawn is pending

Non-loss condition:

- Clearing a wave does not end the run

Game over screen should show:

- Final score
- Wave reached
- Restart prompt

## 6. UI Specification

### 6.1 Start Screen

Must include:

- Game title
- Prompt to start with `Space`

Should include:

- A clearer project title than `"Bevy 2D Template"`

Recommended title strings:

- `"Galaga Clone"`
- `"Galaga Prototype"`
- `"Galaga"`

### 6.2 In-Game HUD

Must include:

- Score
- Lives
- Wave number

Nice to have:

- High score for the current session

### 6.3 Game Over Screen

Must include:

- "Game Over"
- Final score
- Wave reached
- Restart prompt

## 7. Technical Design

### 7.1 Keep The Existing Module Ownership

The current project layout is small and workable. Extend it before creating new modules.

Current modules and intended ownership:

- `src/main.rs`: bootstrapping only
- `src/states.rs`: top-level state enum
- `src/constants.rs`: shared tuning values
- `src/resources.rs`: score, lives, wave, timers, gameplay phase
- `src/components.rs`: shared marker and data components
- `src/player.rs`: player spawn, input, movement, respawn
- `src/enemy.rs`: enemy spawn, formation assignment, dive selection
- `src/combat.rs`: shooting, bullet motion, collisions, scoring, stage-clear detection
- `src/ui.rs`: start screen, HUD, game over screen
- `src/audio.rs`: background music and sound effect lifecycle

Do not add new modules until an existing module becomes clearly overloaded. For example:

- A separate `formation.rs` file is optional, not mandatory
- A separate `paths.rs` file is optional, not mandatory

### 7.2 Resources To Add

Recommended `GameData` expansion:

```rust
struct GameData {
    score: u32,
    wave: u32,
    lives: u32,
    phase: WavePhase,
}
```

Optional follow-up resources:

- `ShotCooldown`
- `RespawnState`
- `EnemyAttackTimer`

### 7.3 Components To Add

Recommended additions to `src/components.rs`:

- `PlayerBullet`
- `EnemyBullet`
- `FormationSlot`
- `DivingEnemy`
- `RespawningPlayer` or equivalent timer-bearing component

Use shared components only when they need to be referenced across modules.

### 7.4 Assets

The current code references these asset paths:

- `player_ship.png`
- `enemy_ufo_green.png`
- `player_laser.png`
- `music_spaceshooter.ogg`
- `sfx_laser1.ogg`

At the time of writing, the `assets/` directory is not present in the repository. That means a clean run from this checkout will fail to display expected visuals or audio correctly.

This must be resolved in one of two ways:

1. Add the required assets under `assets/`
2. Replace asset usage with generated visuals and remove missing asset dependencies

Because the current code is already sprite-and-audio based, option 1 is the more direct path for this repository.

### 7.5 Window And Presentation

The current project uses:

- `WINDOW_WIDTH = 1280.0`
- `WINDOW_HEIGHT = 720.0`

This is acceptable for a prototype. A portrait-oriented playfield can be considered later, but it should not be assumed by the spec until constants and UI are actually changed.

### 7.6 Validation

Expected validation commands from the repo root:

```powershell
cargo check
cargo clippy
```

Use `cargo check` for most gameplay changes. Use `cargo clippy` when broader API usage changes or refactors land.

## 8. Known Design Corrections From The Previous Draft

These corrections are now intentional parts of the spec:

- The game is asset-backed unless and until the codebase is explicitly migrated away from assets.
- Audio is in scope because the repository already has an audio plugin.
- The top-level app state remains three states; wave flow belongs in resources.
- Challenge stages and tractor beam capture are stretch goals, not milestone-1 requirements.
- New modules are optional and should be introduced only when needed.
- Stage clear should lead to the next wave, not to `GameOver`.
- The spec no longer assumes portrait orientation or bloom-based rendering as hard requirements.

## 9. Delivery Plan

### Phase 1: Make The Current Template Behave Like A Fixed-Screen Shooter

- Rename the window title to match the project
- Restrict player movement to horizontal-only
- Add lives and wave tracking
- Change win handling from `GameOver` to stage clear plus next wave
- Expand HUD to show score, lives, wave

### Phase 2: Add Galaga Structure

- Replace the three-static-enemy setup with a formation grid
- Add enemy dive behavior
- Add enemy bullets
- Add player death and respawn flow

### Phase 3: Add Variety And Polish

- Add multiple waves with difficulty scaling
- Add a second enemy type or tougher enemy
- Improve audio feedback
- Improve UI presentation

### Phase 4: Stretch Features

- Boss enemy behavior
- Tractor beam and capture flow
- Dual fighter mode
- Challenge stages

## 10. Acceptance Criteria

The first true Galaga milestone is complete when all of the following are true:

- The player moves only left and right
- The player has multiple lives
- Clearing a wave starts another wave instead of ending the run
- Enemies exist in a visible formation
- Some enemies leave formation and attack
- The player can be hit by bullets or diving enemies
- The HUD shows score, lives, and wave
- `GameOver` occurs only when the player runs out of lives
- The spec, code, and actual assets in the repo no longer contradict one another

## 11. Out Of Scope For This Document

This document does not prescribe:

- Exact sprite art direction
- Exact sound design
- Save-file format
- Mobile support
- Multiplayer

Those can be specified later if they become real project goals.
