# AGENTS.md

Guidance for coding agents working in `c:\dev\games-bevy\tetris`.

## Scope

- Stay scoped to this project directory.
- Do not read, edit, or run git commands against parent or sibling directories.
- Treat unrelated local changes as user-owned unless the task clearly requires touching them.

## Stack

- Rust edition `2024`
- Bevy `0.18.1`
- `rand` `0.9`
- `bevy` currently does **not** enable `dynamic_linking`; the feature is present in `Cargo.toml` as a commented option.
- Current app state: playable core Tetris prototype with board rendering, spawning, movement, rotation, gravity, lock delay, line clears, and line-clear flash effects.

## Build And Validation

Use these commands from the project root:

```powershell
cargo run
cargo build
cargo check
cargo clippy
```

Validation expectations:

- Run `cargo check` for Rust code changes.
- Run `cargo clippy` when changing APIs, gameplay logic patterns, or shared infrastructure in a broader way.
- For docs-only changes, validation is optional.
- If you cannot run validation, say so explicitly.

## Build Configuration

- Target output is redirected by `.cargo/config.toml` to `D:/cargo-target-dir`.
- The crate uses `opt-level = 0` in dev.
- Dependencies use `opt-level = 3` in dev.
- Keep iteration-friendly workflows in mind; prefer targeted changes over broad rewrites.

## Current Project Layout

- `src/main.rs`: app bootstrap, window setup, plugin registration, camera setup
- `src/constants.rs`: window, grid, rendering, timing, scoring, and gameplay constants
- `src/board.rs`: board resource, playfield rendering, board cell syncing, line-clear flash effect
- `src/tetromino.rs`: tetromino definitions, SRS data, active piece resource, 7-bag randomizer, active piece rendering
- `src/input.rs`: keyboard and gamepad input translation into `InputActions`
- `src/gameplay.rs`: gameplay systems for horizontal movement, rotation, hard drop, gravity, lock delay, and respawn after locking
- `SPEC.md`: current product and implementation spec for the Tetris project
- `.cargo/config.toml`: shared cargo target-dir configuration

## Current Runtime Behavior

The current app is a Tetris prototype, not a starter scene:

- `DefaultPlugins` are registered with a custom `WindowPlugin` configuration.
- A `Camera2d` is spawned at startup with bloom, tonemapping, and debanding enabled.
- The playfield border and visible board cell sprites are spawned at startup.
- A first tetromino is spawned from a 7-bag randomizer and rendered as 4 sprites.
- Gameplay input supports keyboard and basic gamepad mappings.
- Horizontal movement includes DAS handling.
- Rotation uses SRS wall kicks, with separate kick data for the I piece.
- Gravity, soft drop, hard drop, lock delay, piece locking, row clearing, and line-clear flash effects are implemented.
- Scoring, HUD, hold, ghost piece, next queue UI, menus, pause/game-over states, and restart flow are not implemented yet.

When making changes, align your work with what actually exists in the repo rather than assuming later phases from `SPEC.md` are already wired up.

## Architecture Guidance

Prefer keeping the current module split and extending the owning module for the behavior you are changing:

- App wiring belongs in `src/main.rs`.
- Tunable values belong in `src/constants.rs`.
- Board storage and board rendering belong in `src/board.rs`.
- Piece definitions, rotation data, and active piece rendering belong in `src/tetromino.rs`.
- Input normalization belongs in `src/input.rs`.
- Piece motion, gravity, locking, and other gameplay flow belong in `src/gameplay.rs`.

Only introduce new modules when the current domain split becomes meaningfully crowded. Good future extraction points include `resources.rs`, `states.rs`, and dedicated UI modules once scoring, menus, hold, and queue systems exist.

## Bevy Conventions To Follow

- Use resources for cross-system gameplay state.
- Keep systems reading `InputActions` instead of raw `KeyCode` outside `src/input.rs`.
- Use explicit ordering or `.chain()` where frame ordering matters.
- Use `OnEnter` and `OnExit` for setup and cleanup once app/gameplay states are added.
- Gate gameplay systems with `run_if(in_state(...))` once states exist.
- Keep rendering and gameplay responsibilities separated when practical.

## Coding Rules For This Repo

- Make the smallest coherent change that solves the task.
- Do not rewrite working structure just to make it "cleaner".
- Preserve the existing Tetris direction; do not reframe the project as a shooter or generic Bevy sample.
- Prefer extending the module that already owns the behavior instead of moving code around preemptively.
- Put new tunable values in `src/constants.rs` rather than scattering magic numbers.
- If a new shared gameplay concept appears often enough, promote it into a resource or a dedicated module instead of hiding it inside one system file.
- Keep input semantics centralized in `src/input.rs`.
- Keep board coordinate conventions consistent: row `0` is the bottom visible row, positive row movement is upward in board logic, and downward falling is represented by decreasing row values.
- Respect the existing hidden buffer-row model for spawn and rotation logic.

## UI And Asset Notes

- The current prototype uses Bevy 2D sprites and primitive rectangles; it does not use textures or audio.
- `SPEC.md` explicitly targets a shape-only presentation with bloom-enabled HDR tetromino colors.
- If future work adds UI such as score, hold, or next queue, keep it aligned with the current Tetris spec rather than the removed space-shooter notes.
- If assets are introduced later, keep asset paths as plain relative strings passed to `asset_server.load(...)`.

## Bevy 0.18.1 Notes

- `despawn()` is recursive by default; do not use removed older APIs like `despawn_recursive()`.
- `WindowResolution::new(width, height)` expects `u32`.
- Use current 2D APIs such as `Camera2d` and `Sprite`.
- Bloom is available via `bevy::post_process::bloom::Bloom`.
- Tonemapping is available via `bevy::core_pipeline::tonemapping`.
- Use `ApplyDeferred` rather than a nonexistent `apply_deferred` helper function if deferred command application becomes necessary.

## Preferred Change Pattern

1. Inspect the current code and local module boundaries before making assumptions.
2. Check `SPEC.md` if the task involves intended gameplay behavior or planned features.
3. Implement the change in the owning file or module.
4. Extract new modules or resources only when the code has clearly grown enough to justify it.
5. Run validation when appropriate, usually `cargo check`.
6. Summarize what changed and any remaining risks.

## Good First Places To Look

- App boot and plugin registration: `src/main.rs`
- Gameplay constants and tuning: `src/constants.rs`
- Board state and rendering: `src/board.rs`
- Falling piece logic and SRS data: `src/tetromino.rs`
- Input mapping: `src/input.rs`
- Gameplay flow: `src/gameplay.rs`
- Planned feature scope: `SPEC.md`
- Build output location: `.cargo/config.toml`
