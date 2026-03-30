# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo check          # Type-check without building
cargo build          # Debug build
cargo run            # Run the game
cargo clippy         # Lint
```

Build target dir is redirected to `D:/cargo-target-dir` via `.cargo/config.toml`.

There are no tests in this project.

## Architecture

Bevy 0.18 Tetris game using a plugin-per-concern architecture. Each module exposes a single `Plugin` struct registered in `main.rs`.

### State Machine

`AppState` (States): `StartScreen` → `Playing` → `GameOver`. `PlayState` (SubStates, active only during `Playing`): `Running` | `Paused`. Gameplay systems run gated on `PlayState::Running`; scoring/effects on `AppState::Playing`.

### Data Flow

Game state lives in resources, not entities. `Board` stores a grid of `Option<TetrominoKind>` (game logic only — color is derived at render time). `ActivePiece` tracks the falling piece's kind, rotation, row, col. `PieceBag` implements 7-bag randomization.

Inter-system communication uses Bevy 0.18 **Messages** (not legacy Events): `LineClearMsg`, `HardDropMsg`, `SoftDropMsg`, `LevelChangedMsg`, `PieceLockedMsg`. These are defined in `resources.rs` and registered in `StatsPlugin`.

### Module Responsibilities

- **gameplay.rs** — Core loop: gravity, DAS movement, rotation (SRS wall kicks), hard/soft drop, lock delay, hold. Systems are `.chain()`ed for deterministic ordering.
- **tetromino.rs** — Piece definitions (SRS cell data, kick tables), `ActivePiece`/`PieceBag` resources, visual sync for active/ghost piece sprites.
- **board.rs** — `Board` grid resource, collision checks, row clearing, playfield rendering, collapse animation.
- **input.rs** — Unified `InputActions` resource polled in `PreUpdate`. A `clear_input` system resets it, then keyboard and gamepad systems OR their inputs in.
- **resources.rs** — `GameStats`, `HoldPiece` resources, scoring logic, message definitions.
- **effects.rs** — Particle spawning on line clears, lock flash overlays. Spawn systems gated on `AppState::Playing`; animate systems run always (self-cleaning).
- **constants.rs** — All tuning values, grid dimensions, colors, timing, and z-index layer constants (`Z_BORDER` through `Z_OVERLAY_TEXT`).

### Key Patterns

- Piece locking is centralized in `perform_lock()` (called by both `handle_hard_drop` and `handle_lock_delay`).
- Rendering entities are spawned once at `Startup` and synced each frame (board cells, active piece cells, ghost cells, sidebar). State-specific UI uses `DespawnOnExit`.
- `WindowResolution::new` takes `u32` in this Bevy version — the f32 constants are cast with `as u32`.
