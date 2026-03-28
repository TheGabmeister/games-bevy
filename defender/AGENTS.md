# AGENTS.md

## Project

`defender` is a small arcade game prototype built with Rust and Bevy `0.18`.
The game uses a wrapped horizontal world, simple geometric rendering, and Bevy
states to drive the gameplay loop.

Work in this file tree only unless the user explicitly asks otherwise. This
workspace contains other game projects as siblings.

## Goals

- Keep the project easy to iterate on.
- Prefer Bevy-friendly ECS patterns over ad hoc global state.
- Preserve current gameplay unless the user asks for design changes.

## Common Commands

- `cargo fmt`
- `cargo check`
- `cargo clippy --all-targets -- -W clippy::all`
- `cargo run`

Run `cargo fmt` after code edits. At minimum, run `cargo check` before wrapping
up. Use Clippy when touching architecture, ECS flow, or gameplay systems.

## Code Layout

- `src/main.rs`: app bootstrapping, plugin registration, global schedule setup
- `src/scheduling.rs`: named gameplay system sets
- `src/states.rs`: app states
- `src/resources.rs`: global resources, seeded RNG, cached gameplay assets
- `src/spawning.rs`: entity spawn helpers
- `src/camera.rs`: wrapped-world camera and transform sync
- `src/collision.rs`: collision handling
- `src/player.rs`: player input, movement, abilities, explosions
- `src/enemies.rs`: enemy AI and enemy movement
- `src/humans.rs`: civilian behavior and rescue/fall logic
- `src/waves.rs`: wave flow and spawning
- `src/ui.rs`: HUD and state-specific UI
- `src/scanner.rs`: minimap/scanner UI
- `src/terrain.rs`: terrain generation and sampling

## Architecture Notes

- The app is organized around feature plugins. Prefer extending an existing
  plugin over growing `main.rs`.
- Gameplay update order is controlled through `GameplaySet`. Keep new gameplay
  systems in the appropriate set instead of wiring many one-off `.after(...)`
  calls.
- `WorldPosition` is the authoritative wrapped X position. `Transform` is used
  for rendered/local placement after camera sync.
- Avoid mirroring ECS entity counts in resources when a query can provide the
  truth directly.
- Use `GameRng` for gameplay randomness. Do not add new `SystemTime`-based
  randomness.
- Use `GameplayAssets` for common mesh/material handles. Do not recreate shared
  meshes/materials on every spawn unless there is a clear need.

## UI and Scanner Conventions

- Prefer change-driven updates over rebuilding UI every frame.
- The scanner currently updates existing dots incrementally. Keep that pattern
  instead of despawn/recreate loops.

## Coding Guidance

- Prefer small, focused systems and helpers over one giant system when logic is
  growing.
- Keep components simple data containers.
- If adding a new gameplay feature, think through:
  1. state transitions
  2. which plugin owns the behavior
  3. which `GameplaySet` it belongs to
  4. whether it should be a component, resource, or derived query

## Validation Expectations

- For small edits: `cargo fmt` and `cargo check`
- For architecture or gameplay refactors: also run
  `cargo clippy --all-targets -- -W clippy::all`

If you cannot run a validation step, say so clearly in your handoff.
