# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Scope

This project lives inside a monorepo. Stay scoped to this directory only — do not read, edit, or run git commands that reference parent or sibling directories.

## Build & Run Commands

```bash
cargo run          # Build and run the game
cargo build        # Build only
cargo check        # Fast type-check without full compilation
cargo clippy       # Lint
```

## Dev Profile

Dependencies compile at `opt-level = 3` while the main crate uses `opt-level = 1` — this gives fast iteration on game code while keeping dependency performance reasonable.

## Tech Stack

- **Bevy 0.18.1** — ECS game engine
- **Rust Edition 2024**

## Architecture

### Two-Layer State Machine

The game uses two layers of state:

1. **`AppState`** (Bevy `States` enum in `states.rs`): drives `OnEnter`/`OnExit` lifecycle for plugins.
   - `StartScreen` → `Playing` → `GameOver` → `StartScreen`

2. **`WavePhase`** (field on `GameData` resource in `resources.rs`): controls flow *within* `Playing`.
   - `Spawning` → `Combat` ↔ `Respawning` → `StageClear` → `Spawning` (loop)
   - Systems gate on phase via `if game_data.phase != WavePhase::Combat { return; }`.

Phase transitions are spread across modules:
- `enemy.rs`: `Spawning → Combat` (when entities exist), `StageClear → Spawning` (new wave)
- `combat.rs`: `Combat → StageClear` (all enemies dead), `Combat → Respawning` (player hit)
- `player.rs`: `Respawning → Combat` (respawn timer fires)

### Module Responsibilities

- **`main.rs`** — App bootstrap, plugin registration, camera
- **`constants.rs`** — All tuning values (window, speeds, radii, scoring, timers)
- **`resources.rs`** — `GameData` (score/lives/wave/phase), timer resources, `FormationSway`
- **`components.rs`** — Shared marker and data components
- **`player.rs`** — Player spawn, horizontal input, movement, respawn timer, invulnerability blink
- **`enemy.rs`** — Formation spawn/sway, dive selection/movement, enemy bullets, wave progression
- **`combat.rs`** — Player shooting (2-bullet cap), bullet movement, all collision pairs, stage clear detection, player death handling
- **`ui.rs`** — Start screen, HUD (score/lives/wave), game over screen (score + wave reached)
- **`audio.rs`** — Background music lifecycle (gracefully skips if asset file missing)

### Rendering Approach

All game entities use code-generated colored `Sprite` components with `custom_size` — no image assets are required for gameplay. Audio assets are optional; `audio.rs` checks file existence before loading.

### Key System Ordering

Combat systems are explicitly chained with `.after()` to prevent double-hit and ensure correct frame ordering:
```
player_shoot → move_player_bullets → laser_enemy_collision
→ enemy_bullet_player_collision → diving_enemy_player_collision → check_stage_clear
```

### Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — it despawns the entity and all children. Do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`. Cast with `as u32` if your constants are `f32`.
- `Timer::just_finished()` is the check method — `finished` is a private field, not a callable method. Use `just_finished()` after calling `.tick(time.delta())`.
- `Option<ResMut<T>>` as a system param allows systems to handle resources that may not exist yet.
- `commands.entity(e).remove::<T>()` removes a component without despawning — used for toggling dive state.
