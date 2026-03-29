# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\galaga`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- Small Galaga-style 2D arcade prototype using Bevy ECS, state-driven UI, and mostly code-generated visuals

## Build And Validation

Use these commands from the project root:

```powershell
cargo run
cargo build
cargo check
cargo clippy
```

Validation expectations:

- Run `cargo check` for most code changes.
- Run `cargo clippy` when changing patterns that may affect API usage or code quality broadly.
- If you cannot run validation, say so explicitly.

## Dev Profile

- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Keep iteration-friendly workflows in mind; prefer targeted changes over broad rewrites.

## Project Layout

- `src/main.rs`: app bootstrap, window configuration, plugin registration, camera setup
- `src/states.rs`: `AppState` state machine
- `src/constants.rs`: window, gameplay, and UI tuning values
- `src/components.rs`: marker and data components
- `src/resources.rs`: shared game-wide resources such as score, wave, lives, gameplay phase, and timers
- `src/player.rs`: player spawn, input, horizontal movement, invulnerability, respawn, cleanup
- `src/enemy.rs`: formation spawn, sway, dive selection, enemy bullets, wave progression, cleanup
- `src/combat.rs`: shooting, projectile movement, collision, scoring, death handling, stage clear/game over logic
- `src/ui.rs`: start screen, HUD, game over screen, state transitions from UI input
- `src/audio.rs`: music lifecycle during gameplay
- `assets/`: optional runtime assets; currently only gameplay music is referenced

## Current Game Flow

The current state machine is:

`StartScreen -> Playing -> GameOver -> StartScreen`

Important behavior tied to state transitions:

- `StartScreen`: title/menu UI exists, `Space` starts the run
- `Playing`: player, enemies, HUD, and optional looping music are spawned
- `GameOver`: final score UI exists, `Space` resets score and returns to `StartScreen`

Within `Playing`, gameplay flow is tracked via `WavePhase` in `GameData`:

- `Spawning`: a formation exists or is being created; once enemies are present, flow advances to combat
- `Combat`: normal play, player fire, enemy dives, collisions, and scoring
- `Respawning`: player is absent temporarily after death while the respawn timer runs
- `StageClear`: all enemies are gone; a short delay runs before the next wave spawns

When adding features, decide first whether they are:

- state-specific systems gated with `run_if(in_state(...))`
- enter/exit lifecycle systems attached to `OnEnter(...)` or `OnExit(...)`
- persistent resources that survive state changes

## Bevy Conventions Used Here

- Prefer small domain plugins over a large `main.rs`
- Use marker components for entity categories like `Player`, `Enemy`, `PlayerBullet`, `EnemyBullet`, and UI roots
- Use resources for cross-system shared state like `GameData`
- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry
- Use explicit system ordering with `.after(...)` where frame ordering matters
- Keep gameplay systems gated behind `AppState::Playing` unless they intentionally span menus
- Prefer deterministic gameplay helpers over time-based pseudo-random selection when debugging or balancing

## Coding Rules For This Repo

- Put new tunable values in `src/constants.rs` instead of scattering magic numbers.
- Add new shared mutable game state to `src/resources.rs`.
- Add marker/data ECS types to `src/components.rs` unless a component is tightly local and clearly better kept nearby.
- Keep module boundaries aligned to gameplay domains.
- Prefer extending an existing plugin in the relevant module over registering ad hoc systems from `main.rs`.
- When spawning entities on `OnEnter`, also define the matching cleanup path on `OnExit` if the entities should not persist.
- If a gameplay concept already exists in a helper function, extend that helper instead of duplicating spawn/setup logic.
- Be careful with deferred ECS commands during gameplay-state transitions; update `GameData.phase` explicitly when flow changes.
- Preserve the current simple architecture unless the task requires a broader refactor.

## UI And Asset Notes

- UI currently uses Bevy UI nodes and text directly, without a custom theme system.
- Text sizes and basic text color live in `src/constants.rs`.
- Asset paths are plain relative strings passed to `asset_server.load(...)`; keep them aligned with files under `assets/`.
- Current gameplay visuals are colored `Sprite` rectangles, not texture-backed sprites.
- `src/audio.rs` deliberately tolerates a missing music file and logs a warning instead of failing hard.
- Reuse existing naming patterns when adding related assets.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- The codebase uses Bevy's current component-style UI and `Camera2d`.
- `Query::is_empty()` is preferred over counting entities when you only need existence checks.

## Working Style

- Make the smallest coherent change that solves the task.
- Do not rewrite working gameplay structure just to make it "cleaner".
- Preserve user changes that are unrelated to the task.
- If you find conflicting local edits in the same area you need to touch, stop and surface the conflict.

## Preferred Change Pattern

1. Inspect the relevant module boundaries and state interactions.
2. Implement the change in the owning module.
3. Update constants/components/resources if the change introduces shared concepts.
4. Run validation, usually `cargo check`.
5. Summarize what changed and any remaining risks.

## Good First Places To Look

- Input or movement bug: `src/player.rs`
- Enemy formation, dive behavior, or wave progression: `src/enemy.rs` and `src/constants.rs`
- Projectile, collision, scoring, or death logic: `src/combat.rs`
- Menu/HUD/game over behavior: `src/ui.rs`
- Music lifecycle: `src/audio.rs`
- App boot or plugin wiring: `src/main.rs`
