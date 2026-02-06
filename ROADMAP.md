# Infinite - Development Roadmap

A Vulkan-based game engine in Rust powering **Infinite** — a game where players traverse a continuous year-based timeline, playing different games within the game across any time period. Think *Like a Dragon* meets *Pokemon* meets time travel.

---

## Current State

### Implemented Systems

| System | Crate | Status |
|--------|-------|--------|
| Core types & timeline system | `infinite-core` | Done |
| Vulkan renderer + egui integration | `src/main.rs`, `infinite-render` | Done (forward renderer) |
| Physics (rapier3d, heightfield terrain) | `infinite-physics` | Done |
| Character controller (capsule, coyote time, jump buffer) | `infinite-game` | Done |
| Camera (FPS/TPS, zoom, collision avoidance) | `infinite-game` | Done |
| Input system (action-based mapping) | `infinite-game` | Done |
| Terrain generation (Perlin noise, biome coloring) | `infinite-world` | Done |
| Time of day + sky color cycle | `infinite-world` | Done |
| Year-based timeline system | `infinite-core` | Done |
| Character creator (sex, body, face, hair, skin — 100+ params) | `src/character/`, `src/ui/` | Done |
| UI screens (loading, main menu, pause, settings, character creator) | `src/ui/` | Done |
| Settings persistence (TOML) | `src/settings.rs` | Done |
| Character persistence (JSON) | `src/character/persistence.rs` | Done |
| Application state machine | `src/state.rs` | Done |
| ECS (generational entities, sparse-set, queries, systems) | `infinite-ecs` | Done |
| Audio engine (kira, music crossfade, SFX, spatial) | `infinite-audio` | Done |
| Asset loading (glTF 2.0, textures, caching, dedup) | `infinite-assets` | Done |
| NPC spawning, persistent identity, chunk lifecycle | `infinite-game/npc` | Done |
| NPC AI dialogue with PixygonServer | `infinite-game/npc` | Done |
| NPC relationship tracking & memory | `infinite-game/npc` | Done |
| PixygonServer integration client | `infinite-integration` | Done |

### Stub Crates (need implementation)

| Crate | Purpose | Dependencies Available |
|-------|---------|----------------------|
| `infinite-net` | Multiplayer networking | tokio, tokio-tungstenite |

---

## Milestone 1: Engine Core ✅

**Goal**: Fill in the stub crates so every system has a working foundation.

### 1.1 — ECS (`infinite-ecs`) ✅

- [x] `World` struct holding component storage, entity allocator, and resource map
- [x] `Entity` as a generational index (u32 index + u32 generation)
- [x] `Component` trait with type-erased `SparseSet<T>` storage (O(1) insert/remove/lookup)
- [x] `System` trait with `run(&mut World)` + closure impl — `SystemSchedule` for ordered execution
- [x] Query API: `world.query::<(&Position, &Velocity)>()` tuple-based iteration (up to 8-tuples)
- [x] Resource storage: type-map via `world.insert_resource(T)` / `world.resource::<T>()`
- [x] Entity commands: spawn, despawn, insert/get/remove components
- [x] Optional component queries via `Option<&T>`
- [x] 23 unit tests passing (entity lifecycle, queries, systems, resources)
- [ ] Parent-child hierarchy (optional `Parent` / `Children` components)
- [ ] `EntityId` (UUID) ↔ `Entity` (generational index) lookup bridge for serialization

### 1.2 — Asset Loading (`infinite-assets`) ✅

- [x] `AssetServer` with handle-based access and path deduplication
- [x] `AssetHandle<T>` typed handles with unique `AssetId`
- [x] glTF 2.0 loader → extract meshes (positions, normals, UVs, vertex colors, indices)
- [x] glTF embedded texture extraction (RGBA8 conversion)
- [x] Texture loader (PNG, JPEG → RGBA8 via `image` crate)
- [x] `MeshAsset` / `MeshPrimitive` renderer-agnostic vertex data
- [x] `TextureAsset` raw pixel data with format tag
- [x] Path deduplication (same path returns same handle)
- [x] Error types for not-found, load-failed, I/O errors
- [x] 4 unit tests passing (error handling, path resolution)
- [ ] glTF material extraction (PBR metallic/roughness)
- [ ] glTF skeleton & animation clip extraction
- [ ] Async loading (background thread with completion polling)
- [ ] Asset hot-reloading in debug builds (file watcher)
- [ ] Handle reference counting and unloading
- [ ] KTX2 texture support

### 1.3 — Audio (`infinite-audio`) ✅

- [x] Kira `AudioManager<DefaultBackend>` initialization and lifecycle
- [x] `AudioConfig` volume settings (master, music, SFX, voice) mapped to `GameSettings`
- [x] `MusicPlayer` with fade-in, fade-out, and crossfade transitions
- [x] `SfxPlayer` with one-shot, looping, and spatial playback + path caching
- [x] Spatial audio: distance-based attenuation + stereo panning from listener/emitter geometry
- [x] Runtime volume updates via `update_volumes(config)`
- [x] Per-frame cleanup of finished sound handles
- [x] 7 unit tests passing (config, spatial math)
- [ ] Time-period ambient tracks (ancient: orchestral, modern: electronic, future: synth)
- [ ] Sound priority system (limit concurrent sounds, drop lowest priority)
- [ ] Audio occlusion (muffled sounds through walls)

### 1.4 — Rendering Upgrades (`infinite-render`)

- [ ] Deferred rendering pipeline (G-Buffer: position, normal, albedo, metallic/roughness)
- [ ] Skinned mesh rendering (skeleton + bone transforms)
- [ ] Animation playback (glTF animation clips, blending)
- [ ] Shadow mapping (cascaded shadow maps for directional sun light)
- [ ] Post-processing: bloom, tone mapping, FXAA
- [ ] Debug rendering (wireframe, collider shapes, navigation meshes)
- [ ] Particle system (emitters, spawn rate, velocity, lifetime, texture atlas)
- [ ] `MeshAsset` → `infinite-render::Mesh` conversion (integration layer)

---

## Milestone 2: World & Exploration

**Goal**: The player can walk through a streaming world, enter buildings, climb ladders, flip switches, and travel between time periods.

### 2.1 — Chunk Streaming ✅

- [x] Chunk grid system (define chunk size, e.g., 64×64 meters)
- [x] Load/unload chunks based on player distance (ring buffer pattern)
- [x] Per-chunk terrain mesh + collider generation
- [x] Chunk data format (terrain heightmap, placed objects, NPC spawn points)
- [ ] Background thread loading with `tokio::spawn_blocking`
- [ ] LOD terrain for distant chunks (reduced subdivision)

### 2.2 — Time Travel Transitions ✅

- [x] Per-chunk time-period variants (terrain changes based on year)
- [x] Transition trigger zones (time portals, story events, items)
- [x] Visual transition effect (tinted fade, color shift, particle burst)
- [ ] Audio crossfade between time-period ambient tracks
- [ ] Gameplay differences per time period (different buildings, NPCs, terrain features)

### 2.3 — World Interactions

Interactable objects the player can engage with via the `Interact` input action:

- [x] **Interaction system**: Raycast from camera, highlight nearest interactable, prompt UI
- [x] **Doors**: Open/close animation, locked/unlocked state, key requirements
- [x] **Levers/Switches**: Toggle state, trigger linked events (open gate, activate bridge)
- [x] **Ladders**: Enter climb mode, vertical movement, dismount at top/bottom
- [x] **Buttons**: Single-press triggers (elevators, traps, puzzles)
- [ ] **Pickups**: Items on the ground, collect into inventory on interact
- [x] **Signs/Readables**: Display text overlay when examined
- [x] **NPCs**: Initiate dialogue on interact
- [x] **Containers**: Chests, crates — open to reveal loot
- [ ] **Sit points**: Benches, chairs — play sit animation, optional rest mechanic
- [ ] **Vehicles**: Mount/dismount (ties into racing sub-game)
- [ ] **Arcade machines**: Enter sub-game on interact

### 2.4 — Save/Load System ✅

- [x] Local save serialization (serde → JSON or bincode)
- [x] Save slot UI (create, load, delete, auto-save indicator)
- [x] Save captures: player position, active year, inventory, quest state, monster party, time-of-day
- [x] Auto-save on time-period transition, entering buildings, and timed interval
- [x] NPC relationship persistence
- [ ] Cloud sync via PixygonServer `/api/v1/savedata` (merge local + remote by timestamp)

---

## Milestone 3: NPC Foundation & AI Dialogue ✅

**Goal**: NPCs exist in the world with persistent identity, AI-powered conversations, and relationship tracking.

### 3.1 — NPC Foundation ✅

- [x] `NpcData` struct: ID, name, species, faction, schedule, home position, role
- [x] NPC spawning per chunk (spawn point data in chunk definition)
- [x] Persistent NPC identity (`persistent_key` from chunk coords + spawn index)
- [x] NPC despawn when chunk unloads, persist state changes
- [ ] NPC visual representation (animated mesh, nametag, faction indicator)

### 3.2 — PixygonServer Integration ✅

Integration client for server communication (non-blocking, async via channels):

- [x] `IntegrationClient` facade with tokio runtime
- [x] `PendingRequest<T>` pattern for non-blocking async operations
- [x] JWT authentication flow (login, token storage, refresh)
- [x] Character API (list, get, create, update characters)
- [x] AI Chat API (`POST /v1/ai/chat` — no auth required)
- [x] Error handling (Network, AuthFailed, ServerError, Offline, Timeout)
- [ ] Connection state monitoring and auto-reconnect
- [ ] Request retry with exponential backoff

### 3.3 — AI-Powered Dialogue ✅

- [x] `AiDialogueManager` for conversation state management
- [x] `GameContext` injection (year, time, weather, GOAP state, location, relationship)
- [x] AI dialogue UI with scrollable message history
- [x] Quick response buttons + text input field
- [x] "Thinking..." indicator while waiting for AI response
- [x] Conversation history per NPC (keyed by `persistent_key`)
- [x] Offline fallback to static dialogue trees
- [ ] Response parsing: extract mood changes, offered items/quests
- [ ] Typing animation effect
- [ ] Character portrait display

### 3.4 — NPC Relationships ✅

- [x] `NpcRelationship` struct: affection (0-100), times_spoken, message history
- [x] `RelationshipTier` enum: Stranger → Acquaintance → Friend → CloseFriend → Trusted → Bonded
- [x] Affection gains per conversation (+2 base, +1 per message, cap +5)
- [x] Message condensation (summarize when >30 messages)
- [x] `RelationshipManager` with save/load support
- [x] Relationship context injected into AI prompts
- [ ] Relationship-gated dialogue options
- [ ] Gift giving system (increase affection)
- [ ] Negative affection events (insults, theft, combat)

### 3.5 — NPC Character Generation ✅

- [x] Era-based archetype mapping (medieval/modern/future based on year)
- [x] `generate_system_prompt()` creates personality from role + era
- [x] `NpcGenerator` for lazy character creation on first interaction
- [x] Character cache with Pending/Ready/Failed states
- [ ] Character visual generation (appearance from server data)
- [ ] Voice style selection based on archetype

---

## Milestone 4: GOAP AI & Enemy NPCs

**Goal**: NPCs have believable autonomous behavior, and hostile NPCs provide combat encounters.

### 4.1 — GOAP (Goal Oriented Action Planning)

Behavior system for NPC autonomy:

- [ ] **World State**: Set of boolean/numeric facts (e.g., `is_hungry`, `has_weapon`, `distance_to_player`)
- [ ] **Goals**: Desired world state changes with priority (e.g., `eat_food`, `patrol_area`, `attack_enemy`)
- [ ] **Actions**: Preconditions + effects + cost (e.g., `go_to_food` requires `knows_food_location`, produces `at_food_location`, cost 2)
- [ ] **Planner**: A* search over action space to find cheapest plan from current state to goal
- [ ] **Plan execution**: Step through action list, re-plan on failure or world state change
- [ ] **Sensor system**: Update world state facts (sight range, hearing, health checks)

**Behavior examples:**
- Shopkeeper: `open_shop` → `wait_for_customer` → `sell_item` → `close_shop` → `go_home` → `sleep`
- Guard: `patrol` → (sees enemy) → `alert_allies` → `engage_combat` → `return_to_post`
- Villager: `wake_up` → `eat` → `go_to_work` → `work` → `eat` → `socialize` → `go_home` → `sleep`

### 4.2 — Enemy NPCs

*Requires: GOAP system, Combat system (Milestone 5)*

- [ ] Enemy NPC type with aggro radius, patrol path, combat stats
- [ ] GOAP goals for enemies: `patrol`, `chase_player`, `attack`, `flee_when_low_hp`, `call_reinforcements`
- [ ] Aggro/de-aggro based on distance and line-of-sight
- [ ] Enemy spawning rules (year-dependent, time-of-day-dependent)
- [ ] Death/defeat handling: drop loot, grant XP, respawn timer

---

## Milestone 5: Player Stats & Combat

**Goal**: Real-time combat with RPG stats, levelling, and damage calculation.

### 5.1 — Player Stats

- [ ] `CharacterStats` struct: HP, Attack, Defense, Speed, Special Attack, Special Defense
- [ ] Base stats from archetype selection (Chronomancer, TemporalHunter, Vanguard, Technomage, ParadoxWeaver)
- [ ] Level system (1–100) with XP curve
- [ ] Stat growth per level (base + per-level scaling per archetype)
- [ ] Derived stats: max HP, damage reduction %, crit chance, dodge chance

### 5.2 — Combat System

- [ ] **Attack input**: Melee (light/heavy), ranged (if weapon equipped)
- [ ] **Hit detection**: Physics-based (rapier3d collider overlap or raycast)
- [ ] **Damage formula**: `(attacker.attack * move.power / defender.defense) * modifiers`
- [ ] **Damage modifiers**: Critical hits (1.5×), type advantage, status effects, equipment bonuses
- [ ] **Knockback**: Apply impulse on hit based on attack power
- [ ] **I-frames**: Brief invincibility after taking damage
- [ ] **Health bar UI**: Player HUD + floating enemy health bars
- [ ] **Death/respawn**: Player respawns at last save point, lose some currency

### 5.3 — Abilities & Moves

- [ ] Ability loadout (equip up to 4 active abilities)
- [ ] Ability types: melee, ranged, AoE, buff, debuff, heal
- [ ] Cooldown system per ability
- [ ] Mana/energy resource for ability usage
- [ ] Time-period themed abilities (ancient: melee/magic, modern: tech/gadgets, future: energy/psychic)

### 5.4 — Levelling & Progression

- [ ] XP gain from combat, quests, exploration, sub-game completion
- [ ] Level-up notification UI with stat increase display
- [ ] Skill points allocated on level-up (choose stat focus)
- [ ] Sync XP to PixygonServer via `gameXp` field on user profile
- [ ] Milestone rewards at key levels (new abilities, cosmetics, areas unlocked)

---

## Milestone 6: Inventory & Equipment

**Goal**: Players collect and equip items that affect their stats.

### 6.1 — Inventory System

- [ ] `Inventory` struct: list of `(ItemId, quantity)` with max capacity
- [ ] Item data from PixygonServer CharacterItem catalog
- [ ] Categories: weapon, armor, consumable, key item, material, collectible
- [ ] Inventory UI: grid view, category tabs, item detail panel, sort/filter
- [ ] Drag-and-drop or select-to-equip
- [ ] Stack limits per item type
- [ ] Drop/destroy items

### 6.2 — Equipment

- [ ] Equipment slots: head, chest, legs, feet, main hand, off hand, accessory (×2), ring (×2)
- [ ] Equipment stat bonuses applied to `CharacterStats`
- [ ] Visual change on character when equipping armor/weapons
- [ ] Equipment comparison tooltip (show stat diff)
- [ ] Set bonuses (wearing full matching set grants extra effect)
- [ ] Time-period equipment (some items only work in certain time periods)

### 6.3 — PixygonServer Item Sync

- [ ] Fetch item catalog: `GET /api/v1/character-items/project/:projectId`
- [ ] Sync player inventory: character inventory endpoints
- [ ] Equipment sync: equip/unequip endpoints
- [ ] Currency stored in save data sections

---

## Milestone 7: Economy & Crafting

**Goal**: Players can buy, sell, and craft items.

*Requires: Inventory System (Milestone 6), NPC Foundation (Milestone 3)*

### 7.1 — Shops & Trading

- [ ] Shop NPC interaction: opens shop UI
- [ ] Shop inventory (per-NPC item list with prices)
- [ ] Buy/sell with currency (coins, gems)
- [ ] Price modifiers (reputation discount, time-period economy)
- [ ] Sell-back at reduced price
- [ ] Special merchants (rare items, time-period exclusive stock)

### 7.2 — Crafting

- [ ] Recipe system: input materials → output item
- [ ] Crafting stations in the world (forge, alchemy table, workbench)
- [ ] Recipe discovery (find recipe scrolls, learn from NPCs)
- [ ] Crafting UI: select recipe, show required materials, craft button
- [ ] Time-period recipes (ancient: blacksmithing, modern: engineering, future: nano-fabrication)

---

## Milestone 8: Quest System

**Goal**: Structured objectives that drive gameplay progression.

*Requires: NPCs (Milestone 3), Inventory (Milestone 6), Combat (Milestone 5)*

### 8.1 — Quest Data

- [ ] `Quest` struct: ID, title, description, objectives, rewards, prerequisites
- [ ] Quest states: Available, Active, Completed, Failed
- [ ] Objective types: kill X enemies, collect X items, talk to NPC, reach location, win sub-game
- [ ] Quest prerequisites (other quests, level, items, relationship tier)
- [ ] Quest rewards: XP, items, currency, monster eggs, area unlock, relationship boost

### 8.2 — Quest Flow

- [ ] Quest giver NPCs (dialogue triggers quest acceptance)
- [ ] Quest acceptance UI (show objectives, rewards, accept/decline)
- [ ] Quest tracking and waypoint system
- [ ] Quest log UI (active, completed, available quests)
- [ ] Quest completion notification and reward screen

### 8.3 — Quest Types

- [ ] Main story quests (unlock areas, progress narrative)
- [ ] Side quests (optional content, extra rewards)
- [ ] Repeatable quests (daily/weekly challenges)
- [ ] Sub-game challenges (complete racing track, catch monster, beat arcade score)
- [ ] Time-period specific quests (only available in certain eras)

---

## Milestone 9: Sub-Game Framework

**Goal**: Shared infrastructure so all sub-games (monsters, racing, arcade) plug into the same system.

### Philosophy

Infinite follows the *Like a Dragon* (Yakuza) approach: the open world is a hub connecting multiple deep, fully-featured sub-games. Each sub-game is a complete experience — not a minigame. All sub-games are designed in parallel with equal priority.

### 9.1 — Sub-Game Architecture

- [ ] `SubGame` trait: `fn enter()`, `fn update()`, `fn render()`, `fn exit()`, `fn is_complete() -> bool`
- [ ] Sub-game state machine (independent from main game state)
- [ ] Transition system: fade out world → load sub-game → fade in sub-game → (play) → fade out → return to world
- [ ] Sub-game saves as sections in save data (`savedata.section("monster_collection")`, `savedata.section("racing")`, etc.)
- [ ] Shared reward pipeline: sub-game completion → XP, items, currency, monster eggs fed back to main game
- [ ] Sub-game leaderboards via PixygonServer `/api/v1/highscores`

### 9.2 — World Entry Points

- [ ] In-world triggers for entering sub-games (NPCs, locations, items, arcade machines)
- [ ] Sub-game availability varies by time period (ancient: jousting arena, modern: racing track, future: VR arcade)
- [ ] Visual indicators on map for sub-game locations
- [ ] Sub-game progress tracked in journal/menu

---

## Milestone 10: Monster Collection

**Goal**: Full creature-collection system powered by PixygonServer's monster backend. Catch, train, evolve, and battle monsters.

*Requires: Sub-Game Framework (Milestone 9), Inventory (Milestone 6)*

### 10.1 — Monster Data Integration

- [ ] Fetch species catalog: `GET /api/v1/monsters/species?projectId=...`
- [ ] Fetch move database: `GET /api/v1/monsters/moves`
- [ ] Fetch player's monsters: `GET /api/v1/monsters/:userId`
- [ ] Fetch player's eggs: `GET /api/v1/monsters/eggs/:userId`
- [ ] Cache species/move data locally, refresh periodically

### 10.2 — Wild Encounters

- [ ] Wild monster spawn zones per chunk (species, level range, rarity, time-period specific)
- [ ] Encounter trigger: walk through tall grass, cave entry, time-of-day events
- [ ] Wild monster 3D model display (use species mesh or placeholder with species colors)
- [ ] Catch mechanic: weaken in battle → throw capture item → catch rate calculation
- [ ] Shiny variant chance (visual indicator + boosted stats)

### 10.3 — Turn-Based Battle System

- [ ] Battle state machine: `ChooseAction` → `ExecuteTurn` → `ApplyEffects` → `CheckFaint` → loop
- [ ] Actions: Fight (choose move), Item (use battle item), Switch (swap party member), Run (flee wild battles)
- [ ] Move execution: type effectiveness (18×18 type chart), STAB bonus, critical hits, accuracy check
- [ ] Damage formula aligned with PixygonServer's stat model (HP, Atk, Def, SpAtk, SpDef, Speed)
- [ ] Speed determines turn order
- [ ] Status conditions: burn (damage over time, halved attack), freeze (can't act), paralysis (speed cut, chance to skip), poison (damage over time), sleep (can't act, wake after turns)
- [ ] Stat stages (-6 to +6 multiplier for each stat)
- [ ] Multi-hit moves, priority moves (-7 to +7)
- [ ] Battle UI: move selection, HP bars, status icons, type effectiveness indicator, catch rate display

### 10.4 — Monster Party & Management

- [ ] Party of up to 6 monsters (synced via `PUT /api/v1/monsters/:userId/party`)
- [ ] Party management UI: reorder, view stats, view moves
- [ ] Monster detail screen: stats (IVs/EVs visible), moves, ability, nature, friendship
- [ ] Nickname editing (`PATCH /api/v1/monsters/:userId/:monsterId`)
- [ ] Healing at rest points or via items (`POST /api/v1/monsters/:userId/:monsterId/heal`)

### 10.5 — Training & Evolution

- [ ] XP gain from battles (`POST /api/v1/monsters/:userId/:monsterId/experience`)
- [ ] EV gain from specific defeated species (`POST /api/v1/monsters/:userId/:monsterId/evs`)
- [ ] Level-up move learning (`POST /api/v1/monsters/:userId/:monsterId/moves`)
- [ ] Evolution triggers (level threshold, item use, friendship, time-period conditions)
- [ ] Evolution animation sequence
- [ ] Held items (stat boosts, evolution stones, battle effects)

### 10.6 — Egg System

- [ ] Eggs received as quest rewards or found in the world
- [ ] Egg hatching via gameplay activity points (`POST /api/v1/monsters/eggs/:userId/:eggId/points`)
- [ ] Activity points from: walking distance, battles won, quests completed, sub-game scores
- [ ] Hatch animation and reveal (`POST /api/v1/monsters/eggs/:userId/:eggId/hatch`)
- [ ] Hatched monster inherits predetermined IVs, nature, ability from egg data

### 10.7 — NPC Trainer Battles

- [ ] NPC trainers with predefined monster teams
- [ ] Challenge triggers: line-of-sight, talk to NPC, story events
- [ ] Trainer AI: move selection based on type advantage and HP thresholds
- [ ] Rewards: currency, items, XP, progression flags

---

## Milestone 11: Racing

**Goal**: Vehicle-based racing as a full sub-game with time-period themed tracks, vehicle customization, and leaderboards.

*Requires: Sub-Game Framework (Milestone 9)*

### 11.1 — Vehicle Physics

- [ ] Vehicle rigid body (rapier3d or custom arcade physics)
- [ ] Suspension simulation (spring-damper per wheel)
- [ ] Steering, acceleration, braking, drifting
- [ ] Surface friction variation (road, dirt, ice, hover-track)
- [ ] Collision with barriers and other vehicles
- [ ] Speed boost pads and jump ramps

### 11.2 — Tracks & Courses

- [ ] Track definition format (spline path, width, surface type, decoration points)
- [ ] Checkpoint system (must pass all checkpoints, lap counting)
- [ ] Time-period themed tracks:
  - Ancient: horse/chariot race through countryside
  - Modern: street racing through city, dirt rally
  - Future: hover-vehicle on anti-gravity tracks
- [ ] Track hazards: obstacles, moving barriers, weather effects
- [ ] Shortcuts and alternate routes

### 11.3 — Race Modes

- [ ] Time trial (solo, beat your best time)
- [ ] Circuit race (multiple laps against AI opponents)
- [ ] Sprint race (point A to point B)
- [ ] Elimination (last place removed each lap)
- [ ] Results screen with position, time, rewards

### 11.4 — Vehicle Customization

- [ ] Vehicle selection (time-period appropriate vehicles)
- [ ] Stat tuning: top speed, acceleration, handling, drift
- [ ] Visual customization: paint, decals
- [ ] Upgrades purchased with currency or won as race rewards

### 11.5 — Leaderboards

- [ ] Submit race times: `POST /api/v1/highscores` (scoreType: time, gameId: track ID)
- [ ] Per-track leaderboard display
- [ ] Ghost replay of leaderboard runs (record input sequence)

---

## Milestone 12: Arcade Machines

**Goal**: In-world playable retro games on arcade cabinets. Walk up, press interact, play a full game.

*Requires: Sub-Game Framework (Milestone 9)*

### 12.1 — Arcade Machine World Object

- [ ] 3D arcade cabinet model placed in the world (time-period appropriate styling)
- [ ] Interact prompt when player is near
- [ ] Camera transition: zoom into screen → full-screen sub-game
- [ ] Exit via pause menu or game-over → camera pulls back to world

### 12.2 — Arcade Game Framework

- [ ] Shared arcade renderer (pixel-art style, low-res render target scaled up)
- [ ] Arcade input mapping (simplified: D-pad + 2 buttons)
- [ ] Score tracking and game-over flow
- [ ] High score entry (initials) → submit to PixygonServer leaderboards

### 12.3 — Example Arcade Games

Each time period has its own arcade lineup:

- **Ancient periods**: Simple games styled as historical pastimes
  - Tile-matching puzzle
  - Jousting timing game
- **Modern periods**: Classic arcade homages
  - Space shooter (vertical scrolling)
  - Racing top-down
  - Beat-em-up side-scroller
- **Future periods**: Abstract/experimental
  - Rhythm game
  - Procedural roguelike
  - Hacking puzzle

### 12.4 — Rewards

- [ ] High scores earn tickets/tokens (currency for prizes)
- [ ] Arcade-exclusive cosmetic items
- [ ] Monster eggs as rare prizes for high scores
- [ ] Achievements for beating score thresholds

---

## Milestone 13: Menus & HUD

**Goal**: Complete in-game menu system, HUD, map, and journal.

### 13.1 — In-Game HUD

- [ ] Health bar, mana/energy bar
- [ ] Minimap (chunk-based, shows nearby NPCs, interactables, sub-game locations)
- [ ] Active quest tracker (objective text + waypoint)
- [ ] Quick-use item slots (consumables, capture items)
- [ ] Year indicator + time-of-day display
- [ ] Interaction prompt (context-sensitive: "Talk", "Open", "Climb", "Play")
- [ ] Notification feed (XP gained, item received, quest update)

### 13.2 — Pause/Game Menu

- [ ] **Inventory tab**: Full inventory management
- [ ] **Equipment tab**: Equip/unequip gear with stat preview
- [ ] **Monster tab**: Party management, monster details, egg progress
- [ ] **Map tab**: Full world map with discovered locations, quest markers, sub-game locations
- [ ] **Journal tab**: Active/completed quests, lore entries, NPC relationship log
- [ ] **Stats tab**: Player stats, play time, completion percentage, sub-game records
- [ ] **Settings tab**: Existing settings menu (video, audio, gameplay)
- [ ] **Save/Load tab**: Manual save, load save slot, cloud sync status

### 13.3 — UI Polish

- [ ] Consistent art style across all menus (time-period themed color palettes)
- [ ] Transition animations between screens
- [ ] Controller/gamepad support for all menus
- [ ] Accessibility: text scaling, colorblind mode, control remapping
- [ ] Tooltip system for items, stats, abilities

---

## Milestone 14: Multiplayer

**Goal**: Multiplayer-aware architecture, single-player implementation first. Networking built on PixygonServer WebSocket infrastructure.

*Requires: All gameplay systems (Milestones 1-12)*

### 14.1 — Network Layer (`infinite-net`)

- [ ] WebSocket client connecting to PixygonServer
- [ ] JWT authentication flow (login → token → connect)
- [ ] Message protocol: binary serialization (bincode or MessagePack)
- [ ] Connection state machine: connecting → authenticating → connected → disconnected
- [ ] Reconnection with exponential backoff
- [ ] Heartbeat / keepalive

### 14.2 — State Synchronization

- [ ] Entity ownership model (each player owns their character + monsters)
- [ ] Position/rotation sync (send at 20Hz, interpolate on receiver)
- [ ] Client-side prediction for local player movement
- [ ] Server reconciliation on mismatch
- [ ] Interest management: only sync entities within player's region
- [ ] Delta compression (only send changed fields)

### 14.3 — Shared World Features

- [ ] See other players in the world (character mesh + nametag)
- [ ] Emotes and chat
- [ ] Cooperative monster battles (2v2 or double battles)
- [ ] Competitive monster battles (PvP with matchmaking)
- [ ] Racing multiplayer (share track, ghost racing or direct)
- [ ] Arcade leaderboard challenges (beat friend's score)
- [ ] Trading items and monsters between players

### 14.4 — Offline Support

- [ ] Offline action queue: record actions when disconnected
- [ ] Sync queue on reconnection
- [ ] Conflict resolution for inventory/save data
- [ ] All single-player content fully playable offline
- [ ] Multiplayer features gracefully degrade when offline

---

## Milestone 15: Launch Preparation

**Goal**: Performance, stability, content, and release readiness.

### 15.1 — Performance

- [ ] GPU profiling and optimization (Vulkan pipeline profiling)
- [ ] Draw call batching and instanced rendering
- [ ] Async asset streaming (no frame hitches on chunk load)
- [ ] Memory budgets per system (mesh, texture, audio, physics)
- [ ] Level-of-detail system for distant objects
- [ ] Occlusion culling

### 15.2 — Testing

- [ ] Unit tests for all game systems (combat math, GOAP planner, inventory logic)
- [ ] Integration tests for PixygonServer API calls
- [ ] Automated gameplay tests (bot plays through key paths)
- [ ] Crash reporting and telemetry
- [ ] Cross-platform testing: Linux, Windows, macOS

### 15.3 — Content Pipeline

- [ ] Asset import tooling (batch convert models, textures)
- [ ] World editor or level data format for designers
- [ ] Monster species/move data editor (or admin panel via Pixygon.io)
- [ ] Item catalog management via PixygonServer admin API
- [ ] Quest authoring format (TOML/JSON quest definitions)

### 15.4 — Release

- [ ] Launcher / auto-updater
- [ ] First-run tutorial / onboarding sequence
- [ ] Settings auto-detect (resolution, quality based on GPU)
- [ ] Beta test program (closed → open)
- [ ] CI/CD pipeline: build → test → package → distribute

---

## PixygonServer Integration Summary

| Game System | Server Endpoints | Sync Strategy | Milestone |
|-------------|-----------------|---------------|-----------|
| **Auth** | JWT login | On launch, refresh periodically | 3 ✅ |
| **AI Dialogue** | `/v1/ai/chat` | Stream during conversation | 3 ✅ |
| **Characters (NPCs)** | `/v1/characters/*` | Fetch/create on first dialogue | 3 ✅ |
| **Save Data** | `/api/v1/savedata/:projectId/:userId` | Auto-save to local + cloud sync | 2 |
| **Items** | `/api/v1/character-items/*` | Cache catalog, sync inventory | 6 |
| **Monsters** | `/api/v1/monsters/*` | Fetch on load, sync after battles | 10 |
| **Monster Species** | `/api/v1/monsters/species` | Cache locally, refresh on start | 10 |
| **Monster Moves** | `/api/v1/monsters/moves` | Cache locally, refresh on start | 10 |
| **Eggs** | `/api/v1/monsters/eggs/*` | Sync points on activity | 10 |
| **Leaderboards** | `/api/v1/highscores/*` | Submit on sub-game completion | 9-12 |
| **User Profile** | User endpoints | Sync XP, level, play time | 5 |

---

## Technical Decisions

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Vulkan Bindings | vulkano (safe) + ash (perf paths) | Safety with escape hatch |
| ECS | Custom (Bevy-inspired) | Full control, no framework lock-in |
| Physics | rapier3d | Pure Rust, well-maintained |
| Audio | kira | Game-focused, Rust native |
| Networking | PixygonServer WebSocket | Leverage existing auth/infrastructure |
| UI | egui | Already integrated, immediate mode |
| Assets | glTF 2.0 + KTX2 | Industry standard |
| NPC AI | GOAP | Flexible, emergent behavior without giant state machines |
| NPC Dialogue | PixygonServer AI Chat | Rich personality, AI-driven conversations |
| Serialization | serde (JSON for saves, bincode for network) | JSON for debuggability, bincode for bandwidth |
| Multiplayer | Multiplayer-aware, single-player first | Ship playable game sooner, add MP incrementally |

---

## Dependency Graph

```
Milestone 1 (Engine Core) ✅
    │
    ▼
Milestone 2 (World & Exploration)
    │
    ▼
Milestone 3 (NPC Foundation & AI Dialogue) ✅
    │
    ├──────────────────────────────────────┐
    ▼                                      ▼
Milestone 4 (GOAP & Enemies)          Milestone 5 (Stats & Combat)
    │                                      │
    │                                      ▼
    │                                 Milestone 6 (Inventory & Equipment)
    │                                      │
    │                                      ▼
    │                                 Milestone 7 (Economy & Crafting)
    │                                      │
    └──────────────────────────────────────┤
                                           ▼
                                      Milestone 8 (Quest System)
                                           │
                                           ▼
                                      Milestone 9 (Sub-Game Framework)
                                           │
                   ┌───────────────────────┼───────────────────────┐
                   ▼                       ▼                       ▼
              Milestone 10            Milestone 11            Milestone 12
              (Monsters)              (Racing)                (Arcade)
                   │                       │                       │
                   └───────────────────────┼───────────────────────┘
                                           ▼
                                      Milestone 13 (Menus & HUD)
                                           │
                                           ▼
                                      Milestone 14 (Multiplayer)
                                           │
                                           ▼
                                      Milestone 15 (Launch)
```

**Key dependency notes:**
- Milestones 10, 11, 12 (Monster Collection, Racing, Arcade) can be developed **in parallel** after the Sub-Game Framework
- Combat (M5) must come before Inventory (M6) so equipment stats can apply to combat
- Inventory (M6) must come before Economy (M7) for shops to work
- Quest System (M8) needs NPCs, Combat, and Inventory for objectives and rewards
- GOAP (M4) and Combat (M5) can be developed in parallel after NPC Foundation
