# Infinite - Vulkan Game Engine

## Overview
A Rust/Vulkan game engine with ray tracing for a time-travel themed game where players traverse a continuous year-based timeline. The world changes organically over time — ancient ruins in one period were once a thriving castle in another.

## Project Structure
```
src/                          # Main application
├── main.rs                   # Entry point, Vulkan + egui
├── state.rs                  # ApplicationState machine
├── settings.rs               # GameSettings persistence
├── save.rs                   # Save/load system
├── character/                # Character data and creator
│   ├── mod.rs                # Character data structures
│   └── persistence.rs        # Save/load characters
└── ui/                       # egui UI screens
    ├── mod.rs                # UI module exports
    ├── loading_screen.rs     # Animated loading
    ├── main_menu.rs          # Title screen
    ├── pause_menu.rs         # In-game pause
    ├── save_load_menu.rs     # Save/load slot UI
    ├── settings_menu.rs      # Settings tabs
    └── character_creator.rs  # Character customization

crates/
├── infinite-core/            # Types, math, time system
├── infinite-ecs/             # Entity Component System
├── infinite-render/          # Vulkan renderer
├── infinite-physics/         # rapier3d physics
│   ├── lib.rs                # PhysicsWorld
│   └── character_controller.rs # Player physics
├── infinite-audio/           # kira audio
├── infinite-world/           # World/chunk management
├── infinite-net/             # WebSocket networking
├── infinite-assets/          # glTF/texture loading
├── infinite-game/            # Game logic
│   ├── lib.rs                # Module exports
│   ├── input.rs              # Input action system
│   ├── interaction.rs        # World interactions (doors, levers, etc.)
│   ├── player/               # Player controller
│   │   ├── mod.rs
│   │   ├── movement.rs       # Movement config
│   │   └── controller.rs     # Player controller
│   ├── camera/               # Camera system
│   │   ├── mod.rs
│   │   ├── config.rs         # Camera config
│   │   └── controller.rs     # Camera controller
│   └── npc/                  # NPC system
│       ├── mod.rs
│       ├── manager.rs        # NPC lifecycle
│       ├── spawn.rs          # Procedural NPC placement
│       ├── goap.rs           # Goal-oriented AI
│       ├── combat.rs         # Combat stats
│       └── dialogue.rs       # NPC dialogue
└── infinite-integration/     # PixygonServer client
```

## Building & Running
```bash
cargo build              # Build
cargo run                # Run
cargo test               # Test
cargo clippy             # Lint
```

## Tech Stack
- Rust, Vulkan (vulkano 0.35), winit 0.30
- egui 0.31 + egui_winit_vulkano
- rapier3d 0.22 (physics)
- kira 0.9 (audio)
- glam 0.29 (math)

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `infinite-core` | Core types (Transform, Color, EntityId), time system (Timeline, GameTime) |
| `infinite-ecs` | Entity Component System |
| `infinite-render` | Vulkan renderer with ray tracing |
| `infinite-physics` | Physics via rapier3d, character controller |
| `infinite-audio` | Audio via kira |
| `infinite-world` | World chunks, year-based terrain, time portals |
| `infinite-net` | Networking, prediction, server sync |
| `infinite-assets` | glTF loading, textures, caching |
| `infinite-game` | Player controller, camera, input system, interactions, NPCs |
| `infinite-integration` | PixygonServer API client |

## Application States

```rust
pub enum ApplicationState {
    Loading(LoadingPhase),              // Loading with progress
    MainMenu,                            // Title screen
    CharacterCreation,                   // Character creator
    Settings { return_to: Box<...> },    // Nested settings
    Paused,                              // Game paused
    SaveLoad { is_saving: bool },        // Save/load menu
    Playing,                             // Active gameplay
    Exiting,                             // Shutdown
}
```

## Controls

| Input | Action |
|-------|--------|
| W/A/S/D | Move |
| Space | Jump |
| Shift | Sprint |
| Mouse | Look |
| Scroll | Zoom (FPS / Third-person) |
| ESC | Pause |
| E | Interact |
| F5 | Quick Save |
| F9 | Quick Load |
| F3 | Debug overlay |

## Pixygon Agent Integration

**Project ID**: `6981e8eda259e89734bd007a`

### Git Workflow
```bash
git checkout main && git pull
git checkout -b feature/<task-description>
# ... work ...
git add -A && git commit -m "feat: description"
git push origin HEAD
```

### Development Guidelines
1. Run `cargo test` before committing
2. Run `cargo clippy` for lints
3. Follow existing code patterns
4. Document public APIs with rustdoc

## Time Travel System

The world exists on a **continuous year-based timeline** (not fixed eras). Any year from deep past to far future can be visited.

- **Present year**: The "now" year — MMO players are synced here.
- **Single-player**: Stories can start at any year. The world changes organically over time — a location that's ancient ruins in one period might be a thriving castle in another.
- **Time portals**: Transport the player to a specific year, triggering terrain regeneration and a tinted fade transition.
- **Terrain**: Varies based on distance from present — distant past has dramatic mountains, present is default, far future is flatter with more detail.
- **NPCs**: Some NPCs have year-range filters (e.g., enemies don't spawn in the deep past).

## Rendering Pipeline

```
Frame
├── G-Buffer Pass (deferred)
├── Shadow Pass (cascaded)
├── Ray Tracing Pass (HW or compute fallback)
├── Denoise Pass (temporal SVGF)
├── Lighting Pass
├── Post-Processing
└── UI Pass (egui)
```

## PixygonServer Integration

The game integrates with PixygonServer for:
- Authentication (JWT)
- Monster/Character persistence (existing models)
- Multiplayer state sync
- Leaderboards and achievements

## Related Projects

- **Dyson**: Agent orchestration (this project's parent in the Pixygon ecosystem)
- **PixygonServer**: Backend API
- **Pixygon.io**: Web admin panel
