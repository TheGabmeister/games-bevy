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

Use `cargo check` for most changes. Use `cargo clippy` when changing API patterns or broader refactors.

## Dev Profile

Dependencies compile at `opt-level = 3` while the main crate uses `opt-level = 1` — fast iteration on game code with performant dependencies.

## Tech Stack

- **Bevy 0.18.1** — ECS game engine
- **Rust Edition 2024**

## Architecture

### Two-Layer State Machine

1. **`AppState`** (Bevy `States` enum in `states.rs`): drives `OnEnter`/`OnExit` lifecycle.
   - `StartScreen` → `Playing` → `GameOver` → `StartScreen`

2. **`WavePhase`** (field on `GameData` resource in `resources.rs`): controls flow *within* `Playing`.
   - `Spawning` → `Combat` ↔ `Respawning` → `StageClear` → `Spawning` (loop)
   - Systems gate on phase via `if game_data.phase != WavePhase::Combat { return; }`.

Phase transitions are spread across modules:
- `enemy.rs`: `Spawning → Combat` (entities exist), `StageClear → Spawning` (new wave)
- `combat.rs`: `Combat → StageClear` (all enemies dead), `Combat → Respawning` (player hit)
- `player.rs`: `Respawning → Combat` (respawn timer fires)

### Module Responsibilities

- **`main.rs`** — App bootstrap, plugin registration, camera, `ClearColor`
- **`constants.rs`** — All tuning values (window, speeds, radii, scoring, timers, effects)
- **`resources.rs`** — `GameData` (score/lives/wave/phase), timer resources, `FormationSway`, `DiveSelectionCursor`
- **`components.rs`** — Shared marker and data components
- **`player.rs`** — Player spawn (triangle mesh), horizontal input, movement, respawn, invulnerability blink
- **`enemy.rs`** — Formation spawn (shaped meshes), sway, dive selection/movement, enemy bullets, wave progression. Exports `score_for_row()` and `enemy_color_for_row()`
- **`combat.rs`** — Player shooting (2-bullet cap), bullet movement, all collision pairs, stage clear detection, player death handling. Calls `spawn_explosion()` on kills/deaths
- **`effects.rs`** — Scrolling starfield (Startup, persists across states), explosion particles (on-demand from combat)
- **`ui.rs`** — Start screen, HUD (score/lives/wave), game over screen (score + wave reached)
- **`audio.rs`** — Background music lifecycle (gracefully skips if asset file missing)

### Rendering Approach

Two rendering strategies coexist:
- **Ships** (player and enemies): `Mesh2d` + `MeshMaterial2d<ColorMaterial>` with shaped meshes (Triangle2d, Rhombus, RegularPolygon). These require `Assets<Mesh>` and `Assets<ColorMaterial>` in spawn functions.
- **Bullets, stars, explosion particles**: `Sprite` with `color` and `custom_size`. Simpler, no asset handles needed.

No image assets are required for gameplay. Audio assets are optional.

Z-ordering: starfield at -10, enemies at 0, player at 1, explosions at 5.

### Key System Ordering

Combat systems are explicitly chained with `.after()`:
```
player_shoot → move_player_bullets → laser_enemy_collision
→ enemy_bullet_player_collision → diving_enemy_player_collision → check_stage_clear
```

Enemy systems are also chained:
```
formation_sway → select_divers / update_divers → diving_enemy_shoot → move_enemy_bullets
```

### Coding Patterns

- Put tunable values in `constants.rs`, not inline.
- Add shared mutable game state to `resources.rs`, marker/data components to `components.rs`.
- When spawning entities on `OnEnter`, define matching cleanup on `OnExit`.
- Prefer extending an existing module's plugin over adding ad hoc systems in `main.rs`.
- Use deterministic helpers over random when possible (see `pseudo_random()` in `effects.rs`).

### Bevy 0.18.1 API Notes

- `despawn()` is recursive by default — despawns entity and all children. Do **not** use `despawn_recursive()` (removed).
- `WindowResolution::new(width, height)` takes `u32`, not `f32`. Cast with `as u32`.
- `Timer::just_finished()` is the check method — `finished` is a private field, not a method.
- `Option<ResMut<T>>` as a system param handles resources that may not exist yet.
- `commands.entity(e).remove::<T>()` removes a component without despawning — used for toggling dive state.
- `ColorMaterial` is in the prelude — import via `use bevy::prelude::*`, not `bevy::sprite::ColorMaterial`.
- `Visibility::Hidden`/`Visibility::Visible` for toggling entity visibility (used for invulnerability blink).
- `Query::is_empty()` is preferred over counting when you only need an existence check.
