# Infinite - AI Agent Instructions

## Project Overview

Infinite is a Vulkan-based game engine in Rust. The engine powers a time-travel themed game where players can traverse different eras (past, present, future).

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `infinite-core` | Core types (Transform, Color, EntityId), time system (Era, Timeline, GameTime) |
| `infinite-ecs` | Entity Component System |
| `infinite-render` | Vulkan renderer with ray tracing |
| `infinite-physics` | Physics via rapier3d |
| `infinite-audio` | Audio via kira |
| `infinite-world` | World chunks, era management, time portals |
| `infinite-net` | Networking, prediction, server sync |
| `infinite-assets` | glTF loading, textures, caching |
| `infinite-game` | Game logic (monsters, battles, quests) |
| `infinite-integration` | PixygonServer API client |

## Building

```bash
cargo build              # Build all crates
cargo run                # Run the game
cargo test               # Run all tests
cargo check              # Fast type checking
```

## Key Dependencies

- **vulkano 0.34** - Safe Vulkan bindings
- **winit 0.28** - Window management
- **rapier3d** - Physics simulation
- **kira** - Audio playback
- **glam** - Math (vectors, matrices, quaternions)
- **tokio** - Async runtime for networking

## Time Travel System

The world exists across multiple Eras:
- **Past**: Historical periods (configurable years_ago)
- **Present**: The "now" moment, synced for MMO
- **Future**: Speculative futures (configurable years_ahead)

Single-player allows free travel between eras. MMO mode locks to Present.

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

## Development Guidelines

1. **Modularity**: Each crate has a single responsibility
2. **Safety**: Prefer vulkano's safe abstractions, use ash only for performance-critical paths
3. **Testing**: Write tests for all game logic
4. **Documentation**: Document public APIs with rustdoc

## Related Projects

- **Dyson**: Agent orchestration (this project's parent in the Pixygon ecosystem)
- **PixygonServer**: Backend API
- **Pixygon.io**: Web admin panel
