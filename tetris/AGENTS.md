# AGENTS.md

Guidance for coding agents working in `tetris/`.

## Scope

- This game lives in `C:/dev/games-bevy/tetris`, inside a larger git repository rooted at `C:/dev/games-bevy`.
- Keep changes scoped to `tetris/` unless the user explicitly asks for cross-project work.
- Do not modify sibling projects such as `adventure/` as part of normal Tetris tasks.

## Build Commands

```bash
cargo check          # Type-check without building
cargo build          # Debug build
cargo run            # Run the game
cargo clippy         # Lint
```

- Build artifacts are redirected to `D:/cargo-target-dir` via `.cargo/config.toml`.
- There are no automated tests in this project right now.
- When behavior changes, prefer at least `cargo check`; run `cargo run` for a quick smoke test when practical.

## Architecture

Bevy 0.18.1 Tetris using a plugin-per-concern architecture. Each gameplay area exposes a single `Plugin` that is registered in [`src/main.rs`](src/main.rs).

### State Machine

- `AppState`: `StartScreen` -> `Playing` -> `GameOver`
- `PlayState` (active only during `Playing`): `Running` | `Paused`
- Gameplay systems are gated on `PlayState::Running`; scoring and visual effects run under `AppState::Playing`

### Data Flow

- Game state lives in resources, not entities.
- `Board` stores a grid of `Option<TetrominoKind>` for game logic only; color is derived during rendering.
- `ActivePiece` tracks the current falling piece's kind, rotation, row, and col.
- `PieceBag` implements 7-bag randomization.

### Messaging

Use Bevy 0.18 `Message`s, not legacy `Event`s. The main messages are:

- `LineClearMsg`
- `HardDropMsg`
- `SoftDropMsg`
- `LevelChangedMsg`
- `PieceLockedMsg`

These are defined in `src/resources.rs` and registered by `StatsPlugin`.

### Module Responsibilities

- `src/gameplay.rs`: gravity, DAS movement, SRS rotation and wall kicks, hard/soft drop, lock delay, hold; systems are `.chain()`ed for deterministic ordering
- `src/tetromino.rs`: piece definitions, SRS cell data, kick tables, `ActivePiece`, `PieceBag`, active/ghost piece visual sync
- `src/board.rs`: `Board` grid resource, collision checks, row clearing, playfield rendering, collapse animation
- `src/input.rs`: unified `InputActions` resource populated in `PreUpdate`; keyboard and gamepad inputs are ORed together, then cleared each frame
- `src/resources.rs`: `GameStats`, `HoldPiece`, scoring, and message definitions
- `src/effects.rs`: line-clear particles and lock flash overlays
- `src/constants.rs`: grid dimensions, timing, colors, tuning values, and z-layer constants
- `src/states.rs`, `src/hud.rs`, `src/sidebar.rs`: state transitions and UI/sidebar presentation

## Working Conventions

- Keep piece locking centralized in `perform_lock()` when changing lock behavior.
- Preserve deterministic system ordering in `src/gameplay.rs`; changes there can alter feel and correctness.
- Prefer syncing long-lived render entities each frame instead of respawning them repeatedly.
- Keep gameplay tuning in `src/constants.rs` unless there is a strong reason not to.
- `WindowResolution::new` takes `u32` in this Bevy version, so window constants are cast with `as u32`.
