# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

- **Run the game:** `cargo run`
- **Check compilation:** `cargo check`
- **Format code:** `cargo fmt`
- **Lint:** `cargo clippy --all-targets --all-features -- -D warnings`
- **Run all tests:** `cargo test`
- **Run a single test:** `cargo test <test_name>` (e.g., `cargo test award_point_returns_winner`)

Before finishing non-trivial changes, run `cargo fmt`, `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.

## Architecture

Bevy 0.18.1 Pong game — single binary, Rust 2024 edition, fixed 960x540 window.

### State Machine

`Phase` enum drives the game flow: **Menu → Playing → Winner → Menu**. State-specific entities are tagged with marker components (`MenuEntity`, `GameplayEntity`, `WinnerEntity`) and despawned on state exit.

### Plugin Structure

- **`PongAudioPlugin`** (`src/audio.rs`): Loads audio assets at startup, starts looping music once on first play, handles paddle-hit SFX via an observer on `PaddleHitEvent`.
- **`PongSystemsPlugin`** (`src/systems/mod.rs`): Wires all state transitions, camera setup, and gameplay systems.

### Gameplay System Ordering

Systems in the `Playing` state run in a strict set chain: **Movement → Collision → Scoring → Presentation**. New gameplay systems that are order-sensitive must be placed in the correct `GameplaySet`.

### Key Patterns

- Shared meshes/materials are cached in `GameplayRenderAssets` (created at startup) and cloned when spawning entities — avoid creating new assets during state transitions.
- Global state lives in resources: `MatchConfig`, `Score`, `Winner`.
- UI text updates use Bevy change detection (`is_changed()`) rather than rewriting every frame.
- Audio events are observer-driven (`PaddleHitEvent`), not polled.

## Coding Conventions

- Keep state-specific behavior scheduled with `OnEnter`/`OnExit`/`run_if(in_state(...))` — don't branch inside systems.
- Match the existing module layout; favor clarity over abstraction.
- Constants live in `src/systems/gameplay.rs` unless growth justifies a dedicated module.
- Tests go in `#[cfg(test)]` modules alongside the code they test; prefer pure logic/helper tests.
