# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\donkey_kong`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- Shape-rendered 2D Donkey Kong-style arcade game using Bevy ECS, state-driven UI, and explicit collision logic

## Build And Validation

Use these commands from the project root:

```powershell
cargo run
cargo build
cargo check
cargo clippy -- -D warnings
```

Validation expectations:

- Run `cargo check` for most code changes.
- Run `cargo clippy -- -D warnings` when changing shared patterns, lifecycle code, or ECS structure.
- If you cannot run validation, say so explicitly.

## Dev Profile

- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Prefer targeted changes that keep compile times and iteration speed reasonable.

## Project Layout

- `src/main.rs`: app bootstrap, window configuration, camera setup, shared mesh/material resources
- `src/states.rs`: `AppState` state machine
- `src/constants.rs`: gameplay, rendering, and UI tuning values
- `src/components.rs`: ECS markers and gameplay state components
- `src/resources.rs`: persistent and per-run resources such as score, wave runtime, and configuration
- `src/level.rs`: stage geometry, spawn helpers, and `Playing`/`Dying` lifecycle orchestration
- `src/player.rs`: player input, movement, climbing, jumping, and hammer timer visuals
- `src/hazards.rs`: DK throw cadence, barrel movement, and fireball behavior
- `src/combat.rs`: collision checks, scoring, bonus items, timer logic, and wave-clear/death triggers
- `src/ui.rs`: title screen, HUD, death overlay, wave tally, game over, and win screen
- `assets/`: legacy assets from the template; current gameplay is primitive-shape based and does not depend on them

## Current Game Flow

The current state machine is:

`StartScreen -> Playing -> Dying -> Playing/WaveTally/GameOver -> StartScreen`

Additional terminal flow:

`WaveTally -> Playing` for the next wave

`WaveTally -> WinScreen -> StartScreen` after wave 5

Important behavior tied to state transitions:

- `StartScreen`: title UI exists, run-scoped state resets, `Space` starts a new run
- `Playing`: stage, player, hazards, pickups, HUD, and timer-driven gameplay are active
- `Dying`: gameplay is frozen while the player flashes and the death cause overlay is shown
- `WaveTally`: remaining bonus timer converts into score, then advances wave or wins
- `GameOver` and `WinScreen`: final summary UI exists, `Space` returns to `StartScreen`

When adding features, decide first whether they are:

- state-specific systems gated with `run_if(in_state(...))`
- enter/exit lifecycle systems attached to `OnEnter(...)` or `OnExit(...)`
- persistent resources that survive state changes
- per-attempt entities that should be cleared on death retry and wave advance

## Bevy Conventions Used Here

- Prefer small domain plugins over a large `main.rs`
- Keep app bootstrap in `main.rs` and gameplay lifecycle ownership in the domain module that owns it
- Use `StageEntity` for persistent stage actors within a run and `AttemptEntity` for hazards and pickups that reset on death or wave advance
- Use resources for cross-system shared state such as `SessionData`, `RunData`, `WaveRuntime`, `WaveConfig`, and `StageData`
- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry
- Use explicit system ordering with `.after(...)` where frame ordering matters
- Keep gameplay systems gated behind `AppState::Playing` unless they intentionally span menus or transitions

## Coding Rules For This Repo

- Put new tunable values in `src/constants.rs` instead of scattering magic numbers.
- Add new shared mutable game state to `src/resources.rs`.
- Add reusable marker/data ECS types to `src/components.rs` unless a component is tightly local and clearly better kept nearby.
- Keep module boundaries aligned to gameplay domains.
- Prefer extending the owning plugin in the relevant module over registering ad hoc systems from `main.rs`.
- If a spawned entity should disappear on death retry or next wave, give it `AttemptEntity`.
- If a spawned entity should persist for the whole run until game over or a full restart, give it `StageEntity`.
- Preserve the current small, explicit architecture unless the task requires a broader refactor.

## UI And Asset Notes

- UI uses Bevy UI nodes and `Text` directly, without a custom theme system.
- Text sizes and shared text color live in `src/constants.rs`.
- The active game currently renders with colored primitives, not sprites.
- Do not add sprite or audio dependencies unless the task explicitly calls for that change.
- If you reuse files from `assets/`, confirm they are actually wired into gameplay rather than assuming the old template still uses them.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- The codebase uses Bevy's current component-style UI and `Camera2d`.
- Keep query assumptions explicit. This codebase often expects one player, one DK, and one HUD root.

## Working Style

- Make the smallest coherent change that solves the task.
- Do not rewrite working gameplay structure just to make it "cleaner".
- Preserve user changes that are unrelated to the task.
- If you find conflicting local edits in the same area you need to touch, stop and surface the conflict.

## Preferred Change Pattern

1. Inspect the relevant module boundaries and state interactions.
2. Implement the change in the owning module or plugin.
3. Update constants, components, or resources if the change introduces shared concepts.
4. Run validation, usually `cargo check`, and `cargo clippy -- -D warnings` for broader changes.
5. Summarize what changed and any remaining risks.

## Good First Places To Look

- Input, movement, jumping, climbing, or hammer timing: `src/player.rs`
- Stage geometry, spawn/reset behavior, and playing-state lifecycle: `src/level.rs`
- Barrel, fireball, and DK behavior: `src/hazards.rs`
- Collision, score, timer, bonus items, and wave clear logic: `src/combat.rs`
- Title/HUD/death/tally/final screens: `src/ui.rs`
- App boot and plugin wiring: `src/main.rs`
