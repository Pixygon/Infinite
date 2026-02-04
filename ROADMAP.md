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

### Stub Crates (need implementation)

| Crate | Purpose | Dependencies Available |
|-------|---------|----------------------|
| `infinite-net` | Multiplayer networking | tokio, tokio-tungstenite |
| `infinite-integration` | PixygonServer API client | tokio, serde |

---

## Milestone 1: Engine Core

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

### 2.1 — Chunk Streaming

- [x] Chunk grid system (define chunk size, e.g., 64×64 meters)
- [x] Load/unload chunks based on player distance (ring buffer pattern)
- [x] Per-chunk terrain mesh + collider generation
- [x] Chunk data format (terrain heightmap, placed objects, NPC spawn points)
- [ ] Background thread loading with `tokio::spawn_blocking`
- [ ] LOD terrain for distant chunks (reduced subdivision)

### 2.2 — Time Travel Transitions

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
- [x] **NPCs**: Initiate dialogue on interact (see Milestone 3)
- [x] **Containers**: Chests, crates — open to reveal loot
- [ ] **Sit points**: Benches, chairs — play sit animation, optional rest mechanic
- [ ] **Vehicles**: Mount/dismount (ties into racing sub-game)
- [ ] **Arcade machines**: Enter sub-game on interact (see Milestone 9)

### 2.4 — Save/Load System

- [x] Local save serialization (serde → JSON or bincode)
- [x] Save slot UI (create, load, delete, auto-save indicator)
- [x] Save captures: player position, active year, inventory, quest state, monster party, time-of-day
- [x] Auto-save on time-period transition, entering buildings, and timed interval
- [ ] Cloud sync via PixygonServer `/api/v1/savedata` (merge local + remote by timestamp)

---

## Milestone 3: NPCs & AI

**Goal**: The world has friendly and hostile NPCs with believable behavior and conversational dialogue.

### 3.1 — NPC Foundation

- [x] `NpcData` struct: ID, name, species, faction, schedule, home position, role
- [x] NPC spawning per chunk (spawn point data in chunk definition)
- [ ] NPC visual representation (animated mesh, nametag, faction indicator)
- [x] NPC despawn when chunk unloads, persist state changes

### 3.2 — GOAP (Goal Oriented Action Planning)

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

### 3.3 — NPC Dialogue (BimboChat Integration)

Dialogue uses generated AI characters from PixygonServer's Character system:

- [ ] **Dialogue UI**: Text box with character portrait, name, typing animation
- [ ] **Character binding**: Each NPC links to a PixygonServer Character (personality, backstory, style)
- [ ] **Dialogue flow**: Player approaches → interact → send context to AI → stream response
- [ ] **Context injection**: NPC's current GOAP state, location, time of day, active year, relationship level
- [ ] **Response parsing**: Extract dialogue text, mood/expression changes, offered items/quests
- [ ] **Conversation history**: Cache recent exchanges per NPC (store in save data)
- [ ] **Offline fallback**: Pre-written dialogue trees for when AI is unavailable
- [ ] **PixygonServer endpoints used**:
  - `GET /api/v1/characters/:projectId/:userId/:characterId` — fetch NPC character data
  - AI services endpoint — generate dialogue responses with character personality

### 3.4 — Enemy NPCs

- [ ] Enemy NPC type with aggro radius, patrol path, combat stats
- [ ] GOAP goals for enemies: `patrol`, `chase_player`, `attack`, `flee_when_low_hp`, `call_reinforcements`
- [ ] Aggro/de-aggro based on distance and line-of-sight
- [ ] Enemy spawning rules (year-dependent, time-of-day-dependent)
- [ ] Death/defeat handling: drop loot, grant XP, respawn timer

---

## Milestone 4: Combat & Stats

**Goal**: Real-time combat with RPG stats, levelling, and damage calculation.

### 4.1 — Player Stats

- [ ] `CharacterStats` struct: HP, Attack, Defense, Speed, Special Attack, Special Defense
- [ ] Base stats from archetype selection (Chronomancer, TemporalHunter, Vanguard, Technomage, ParadoxWeaver)
- [ ] Level system (1–100) with XP curve
- [ ] Stat growth per level (base + per-level scaling per archetype)
- [ ] Derived stats: max HP, damage reduction %, crit chance, dodge chance

### 4.2 — Combat System

- [ ] **Attack input**: Melee (light/heavy), ranged (if weapon equipped)
- [ ] **Hit detection**: Physics-based (rapier3d collider overlap or raycast)
- [ ] **Damage formula**: `(attacker.attack * move.power / defender.defense) * modifiers`
- [ ] **Damage modifiers**: Critical hits (1.5×), type advantage, status effects, equipment bonuses
- [ ] **Knockback**: Apply impulse on hit based on attack power
- [ ] **I-frames**: Brief invincibility after taking damage
- [ ] **Health bar UI**: Player HUD + floating enemy health bars
- [ ] **Death/respawn**: Player respawns at last save point, lose some currency

### 4.3 — Abilities & Moves

- [ ] Ability loadout (equip up to 4 active abilities)
- [ ] Ability types: melee, ranged, AoE, buff, debuff, heal
- [ ] Cooldown system per ability
- [ ] Mana/energy resource for ability usage
- [ ] Time-period themed abilities (ancient: melee/magic, modern: tech/gadgets, future: energy/psychic)

### 4.4 — Levelling & Progression

- [ ] XP gain from combat, quests, exploration, sub-game completion
- [ ] Level-up notification UI with stat increase display
- [ ] Skill points allocated on level-up (choose stat focus)
- [ ] Sync XP to PixygonServer via `gameXp` field on user profile
- [ ] Milestone rewards at key levels (new abilities, cosmetics, areas unlocked)

---

## Milestone 5: Inventory & Economy

**Goal**: Players collect, equip, buy, sell, and craft items.

### 5.1 — Inventory System

- [ ] `Inventory` struct: list of `(ItemId, quantity)` with max capacity
- [ ] Item data from PixygonServer CharacterItem catalog
- [ ] Categories: weapon, armor, consumable, key item, material, collectible
- [ ] Inventory UI: grid view, category tabs, item detail panel, sort/filter
- [ ] Drag-and-drop or select-to-equip
- [ ] Stack limits per item type
- [ ] Drop/destroy items

### 5.2 — Equipment

- [ ] Equipment slots: head, chest, legs, feet, main hand, off hand, accessory (×2), ring (×2)
- [ ] Equipment stat bonuses applied to `CharacterStats`
- [ ] Visual change on character when equipping armor/weapons
- [ ] Equipment comparison tooltip (show stat diff)
- [ ] Set bonuses (wearing full matching set grants extra effect)
- [ ] Time-period equipment (some items only work in certain time periods)

### 5.3 — Shops & Trading

- [ ] Shop NPC interaction: opens shop UI
- [ ] Shop inventory (per-NPC item list with prices)
- [ ] Buy/sell with currency (coins, gems)
- [ ] Price modifiers (reputation discount, time-period economy)
- [ ] Sell-back at reduced price
- [ ] Special merchants (rare items, time-period exclusive stock)

### 5.4 — Crafting

- [ ] Recipe system: input materials → output item
- [ ] Crafting stations in the world (forge, alchemy table, workbench)
- [ ] Recipe discovery (find recipe scrolls, learn from NPCs)
- [ ] Crafting UI: select recipe, show required materials, craft button
- [ ] Time-period recipes (ancient: blacksmithing, modern: engineering, future: nano-fabrication)

### 5.5 — PixygonServer Sync

- [ ] Fetch item catalog: `GET /api/v1/character-items/project/:projectId`
- [ ] Sync player inventory: character inventory endpoints
- [ ] Equipment sync: equip/unequip endpoints
- [ ] Currency stored in save data sections

---

## Milestone 6: Sub-Game Framework

**Goal**: Shared infrastructure so all sub-games (monsters, racing, arcade) plug into the same system.

### Philosophy

Infinite follows the *Like a Dragon* (Yakuza) approach: the open world is a hub connecting multiple deep, fully-featured sub-games. Each sub-game is a complete experience — not a minigame. All sub-games are designed in parallel with equal priority.

### 6.1 — Sub-Game Architecture

- [ ] `SubGame` trait: `fn enter()`, `fn update()`, `fn render()`, `fn exit()`, `fn is_complete() -> bool`
- [ ] Sub-game state machine (independent from main game state)
- [ ] Transition system: fade out world → load sub-game → fade in sub-game → (play) → fade out → return to world
- [ ] Sub-game saves as sections in save data (`savedata.section("monster_collection")`, `savedata.section("racing")`, etc.)
- [ ] Shared reward pipeline: sub-game completion → XP, items, currency, monster eggs fed back to main game
- [ ] Sub-game leaderboards via PixygonServer `/api/v1/highscores`

### 6.2 — World Entry Points

- [ ] In-world triggers for entering sub-games (NPCs, locations, items, arcade machines)
- [ ] Sub-game availability varies by time period (ancient: jousting arena, modern: racing track, future: VR arcade)
- [ ] Visual indicators on map for sub-game locations
- [ ] Sub-game progress tracked in journal/menu

---

## Milestone 7: Monster Collection

**Goal**: Full creature-collection system powered by PixygonServer's monster backend. Catch, train, evolve, and battle monsters.

### 7.1 — Monster Data Integration

- [ ] Fetch species catalog: `GET /api/v1/monsters/species?projectId=...`
- [ ] Fetch move database: `GET /api/v1/monsters/moves`
- [ ] Fetch player's monsters: `GET /api/v1/monsters/:userId`
- [ ] Fetch player's eggs: `GET /api/v1/monsters/eggs/:userId`
- [ ] Cache species/move data locally, refresh periodically

### 7.2 — Wild Encounters

- [ ] Wild monster spawn zones per chunk (species, level range, rarity, time-period specific)
- [ ] Encounter trigger: walk through tall grass, cave entry, time-of-day events
- [ ] Wild monster 3D model display (use species mesh or placeholder with species colors)
- [ ] Catch mechanic: weaken in battle → throw capture item → catch rate calculation
- [ ] Shiny variant chance (visual indicator + boosted stats)

### 7.3 — Turn-Based Battle System

- [ ] Battle state machine: `ChooseAction` → `ExecuteTurn` → `ApplyEffects` → `CheckFaint` → loop
- [ ] Actions: Fight (choose move), Item (use battle item), Switch (swap party member), Run (flee wild battles)
- [ ] Move execution: type effectiveness (18×18 type chart), STAB bonus, critical hits, accuracy check
- [ ] Damage formula aligned with PixygonServer's stat model (HP, Atk, Def, SpAtk, SpDef, Speed)
- [ ] Speed determines turn order
- [ ] Status conditions: burn (damage over time, halved attack), freeze (can't act), paralysis (speed cut, chance to skip), poison (damage over time), sleep (can't act, wake after turns)
- [ ] Stat stages (-6 to +6 multiplier for each stat)
- [ ] Multi-hit moves, priority moves (-7 to +7)
- [ ] Battle UI: move selection, HP bars, status icons, type effectiveness indicator, catch rate display

### 7.4 — Monster Party & Management

- [ ] Party of up to 6 monsters (synced via `PUT /api/v1/monsters/:userId/party`)
- [ ] Party management UI: reorder, view stats, view moves
- [ ] Monster detail screen: stats (IVs/EVs visible), moves, ability, nature, friendship
- [ ] Nickname editing (`PATCH /api/v1/monsters/:userId/:monsterId`)
- [ ] Healing at rest points or via items (`POST /api/v1/monsters/:userId/:monsterId/heal`)

### 7.5 — Training & Evolution

- [ ] XP gain from battles (`POST /api/v1/monsters/:userId/:monsterId/experience`)
- [ ] EV gain from specific defeated species (`POST /api/v1/monsters/:userId/:monsterId/evs`)
- [ ] Level-up move learning (`POST /api/v1/monsters/:userId/:monsterId/moves`)
- [ ] Evolution triggers (level threshold, item use, friendship, time-period conditions)
- [ ] Evolution animation sequence
- [ ] Held items (stat boosts, evolution stones, battle effects)

### 7.6 — Egg System

- [ ] Eggs received as quest rewards or found in the world
- [ ] Egg hatching via gameplay activity points (`POST /api/v1/monsters/eggs/:userId/:eggId/points`)
- [ ] Activity points from: walking distance, battles won, quests completed, sub-game scores
- [ ] Hatch animation and reveal (`POST /api/v1/monsters/eggs/:userId/:eggId/hatch`)
- [ ] Hatched monster inherits predetermined IVs, nature, ability from egg data

### 7.7 — NPC Trainer Battles

- [ ] NPC trainers with predefined monster teams
- [ ] Challenge triggers: line-of-sight, talk to NPC, story events
- [ ] Trainer AI: move selection based on type advantage and HP thresholds
- [ ] Rewards: currency, items, XP, progression flags

---

## Milestone 8: Racing

**Goal**: Vehicle-based racing as a full sub-game with time-period themed tracks, vehicle customization, and leaderboards.

### 8.1 — Vehicle Physics

- [ ] Vehicle rigid body (rapier3d or custom arcade physics)
- [ ] Suspension simulation (spring-damper per wheel)
- [ ] Steering, acceleration, braking, drifting
- [ ] Surface friction variation (road, dirt, ice, hover-track)
- [ ] Collision with barriers and other vehicles
- [ ] Speed boost pads and jump ramps

### 8.2 — Tracks & Courses

- [ ] Track definition format (spline path, width, surface type, decoration points)
- [ ] Checkpoint system (must pass all checkpoints, lap counting)
- [ ] Time-period themed tracks:
  - Ancient: horse/chariot race through countryside
  - Modern: street racing through city, dirt rally
  - Future: hover-vehicle on anti-gravity tracks
- [ ] Track hazards: obstacles, moving barriers, weather effects
- [ ] Shortcuts and alternate routes

### 8.3 — Race Modes

- [ ] Time trial (solo, beat your best time)
- [ ] Circuit race (multiple laps against AI opponents)
- [ ] Sprint race (point A to point B)
- [ ] Elimination (last place removed each lap)
- [ ] Results screen with position, time, rewards

### 8.4 — Vehicle Customization

- [ ] Vehicle selection (time-period appropriate vehicles)
- [ ] Stat tuning: top speed, acceleration, handling, drift
- [ ] Visual customization: paint, decals
- [ ] Upgrades purchased with currency or won as race rewards

### 8.5 — Leaderboards

- [ ] Submit race times: `POST /api/v1/highscores` (scoreType: time, gameId: track ID)
- [ ] Per-track leaderboard display
- [ ] Ghost replay of leaderboard runs (record input sequence)

---

## Milestone 9: Arcade Machines

**Goal**: In-world playable retro games on arcade cabinets. Walk up, press interact, play a full game.

### 9.1 — Arcade Machine World Object

- [ ] 3D arcade cabinet model placed in the world (time-period appropriate styling)
- [ ] Interact prompt when player is near
- [ ] Camera transition: zoom into screen → full-screen sub-game
- [ ] Exit via pause menu or game-over → camera pulls back to world

### 9.2 — Arcade Game Framework

- [ ] Shared arcade renderer (pixel-art style, low-res render target scaled up)
- [ ] Arcade input mapping (simplified: D-pad + 2 buttons)
- [ ] Score tracking and game-over flow
- [ ] High score entry (initials) → submit to PixygonServer leaderboards

### 9.3 — Example Arcade Games

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

### 9.4 — Rewards

- [ ] High scores earn tickets/tokens (currency for prizes)
- [ ] Arcade-exclusive cosmetic items
- [ ] Monster eggs as rare prizes for high scores
- [ ] Achievements for beating score thresholds

---

## Milestone 10: Menus & Polish

**Goal**: Complete in-game menu system, HUD, map, journal, and UI polish.

### 10.1 — In-Game HUD

- [ ] Health bar, mana/energy bar
- [ ] Minimap (chunk-based, shows nearby NPCs, interactables, sub-game locations)
- [ ] Active quest tracker (objective text + waypoint)
- [ ] Quick-use item slots (consumables, capture items)
- [ ] Year indicator + time-of-day display
- [ ] Interaction prompt (context-sensitive: "Talk", "Open", "Climb", "Play")
- [ ] Notification feed (XP gained, item received, quest update)

### 10.2 — Pause/Game Menu

- [ ] **Inventory tab**: Full inventory management (see Milestone 5)
- [ ] **Equipment tab**: Equip/unequip gear with stat preview
- [ ] **Monster tab**: Party management, monster details, egg progress
- [ ] **Map tab**: Full world map with discovered locations, quest markers, sub-game locations
- [ ] **Journal tab**: Active/completed quests, lore entries, NPC relationship log
- [ ] **Stats tab**: Player stats, play time, completion percentage, sub-game records
- [ ] **Settings tab**: Existing settings menu (video, audio, gameplay)
- [ ] **Save/Load tab**: Manual save, load save slot, cloud sync status

### 10.3 — Quest System

- [ ] `Quest` struct: ID, title, description, objectives, rewards, prerequisites
- [ ] Objective types: kill X enemies, collect X items, talk to NPC, reach location, win sub-game
- [ ] Quest giver NPCs (dialogue triggers quest acceptance)
- [ ] Quest tracking and waypoint system
- [ ] Quest completion rewards: XP, items, currency, monster eggs, area unlock
- [ ] Main story quests vs side quests vs sub-game challenges

### 10.4 — UI Polish

- [ ] Consistent art style across all menus (time-period themed color palettes)
- [ ] Transition animations between screens
- [ ] Controller/gamepad support for all menus
- [ ] Accessibility: text scaling, colorblind mode, control remapping
- [ ] Tooltip system for items, stats, abilities

---

## Milestone 11: Multiplayer

**Goal**: Multiplayer-aware architecture, single-player implementation first. Networking built on PixygonServer WebSocket infrastructure.

### 11.1 — Network Layer (`infinite-net`)

- [ ] WebSocket client connecting to PixygonServer
- [ ] JWT authentication flow (login → token → connect)
- [ ] Message protocol: binary serialization (bincode or MessagePack)
- [ ] Connection state machine: connecting → authenticating → connected → disconnected
- [ ] Reconnection with exponential backoff
- [ ] Heartbeat / keepalive

### 11.2 — State Synchronization

- [ ] Entity ownership model (each player owns their character + monsters)
- [ ] Position/rotation sync (send at 20Hz, interpolate on receiver)
- [ ] Client-side prediction for local player movement
- [ ] Server reconciliation on mismatch
- [ ] Interest management: only sync entities within player's region
- [ ] Delta compression (only send changed fields)

### 11.3 — Shared World Features

- [ ] See other players in the world (character mesh + nametag)
- [ ] Emotes and chat
- [ ] Cooperative monster battles (2v2 or double battles)
- [ ] Competitive monster battles (PvP with matchmaking)
- [ ] Racing multiplayer (share track, ghost racing or direct)
- [ ] Arcade leaderboard challenges (beat friend's score)
- [ ] Trading items and monsters between players

### 11.4 — Offline Support

- [ ] Offline action queue: record actions when disconnected
- [ ] Sync queue on reconnection
- [ ] Conflict resolution for inventory/save data
- [ ] All single-player content fully playable offline
- [ ] Multiplayer features gracefully degrade when offline

---

## Milestone 12: Launch Preparation

**Goal**: Performance, stability, content, and release readiness.

### 12.1 — Performance

- [ ] GPU profiling and optimization (Vulkan pipeline profiling)
- [ ] Draw call batching and instanced rendering
- [ ] Async asset streaming (no frame hitches on chunk load)
- [ ] Memory budgets per system (mesh, texture, audio, physics)
- [ ] Level-of-detail system for distant objects
- [ ] Occlusion culling

### 12.2 — Testing

- [ ] Unit tests for all game systems (combat math, GOAP planner, inventory logic)
- [ ] Integration tests for PixygonServer API calls
- [ ] Automated gameplay tests (bot plays through key paths)
- [ ] Crash reporting and telemetry
- [ ] Cross-platform testing: Linux, Windows, macOS

### 12.3 — Content Pipeline

- [ ] Asset import tooling (batch convert models, textures)
- [ ] World editor or level data format for designers
- [ ] Monster species/move data editor (or admin panel via Pixygon.io)
- [ ] Item catalog management via PixygonServer admin API
- [ ] Quest authoring format (TOML/JSON quest definitions)

### 12.4 — Release

- [ ] Launcher / auto-updater
- [ ] First-run tutorial / onboarding sequence
- [ ] Settings auto-detect (resolution, quality based on GPU)
- [ ] Beta test program (closed → open)
- [ ] CI/CD pipeline: build → test → package → distribute

---

## PixygonServer Integration Summary

| Game System | Server Endpoints | Sync Strategy |
|-------------|-----------------|---------------|
| **Auth** | JWT login | On launch, refresh periodically |
| **Save Data** | `/api/v1/savedata/:projectId/:userId` | Auto-save to local + cloud sync on key events |
| **Monsters** | `/api/v1/monsters/*` | Fetch on load, sync after battles/catches |
| **Monster Species** | `/api/v1/monsters/species` | Cache locally, refresh on game start |
| **Monster Moves** | `/api/v1/monsters/moves` | Cache locally, refresh on game start |
| **Eggs** | `/api/v1/monsters/eggs/*` | Sync points on activity, hatch on threshold |
| **Items** | `/api/v1/character-items/*` | Cache catalog, sync inventory changes |
| **Characters (NPCs)** | `/api/v1/characters/*` | Fetch NPC personality on first dialogue |
| **Leaderboards** | `/api/v1/highscores/*` | Submit on sub-game completion, fetch for display |
| **AI Dialogue** | AI service endpoints | Stream during conversation, cache recent exchanges |
| **User Profile** | User endpoints | Sync XP, level, play time |

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
| NPC Dialogue | BimboChat character system | Rich personality, AI-driven conversations |
| Serialization | serde (JSON for saves, bincode for network) | JSON for debuggability, bincode for bandwidth |
| Multiplayer | Multiplayer-aware, single-player first | Ship playable game sooner, add MP incrementally |

---

## Dependency Graph

```
Milestone 1 (Engine Core)
    ├── ECS ──────────────┐
    ├── Assets ───────────┤
    ├── Audio ────────────┤
    └── Render ───────────┤
                          ▼
Milestone 2 (World) ─────┤
                          ▼
    ┌─────────────────────┼─────────────────────┐
    ▼                     ▼                     ▼
Milestone 3 (NPCs)   Milestone 4 (Combat)  Milestone 6 (Sub-Game Framework)
    │                     │                     │
    ▼                     ▼                 ┌───┼───┐
Milestone 5 (Inventory)  │                 ▼   ▼   ▼
    │                     │                M7  M8  M9
    ▼                     ▼              (Mon)(Race)(Arcade)
Milestone 10 (Menus & Polish) ◄────────────┘
    │
    ▼
Milestone 11 (Multiplayer)
    │
    ▼
Milestone 12 (Launch)
```

Milestones 7, 8, and 9 (Monster Collection, Racing, Arcade Machines) are designed **in parallel** — they share the sub-game framework from Milestone 6 and can be developed simultaneously with equal priority.
