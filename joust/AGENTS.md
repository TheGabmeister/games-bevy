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
- Current app state: minimal Bevy starter, not yet a full arcade/shooter implementation

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

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 1` in dev.
- Dependencies use `opt-level = 3` in dev.
- Keep iteration-friendly workflows in mind; prefer targeted changes over broad rewrites.

## Current Project Layout

- `src/main.rs`: current app bootstrap and all gameplay/UI code that exists today
- `assets/`: space-shooter themed sprite and audio assets available for future gameplay work
- `.cargo/config.toml`: shared cargo target-dir configuration

## Current Runtime Behavior

The current app is a starter scene:

- `DefaultPlugins` are registered with no custom plugin split yet.
- A `Camera2d` is spawned at startup.
- A centered `Hello, World!` UI text node is spawned at startup.
- There is no state machine, gameplay loop, audio lifecycle, or asset loading wired up yet.

When making changes, align your work with what actually exists in the repo rather than assuming the larger game architecture is already implemented.

## Architecture Guidance For Future Expansion

If the user grows this starter into the intended arcade/shooter template, prefer this structure:

- `src/main.rs`: app setup, plugin registration, high-level wiring
- `src/constants.rs`: tunable values such as window size, speeds, cooldowns, UI sizing
- `src/components.rs`: marker and data ECS components
- `src/resources.rs`: shared mutable game-wide state
- `src/states.rs`: `AppState` enum and state-related helpers
- Domain modules such as `src/player.rs`, `src/enemy.rs`, `src/combat.rs`, `src/ui.rs`, and `src/audio.rs`

Prefer small domain plugins over growing `main.rs` indefinitely once the game has more than a handful of systems.

## Bevy Conventions To Follow

- Use `OnEnter` and `OnExit` for spawn/cleanup symmetry once states are introduced.
- Gate gameplay systems with `run_if(in_state(...))` once an `AppState` exists.
- Use resources for cross-system shared state.
- Use marker components for entity categories.
- Use explicit system ordering with `.after(...)` where frame ordering matters.

## Coding Rules For This Repo

- Make the smallest coherent change that solves the task.
- Do not rewrite working structure just to make it "cleaner".
- Keep module boundaries aligned to gameplay domains once modules are introduced.
- Put new tunable values in `src/constants.rs` instead of scattering magic numbers once that module exists.
- Add new shared mutable game state to `src/resources.rs` once that module exists.
- Add shared marker/data ECS types to `src/components.rs` once that module exists.
- Prefer extending an existing domain plugin over registering many ad hoc systems from `main.rs`.
- When spawning entities tied to a state, define the matching cleanup path on `OnExit` if they should not persist.

## UI And Asset Notes

- UI currently uses Bevy's component-based UI directly.
- Asset paths should remain plain relative strings passed to `asset_server.load(...)`.
- Keep asset references aligned with files under `assets/`.
- Reuse the existing space-shooter asset naming pattern when adding related content.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- `ScalingMode` is in `bevy::camera::ScalingMode`.
- Use `ApplyDeferred` rather than a nonexistent `apply_deferred` helper function.
- 2D rendering uses current Bevy APIs such as `Camera2d`, `Sprite`, `Mesh2d`, and `MeshMaterial2d`.

## Preferred Change Pattern

1. Inspect the current code and local module boundaries before making assumptions.
2. Implement the change in the owning file or module.
3. Extract constants/resources/components only when the code has grown enough to justify it.
4. Run validation, usually `cargo check`.
5. Summarize what changed and any remaining risks.

## Good First Places To Look

- App boot or current behavior: `src/main.rs`
- Build output location: `.cargo/config.toml`
- Dependency/runtime configuration: `Cargo.toml`
- Available art/audio for future gameplay work: `assets/`
