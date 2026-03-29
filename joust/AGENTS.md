# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\joust`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- `bevy` is enabled with the `dynamic_linking` feature
- Current app state: a playable primitive-rendered Joust prototype with menus, waves, combat, eggs, lava, HUD, and game-over flow

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
- Run `cargo clippy` when changing API usage, scheduling, or broader architecture.
- If you cannot run validation, say so explicitly.

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Prefer targeted changes over broad rewrites to keep iteration fast.

## Current Project Layout

- `src/main.rs`: app bootstrap, state/message registration, plugin wiring, camera setup, arena spawn hook
- `src/constants.rs`: tunable physics, arena, scoring, timing, and spawn constants
- `src/components.rs`: gameplay components, marker types, and buffered message definitions
- `src/resources.rs`: game-wide mutable state, cached mesh/material handles, wave definitions
- `src/states.rs`: `AppState`, `PlayState`, and ordered gameplay `GameSet`s
- `src/player.rs`: player spawning, input, and respawn behavior
- `src/enemy.rs`: enemy spawning and AI
- `src/physics.rs`: gravity, drag, movement, platform landing, and horizontal wrap helpers
- `src/combat.rs`: invincibility, jousts, eggs, lava kills, score/life handling, game-over detection
- `src/waves.rs`: game reset, wave intro/clear transitions, wave timers
- `src/ui.rs`: start screen, HUD, game-over overlay
- `src/rendering.rs`: shared primitive mesh/material setup and animation helpers
- `src/effects.rs`: death particles
- `assets/`: currently available for future audio/art work, but core gameplay does not depend on them

## Current Runtime Behavior

The current game is no longer a starter scene. It currently does all of the following:

- Starts in `AppState::StartScreen`
- Lets the player toggle between 1-player and 2-player mode from the menu
- Transitions into `AppState::Playing` with `PlayState::{WaveIntro, WaveActive, WaveClear}`
- Spawns a primitive-rendered arena, players, enemies, lava, HUD, and wave text
- Supports flap-based movement, one-way platform landings, horizontal wrap, and joust combat
- Spawns eggs from defeated enemies, allows egg collection, and hatches eggs into stronger enemies
- Tracks score, lives, respawns, wave progression, and game over

Important current scheduling detail:

- Core simulation is still in `Update`, not `FixedUpdate`
- Gameplay systems that should be live only during active play are gated with `run_if(in_state(PlayState::WaveActive))`
- Some transitions and lifecycle systems still run across `AppState::Playing`

## Architecture Guidance

Prefer the existing modular plugin layout over collapsing logic back into `main.rs`.

When extending the game:

- Keep app wiring in `src/main.rs`
- Put tunable values in `src/constants.rs`
- Put new ECS markers/data in `src/components.rs`
- Put shared mutable state in `src/resources.rs`
- Keep state definitions in `src/states.rs`
- Extend the owning gameplay domain module instead of sprinkling ad hoc systems across files

## Bevy Conventions To Follow

- Use `OnEnter`/`OnExit` or `DespawnOnExit(...)` for state-scoped entities.
- Gate active gameplay systems with `run_if(in_state(PlayState::WaveActive))` unless they intentionally need to run across all of `AppState::Playing`.
- Register new buffered messages in `main.rs` with `add_message::<T>()` before using `MessageReader<T>` or `MessageWriter<T>`.
- Use resources for cross-system shared state and components for per-entity state.
- Keep explicit set ordering through `GameSet::{Input, Ai, Physics, Combat, Progression}`.
- Use `ApplyDeferred` only when a later system in the same frame must observe earlier command results immediately.
- `despawn()` is recursive by default in Bevy `0.18.1`; do not use removed APIs like `despawn_recursive()`.

## Coding Rules For This Repo

- Make the smallest coherent change that solves the task.
- Do not rewrite working structure just to make it "cleaner".
- Preserve module ownership by gameplay domain.
- Prefer modifying the owning plugin/module instead of introducing parallel patterns.
- Keep constants out of system bodies when they are tuning values.
- When spawning persistent gameplay entities for a state, define their cleanup path.
- If you introduce a new message-driven gameplay flow, make sure the message is registered and validated with `cargo check`.

## UI And Asset Notes

- UI uses Bevy's component-based UI directly.
- In-world wave text uses `Text2d`.
- Core actors and arena pieces are rendered from primitive meshes and shared `ColorMaterial` handles.
- Asset paths should remain plain relative strings passed to `asset_server.load(...)` if assets are introduced later.
- Do not assume the existing `assets/` directory is wired into current gameplay; most current visuals are code-built primitives.

## Current Gameplay Notes

- Player 1 supports `A/D` or arrow keys for horizontal movement and `W`, `Space`, or `Up` to flap.
- Player 2 supports `J/L` to move and `I` to flap.
- Flapping is press-based, not hold-based.
- Enemy AI uses the same wrap helpers as gameplay movement/combat.
- Combat currently uses buffered messages for kills, score changes, and player deaths.
- Wave clear requires both enemies and eggs to be gone.

## Preferred Change Pattern

1. Inspect the owning module and nearby state/resources before making assumptions.
2. Confirm how the relevant system is gated by `AppState`/`PlayState`.
3. Implement the change in the owning file or plugin.
4. Run validation, usually `cargo check`.
5. Run `cargo clippy` if scheduling or API patterns changed.
6. Summarize what changed and any remaining risks.

## Good First Places To Look

- App boot, state/message registration, and plugin wiring: `src/main.rs`
- Shared state and cached handles: `src/resources.rs`
- App/play state flow: `src/states.rs`
- Current gameplay rules: `src/player.rs`, `src/enemy.rs`, `src/physics.rs`, `src/combat.rs`, `src/waves.rs`
- Menu and HUD behavior: `src/ui.rs`
- Primitive rendering helpers: `src/rendering.rs`
