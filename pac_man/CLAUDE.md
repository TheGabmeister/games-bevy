# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo check          # Type-check without building
cargo clippy         # Lint
cargo test           # Run unit tests (in level.rs and logic.rs)
cargo run            # Launch the game (use for behavior-sensitive changes)
```

Build artifacts go to `D:/cargo-target-dir` (configured in `.cargo/config.toml`). Dynamic linking is enabled for faster dev builds via Bevy's feature flag.

## Stack

- Rust edition 2024, Bevy 0.18.1 (sole dependency)
- 2D game: meshes, color materials, no asset files on disk

## Architecture

Single plugin (`PacmanPlugin` in `src/game/mod.rs`) wires all systems. The plugin is the source of truth for schedules, states, and system ordering.

**Module responsibilities:**

| Module | Role |
|--------|------|
| `game/mod.rs` | Plugin orchestration and system scheduling only |
| `game/components.rs` | ECS components, marker types, `Direction`/`GhostPersonality` enums |
| `game/resources.rs` | `GameSession` (score, lives, timers), `GameMeshes`, `GameMaterials` |
| `game/constants.rs` | Tuning values, colors, z-layers, and the embedded ASCII level map |
| `game/level.rs` | `LevelLayout` resource — ASCII parsing, tile/world coordinate conversion, wall queries |
| `game/logic.rs` | Pure functions for ghost AI targeting (no ECS dependencies, unit-tested) |
| `game/systems/setup.rs` | Asset creation, camera/HUD/wall spawning, round entity spawning |
| `game/systems/flow.rs` | `RoundState` transitions and timer advancement |
| `game/systems/input.rs` | Player movement (arrows/WASD) and restart (Space/Enter) |
| `game/systems/simulation.rs` | Fixed-timestep movement, ghost AI planning, pellet collection, collisions |
| `game/systems/presentation.rs` | Ghost appearance sync, Pac-Man mouth animation, power pellet pulse, HUD text |

**Schedule layout:**
- `Startup` — assets → scene → round entities (chained sets)
- `Update` — input handling, then visual/HUD sync
- `FixedUpdate` — round state timers → ghost planning → movement → interactions (chained sets)

**State machine** (`RoundState`): `Ready` → `Playing` → `Won`/`GameOver`. Entering `Ready` resets actor positions and timers. `Won`/`GameOver` stop simulation and wait for restart input.

**Entity lifetimes:**
- `LevelEntity` — static geometry, persists for the app lifetime
- `RoundEntity` — gameplay entities (player, ghosts, pellets), destroyed and recreated on round reset
- HUD entities are persistent and not tied to round resets

## Key Design Rules

- Simulation in `FixedUpdate`, rendering/animation/UI in `Update` — never mix
- Pure logic belongs in `logic.rs` or `level.rs`, not in system functions
- Extend Bevy `States` for new game modes; don't add boolean flags
- Keep `mod.rs` as orchestration only — no gameplay logic
- Small focused system modules; do not consolidate into a monolithic `systems.rs`
- Add unit tests for any new pure helper or parsing logic
