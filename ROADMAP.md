# Infinite - Development Roadmap

## Vision

A Vulkan-based game engine in Rust with ray tracing by default, powering "Infinite" - a game where players traverse time, playing different games within the game across past, present, and future.

---

## Phase 1: Foundation

### Goals
- Project scaffold with Cargo workspace
- Core types (Vec3, Quat, Transform)
- Vulkan context initialization
- Basic forward renderer
- Simple ECS implementation
- glTF asset loading
- Camera controls
- CI/CD pipeline

### Deliverable
Interactive 3D scene viewer

### Milestones
- [x] Cargo workspace setup
- [x] infinite-core crate with types and time system
- [x] Vulkan instance and device creation
- [x] Window with event loop
- [x] CI/CD pipeline
- [ ] Basic triangle rendering
- [ ] glTF model loading
- [ ] Camera controls (orbit, fly)
- [ ] ECS foundation

---

## Phase 2: Ray Tracing

### Goals
- G-Buffer deferred renderer
- Hardware RT initialization
- BVH construction (CPU-built, GPU-resident)
- Compute shader RT fallback
- Temporal denoising
- PBR material system

### Deliverable
Cornell box with real-time global illumination

### Milestones
- [ ] Deferred rendering pipeline
- [ ] G-Buffer (position, normal, albedo, metallic/roughness)
- [ ] Hardware RT initialization (VK_KHR_ray_tracing_pipeline)
- [ ] BVH construction and management
- [ ] Compute shader ray traversal (fallback)
- [ ] SVGF temporal denoising
- [ ] PBR materials

---

## Phase 3: World System

### Goals
- Chunk-based world streaming
- Timeline/Era system
- Time portal mechanics
- Save/load system
- Physics integration
- Audio system

### Deliverable
Single-player demo with 3 eras

### Milestones
- [ ] World chunk loading/unloading
- [ ] Era system (Past, Present, Future)
- [ ] Era transitions with visual effects
- [ ] Time portal placement and traversal
- [ ] Physics via rapier3d
- [ ] Audio via kira
- [ ] Save/load serialization

---

## Phase 4: Game Systems

### Goals
- Player controller
- Monster system (sync with PixygonServer)
- Turn-based battle system
- Inventory/items
- Quest system foundation

### Deliverable
Playable single-player vertical slice

### Milestones
- [ ] First/third person player controller
- [ ] Monster spawning and AI
- [ ] Battle system with PixygonServer monster models
- [ ] Inventory UI
- [ ] Item pickup and usage
- [ ] Basic quest tracking

---

## Phase 5: Networking

### Goals
- PixygonServer game namespace
- Client networking layer
- Client-side prediction
- State synchronization
- Offline queue and sync
- Region-based interest management

### Deliverable
10-player shared world test

### Milestones
- [ ] PixygonServer WebSocket game namespace
- [ ] Authentication flow
- [ ] Client-side prediction
- [ ] Server reconciliation
- [ ] Offline action queueing
- [ ] Interest management (only sync nearby entities)
- [ ] Lag compensation

---

## Phase 6: Polish & Launch

### Goals
- UI/UX polish
- Performance optimization
- Cross-platform testing
- Content pipeline tooling
- Analytics integration
- Beta testing

### Deliverable
Public release

### Milestones
- [ ] UI redesign and polish
- [ ] GPU profiling and optimization
- [ ] Linux, Windows, macOS testing
- [ ] Asset pipeline tools
- [ ] Analytics events
- [ ] Closed beta
- [ ] Open beta
- [ ] Launch

---

## Technical Decisions

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Vulkan Bindings | vulkano (safe) + ash (perf paths) | Safety with escape hatch |
| Ray Tracing | Hybrid HW+SW | HW RT when available, compute fallback |
| ECS | Custom (Bevy-inspired) | Full control for game needs |
| Physics | rapier3d | Pure Rust, excellent |
| Audio | kira | Game-focused features |
| Networking | Extend PixygonServer | Leverage existing WebSocket/auth |
| UI | egui | Already used in Dyson |
| Assets | glTF 2.0 + KTX2 | Industry standard |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| SW RT performance | Dynamic sample counts, hybrid mode |
| Network latency | Aggressive prediction, interpolation |
| Scope creep | Strict phase boundaries, MVP mindset |
| Cross-platform | CI testing on all platforms from Phase 1 |
| Content bottleneck | Procedural generation, AI-assisted |

---

## Integration Points

### PixygonServer
- Existing Monster/Character models
- JWT authentication
- WebSocket real-time sync
- REST API for persistence

### Dyson
- Agent task tracking
- Automated testing triggers
- Deployment pipeline
