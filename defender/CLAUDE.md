# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`defender` is an arcade game built with Rust and Bevy 0.18. It features a wrapped horizontal world (8000 units), simple geometric rendering, and Bevy states for gameplay flow. Work in this file tree only — sibling directories are separate game projects.

## Build Commands

- `cargo fmt` — run after code edits
- `cargo check` — minimum validation before wrapping up
- `cargo clippy --all-targets -- -W clippy::all` — run when touching architecture, ECS flow, or gameplay systems
- `cargo run` — launch the game

Note: target-dir is set to `D:/cargo-target-dir` via `.cargo/config.toml`.

## Architecture

**Plugin-based organization.** Each major system is a Bevy plugin registered in `main.rs`. Extend existing plugins rather than growing `main.rs`.

**System ordering via GameplaySet** (`scheduling.rs`): Input → Movement → Collision → Camera → Sync → Post. All gameplay systems must use the appropriate set instead of ad-hoc `.after()` calls. Systems run only in `AppState::Playing`.

**Game states** (`states.rs`): StartScreen → WaveIntro (2s countdown) → Playing → PlayerDeath (1.5s) → GameOver. State transitions are managed in `ui.rs`.

**WorldPosition is authoritative.** `WorldPosition` holds the wrapped X coordinate; `Transform` is derived after camera sync in `camera.rs`. Never set Transform.x directly for gameplay entities — update WorldPosition instead.

**Seeded RNG.** All gameplay randomness uses `GameRng` (seeded SmallRng). Never use `SystemTime`-based randomness.

**Shared assets.** `GameplayAssets` (in `resources.rs`, built via `FromWorld`) caches all meshes and materials. Do not recreate shared meshes/materials per spawn.

**Change-driven UI.** Scanner dots update incrementally, not via despawn/recreate. HUD text updates on resource change detection. Maintain this pattern.

## Key Patterns

- Components are simple data containers; behavior lives in systems
- Collision uses circle-based detection with world-wrapping-aware distance (`world_distance` in `camera.rs`)
- Wave progression scales enemy counts by wave number (see `waves.rs`)
- When all humans die, remaining Landers mutate into Mutants
- Avoid mirroring ECS entity counts in resources — use queries for truth
- New gameplay features should consider: state transitions, owning plugin, GameplaySet placement, component vs resource vs query
