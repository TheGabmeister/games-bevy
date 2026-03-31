# SPEC

## Project Goal

Recreate the core gameplay of the 1986 NES **The Legend of Zelda** in **Bevy 0.18.1** as a **2D-only** game using **primitive shapes and flat colors only**.

This project should prioritize:

- Mechanical faithfulness over visual fidelity
- Learning-oriented recreation of core systems over exact historical accuracy
- Idiomatic Bevy architecture with clear plugins, states, resources, components, and events
- Small, testable implementation steps
- Data-driven content where practical so maps, enemies, items, and encounters are not hardcoded into one-off systems

## Visual And Presentation Constraints

- No spritesheets
- No textures
- No tilemap plugin requirement
- No pre-rendered art
- Primitive visuals only: rectangles, circles, lines, polygons, text, and simple color coding
- Temporary readability labels are allowed: a short text label may be layered on or near a primitive shape when needed to clarify what a prototype object represents
- UI may use Bevy UI text and simple panels
- Audio is allowed, but should be simple and lightweight if added early

## Scope Target

The target is a faithful gameplay remake of the original NES experience:

- Title flow
- Overworld exploration
- Screen-by-screen traversal
- Secret caves and overworld interactions
- Full dungeon loop
- Core enemy roster and bosses
- Core item progression
- Health, rupees, bombs, keys, heart containers, Triforce pieces
- Final Ganon encounter and ending flow
- Save/load flow suitable for a modern local build

To keep delivery practical, development should be staged:

- **Phase A:** fully playable first quest from start to ending
- **Phase B:** polish, balancing, presentation, and missing edge cases
- **Phase C:** optional second quest support

This remake is intended for learning purposes. The goal is to recreate the game's mechanics and progression structure in a recognizable, faithful way without requiring pixel-perfect rendering, exact NES timing, or exact internal numbers.

## Design Pillars

- The player should always understand what is solid, harmful, interactive, collectible, or secret from shape language and color.
- During early prototyping, shape-plus-label is acceptable for ambiguous entities, but the long-term goal should still be readability from shape, color, motion, and placement alone.
- The overworld and dungeons should remain screen-based rather than freely scrolling.
- Systems should be deterministic and data-driven enough to support many rooms without duplicating logic.
- Combat should feel readable and consistent, with explicit hitboxes, damage windows, invulnerability windows, and knockback.
- State transitions should be explicit and owned by app states rather than ad hoc flags.

## Idiomatic Bevy Architecture

### App States

Extend `AppState` into a fuller game flow:

- `Boot`
- `Title`
- `FileSelect`
- `PlayingOverworld`
- `PlayingDungeon`
- `RoomTransition`
- `PausedInventory`
- `PlayerDeath`
- `GameOver`
- `Ending`

If preferred, keep a top-level `Playing` state and add nested substates/resources for mode-specific flow, but room transitions and pause flow should still be explicit.

The remake should use a dedicated `PausedInventory` state for the original game's subscreen behavior: opening the inventory pauses gameplay, displays inventory/status information, and allows item selection before returning to play.

### Plugin Layout

Keep `src/main.rs` thin and register focused plugins. Recommended near-term plugin set:

- `InputPlugin`
- `GameStatePlugin`
- `WorldPlugin`
- `RoomPlugin`
- `PlayerPlugin`
- `CombatPlugin`
- `EnemyPlugin`
- `ItemPlugin`
- `InteractionPlugin`
- `DungeonPlugin`
- `CameraPlugin`
- `UiPlugin`
- `AudioPlugin`
- `SavePlugin`
- `DebugPlugin` for optional developer overlays and cheats

These do not all need to exist immediately, but the spec should guide growth in that direction.

### World-Space And Rendering Conventions

The project should define one canonical logical play-space in `src/constants.rs` and use it everywhere:

- Logical room width and height
- Reserved HUD area, if any
- World origin convention for room-local placement
- Cardinal door anchor positions for north, south, east, and west exits
- Player entry spawn offsets per doorway and staircase
- Collision/grid unit size for room authoring
- Shared Z-layer conventions for floor, walls, entities, pickups, projectiles, UI, and debug overlays

Overworld rooms and dungeon rooms should use the same placement conventions unless a strong gameplay reason requires a special-case layout.

### Core ECS/Data Model

Recommended shared components:

- `Player`
- `Enemy`
- `Projectile`
- `Pickup`
- `Door`
- `Wall`
- `PushBlock`
- `Hazard`
- `Npc`
- `ShopOffer`
- `Interactable`
- `RoomEntity`
- `OverworldEntity`
- `DungeonEntity`
- `CleanupOnRoomExit`
- `Health`
- `Damage`
- `Hitbox`
- `Hurtbox`
- `Facing`
- `MoveSpeed`
- `Knockback`
- `InvulnerabilityTimer`
- `Lifetime`

Recommended shared resources:

- `InputActions`
- `CurrentRoom`
- `CurrentLevel`
- `WorldMapState`
- `DungeonState`
- `Inventory`
- `PlayerProgress`
- `EconomyState`
- `GameFlags`
- `RoomTransitionState`
- `HudState`
- `SaveSlots`
- `AudioState`
- `DropTableState`
- `RespawnState`

Recommended events:

- `RoomChangeEvent`
- `UseItemEvent`
- `DamageEvent`
- `DeathEvent`
- `PickupEvent`
- `OpenDoorEvent`
- `SpawnRoomEvent`
- `SecretFoundEvent`
- `SaveRequestedEvent`
- `RoomClearedEvent`
- `EnemyDropEvent`
- `RespawnRequestedEvent`

### Data-Driven Content

Prefer game data definitions over hardcoded room logic:

- Overworld room definitions
- Dungeon room definitions
- Enemy spawn tables
- Item definitions
- Shop definitions
- Secret triggers
- Dungeon layouts and door connections
- Boss placement
- Text/hint data

Rust should own the content schema and loading code, but world and room content should live in external `RON` files so maps, encounters, exits, and secrets are easy to edit without changing gameplay code.

Minimum room schema should include:

- Stable room id
- Room type: overworld, cave, dungeon, boss, shop, hint, transport
- Neighbor links and transition targets
- Entry spawn points and facing by entrance type
- Static geometry and collision blockers
- Door definitions and unlock conditions
- Staircase definitions and destinations
- Enemy spawn groups and wave/clear requirements
- Pickup and reward placements
- Secret triggers and reveal results
- NPCs, shop offers, and text references
- Persistence policy for room-local state

Use one unified room schema for overworld, dungeon, cave, shop, hint, boss, and transport rooms, with room-kind-specific optional fields where needed rather than separate unrelated schemas.

### Persistence And Reset Rules

The remake should classify state explicitly so room loading and save/load behavior stay deterministic.

Persistent across saves:

- Inventory, upgrades, and major progression items
- Max health and current heart container count
- Rupees, bombs, keys, and Triforce pieces
- Dungeon completion and boss defeat state
- Permanent overworld secrets and revealed entrances
- Collected unique heart containers and unique item pickups
- Purchased shop stock if the design treats it as one-time
- Completed game and second-quest unlock state

Persistent only for the current run/session unless saved:

- Current room/location
- Current health value
- Current equipped item
- Temporary dungeon position/progress that should resume on continue

Ephemeral and recreated on room load unless flagged otherwise:

- Common enemy spawns
- Temporary enemy projectiles
- Temporary pickups dropped by enemies
- Combat timers, invulnerability timers, and knockback state
- Most shutter-door and room-clear combat state

Secret reveals, opened cave entrances, bombed overworld passages, and other major traversal changes should persist once discovered.

Temporary room-clear state should not be treated as permanent progress. Cleared enemies, temporary shutter conditions, and similar combat-state flags should reset based on room reload / dungeon re-entry rules rather than persisting indefinitely.

### Death And Continue Rules

Death flow should be specified early because it touches states, room reload logic, and save behavior.

- On death, gameplay should transition into a dedicated death/continue flow rather than directly resetting the player in-place.
- Continuing after death should preserve inventory and long-term progression, but restore the player with a limited health value rather than full health.
- Overworld deaths should respawn the player at the overworld start area or other chosen canonical continue point.
- Dungeon deaths should respawn the player at the entrance of the current dungeon.
- Continue should restore the player to `3 hearts` to mirror the original game's general continue behavior.
- Room-local enemies, temporary pickups, and shutter-door combat state should reset after death according to the room's normal reload rules.
- Save-and-quit behavior should be explicit and may be modernized as long as it stays consistent and predictable.

## Feature Inventory

### 1. Core Game Flow

- Title screen
- File select
- Start game
- Continue after death
- Save game
- Ending screen
- Optional second quest unlock after first completion

### 2. Player Movement And Feel

- 4-direction movement
- Facing direction updates
- Screen-edge locking and transition triggers
- Collision with world geometry
- Simple but readable acceleration or immediate movement response
- Damage knockback
- Temporary invulnerability after taking damage
- Death state and respawn flow

### 3. Camera And Room Structure

- Fixed single-screen rooms
- Discrete screen transitions between neighboring rooms
- Overworld room graph
- Dungeon room graph
- Camera snap or short transition motion
- Room spawn and cleanup lifecycle

### 4. World Geometry And Traversal

- Solid walls
- Water
- Trees/rocks/obstacles as primitive-shape blockers
- Stairs and cave entrances
- Dungeon doors
- Locked doors
- Shutter doors
- Bombable walls
- Burnable bushes
- Push-block secrets
- Raft docks
- Ladder-crossable gaps or water strips

### 5. Combat Systems

- Melee attack timing
- Sword projectile at full health
- Enemy contact damage
- Enemy projectiles
- Enemy drop generation
- Projectile collision and despawn
- Shield blocking rules
- Damage values by source
- Enemy stun/knockback where appropriate
- Boss-specific damage rules where needed

### 6. Player Items And Equipment

Required progression items and equipment:

- Wooden Sword
- White Sword
- Magical Sword
- Wooden Shield
- Magical Shield
- Boomerang
- Magical Boomerang
- Bombs
- Bow
- Arrows
- Silver Arrows
- Blue Candle
- Red Candle
- Recorder
- Food
- Letter
- Potion
- Magic Rod
- Book of Magic
- Raft
- Ladder
- Magical Key
- Power Bracelet
- Blue Ring
- Red Ring

Item rules to support:

- Acquisition
- Inventory ownership
- Active item selection
- Consumable counts
- Usage gating
- World interaction hooks
- Damage/stat upgrades from equipment tiers

### 7. Health, Economy, And Progression

- Heart health
- Heart containers
- Rupees
- Bomb count
- Key count
- Dungeon keys
- Enemy-dropped pickups such as hearts, rupees, bombs, fairies, and clock/time-stop effects
- Triforce shard collection
- Max health upgrades
- Shop purchases
- Persistent progression flags

### 8. Overworld Content

- First quest overworld layout
- Starting cave
- Secret caves
- Shops
- Hint caves
- Sword caves
- Heart container caves
- Money secrets
- Armos-like encounter triggers
- Dungeon entrances
- Recorder warp destinations if included in first-quest scope

### 9. Dungeon Content

- 9 first-quest dungeons
- Room-based dungeon layout
- Enemy-cleared shutters
- Locked doors and keys
- Map item
- Compass item
- Old man hint rooms
- Item rooms
- Boss rooms
- Triforce room reward flow
- Staircases and transport rooms

### 10. Enemy Roster

Implement enough enemy variety to recreate encounter structure. Group by behavior families first, then add exact variants.

Core enemy families:

- Simple walkers
- Random movers
- Chargers
- Shielded frontal blockers
- Projectile shooters
- Teleporters
- Burrowers or intermittent hazards
- Flying erratic enemies
- Aquatic enemies
- Trap-like room hazards

Dungeon and overworld encounters should be defined by spawn data, not custom room code.

### 11. Bosses And Major Encounters

- Dungeon bosses for each first-quest dungeon
- Distinct boss patterns and weak points
- Final Ganon encounter
- Zelda rescue / ending trigger

Bosses should have dedicated state machines rather than overloaded generic enemy logic.

### 12. NPCs, Shops, And Text

- Old man / hint NPCs
- Shops with rupee costs
- One-item choice rooms
- Text prompts and hint messages
- Buy and receive item flow
- Conditional text based on progression if needed

### 13. UI

- Hearts HUD
- Rupee counter
- Key counter
- Bomb counter
- Equipped item display
- Triforce progression display
- Dungeon map/compass feedback
- Pause/inventory subscreen
- Title and file select UI
- Death/game over UI

### 14. Audio

- Title music placeholder or simple theme
- Overworld music
- Dungeon music
- Item pickup cue
- Sword swing cue
- Hit cue
- Secret discovery cue
- Boss cue
- Ending cue

Primitive audio implementation is acceptable at first; gameplay should come before sound fidelity.

### 15. Persistence

- Save slot data
- Current health and max health
- Inventory and upgrades
- Rupees, bombs, keys
- Dungeon completion state
- Collected heart containers
- Opened secrets and world progression flags
- Completed game / unlocked second quest flag

### 16. Debug And Developer Tools

- Room warp/debug load
- Damage toggle or god mode
- Inventory grant commands
- Collision overlay
- Spawn marker visualization
- FPS and state overlay

These are strongly recommended for implementation speed even if excluded from the final player-facing build.

## Non-Goals For Initial Delivery

- Sprite-authentic visual remake
- NES-perfect pixel rendering
- Multiplayer
- Procedural world generation
- Networked save sync
- Full second quest before the first quest is playable end-to-end

## Recommended Build Order

The project should be built from stable foundations upward:

1. Core app states and room lifecycle
2. World-space conventions, primitive rendering conventions, and shared collision model
3. Player movement and combat
4. Screen transitions, respawn rules, and room loading
5. Overworld traversal, secrets, and persistence rules
6. Dungeon framework
7. Inventory, item progression, and pickup/drop economy
8. Enemy families
9. Bosses
10. UI, pause, and save/load flow
11. Audio and polish
12. Second quest support

## Implementation Task List

Use this as the main backlog. Each item should produce a playable improvement and keep the game runnable.

- [x] Replace the starter scene with a real top-level game bootstrap using explicit app states.
- [x] Expand `InputActions` from the current prototype controls to Zelda-style movement, attack, pause, confirm, cancel, and item-use actions.
- [x] Define canonical room-space constants: logical room size, door anchors, entry offsets, collision unit size, and render layers.
- [x] Add a shared primitive-rendering layer for rectangles, circles, and color conventions used across world entities.
- [x] Add a camera setup tuned for screen-based 2D rooms.
- [x] Add a room lifecycle system that can spawn, despawn, and transition a single room cleanly.
- [x] Define and implement room persistence categories so unique pickups, secrets, and temporary room state reset correctly.
- [x] Introduce core gameplay components: player, health, facing, hitbox, hurtbox, damage, knockback, lifetime.
- [x] Implement player spawning, 4-direction movement, facing, and collision with static blockers.
- [x] Implement a basic overworld test room using primitive walls and doors.
- [x] Implement screen-edge transitions between adjacent overworld rooms.
- [x] Add room-boundary locking during transitions so the player cannot break screen flow.
- [x] Implement sword attacks with hit timing and short-lived attack entities.
- [x] Implement player damage, knockback, invulnerability, death, respawn, and continue flow with explicit reset rules.
- [x] Add a HUD showing hearts plus basic counters for rupees, bombs, and keys.
- [x] Create a persistent `Inventory` resource and wire equipped-item selection.
- [x] Implement pickups for rupees, hearts, bombs, keys, and heart containers.
- [x] Add short text labels over entities to visually indicate what they represent.
- [x] Define an item data table centralizing ID, label, description, color, and pickup effect for every item type.
- [x] Add enemy drop tables and temporary pickup behavior for common combat rewards.
- [x] Add a data format for overworld room definitions and spawn the overworld from data.
- [x] Define a shared room schema covering exits, spawn points, geometry, doors, secrets, encounters, rewards, and persistence flags.
- [x] Implement cave entrances, interior cave rooms, and return-to-overworld flow.
- [x] Implement shops and hint rooms with text UI and purchase/interaction flow.
- [x] Implement secret triggers: burnable bush, bombable wall, push block, and hidden staircase reveal.
- [x] Implement dungeon room definitions, dungeon transitions, and dungeon-specific room cleanup.
- [x] Add dungeon doors: normal, locked, shutter, bombable, and staircase exits.
- [x] Implement dungeon keys, map, compass, and Triforce reward flow.
- [x] Build the pause/inventory subscreen and item-selection UI.
- [ ] Implement the boomerang and its stun/return behavior.
- [ ] Implement bombs with fuse, explosion hitbox, block destruction, and secret interactions.
- [ ] Implement the candle with room interaction rules for bush burning.
- [ ] Implement the bow and arrow economy rules.
- [ ] Implement the ladder and raft traversal rules.
- [ ] Implement the recorder and any first-quest-required warp/interaction behavior.
- [ ] Implement potion, letter, and healing interactions.
- [ ] Implement sword upgrades and full-health sword beams.
- [ ] Implement shield rules and shield upgrades.
- [ ] Implement ring upgrades and damage reduction.
- [ ] Add the first enemy family set: simple walkers, random movers, projectile shooters, and shielded enemies.
- [ ] Add encounter resolution rules, room-clear detection, and shutter-door unlocking.
- [ ] Add more enemy families needed for first-quest encounter coverage.
- [ ] Create boss framework with dedicated state machines and room-specific win conditions.
- [ ] Implement first-quest bosses and boss rewards.
- [ ] Build the full first-quest overworld layout.
- [ ] Build all first-quest dungeons with data-driven room content.
- [ ] Implement title screen, file select, save slots, and game start flow.
- [ ] Implement persistent save/load of inventory, health, room progress, and dungeon completion.
- [ ] Implement ending flow and post-completion state changes.
- [ ] Add lightweight audio cues and music routing.
- [ ] Add debug tools for warping, inventory grants, collision overlays, and state inspection.
- [ ] Playtest and tune combat timings, damage values, room transitions, economy, and progression gating.
- [ ] Add optional second-quest support after the first quest is stable and complete.

## Milestone Definition Of Done

### Milestone 1: Playable Prototype

- Player can move, attack, take damage, and transition between rooms
- At least a few overworld rooms exist
- HUD is visible
- One or two collectible items work
- Death and continue behavior are functional and deterministic

### Milestone 2: Vertical Slice

- One dungeon is fully playable
- Keys, doors, map/compass, miniboss/boss, and Triforce reward all work
- At least one shop, one secret, and one cave interaction work
- Enemy drops and room reset behavior feel coherent

### Milestone 3: First Quest Complete

- Full overworld and all first-quest dungeons implemented
- Required items and bosses implemented
- Save/load works
- Ending is reachable
- Major persistence behavior matches the intended design consistently

### Milestone 4: Polish

- Audio, balancing, UI refinement, bug fixing, and edge-case cleanup complete
- Debug tooling exists to support maintenance

## Open Decisions

These should be resolved early because they affect architecture:

- Whether second quest is required for version `1.0`
