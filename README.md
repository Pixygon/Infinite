# Infinite

A Vulkan-based game engine in Rust with ray tracing by default.

## Overview

Infinite powers a game where players traverse time, playing different games within the game across past, present, and future. Features both single-player (free time travel) and MMO mode (shared world locked to "now").

## Features

- **Vulkan Rendering** - Modern graphics with deferred rendering pipeline
- **Ray Tracing** - Hardware RT when available, compute shader fallback for universal compatibility
- **Custom ECS** - Entity Component System optimized for the game's needs
- **Time Travel System** - Era-based world with seamless transitions
- **PixygonServer Integration** - Multiplayer, auth, and persistent state

## Building

```bash
# Build
cargo build

# Run
cargo run

# Run tests
cargo test
```

## Requirements

- Rust 1.75+
- Vulkan 1.2+ capable GPU
- For hardware ray tracing: GPU with VK_KHR_ray_tracing_pipeline support

## Architecture

```
crates/
├── infinite-core/       # Types, time, events, errors
├── infinite-ecs/        # Entity Component System
├── infinite-render/     # Vulkan + Ray Tracing
├── infinite-physics/    # Physics (rapier3d)
├── infinite-audio/      # Audio (kira)
├── infinite-world/      # Timeline/Era system, chunks
├── infinite-net/        # Networking, prediction, sync
├── infinite-assets/     # Asset loading, formats
├── infinite-game/       # Game logic, monsters, battles
└── infinite-integration/ # PixygonServer API client
```

## Time System

```rust
pub enum Era {
    Past(PastConfig),    // Historical periods
    Present,             // "Now" - MMO synced to real time
    Future(FutureConfig), // Speculative futures
}
```

### Game Modes
- **Single-Player**: Local state, free time travel, pausable
- **MMO**: Server-authoritative, locked to Present era, real-time

## License

MIT OR Apache-2.0
