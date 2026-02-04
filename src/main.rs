//! Infinite - A Vulkan-based game engine with ray tracing
//!
//! This is the main entry point for the Infinite engine and game.

mod character;
mod save;
mod settings;
mod state;
mod ui;

use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use egui_winit_vulkano::{Gui, GuiConfig};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue,
        QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageUsage},
    instance::{
        debug::{
            DebugUtilsMessageSeverity, DebugUtilsMessenger,
            DebugUtilsMessengerCreateInfo,
        },
        Instance, InstanceCreateFlags, InstanceCreateInfo,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, PolygonMode, RasterizationState},
            vertex_input::VertexDefinition,
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{self, GpuFuture},
    Validated, VulkanError, VulkanLibrary,
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey, PhysicalKey, KeyCode},
    window::{CursorGrabMode, Window, WindowAttributes, WindowId},
};

use glam::{Mat4, Vec3};
use infinite_core::{GameTime, Timeline, time::format_year};
use infinite_game::{
    CameraController, InputAction, InputHandler, Interactable, InteractionResult,
    InteractionSystem, NpcId, PlayerController,
};
use infinite_game::npc::combat::PlayerCombatState;
use infinite_game::npc::dialogue::DialogueSystem;
use infinite_game::npc::manager::NpcManager;
use infinite_physics::PhysicsWorld;
use infinite_render::{BasicPushConstants, Mesh, SkyMesh, SkyPushConstants, Vertex3D, SkyVertex};
use infinite_world::{
    ChunkConfig, ChunkCoord, ChunkManager, TimeTerrainConfig, Terrain, TerrainConfig, TimeOfDay,
    Weather,
};

use crate::character::CharacterData;
use crate::save::{SaveData, PlayerSaveData, WorldSaveData};
use crate::settings::GameSettings;
use crate::state::{ApplicationState, StateTransition};
use crate::ui::{CharacterCreator, LoadingScreen, MainMenu, PauseMenu, SaveLoadAction, SaveLoadMenu, SettingsMenu};
use std::collections::HashMap;

/// Mesh buffers for GPU rendering
struct MeshBuffers {
    vertex_buffer: Subbuffer<[Vertex3D]>,
    index_buffer: Subbuffer<[u32]>,
    index_count: u32,
}

/// Sky mesh buffers
struct SkyMeshBuffers {
    vertex_buffer: Subbuffer<[SkyVertex]>,
    index_buffer: Subbuffer<[u32]>,
    index_count: u32,
}

/// Vulkan rendering context
struct RenderContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    _descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    // Depth buffer
    depth_buffer: Arc<ImageView>,

    // 3D pipelines
    basic_pipeline: Option<Arc<GraphicsPipeline>>,
    sky_pipeline: Option<Arc<GraphicsPipeline>>,
    wireframe_pipeline: Option<Arc<GraphicsPipeline>>,

    // Mesh buffers
    capsule_mesh: Option<MeshBuffers>,
    terrain_mesh: Option<MeshBuffers>,
    /// Per-chunk terrain meshes (keyed by ChunkCoord)
    chunk_meshes: HashMap<ChunkCoord, MeshBuffers>,
    /// Shared NPC capsule mesh (reused for all NPCs with per-NPC push constants)
    npc_capsule_mesh: Option<MeshBuffers>,
    sky_mesh: Option<SkyMeshBuffers>,
    debug_capsule_mesh: Option<MeshBuffers>,
}

/// Application state
struct InfiniteApp {
    /// Vulkan instance
    instance: Arc<Instance>,
    /// Window (created on resumed)
    window: Option<Arc<Window>>,
    /// Vulkan surface
    surface: Option<Arc<Surface>>,
    /// Render context
    render_ctx: Option<RenderContext>,
    /// egui renderer
    gui: Option<Gui>,
    /// Application state machine
    app_state: ApplicationState,
    /// State stack (for Push/Pop transitions)
    state_stack: Vec<ApplicationState>,
    /// Game time
    game_time: GameTime,
    /// Timeline
    timeline: Timeline,
    /// Last frame time
    last_frame: Instant,
    /// Game settings
    settings: GameSettings,
    /// Loading screen UI
    loading_screen: LoadingScreen,
    /// Main menu UI
    main_menu: MainMenu,
    /// Pause menu UI
    pause_menu: PauseMenu,
    /// Settings menu UI (created when needed)
    settings_menu: Option<SettingsMenu>,
    /// Save/Load menu UI (created when needed)
    save_load_menu: Option<SaveLoadMenu>,
    /// Character creator UI
    character_creator: CharacterCreator,
    /// Current character (when playing)
    current_character: Option<CharacterData>,
    /// Simulated loading timer
    loading_timer: f32,

    // Game systems
    /// Physics world
    physics_world: Option<PhysicsWorld>,
    /// Player controller
    player: Option<PlayerController>,
    /// Camera controller
    camera: Option<CameraController>,
    /// Input handler
    input_handler: InputHandler,
    /// Whether cursor is currently captured
    cursor_captured: bool,

    // World systems
    /// Terrain data (legacy single terrain, kept for reference)
    terrain: Option<Terrain>,
    /// Chunk manager for streaming terrain
    chunk_manager: Option<ChunkManager>,
    /// Time of day system
    time_of_day: TimeOfDay,
    /// Weather system
    weather: Weather,
    /// Interaction system
    interaction_system: InteractionSystem,
    /// NPC manager
    npc_manager: Option<NpcManager>,
    /// Dialogue system
    dialogue_system: DialogueSystem,
    /// Player combat state
    player_combat: PlayerCombatState,
    /// Text overlay to show (from sign interactions)
    interaction_text: Option<String>,
    /// Timer for hiding interaction text
    interaction_text_timer: f32,
    /// Notification text (e.g., "Game Saved")
    notification_text: Option<String>,
    /// Timer for hiding notification
    notification_timer: f32,
    /// Time transition fade alpha (0.0 = clear, 1.0 = black)
    time_transition_alpha: f32,
    /// Target year for pending transition
    pending_time_transition: Option<i64>,
    /// Whether we're in the middle of a time transition
    time_transitioning: bool,
    /// Source year for tinted transition
    time_transition_source: i64,

    // Climbing state
    /// Whether the player is currently climbing a ladder
    climbing: bool,
    /// Direction of climbing (typically Vec3::Y)
    climb_direction: Vec3,
    /// Remaining distance to climb
    climb_remaining: f32,

    // Collected items (pre-inventory)
    /// Items the player has collected
    collected_items: Vec<String>,
    /// Total play time in seconds
    play_time: f64,
    /// Auto-save countdown timer
    auto_save_timer: f32,

    // Debug
    /// Whether the debug overlay is visible
    debug_visible: bool,
    /// Render terrain in wireframe mode
    debug_wireframe: bool,
    /// Show collider shapes
    debug_colliders: bool,
}

impl InfiniteApp {
    fn new(instance: Arc<Instance>) -> Self {
        let settings = GameSettings::load();

        Self {
            instance,
            window: None,
            surface: None,
            render_ctx: None,
            gui: None,
            app_state: ApplicationState::default(),
            state_stack: Vec::new(),
            game_time: GameTime::default(),
            timeline: Timeline::default(),
            last_frame: Instant::now(),
            settings,
            loading_screen: LoadingScreen::new(),
            main_menu: MainMenu::new(),
            pause_menu: PauseMenu::new(),
            settings_menu: None,
            save_load_menu: None,
            character_creator: CharacterCreator::new(),
            current_character: None,
            loading_timer: 0.0,
            physics_world: None,
            player: None,
            camera: None,
            input_handler: InputHandler::new(),
            cursor_captured: false,

            terrain: None,
            chunk_manager: None,
            time_of_day: TimeOfDay::default(),
            weather: Weather::default(),
            interaction_system: InteractionSystem::new(),
            npc_manager: None,
            dialogue_system: DialogueSystem::new(),
            player_combat: PlayerCombatState::new(),
            interaction_text: None,
            interaction_text_timer: 0.0,
            notification_text: None,
            notification_timer: 0.0,
            time_transition_alpha: 0.0,
            pending_time_transition: None,
            time_transitioning: false,
            time_transition_source: 2025,

            climbing: false,
            climb_direction: Vec3::ZERO,
            climb_remaining: 0.0,

            collected_items: Vec::new(),
            play_time: 0.0,
            auto_save_timer: 300.0,

            debug_visible: false,
            debug_wireframe: false,
            debug_colliders: false,
        }
    }

    /// Initialize game systems when entering Playing state
    fn init_game_systems(&mut self) {
        // Create physics world
        let mut physics = PhysicsWorld::new();

        // Create a fallback ground plane well below terrain as safety net
        physics.create_ground(-50.0);

        // Set up chunk manager for streaming terrain
        let chunk_config = ChunkConfig {
            chunk_size: 64.0,
            subdivisions: 32,
            load_radius: 3,
            unload_radius: 4,
        };
        let terrain_config = TerrainConfig {
            size: 64.0, // matches chunk_size
            subdivisions: 32,
            max_height: 5.0,
            noise_scale: 0.02,
            seed: 42,
            ..Default::default()
        };

        let mut chunk_manager = ChunkManager::new(chunk_config.clone(), terrain_config.clone());

        // Apply time-period terrain config if not in the present year
        if !self.timeline.is_present() {
            chunk_manager.set_time_terrain_config(Some(TimeTerrainConfig::for_year(
                self.timeline.active_year,
                self.timeline.present_year,
            )));
        }

        // Initial chunk load around spawn
        let spawn_pos = Vec3::new(0.0, 0.0, 0.0);
        chunk_manager.update(spawn_pos, &mut physics);

        // Get spawn height from chunk manager
        let spawn_height = chunk_manager.height_at(0.0, 0.0);

        // Also generate a legacy single terrain for height_at queries outside chunks
        let legacy_terrain = Terrain::generate(TerrainConfig {
            size: 100.0,
            subdivisions: 64,
            max_height: 5.0,
            noise_scale: 0.02,
            seed: 42,
            ..Default::default()
        });
        self.terrain = Some(legacy_terrain);

        // Create chunk terrain meshes for initially loaded chunks
        if let Some(render_ctx) = &mut self.render_ctx {
            for chunk in chunk_manager.loaded_chunks() {
                let terrain = &chunk.terrain;
                let terrain_mesh_data = Mesh::terrain(
                    terrain.config.size,
                    terrain.config.subdivisions,
                    &terrain.heights,
                    |x, h, z| terrain.color_at(x, h, z),
                );

                if let Ok(buffers) = create_mesh_buffers(
                    render_ctx.memory_allocator.clone(),
                    &terrain_mesh_data.vertices,
                    &terrain_mesh_data.indices,
                ) {
                    render_ctx.chunk_meshes.insert(chunk.coord, buffers);
                }
            }
        }

        // Create NPC manager and spawn NPCs for initial chunks
        let mut npc_manager = NpcManager::new(chunk_config.chunk_size);
        let active_year = self.timeline.active_year;
        for chunk in chunk_manager.loaded_chunks() {
            let coord = chunk.coord;
            let cm_ref = &chunk_manager;
            npc_manager.on_chunk_loaded(coord, active_year, |x, z| cm_ref.height_at(x, z));
        }
        self.npc_manager = Some(npc_manager);

        // Create NPC capsule mesh (smaller than player)
        if let Some(render_ctx) = &mut self.render_ctx {
            if render_ctx.npc_capsule_mesh.is_none() {
                let npc_mesh_data = Mesh::capsule(1.6, 0.35, 12, 8, [1.0, 1.0, 1.0, 1.0]);
                if let Ok(buffers) = create_mesh_buffers(
                    render_ctx.memory_allocator.clone(),
                    &npc_mesh_data.vertices,
                    &npc_mesh_data.indices,
                ) {
                    render_ctx.npc_capsule_mesh = Some(buffers);
                }
            }
        }

        self.chunk_manager = Some(chunk_manager);

        // Reset dialogue and combat state
        self.dialogue_system = DialogueSystem::new();
        self.player_combat = PlayerCombatState::new();

        // Create player - spawn above terrain
        let mut player = PlayerController::new();
        player.spawn(&mut physics, Vec3::new(0.0, spawn_height + 2.0, 0.0));

        // Update query pipeline so the character controller can see terrain on the first frame
        physics.update_query_pipeline();

        // Create camera
        let camera = CameraController::new();

        self.physics_world = Some(physics);
        self.player = Some(player);
        self.camera = Some(camera);

        // Set up test interactables
        self.interaction_system.clear();
        self.interaction_system.add(Interactable::sign(
            Vec3::new(5.0, spawn_height + 1.0, 5.0),
            "Welcome to Infinite!\nExplore the world, travel through time.",
        ));
        self.interaction_system.add(Interactable::sign(
            Vec3::new(-10.0, spawn_height + 1.0, 0.0),
            "The terrain stretches infinitely in all directions.\nChunks load and unload as you walk.",
        ));
        // Time portals for testing
        self.interaction_system.add(Interactable::time_portal(
            Vec3::new(20.0, spawn_height + 1.0, 0.0),
            -5000,
            "Ancient Past (5001 BCE)",
        ));
        self.interaction_system.add(Interactable::time_portal(
            Vec3::new(20.0, spawn_height + 1.0, 10.0),
            3500,
            "Far Future (3500 CE)",
        ));

        // Stateful interactables for testing
        let locked_door_id = self.interaction_system.add_door(
            Vec3::new(10.0, spawn_height + 1.0, -5.0),
            true, // locked
        );
        self.interaction_system.add_door(
            Vec3::new(-5.0, spawn_height + 1.0, -5.0),
            false, // unlocked
        );
        self.interaction_system.add_lever(
            Vec3::new(8.0, spawn_height + 1.0, -5.0),
            vec![locked_door_id],
        );
        self.interaction_system.add_container(
            Vec3::new(0.0, spawn_height + 0.5, -8.0),
            vec!["Ancient Coin".to_string(), "Health Potion".to_string()],
        );
        self.interaction_system.add_ladder(
            Vec3::new(-8.0, spawn_height + 0.5, 0.0),
            6.0,
            Vec3::Y,
        );

        info!("Game systems initialized with chunk-based terrain");
    }

    /// Cleanup game systems when leaving Playing state
    fn cleanup_game_systems(&mut self) {
        self.physics_world = None;
        self.player = None;
        self.camera = None;
        self.terrain = None;
        self.chunk_manager = None;
        self.npc_manager = None;
        self.dialogue_system.end_dialogue();
        self.interaction_system.clear();
        self.interaction_text = None;
        self.notification_text = None;
        self.time_transitioning = false;
        self.pending_time_transition = None;
        self.climbing = false;
        self.climb_remaining = 0.0;

        // Clear terrain meshes
        if let Some(render_ctx) = &mut self.render_ctx {
            render_ctx.terrain_mesh = None;
            render_ctx.chunk_meshes.clear();
        }

        info!("Game systems cleaned up");
    }

    /// Gather all current game state into a SaveData struct
    fn gather_save_data(&self, slot_name: &str) -> SaveData {
        let player_pos = self.player.as_ref().map(|p| p.position()).unwrap_or(Vec3::ZERO);
        let (yaw, pitch) = self.camera.as_ref().map(|c| (c.yaw, c.pitch)).unwrap_or((0.0, 0.0));
        let char_name = self.current_character.as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Player".to_string());

        SaveData {
            version: 1,
            player: PlayerSaveData {
                position: [player_pos.x, player_pos.y, player_pos.z],
                rotation_yaw: yaw,
                rotation_pitch: pitch,
                character_name: char_name,
            },
            world: WorldSaveData {
                active_year: self.timeline.active_year,
                time_of_day: self.time_of_day.time_hours,
            },
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            slot_name: slot_name.to_string(),
            collected_items: self.collected_items.clone(),
            play_time_seconds: self.play_time,
            interactions: self.interaction_system.save_states(),
        }
    }

    /// Quick save the game (F5)
    fn do_quicksave(&mut self) {
        let data = self.gather_save_data("");

        match save::save_game(&data) {
            Ok(()) => {
                self.notification_text = Some("Game Saved".to_string());
                self.notification_timer = 2.0;
                info!("Game saved successfully");
            }
            Err(e) => {
                self.notification_text = Some(format!("Save failed: {}", e));
                self.notification_timer = 3.0;
                tracing::error!("Failed to save game: {}", e);
            }
        }
    }

    /// Auto-save the game
    fn do_autosave(&mut self) {
        let data = self.gather_save_data("Autosave");

        match save::autosave(&data) {
            Ok(()) => {
                self.notification_text = Some("Auto-saved".to_string());
                self.notification_timer = 1.5;
            }
            Err(e) => {
                tracing::error!("Failed to auto-save: {}", e);
            }
        }
    }

    /// Restore game state from save data
    fn restore_from_save(&mut self, data: SaveData) {
        // Restore player position
        if let (Some(player), Some(physics)) = (&mut self.player, &mut self.physics_world) {
            let pos = Vec3::new(data.player.position[0], data.player.position[1], data.player.position[2]);
            player.teleport(physics, pos);
        }

        // Restore camera rotation
        if let Some(camera) = &mut self.camera {
            camera.set_yaw(data.player.rotation_yaw);
            camera.set_pitch(data.player.rotation_pitch);
        }

        // Restore year (if different)
        if data.world.active_year != self.timeline.active_year {
            if !self.time_transitioning {
                self.time_transition_source = self.timeline.active_year;
                self.pending_time_transition = Some(data.world.active_year);
                self.time_transitioning = true;
                self.time_transition_alpha = 0.0;
            }
        }

        // Restore time of day
        self.time_of_day.set_time(data.world.time_of_day);

        // Restore collected items and play time
        self.collected_items = data.collected_items;
        self.play_time = data.play_time_seconds;

        // Restore interaction states
        self.interaction_system.load_states(data.interactions);

        // Reset climbing state
        self.climbing = false;
        self.climb_remaining = 0.0;
    }

    /// Quick load the game (F9)
    fn do_quickload(&mut self) {
        match save::load_game() {
            Ok(data) => {
                self.restore_from_save(data);
                self.notification_text = Some("Game Loaded".to_string());
                self.notification_timer = 2.0;
                info!("Game loaded successfully");
            }
            Err(e) => {
                self.notification_text = Some(format!("Load failed: {}", e));
                self.notification_timer = 3.0;
                tracing::error!("Failed to load game: {}", e);
            }
        }
    }

    /// Update cursor capture state
    fn update_cursor_capture(&mut self, should_capture: bool) {
        if self.cursor_captured == should_capture {
            return;
        }

        if let Some(window) = &self.window {
            if should_capture {
                let _ = window.set_cursor_grab(CursorGrabMode::Locked)
                    .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
                window.set_cursor_visible(false);
            } else {
                let _ = window.set_cursor_grab(CursorGrabMode::None);
                window.set_cursor_visible(true);
            }
            self.cursor_captured = should_capture;
            self.input_handler.set_cursor_captured(should_capture);
        }
    }

    fn create_swapchain_and_framebuffers(
        device: Arc<Device>,
        surface: Arc<Surface>,
        window: Arc<Window>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<(
        Arc<Swapchain>,
        Vec<Arc<Image>>,
        Arc<RenderPass>,
        Vec<Arc<Framebuffer>>,
        Arc<ImageView>,
    )> {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .context("Failed to get surface capabilities")?;

        let image_format = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .context("Failed to get surface formats")?[0]
            .0;

        let window_size = window.inner_size();

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count.max(2),
                image_format,
                image_extent: [window_size.width, window_size.height],
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .context("Failed to create swapchain")?;

        // Create depth buffer
        let depth_buffer = ImageView::new_default(
            Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: vulkano::image::ImageType::Dim2d,
                    format: Format::D32_SFLOAT,
                    extent: [window_size.width, window_size.height, 1],
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .context("Failed to create depth buffer")?,
        )
        .context("Failed to create depth buffer view")?;

        // Create render pass with two subpasses:
        // Subpass 0: 3D scene rendering with depth
        // Subpass 1: UI overlay (no depth)
        let render_pass = vulkano::ordered_passes_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: image_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },
            passes: [
                // Subpass 0: 3D scene with depth
                {
                    color: [color],
                    depth_stencil: {depth},
                    input: []
                },
                // Subpass 1: UI overlay (no depth)
                {
                    color: [color],
                    depth_stencil: {},
                    input: []
                }
            ]
        )
        .context("Failed to create render pass")?;

        let framebuffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_buffer.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect();

        Ok((swapchain, images, render_pass, framebuffers, depth_buffer))
    }

    fn recreate_swapchain(&mut self) {
        let Some(window) = &self.window else { return };
        let Some(_surface) = &self.surface else { return };
        let Some(render_ctx) = &mut self.render_ctx else {
            return;
        };

        let window_size = window.inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        let (new_swapchain, new_images) = render_ctx
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: [window_size.width, window_size.height],
                ..render_ctx.swapchain.create_info()
            })
            .expect("Failed to recreate swapchain");

        // Recreate depth buffer with new size
        let new_depth_buffer = ImageView::new_default(
            Image::new(
                render_ctx.memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: vulkano::image::ImageType::Dim2d,
                    format: Format::D32_SFLOAT,
                    extent: [window_size.width, window_size.height, 1],
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .expect("Failed to recreate depth buffer"),
        )
        .expect("Failed to create depth buffer view");

        render_ctx.swapchain = new_swapchain;
        render_ctx.images = new_images.clone();
        render_ctx.depth_buffer = new_depth_buffer.clone();
        render_ctx.framebuffers = new_images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_ctx.render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, new_depth_buffer.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect();

        render_ctx.recreate_swapchain = false;
    }

    fn update(&mut self, delta: f32) {
        self.game_time.update(delta);

        // Update based on current state
        match &self.app_state {
            ApplicationState::Loading(phase) => {
                self.loading_timer += delta;
                self.loading_screen.update(delta, phase.progress());

                // Simulate loading phases (advance every 0.5 seconds)
                if self.loading_timer >= 0.5 {
                    self.loading_timer = 0.0;
                    if let Some(next_phase) = phase.next() {
                        self.app_state = ApplicationState::Loading(next_phase);
                    } else {
                        // Loading complete, go to main menu
                        self.app_state = ApplicationState::MainMenu;
                        info!(
                            "Loading complete - Year: {}",
                            self.timeline.year_label()
                        );
                    }
                }
            }
            ApplicationState::Playing => {
                // Release cursor when debug overlay is open so user can interact with it
                self.update_cursor_capture(!self.debug_visible);

                // Update world systems
                self.time_of_day.update(delta);
                self.weather.update(delta);

                // --- Time transition fade ---
                if self.time_transitioning {
                    if let Some(target_year) = self.pending_time_transition {
                        if self.time_transition_alpha < 1.0 {
                            // Fade to black
                            self.time_transition_alpha = (self.time_transition_alpha + delta * 2.0).min(1.0);
                        } else {
                            // At full black: switch year, regenerate terrain
                            if let Err(e) = self.timeline.travel_to_year(target_year) {
                                tracing::error!("Failed to travel to year {}: {}", target_year, e);
                            } else {
                                info!("Switched to year: {}", self.timeline.year_label());
                            }

                            // Regenerate chunks with new time-period terrain config
                            let time_config = if self.timeline.is_present() {
                                None
                            } else {
                                Some(TimeTerrainConfig::for_year(target_year, self.timeline.present_year))
                            };

                            if let (Some(chunk_manager), Some(physics)) =
                                (&mut self.chunk_manager, &mut self.physics_world)
                            {
                                chunk_manager.set_time_terrain_config(time_config);
                                let player_pos = self.player.as_ref()
                                    .map(|p| p.position())
                                    .unwrap_or(Vec3::ZERO);
                                chunk_manager.reload_all(player_pos, physics);
                                physics.update_query_pipeline();

                                // Rebuild chunk meshes
                                if let Some(render_ctx) = &mut self.render_ctx {
                                    render_ctx.chunk_meshes.clear();
                                    for chunk in chunk_manager.loaded_chunks() {
                                        let terrain = &chunk.terrain;
                                        let mesh_data = Mesh::terrain(
                                            terrain.config.size,
                                            terrain.config.subdivisions,
                                            &terrain.heights,
                                            |x, h, z| terrain.color_at(x, h, z),
                                        );
                                        if let Ok(buffers) = create_mesh_buffers(
                                            render_ctx.memory_allocator.clone(),
                                            &mesh_data.vertices,
                                            &mesh_data.indices,
                                        ) {
                                            render_ctx.chunk_meshes.insert(chunk.coord, buffers);
                                        }
                                    }
                                }
                            }

                            self.pending_time_transition = None;

                            // Auto-save on time transition
                            if self.settings.gameplay.auto_save {
                                self.do_autosave();
                                self.auto_save_timer = self.settings.gameplay.auto_save_interval as f32;
                            }
                        }
                    } else {
                        // No pending transition, fade back in
                        self.time_transition_alpha = (self.time_transition_alpha - delta * 2.0).max(0.0);
                        if self.time_transition_alpha <= 0.0 {
                            self.time_transitioning = false;
                        }
                    }
                }

                // --- Climbing mode update ---
                if self.climbing {
                    let climb_speed = 3.0;
                    let climb_step = climb_speed * delta;

                    // Check for exit: Jump to dismount, or reached top/bottom
                    if self.input_handler.state.is_just_pressed(InputAction::Jump)
                        || self.climb_remaining <= 0.0
                    {
                        self.climbing = false;
                        self.climb_remaining = 0.0;
                    } else if let (Some(player), Some(physics)) =
                        (&mut self.player, &mut self.physics_world)
                    {
                        // Move player up/down based on W/S input
                        let move_dir = if self.input_handler.state.is_held(InputAction::MoveForward) {
                            1.0
                        } else if self.input_handler.state.is_held(InputAction::MoveBackward) {
                            -1.0
                        } else {
                            0.0
                        };

                        if move_dir != 0.0 {
                            let displacement = self.climb_direction * move_dir * climb_step;
                            let pos = player.position() + displacement;
                            player.teleport(physics, pos);
                            if move_dir > 0.0 {
                                self.climb_remaining -= climb_step;
                            }
                        }
                    }
                }

                // --- Fixed timestep physics update ---
                let fixed_dt = self.game_time.config.fixed_timestep;
                let steps = self.game_time.fixed_steps();

                for _ in 0..steps {
                    if let (Some(physics), Some(player), Some(camera)) =
                        (&mut self.physics_world, &mut self.player, &self.camera)
                    {
                        if !self.climbing {
                            player.fixed_update(
                                physics,
                                &self.input_handler.state,
                                camera.yaw,
                                fixed_dt,
                            );
                        }
                        physics.step();
                    }
                }

                // --- Variable timestep camera update ---
                if let (Some(physics), Some(player), Some(camera)) =
                    (&self.physics_world, &self.player, &mut self.camera)
                {
                    camera.update(
                        &self.input_handler.state,
                        player.eye_position(),
                        Some(physics),
                        delta,
                    );
                }

                // --- Chunk streaming ---
                let player_pos = self.player.as_ref().map(|p| p.position()).unwrap_or(Vec3::ZERO);
                if let (Some(chunk_manager), Some(physics)) =
                    (&mut self.chunk_manager, &mut self.physics_world)
                {
                    chunk_manager.update(player_pos, physics);

                    // Remove meshes for unloaded chunks
                    if let Some(render_ctx) = &mut self.render_ctx {
                        for coord in &chunk_manager.newly_unloaded {
                            render_ctx.chunk_meshes.remove(coord);
                        }

                        // Create meshes for newly loaded chunks
                        for coord in &chunk_manager.newly_loaded {
                            if let Some(chunk) = chunk_manager.get_chunk(coord) {
                                let terrain = &chunk.terrain;
                                let mesh_data = Mesh::terrain(
                                    terrain.config.size,
                                    terrain.config.subdivisions,
                                    &terrain.heights,
                                    |x, h, z| terrain.color_at(x, h, z),
                                );
                                if let Ok(buffers) = create_mesh_buffers(
                                    render_ctx.memory_allocator.clone(),
                                    &mesh_data.vertices,
                                    &mesh_data.indices,
                                ) {
                                    render_ctx.chunk_meshes.insert(*coord, buffers);
                                }
                            }
                        }
                    }

                    physics.update_query_pipeline();
                }

                // --- NPC spawning/despawning with chunks ---
                if let (Some(npc_manager), Some(chunk_manager)) =
                    (&mut self.npc_manager, &self.chunk_manager)
                {
                    let active_year = self.timeline.active_year;

                    // Despawn NPCs from unloaded chunks
                    for coord in &chunk_manager.newly_unloaded {
                        npc_manager.on_chunk_unloaded(*coord);
                    }

                    // Spawn NPCs for newly loaded chunks
                    for coord in &chunk_manager.newly_loaded {
                        let cm_ref = chunk_manager;
                        npc_manager.on_chunk_loaded(*coord, active_year, |x, z| cm_ref.height_at(x, z));
                    }
                }

                // --- NPC update ---
                if let (Some(npc_manager), Some(chunk_manager)) =
                    (&mut self.npc_manager, &self.chunk_manager)
                {
                    let cm_ref = chunk_manager;
                    npc_manager.update(delta, player_pos, |x, z| cm_ref.height_at(x, z));

                    // Sync NPC positions to interaction system:
                    // Remove old NPC interactables
                    self.interaction_system.retain(|i| {
                        !matches!(i.kind, infinite_game::InteractableKind::Npc { .. })
                    });

                    // Add current NPC interactables (non-hostile only)
                    for npc in npc_manager.npcs_iter() {
                        if npc.data.faction != infinite_game::NpcFaction::Hostile {
                            self.interaction_system.add(Interactable::npc(
                                npc.position,
                                npc.id,
                                npc.name(),
                                npc.data.interaction_radius,
                            ));
                        }
                    }

                    // --- Enemy combat: enemies damage player ---
                    // Collect attacking enemy IDs first (avoids borrow conflicts)
                    let attacking_enemies: Vec<NpcId> = npc_manager.npcs_iter()
                        .filter(|n| n.data.role == infinite_game::NpcRole::Enemy)
                        .filter(|n| {
                            n.brain.as_ref()
                                .and_then(|b| b.current_action_name())
                                .map(|name| name == "attack_melee")
                                .unwrap_or(false)
                        })
                        .map(|n| n.id)
                        .collect();
                    for npc_id in &attacking_enemies {
                        if let Some(stats) = npc_manager.combat_stats.get_mut(npc_id) {
                            if stats.is_alive() && stats.update_attack(delta) {
                                let dmg = stats.attack;
                                self.player_combat.take_damage(dmg);
                            }
                        }
                    }
                }

                // --- Player combat update ---
                self.player_combat.update(delta);

                // --- Interaction system ---
                if let Some(camera) = &self.camera {
                    let forward = camera.forward();
                    self.interaction_system.update(player_pos, forward);
                }

                // Handle Interact input (E key)
                if !self.dialogue_system.is_active() && self.input_handler.state.is_just_pressed(InputAction::Interact) {
                    if let Some(result) = self.interaction_system.interact() {
                        match result {
                            InteractionResult::ShowText(text) => {
                                self.interaction_text = Some(text);
                                self.interaction_text_timer = 5.0;
                            }
                            InteractionResult::ChangeTimePeriod(target_year) => {
                                if !self.time_transitioning {
                                    self.time_transition_source = self.timeline.active_year;
                                    self.pending_time_transition = Some(target_year);
                                    self.time_transitioning = true;
                                    self.time_transition_alpha = 0.0;
                                    info!("Starting time transition to year {}", target_year);
                                }
                            }
                            InteractionResult::PickupItem(name) => {
                                self.notification_text = Some(format!("Picked up: {}", name));
                                self.notification_timer = 3.0;
                            }
                            InteractionResult::TalkToNpc(npc_id) => {
                                if let Some(npc_manager) = &mut self.npc_manager {
                                    if let Some(npc) = npc_manager.get_mut(npc_id) {
                                        let npc_name = npc.name().to_string();
                                        let role = npc.data.role;
                                        npc.state = infinite_game::npc::NpcBehaviorState::Talking;
                                        self.dialogue_system.start_dialogue(npc_id, npc_name, role);
                                    }
                                }
                            }
                            InteractionResult::ToggleDoor { now_open, .. } => {
                                let msg = if now_open { "Door opened" } else { "Door closed" };
                                self.notification_text = Some(msg.to_string());
                                self.notification_timer = 1.5;
                            }
                            InteractionResult::ToggleLever { now_on, linked, .. } => {
                                let msg = if now_on { "Lever activated" } else { "Lever deactivated" };
                                self.notification_text = Some(msg.to_string());
                                self.notification_timer = 1.5;
                                self.interaction_system.trigger_linked(&linked);
                            }
                            InteractionResult::PressButton { .. } => {
                                self.notification_text = Some("Button pressed".to_string());
                                self.notification_timer = 1.5;
                            }
                            InteractionResult::OpenContainer { items, .. } => {
                                if items.is_empty() {
                                    self.notification_text = Some("Container is empty".to_string());
                                } else {
                                    let item_list = items.join(", ");
                                    self.notification_text = Some(format!("Found: {}", item_list));
                                    self.collected_items.extend(items);
                                }
                                self.notification_timer = 3.0;
                            }
                            InteractionResult::StartClimbing { height, direction } => {
                                self.climbing = true;
                                self.climb_direction = direction;
                                self.climb_remaining = height;
                                self.notification_text = Some("Climbing...".to_string());
                                self.notification_timer = 1.5;
                            }
                            InteractionResult::Locked => {
                                self.notification_text = Some("It's locked".to_string());
                                self.notification_timer = 2.0;
                            }
                        }
                    }
                }

                // --- Save/Load ---
                if self.input_handler.state.is_just_pressed(InputAction::QuickSave) {
                    self.do_quicksave();
                }
                if self.input_handler.state.is_just_pressed(InputAction::QuickLoad) {
                    self.do_quickload();
                }

                // --- Play time & auto-save ---
                self.play_time += delta as f64;
                if self.settings.gameplay.auto_save {
                    self.auto_save_timer -= delta;
                    if self.auto_save_timer <= 0.0 {
                        self.do_autosave();
                        self.auto_save_timer = self.settings.gameplay.auto_save_interval as f32;
                    }
                }

                // --- Timers ---
                if self.interaction_text_timer > 0.0 {
                    self.interaction_text_timer -= delta;
                    if self.interaction_text_timer <= 0.0 {
                        self.interaction_text = None;
                    }
                }
                if self.notification_timer > 0.0 {
                    self.notification_timer -= delta;
                    if self.notification_timer <= 0.0 {
                        self.notification_text = None;
                    }
                }

                // Clear frame input
                self.input_handler.end_frame();
            }
            _ => {
                // Release cursor in menus
                self.update_cursor_capture(false);
            }
        }
    }

    fn apply_transition(&mut self, transition: StateTransition) {
        let old_state = self.app_state.clone();

        match transition {
            StateTransition::None => return,
            StateTransition::Push(state) => {
                let current = std::mem::replace(&mut self.app_state, state);
                self.state_stack.push(current);
            }
            StateTransition::Pop => {
                if let Some(previous) = self.state_stack.pop() {
                    self.app_state = previous;
                }
            }
            StateTransition::Replace(state) => {
                self.app_state = state;
                self.state_stack.clear();
            }
        }

        // Handle state-specific initialization/cleanup
        match &self.app_state {
            ApplicationState::Settings { .. } => {
                self.settings_menu = Some(SettingsMenu::new(self.settings.clone()));
            }
            ApplicationState::CharacterCreation => {
                // Reset character creator for new character
                self.character_creator.reset();
            }
            ApplicationState::Playing => {
                // Initialize game systems if coming from character creation
                if matches!(old_state, ApplicationState::CharacterCreation) {
                    self.init_game_systems();
                }
            }
            ApplicationState::MainMenu => {
                // Cleanup game systems when returning to main menu
                if matches!(old_state, ApplicationState::Playing | ApplicationState::Paused | ApplicationState::SaveLoad { .. }) {
                    self.cleanup_game_systems();
                    self.current_character = None;
                    self.save_load_menu = None;
                }
            }
            _ => {}
        }
    }

    fn render(&mut self) {
        // Get window size before borrowing other things
        let window_size = match &self.window {
            Some(w) => w.inner_size(),
            None => return,
        };

        // Check if we need to recreate swapchain first
        if let Some(render_ctx) = &self.render_ctx {
            if render_ctx.recreate_swapchain {
                self.recreate_swapchain();
                return;
            }
        } else {
            return;
        }

        // Cleanup finished work
        if let Some(render_ctx) = &mut self.render_ctx {
            if let Some(future) = render_ctx.previous_frame_end.as_mut() {
                future.cleanup_finished();
            }
        }

        // Acquire next swapchain image
        let (image_index, suboptimal, acquire_future) = {
            let render_ctx = self.render_ctx.as_mut().unwrap();
            match acquire_next_image(render_ctx.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    render_ctx.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {e}"),
            }
        };

        if suboptimal {
            if let Some(render_ctx) = &mut self.render_ctx {
                render_ctx.recreate_swapchain = true;
            }
        }

        // Build egui UI - collect transition to apply later
        let mut pending_transition = StateTransition::None;
        let mut should_save_settings = false;
        let mut save_load_pending_action: Option<(StateTransition, SaveLoadAction)> = None;

        if let Some(gui) = &mut self.gui {
            gui.immediate_ui(|gui| {
                let ctx = gui.context();

                // Dark theme background
                let mut style = (*ctx.style()).clone();
                style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 40);
                style.visuals.panel_fill = egui::Color32::from_rgb(30, 30, 40);
                ctx.set_style(style);

                // For states that need 3D rendering (Playing, CharacterCreation), use transparent background
                // For UI-only states (Loading, MainMenu, Settings, Paused), use opaque background
                let needs_transparent = matches!(
                    self.app_state,
                    ApplicationState::Playing | ApplicationState::CharacterCreation
                );
                let panel_fill = if needs_transparent {
                    egui::Color32::TRANSPARENT
                } else {
                    egui::Color32::from_rgb(20, 20, 30)
                };

                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(panel_fill))
                    .show(&ctx, |ui| {
                        let transition = match &mut self.app_state {
                            ApplicationState::Loading(phase) => {
                                self.loading_screen.render(ui, phase);
                                StateTransition::None
                            }
                            ApplicationState::MainMenu => self.main_menu.render(ui),
                            ApplicationState::CharacterCreation => {
                                self.character_creator.render(ui)
                            }
                            ApplicationState::Settings { .. } => {
                                if let Some(settings_menu) = &mut self.settings_menu {
                                    let (transition, apply) = settings_menu.render(ui);
                                    if apply {
                                        self.settings = settings_menu.working_settings().clone();
                                        should_save_settings = true;
                                    }
                                    transition
                                } else {
                                    StateTransition::Pop
                                }
                            }
                            ApplicationState::Paused => self.pause_menu.render(ui),
                            ApplicationState::SaveLoad { is_saving } => {
                                let is_saving = *is_saving;
                                if self.save_load_menu.is_none() {
                                    self.save_load_menu = Some(SaveLoadMenu::new(is_saving));
                                }
                                // Render menu and capture action
                                let (menu_transition, action) = if let Some(menu) = &mut self.save_load_menu {
                                    menu.render(ui)
                                } else {
                                    (StateTransition::None, SaveLoadAction::None)
                                };
                                // Store action to process after match (avoids borrow conflict with gather_save_data)
                                save_load_pending_action = Some((menu_transition, action));
                                StateTransition::None
                            }
                            ApplicationState::Playing => {
                                // Player stats (placeholder values for now)
                                let hp = 85.0f32;
                                let max_hp = 100.0f32;
                                let mana = 60.0f32;
                                let max_mana = 100.0f32;
                                let level = 1u32;

                                // Get time and weather info
                                let time_str = self.time_of_day.formatted_time();
                                let period = self.time_of_day.period_name();
                                let weather_name = self.weather.current.name();

                                // Top-left: HP, Level, Mana
                                egui::Area::new(egui::Id::new("player_stats"))
                                    .fixed_pos([10.0, 10.0])
                                    .show(&ctx, |ui| {
                                        egui::Frame::new()
                                            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200))
                                            .corner_radius(8.0)
                                            .inner_margin(12.0)
                                            .show(ui, |ui| {
                                                ui.set_min_width(180.0);

                                                // Level
                                                ui.label(
                                                    egui::RichText::new(format!("Level {}", level))
                                                        .font(egui::FontId::proportional(18.0))
                                                        .color(egui::Color32::from_rgb(255, 215, 0))
                                                );

                                                ui.add_space(8.0);

                                                // HP Bar
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new("HP")
                                                            .font(egui::FontId::proportional(12.0))
                                                            .color(egui::Color32::from_rgb(200, 80, 80))
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(format!("{:.0}/{:.0}", hp, max_hp))
                                                            .font(egui::FontId::proportional(12.0))
                                                            .color(egui::Color32::from_rgb(180, 180, 180))
                                                    );
                                                });
                                                let hp_rect = ui.available_rect_before_wrap();
                                                let hp_bar_rect = egui::Rect::from_min_size(
                                                    hp_rect.min,
                                                    egui::vec2(160.0, 12.0)
                                                );
                                                ui.painter().rect_filled(hp_bar_rect, 3.0, egui::Color32::from_rgb(60, 20, 20));
                                                let hp_fill = egui::Rect::from_min_size(
                                                    hp_bar_rect.min,
                                                    egui::vec2(160.0 * (hp / max_hp), 12.0)
                                                );
                                                ui.painter().rect_filled(hp_fill, 3.0, egui::Color32::from_rgb(200, 50, 50));
                                                ui.allocate_space(egui::vec2(160.0, 12.0));

                                                ui.add_space(6.0);

                                                // Mana Bar
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new("MP")
                                                            .font(egui::FontId::proportional(12.0))
                                                            .color(egui::Color32::from_rgb(80, 120, 200))
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(format!("{:.0}/{:.0}", mana, max_mana))
                                                            .font(egui::FontId::proportional(12.0))
                                                            .color(egui::Color32::from_rgb(180, 180, 180))
                                                    );
                                                });
                                                let mana_rect = ui.available_rect_before_wrap();
                                                let mana_bar_rect = egui::Rect::from_min_size(
                                                    mana_rect.min,
                                                    egui::vec2(160.0, 12.0)
                                                );
                                                ui.painter().rect_filled(mana_bar_rect, 3.0, egui::Color32::from_rgb(20, 30, 60));
                                                let mana_fill = egui::Rect::from_min_size(
                                                    mana_bar_rect.min,
                                                    egui::vec2(160.0 * (mana / max_mana), 12.0)
                                                );
                                                ui.painter().rect_filled(mana_fill, 3.0, egui::Color32::from_rgb(50, 100, 200));
                                                ui.allocate_space(egui::vec2(160.0, 12.0));
                                            });
                                    });

                                // Top-right: Time of day + Weather
                                egui::Area::new(egui::Id::new("time_weather"))
                                    .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                                    .show(&ctx, |ui| {
                                        egui::Frame::new()
                                            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200))
                                            .corner_radius(8.0)
                                            .inner_margin(12.0)
                                            .show(ui, |ui| {
                                                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                                                    // Time
                                                    ui.label(
                                                        egui::RichText::new(&time_str)
                                                            .font(egui::FontId::proportional(24.0))
                                                            .color(egui::Color32::from_rgb(255, 255, 200))
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(period)
                                                            .font(egui::FontId::proportional(14.0))
                                                            .color(egui::Color32::from_rgb(180, 180, 150))
                                                    );

                                                    ui.add_space(4.0);

                                                    // Weather
                                                    let weather_color = match self.weather.current {
                                                        infinite_world::WeatherState::Clear => egui::Color32::from_rgb(135, 206, 250),
                                                        infinite_world::WeatherState::Cloudy => egui::Color32::from_rgb(180, 180, 190),
                                                        infinite_world::WeatherState::Rain => egui::Color32::from_rgb(100, 130, 180),
                                                        infinite_world::WeatherState::Storm => egui::Color32::from_rgb(80, 80, 120),
                                                    };
                                                    ui.label(
                                                        egui::RichText::new(weather_name)
                                                            .font(egui::FontId::proportional(14.0))
                                                            .color(weather_color)
                                                    );
                                                });
                                            });
                                    });

                                // Controls hint at bottom
                                egui::Area::new(egui::Id::new("controls_hint"))
                                    .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
                                    .show(&ctx, |ui| {
                                        ui.label(
                                            egui::RichText::new("WASD: Move | Space: Jump | Shift: Sprint | Scroll: Zoom | E: Interact | F5: Save | F9: Load | ESC: Pause | F3: Debug")
                                                .color(egui::Color32::from_rgba_unmultiplied(150, 150, 170, 200))
                                                .font(egui::FontId::proportional(12.0)),
                                        );
                                    });

                                // Interaction prompt (when focused on an interactable)
                                if let Some(focused) = self.interaction_system.focused() {
                                    egui::Area::new(egui::Id::new("interaction_prompt"))
                                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 50.0])
                                        .show(&ctx, |ui| {
                                            egui::Frame::new()
                                                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220))
                                                .corner_radius(6.0)
                                                .inner_margin(10.0)
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!("Press E to {}", &focused.prompt))
                                                            .font(egui::FontId::proportional(16.0))
                                                            .color(egui::Color32::from_rgb(255, 220, 100)),
                                                    );
                                                });
                                        });
                                }

                                // Interaction text overlay (sign content)
                                if let Some(text) = &self.interaction_text {
                                    egui::Area::new(egui::Id::new("interaction_text"))
                                        .anchor(egui::Align2::CENTER_CENTER, [0.0, -50.0])
                                        .show(&ctx, |ui| {
                                            egui::Frame::new()
                                                .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 40, 230))
                                                .corner_radius(8.0)
                                                .inner_margin(16.0)
                                                .show(ui, |ui| {
                                                    ui.set_max_width(400.0);
                                                    ui.label(
                                                        egui::RichText::new(text)
                                                            .font(egui::FontId::proportional(16.0))
                                                            .color(egui::Color32::from_rgb(230, 230, 240)),
                                                    );
                                                });
                                        });
                                }

                                // Notification (save/load/pickup)
                                if let Some(notif) = &self.notification_text {
                                    egui::Area::new(egui::Id::new("notification"))
                                        .anchor(egui::Align2::CENTER_TOP, [0.0, 60.0])
                                        .show(&ctx, |ui| {
                                            egui::Frame::new()
                                                .fill(egui::Color32::from_rgba_unmultiplied(0, 80, 0, 200))
                                                .corner_radius(6.0)
                                                .inner_margin(10.0)
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        egui::RichText::new(notif)
                                                            .font(egui::FontId::proportional(16.0))
                                                            .color(egui::Color32::WHITE),
                                                    );
                                                });
                                        });
                                }

                                // --- Dialogue UI ---
                                if self.dialogue_system.is_active() {
                                    let should_close = {
                                        let mut close = false;
                                        egui::Area::new(egui::Id::new("dialogue_ui"))
                                            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -40.0])
                                            .show(&ctx, |ui| {
                                                egui::Frame::new()
                                                    .fill(egui::Color32::from_rgba_unmultiplied(15, 15, 30, 240))
                                                    .corner_radius(10.0)
                                                    .inner_margin(16.0)
                                                    .show(ui, |ui| {
                                                        ui.set_min_width(400.0);
                                                        ui.set_max_width(500.0);

                                                        if let Some(active) = self.dialogue_system.active() {
                                                            ui.label(
                                                                egui::RichText::new(&active.npc_name)
                                                                    .font(egui::FontId::proportional(14.0))
                                                                    .color(egui::Color32::from_rgb(180, 180, 200))
                                                            );
                                                            ui.separator();
                                                        }

                                                        if let Some(node) = self.dialogue_system.current_node() {
                                                            ui.label(
                                                                egui::RichText::new(&node.text)
                                                                    .font(egui::FontId::proportional(16.0))
                                                                    .color(egui::Color32::from_rgb(230, 230, 240))
                                                            );
                                                            ui.add_space(10.0);

                                                            let responses: Vec<(usize, String)> = node.responses.iter()
                                                                .enumerate()
                                                                .map(|(i, r)| (i, r.text.clone()))
                                                                .collect();
                                                            for (i, text) in responses {
                                                                if ui.button(
                                                                    egui::RichText::new(format!("  {}  ", text))
                                                                        .font(egui::FontId::proportional(14.0))
                                                                ).clicked() {
                                                                    close = false;
                                                                    self.dialogue_system.choose_response(i);
                                                                }
                                                            }
                                                        } else {
                                                            close = true;
                                                        }
                                                    });
                                            });
                                        close
                                    };
                                    if should_close {
                                        self.dialogue_system.end_dialogue();
                                    }
                                }

                                // --- Player Health Bar ---
                                if self.player_combat.current_hp < self.player_combat.max_hp {
                                    egui::Area::new(egui::Id::new("player_health"))
                                        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
                                        .show(&ctx, |ui| {
                                            egui::Frame::new()
                                                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180))
                                                .corner_radius(4.0)
                                                .inner_margin(6.0)
                                                .show(ui, |ui| {
                                                    ui.label(egui::RichText::new("HP")
                                                        .font(egui::FontId::proportional(12.0))
                                                        .color(egui::Color32::WHITE));
                                                    let hp_frac = self.player_combat.hp_fraction();
                                                    let bar = egui::ProgressBar::new(hp_frac)
                                                        .text(format!("{:.0}/{:.0}", self.player_combat.current_hp, self.player_combat.max_hp));
                                                    ui.add_sized([150.0, 16.0], bar);
                                                });
                                        });
                                }

                                // --- Damage Flash ---
                                if self.player_combat.damage_flash_timer > 0.0 {
                                    let alpha = (self.player_combat.damage_flash_timer / 0.3 * 80.0) as u8;
                                    egui::Area::new(egui::Id::new("damage_flash"))
                                        .fixed_pos([0.0, 0.0])
                                        .order(egui::Order::Foreground)
                                        .show(&ctx, |ui| {
                                            let screen_rect = ctx.screen_rect();
                                            ui.painter().rect_filled(
                                                screen_rect,
                                                0.0,
                                                egui::Color32::from_rgba_unmultiplied(200, 0, 0, alpha),
                                            );
                                            ui.allocate_space(screen_rect.size());
                                        });
                                }

                                // Time transition fade overlay (tinted by time period)
                                if self.time_transition_alpha > 0.01 {
                                    let alpha = (self.time_transition_alpha * 255.0) as u8;
                                    // Blend between source tint and target tint
                                    // First half fades to black through source tint,
                                    // second half fades from black through target tint
                                    let (tint_r, tint_g, tint_b) = if self.time_transition_alpha > 0.5 {
                                        // Fading to black: use source year tint, fade toward black
                                        let (sr, sg, sb) = time_tint_color(self.time_transition_source, self.timeline.present_year);
                                        let t = (self.time_transition_alpha - 0.5) * 2.0;
                                        let r = (sr as f32 * (1.0 - t)) as u8;
                                        let g = (sg as f32 * (1.0 - t)) as u8;
                                        let b = (sb as f32 * (1.0 - t)) as u8;
                                        (r, g, b)
                                    } else {
                                        // Fading from black: use target year tint, fade from black
                                        let target_year = self.timeline.active_year;
                                        let (tr, tg, tb) = time_tint_color(target_year, self.timeline.present_year);
                                        let t = self.time_transition_alpha * 2.0;
                                        let r = (tr as f32 * (1.0 - t)) as u8;
                                        let g = (tg as f32 * (1.0 - t)) as u8;
                                        let b = (tb as f32 * (1.0 - t)) as u8;
                                        (r, g, b)
                                    };
                                    egui::Area::new(egui::Id::new("time_transition"))
                                        .fixed_pos([0.0, 0.0])
                                        .order(egui::Order::Foreground)
                                        .show(&ctx, |ui| {
                                            let screen_rect = ctx.screen_rect();
                                            ui.painter().rect_filled(
                                                screen_rect,
                                                0.0,
                                                egui::Color32::from_rgba_unmultiplied(tint_r, tint_g, tint_b, alpha),
                                            );
                                            ui.allocate_space(screen_rect.size());
                                        });
                                }

                                // Debug overlay (F3)
                                if self.debug_visible {
                                    let player_pos = self.player.as_ref().map(|p| p.position()).unwrap_or(Vec3::ZERO);
                                    let player_grounded = self.player.as_ref().map(|p| p.is_grounded()).unwrap_or(false);
                                    let chunk_height = self.chunk_manager.as_ref()
                                        .map(|cm| cm.height_at(player_pos.x, player_pos.z))
                                        .unwrap_or(0.0);
                                    let chunks_loaded = self.chunk_manager.as_ref()
                                        .map(|cm| cm.loaded_count())
                                        .unwrap_or(0);

                                    egui::Window::new("Debug")
                                        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -40.0])
                                        .resizable(false)
                                        .collapsible(true)
                                        .default_width(280.0)
                                        .show(&ctx, |ui| {
                                            ui.heading("Player");
                                            ui.label(format!("Position: ({:.1}, {:.1}, {:.1})", player_pos.x, player_pos.y, player_pos.z));
                                            ui.label(format!("Grounded: {}", player_grounded));
                                            ui.label(format!("Terrain height: {:.2}", chunk_height));
                                            ui.label(format!("Above terrain: {:.2}", player_pos.y - chunk_height));

                                            ui.separator();
                                            ui.heading("Chunks");
                                            ui.label(format!("Loaded: {}", chunks_loaded));
                                            if let Some(cm) = &self.chunk_manager {
                                                let pc = cm.player_chunk(player_pos);
                                                ui.label(format!("Player chunk: ({}, {})", pc.x, pc.z));
                                            }

                                            ui.separator();
                                            ui.heading("Rendering");
                                            ui.checkbox(&mut self.debug_wireframe, "Wireframe terrain");
                                            ui.checkbox(&mut self.debug_colliders, "Show colliders");

                                            ui.separator();
                                            ui.heading("World");
                                            ui.label(format!("Year: {}", self.timeline.year_label()));
                                            ui.label(format!("Time: {} ({})", self.time_of_day.formatted_time(), self.time_of_day.period_name()));
                                            ui.label(format!("Weather: {}", self.weather.current.name()));

                                            // NPC info
                                            ui.separator();
                                            ui.heading("NPCs");
                                            if let Some(npc_mgr) = &self.npc_manager {
                                                ui.label(format!("Total: {}", npc_mgr.count()));
                                                ui.label(format!("Friendly: {}", npc_mgr.count_by_faction(infinite_game::NpcFaction::Friendly)));
                                                ui.label(format!("Hostile: {}", npc_mgr.count_by_faction(infinite_game::NpcFaction::Hostile)));
                                                ui.label(format!("Neutral: {}", npc_mgr.count_by_faction(infinite_game::NpcFaction::Neutral)));
                                            }
                                            ui.label(format!("Player HP: {:.0}/{:.0}", self.player_combat.current_hp, self.player_combat.max_hp));

                                            // Time travel debug buttons
                                            ui.separator();
                                            ui.heading("Time Travel");
                                            let test_years: &[i64] = &[-5000, -1000, 1000, 2025, 2500, 3500];
                                            for &year in test_years {
                                                let label_text = format_year(year);
                                                let is_active = self.timeline.active_year == year;
                                                let label = if is_active {
                                                    format!("> {} <", label_text)
                                                } else {
                                                    label_text
                                                };
                                                if ui.button(&label).clicked() && !is_active && !self.time_transitioning {
                                                    self.time_transition_source = self.timeline.active_year;
                                                    self.pending_time_transition = Some(year);
                                                    self.time_transitioning = true;
                                                    self.time_transition_alpha = 0.0;
                                                }
                                            }
                                        });
                                }

                                StateTransition::None
                            }
                            ApplicationState::Exiting => StateTransition::None,
                        };

                        pending_transition = transition;
                    });
            });
        }

        // Process save/load actions (deferred to avoid borrow conflicts in the UI closure)
        if let Some((menu_transition, action)) = save_load_pending_action {
            let mut result_transition = menu_transition;
            match action {
                SaveLoadAction::SaveNew(name) => {
                    let data = self.gather_save_data(&name);
                    match save::save_to_slot(&name, &data) {
                        Ok(()) => {
                            self.notification_text = Some(format!("Saved: {}", name));
                            self.notification_timer = 2.0;
                        }
                        Err(e) => {
                            self.notification_text = Some(format!("Save failed: {}", e));
                            self.notification_timer = 3.0;
                        }
                    }
                    if let Some(menu) = &mut self.save_load_menu {
                        menu.mark_needs_refresh();
                    }
                }
                SaveLoadAction::Load(filename) => {
                    match save::load_from_slot(&filename) {
                        Ok(data) => {
                            self.restore_from_save(data);
                            self.notification_text = Some("Game Loaded".to_string());
                            self.notification_timer = 2.0;
                            self.save_load_menu = None;
                            result_transition = StateTransition::Replace(ApplicationState::Playing);
                        }
                        Err(e) => {
                            self.notification_text = Some(format!("Load failed: {}", e));
                            self.notification_timer = 3.0;
                        }
                    }
                }
                SaveLoadAction::Delete(filename) => {
                    if let Err(e) = save::delete_slot(&filename) {
                        self.notification_text = Some(format!("Delete failed: {}", e));
                        self.notification_timer = 3.0;
                    }
                    if let Some(menu) = &mut self.save_load_menu {
                        menu.mark_needs_refresh();
                    }
                }
                SaveLoadAction::None => {}
            }
            if matches!(result_transition, StateTransition::Pop) {
                self.save_load_menu = None;
            }
            if !matches!(result_transition, StateTransition::None) {
                pending_transition = result_transition;
            }
        }

        // Apply state transition after UI is done
        if !matches!(pending_transition, StateTransition::None) {
            self.apply_transition(pending_transition);
        }

        // Save settings if needed
        if should_save_settings {
            if let Err(e) = self.settings.save() {
                tracing::error!("Failed to save settings: {}", e);
            }
        }

        // Build command buffer and submit
        let render_ctx = self.render_ctx.as_mut().unwrap();
        let gui = self.gui.as_mut().unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            render_ctx.command_buffer_allocator.clone(),
            render_ctx.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Get sky colors from time of day, modified by weather
        let sky_colors = self.time_of_day.sky_colors();
        let weather_tint = self.weather.sky_tint();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([
                            sky_colors.horizon.x * weather_tint[0] * 0.3,
                            sky_colors.horizon.y * weather_tint[1] * 0.3,
                            sky_colors.horizon.z * weather_tint[2] * 0.3,
                            1.0
                        ].into()),
                        Some(1.0f32.into()), // Depth clear value
                    ],
                    ..RenderPassBeginInfo::framebuffer(
                        render_ctx.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap();

        // === SUBPASS 0: 3D Scene Rendering ===
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [window_size.width as f32, window_size.height as f32],
            depth_range: 0.0..=1.0,
        };

        // Calculate view and projection matrices
        let aspect_ratio = window_size.width as f32 / window_size.height as f32;
        let (view_matrix, mut projection_matrix) = if matches!(self.app_state, ApplicationState::Playing) {
            if let Some(camera) = &self.camera {
                (camera.view_matrix(), camera.projection_matrix(aspect_ratio, 60.0))
            } else {
                default_matrices(aspect_ratio)
            }
        } else {
            default_matrices(aspect_ratio)
        };
        // Vulkan Y-axis is inverted compared to OpenGL, flip it in projection
        projection_matrix.y_axis.y *= -1.0;

        // Get lighting from time of day and weather
        let sun_direction = self.time_of_day.light_direction();
        let sun_intensity = self.time_of_day.light_intensity() * self.weather.sun_modifier();
        let ambient_intensity = 0.3 * self.weather.ambient_modifier();

        // Set viewport and scissor for all 3D rendering in subpass 0
        // Both must be set when using dynamic state
        let scissor = vulkano::pipeline::graphics::viewport::Scissor {
            offset: [0, 0],
            extent: [window_size.width, window_size.height],
        };
        builder
            .set_viewport(0, [viewport.clone()].into_iter().collect())
            .unwrap()
            .set_scissor(0, [scissor.clone()].into_iter().collect())
            .unwrap();

        // Render 3D scene if playing
        if matches!(self.app_state, ApplicationState::Playing) {
            // Log 3D state (first frame only via static flag)
            static LOGGED_PLAYING_STATE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !LOGGED_PLAYING_STATE.swap(true, std::sync::atomic::Ordering::Relaxed) {
                let has_sky = render_ctx.sky_pipeline.is_some() && render_ctx.sky_mesh.is_some();
                let has_terrain = render_ctx.basic_pipeline.is_some() && render_ctx.terrain_mesh.is_some();
                let has_capsule = render_ctx.basic_pipeline.is_some() && render_ctx.capsule_mesh.is_some() && self.player.is_some();
                info!("Playing 3D state: sky={}, terrain={}, capsule={}, player={}",
                      has_sky, has_terrain, has_capsule, self.player.is_some());
            }

            // Render sky dome
            if let (Some(sky_pipeline), Some(sky_mesh)) = (&render_ctx.sky_pipeline, &render_ctx.sky_mesh) {
                let sky_push = SkyPushConstants::new(
                    view_matrix,
                    projection_matrix,
                    sun_direction,
                    sun_intensity,
                    &infinite_render::SkyColors {
                        zenith: sky_colors.zenith * Vec3::from_array(weather_tint),
                        horizon: sky_colors.horizon * Vec3::from_array(weather_tint),
                        sun_glow: sky_colors.sun_glow,
                        sun_size: sky_colors.sun_size,
                    },
                    self.time_of_day.time_hours,
                );

                unsafe {
                    builder
                        .bind_pipeline_graphics(sky_pipeline.clone())
                        .unwrap()
                        .push_constants(sky_pipeline.layout().clone(), 0, sky_push)
                        .unwrap()
                        .bind_vertex_buffers(0, sky_mesh.vertex_buffer.clone())
                        .unwrap()
                        .bind_index_buffer(sky_mesh.index_buffer.clone())
                        .unwrap()
                        .draw_indexed(sky_mesh.index_count, 1, 0, 0, 0)
                        .unwrap();
                }
            }

            // Render chunk terrain meshes
            {
                let terrain_pipeline = if self.debug_wireframe {
                    render_ctx.wireframe_pipeline.as_ref().or(render_ctx.basic_pipeline.as_ref())
                } else {
                    render_ctx.basic_pipeline.as_ref()
                };

                if let (Some(pipeline), Some(chunk_manager)) = (terrain_pipeline, &self.chunk_manager) {
                    let chunk_size = chunk_manager.config.chunk_size;

                    for chunk in chunk_manager.loaded_chunks() {
                        if let Some(mesh) = render_ctx.chunk_meshes.get(&chunk.coord) {
                            let origin = chunk.coord.world_center(chunk_size);
                            let model = Mat4::from_translation(origin);

                            let push = BasicPushConstants::new(
                                model,
                                view_matrix,
                                projection_matrix,
                                sun_direction,
                                sun_intensity,
                                Vec3::new(1.0, 0.95, 0.85),
                                ambient_intensity,
                            );

                            unsafe {
                                builder
                                    .bind_pipeline_graphics(pipeline.clone())
                                    .unwrap()
                                    .push_constants(pipeline.layout().clone(), 0, push)
                                    .unwrap()
                                    .bind_vertex_buffers(0, mesh.vertex_buffer.clone())
                                    .unwrap()
                                    .bind_index_buffer(mesh.index_buffer.clone())
                                    .unwrap()
                                    .draw_indexed(mesh.index_count, 1, 0, 0, 0)
                                    .unwrap();
                            }
                        }
                    }
                }
            }

            // Also render legacy single terrain if present (fallback)
            if let Some(terrain_mesh) = &render_ctx.terrain_mesh {
                if let Some(pipeline) = render_ctx.basic_pipeline.as_ref() {
                    let push = BasicPushConstants::new(
                        Mat4::IDENTITY,
                        view_matrix,
                        projection_matrix,
                        sun_direction,
                        sun_intensity,
                        Vec3::new(1.0, 0.95, 0.85),
                        ambient_intensity,
                    );
                    unsafe {
                        builder
                            .bind_pipeline_graphics(pipeline.clone())
                            .unwrap()
                            .push_constants(pipeline.layout().clone(), 0, push)
                            .unwrap()
                            .bind_vertex_buffers(0, terrain_mesh.vertex_buffer.clone())
                            .unwrap()
                            .bind_index_buffer(terrain_mesh.index_buffer.clone())
                            .unwrap()
                            .draw_indexed(terrain_mesh.index_count, 1, 0, 0, 0)
                            .unwrap();
                    }
                }
            }

            // Render player capsule (debug visualization)
            if let (Some(basic_pipeline), Some(capsule_mesh), Some(player)) =
                (&render_ctx.basic_pipeline, &render_ctx.capsule_mesh, &self.player)
            {
                let player_pos = player.position();
                let model = Mat4::from_translation(player_pos);

                let push = BasicPushConstants::new(
                    model,
                    view_matrix,
                    projection_matrix,
                    sun_direction,
                    sun_intensity,
                    Vec3::new(1.0, 0.95, 0.85),
                    ambient_intensity,
                );

                unsafe {
                    builder
                        .bind_pipeline_graphics(basic_pipeline.clone())
                        .unwrap()
                        .push_constants(basic_pipeline.layout().clone(), 0, push)
                        .unwrap()
                        .bind_vertex_buffers(0, capsule_mesh.vertex_buffer.clone())
                        .unwrap()
                        .bind_index_buffer(capsule_mesh.index_buffer.clone())
                        .unwrap()
                        .draw_indexed(capsule_mesh.index_count, 1, 0, 0, 0)
                        .unwrap();
                }
            }

            // Render NPC capsules
            if let (Some(basic_pipeline), Some(npc_mesh)) =
                (&render_ctx.basic_pipeline, &render_ctx.npc_capsule_mesh)
            {
                if let Some(npc_manager) = &self.npc_manager {
                    for npc in npc_manager.npcs_iter() {
                        let model = Mat4::from_translation(npc.position);
                        let color = npc.data.color;

                        let push = BasicPushConstants::new(
                            model,
                            view_matrix,
                            projection_matrix,
                            sun_direction,
                            sun_intensity,
                            Vec3::new(color[0], color[1], color[2]),
                            ambient_intensity,
                        );

                        unsafe {
                            builder
                                .bind_pipeline_graphics(basic_pipeline.clone())
                                .unwrap()
                                .push_constants(basic_pipeline.layout().clone(), 0, push)
                                .unwrap()
                                .bind_vertex_buffers(0, npc_mesh.vertex_buffer.clone())
                                .unwrap()
                                .bind_index_buffer(npc_mesh.index_buffer.clone())
                                .unwrap()
                                .draw_indexed(npc_mesh.index_count, 1, 0, 0, 0)
                                .unwrap();
                        }
                    }
                }
            }

            // Debug: render collider wireframes
            if self.debug_colliders {
                if let Some(wireframe_pipeline) = &render_ctx.wireframe_pipeline {
                    // Render player collider capsule as wireframe
                    if let (Some(capsule_mesh), Some(player)) = (&render_ctx.capsule_mesh, &self.player) {
                        let player_pos = player.position();
                        let model = Mat4::from_translation(player_pos);

                        let push = BasicPushConstants::new(
                            model,
                            view_matrix,
                            projection_matrix,
                            Vec3::new(0.0, 1.0, 0.0), // green light direction for debug color
                            0.0,                        // no sun influence
                            Vec3::new(0.0, 1.0, 0.0),  // green wireframe
                            1.0,                        // full ambient for uniform color
                        );

                        unsafe {
                            builder
                                .bind_pipeline_graphics(wireframe_pipeline.clone())
                                .unwrap()
                                .push_constants(wireframe_pipeline.layout().clone(), 0, push)
                                .unwrap()
                                .bind_vertex_buffers(0, capsule_mesh.vertex_buffer.clone())
                                .unwrap()
                                .bind_index_buffer(capsule_mesh.index_buffer.clone())
                                .unwrap()
                                .draw_indexed(capsule_mesh.index_count, 1, 0, 0, 0)
                                .unwrap();
                        }
                    }

                    // Render heightfield bounds as wireframe box
                    if let Some(terrain) = &self.terrain {
                        let half_size = terrain.config.size / 2.0;
                        let min_h = terrain.min_height;
                        let max_h = terrain.max_height;

                        // Create or reuse a debug box mesh for the heightfield bounds
                        if render_ctx.debug_capsule_mesh.is_none() {
                            let box_mesh = Mesh::capsule(
                                max_h - min_h,
                                half_size.min(10.0),
                                4, 4,
                                [0.0, 1.0, 0.0, 0.3],
                            );
                            if let Ok(buffers) = create_mesh_buffers(
                                render_ctx.memory_allocator.clone(),
                                &box_mesh.vertices,
                                &box_mesh.indices,
                            ) {
                                render_ctx.debug_capsule_mesh = Some(buffers);
                            }
                        }
                    }
                }
            }
        }

        // Character creator 3D preview
        if matches!(self.app_state, ApplicationState::CharacterCreation) {
            // Regenerate capsule mesh if appearance changed
            if self.character_creator.appearance_dirty {
                let appearance = &self.character_creator.appearance;
                let mesh_data = infinite_render::Mesh::character_capsule(
                    appearance.body.height,
                    appearance.body.build,
                    appearance.body.shoulder_width,
                    appearance.body.hip_width,
                    appearance.skin.tone,
                    appearance.skin.undertone,
                );

                match create_mesh_buffers(
                    render_ctx.memory_allocator.clone(),
                    &mesh_data.vertices,
                    &mesh_data.indices,
                ) {
                    Ok(buffers) => {
                        render_ctx.capsule_mesh = Some(buffers);
                    }
                    Err(e) => {
                        tracing::error!("Failed to rebuild character mesh: {}", e);
                    }
                }
                self.character_creator.appearance_dirty = false;
            }

            // Check if we have a preview rect from egui
            let preview_rect: Option<[f32; 4]> = gui.egui_winit.egui_ctx().data(|data: &egui::util::IdTypeMap| {
                data.get_temp(egui::Id::new("character_preview_rect"))
            });

            // Log preview state (first frame only via static flag)
            static LOGGED_PREVIEW_STATE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !LOGGED_PREVIEW_STATE.swap(true, std::sync::atomic::Ordering::Relaxed) {
                let has_rect = preview_rect.is_some();
                let has_pipeline = render_ctx.basic_pipeline.is_some();
                let has_capsule = render_ctx.capsule_mesh.is_some();
                info!("Character preview state: rect={}, pipeline={}, capsule={}", has_rect, has_pipeline, has_capsule);
            }

            if let Some([px, py, pw, ph]) = preview_rect {
                // Log rect details once
                static LOGGED_RECT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                if !LOGGED_RECT.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    info!("Preview rect: pos=({}, {}), size=({}, {})", px, py, pw, ph);
                }
                if pw > 10.0 && ph > 10.0 {
                    // Set viewport and scissor to preview area
                    let preview_viewport = Viewport {
                        offset: [px, py],
                        extent: [pw, ph],
                        depth_range: 0.0..=1.0,
                    };
                    let preview_scissor = vulkano::pipeline::graphics::viewport::Scissor {
                        offset: [px as u32, py as u32],
                        extent: [pw as u32, ph as u32],
                    };
                    builder
                        .set_viewport(0, [preview_viewport].into_iter().collect())
                        .unwrap()
                        .set_scissor(0, [preview_scissor].into_iter().collect())
                        .unwrap();

                    // Calculate preview camera (orbit around capsule)
                    let rotation = self.character_creator.preview_rotation.to_radians();
                    let distance = 3.0 / self.character_creator.preview_zoom;
                    let cam_pos = Vec3::new(rotation.sin() * distance, 1.0, rotation.cos() * distance);
                    let target = Vec3::new(0.0, 0.9, 0.0);

                    let preview_view = Mat4::look_at_rh(cam_pos, target, Vec3::Y);
                    // Vulkan Y-axis is inverted compared to OpenGL, flip it in projection
                    let mut preview_proj = Mat4::perspective_rh(45f32.to_radians(), pw / ph, 0.1, 100.0);
                    preview_proj.y_axis.y *= -1.0;

                    // Render capsule with fixed lighting
                    if let (Some(basic_pipeline), Some(capsule_mesh)) =
                        (&render_ctx.basic_pipeline, &render_ctx.capsule_mesh)
                    {
                        // Log first draw call
                        static LOGGED_DRAW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                        if !LOGGED_DRAW.swap(true, std::sync::atomic::Ordering::Relaxed) {
                            info!("Drawing capsule preview: {} indices, viewport=({}, {}, {}, {})",
                                  capsule_mesh.index_count, px, py, pw, ph);
                        }
                        let push = BasicPushConstants::new(
                            Mat4::IDENTITY,
                            preview_view,
                            preview_proj,
                            Vec3::new(0.5, 0.8, 0.3).normalize(),
                            1.0,
                            Vec3::new(1.0, 0.95, 0.85),
                            0.3,
                        );

                        unsafe {
                            builder
                                .bind_pipeline_graphics(basic_pipeline.clone())
                                .unwrap()
                                .push_constants(basic_pipeline.layout().clone(), 0, push)
                                .unwrap()
                                .bind_vertex_buffers(0, capsule_mesh.vertex_buffer.clone())
                                .unwrap()
                                .bind_index_buffer(capsule_mesh.index_buffer.clone())
                                .unwrap()
                                .draw_indexed(capsule_mesh.index_count, 1, 0, 0, 0)
                                .unwrap();
                        }
                    }

                    // Reset viewport and scissor for UI
                    builder
                        .set_viewport(0, [viewport].into_iter().collect())
                        .unwrap()
                        .set_scissor(0, [scissor].into_iter().collect())
                        .unwrap();
                }
            }
        }

        // === SUBPASS 1: UI Overlay ===
        builder
            .next_subpass(
                SubpassEndInfo::default(),
                SubpassBeginInfo {
                    contents: SubpassContents::SecondaryCommandBuffers,
                    ..Default::default()
                },
            )
            .unwrap();

        // Draw egui
        let cb = gui.draw_on_subpass_image([window_size.width, window_size.height]);
        builder.execute_commands(cb).unwrap();

        builder.end_render_pass(Default::default()).unwrap();

        let command_buffer = builder.build().unwrap();

        // Submit
        let future = render_ctx
            .previous_frame_end
            .take()
            .unwrap_or_else(|| sync::now(render_ctx.device.clone()).boxed())
            .join(acquire_future)
            .then_execute(render_ctx.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                render_ctx.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    render_ctx.swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                render_ctx.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                render_ctx.recreate_swapchain = true;
                render_ctx.previous_frame_end = Some(sync::now(render_ctx.device.clone()).boxed());
            }
            Err(e) => {
                tracing::error!("Failed to flush future: {e}");
                render_ctx.previous_frame_end = Some(sync::now(render_ctx.device.clone()).boxed());
            }
        }
    }
}

impl ApplicationHandler for InfiniteApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("Application resumed, creating window...");

        let window_attrs = WindowAttributes::default()
            .with_title("Infinite")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.settings.video.width,
                self.settings.video.height,
            ));

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Failed to create window"),
        );

        let surface =
            Surface::from_window(self.instance.clone(), window.clone()).expect("Failed to create surface");

        // Select physical device
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = self
            .instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .expect("No suitable GPU found");

        info!(
            "Using GPU: {} ({:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        // Check for ray tracing support
        let rt_supported = physical_device
            .supported_extensions()
            .khr_ray_tracing_pipeline;
        if rt_supported {
            info!("Hardware ray tracing supported");
        } else {
            info!("Hardware ray tracing NOT supported - will use compute fallback");
        }

        // Create logical device
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                enabled_features: DeviceFeatures {
                    fill_mode_non_solid: true,
                    wide_lines: true,
                    ..DeviceFeatures::empty()
                },
                ..Default::default()
            },
        )
        .expect("Failed to create logical device");

        let queue = queues.next().unwrap();

        // Create allocators first (needed for depth buffer creation)
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        // Create swapchain and framebuffers (with depth buffer)
        let (swapchain, images, render_pass, framebuffers, depth_buffer) =
            Self::create_swapchain_and_framebuffers(
                device.clone(),
                surface.clone(),
                window.clone(),
                memory_allocator.clone(),
            )
            .expect("Failed to create swapchain");

        // Create 3D pipelines
        let basic_pipeline = create_basic_pipeline(device.clone(), render_pass.clone());
        if basic_pipeline.is_none() {
            tracing::error!("Failed to create basic 3D pipeline!");
        } else {
            info!("Basic 3D pipeline created successfully");
        }

        let sky_pipeline = create_sky_pipeline(device.clone(), render_pass.clone());
        if sky_pipeline.is_none() {
            tracing::error!("Failed to create sky pipeline!");
        } else {
            info!("Sky pipeline created successfully");
        }

        let wireframe_pipeline = create_wireframe_pipeline(device.clone(), render_pass.clone());
        if wireframe_pipeline.is_some() {
            info!("Wireframe debug pipeline created successfully");
        }

        // Create capsule mesh for player/preview
        let capsule_mesh_data = Mesh::capsule(1.8, 0.4, 16, 16, [0.6, 0.7, 0.8, 1.0]);
        let capsule_mesh = match create_mesh_buffers(
            memory_allocator.clone(),
            &capsule_mesh_data.vertices,
            &capsule_mesh_data.indices,
        ) {
            Ok(mesh) => {
                info!("Capsule mesh created: {} vertices, {} indices",
                      capsule_mesh_data.vertices.len(), capsule_mesh_data.indices.len());
                Some(mesh)
            }
            Err(e) => {
                tracing::error!("Failed to create capsule mesh: {}", e);
                None
            }
        };

        // Create sky dome mesh
        let sky_mesh_data = SkyMesh::dome(32, 16);
        let sky_mesh = match create_sky_mesh_buffers(
            memory_allocator.clone(),
            &sky_mesh_data.vertices,
            &sky_mesh_data.indices,
        ) {
            Ok(mesh) => {
                info!("Sky mesh created: {} vertices, {} indices",
                      sky_mesh_data.vertices.len(), sky_mesh_data.indices.len());
                Some(mesh)
            }
            Err(e) => {
                tracing::error!("Failed to create sky mesh: {}", e);
                None
            }
        };

        // Create egui renderer (subpass 1 - UI overlay)
        let gui = Gui::new_with_subpass(
            event_loop,
            surface.clone(),
            queue.clone(),
            Subpass::from(render_pass.clone(), 1).unwrap(),
            swapchain.image_format(),
            GuiConfig::default(),
        );

        self.window = Some(window);
        self.surface = Some(surface);
        self.render_ctx = Some(RenderContext {
            device,
            queue,
            swapchain,
            images,
            render_pass,
            framebuffers,
            memory_allocator,
            command_buffer_allocator,
            _descriptor_set_allocator: descriptor_set_allocator,
            recreate_swapchain: false,
            previous_frame_end: None,
            depth_buffer,
            basic_pipeline,
            sky_pipeline,
            wireframe_pipeline,
            capsule_mesh,
            terrain_mesh: None,
            chunk_meshes: HashMap::new(),
            npc_capsule_mesh: None,
            sky_mesh,
            debug_capsule_mesh: None,
        });
        self.gui = Some(gui);
        self.last_frame = Instant::now();

        info!("Window and Vulkan context created successfully with 3D rendering");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Pass events to egui first (only if not in Playing state or if cursor not captured)
        if !self.cursor_captured {
            if let Some(gui) = &mut self.gui {
                let _ = gui.update(&event);
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(_size) => {
                if let Some(render_ctx) = &mut self.render_ctx {
                    render_ctx.recreate_swapchain = true;
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        logical_key,
                        ..
                    },
                ..
            } => {
                // Handle ESC specially
                if logical_key == Key::Named(NamedKey::Escape) && state == ElementState::Pressed {
                    match &self.app_state {
                        ApplicationState::Playing => {
                            self.apply_transition(StateTransition::Push(ApplicationState::Paused));
                        }
                        ApplicationState::Paused => {
                            self.apply_transition(StateTransition::Pop);
                        }
                        ApplicationState::Settings { .. } => {
                            self.apply_transition(StateTransition::Pop);
                        }
                        ApplicationState::SaveLoad { .. } => {
                            self.save_load_menu = None;
                            self.apply_transition(StateTransition::Pop);
                        }
                        ApplicationState::CharacterCreation => {
                            self.apply_transition(StateTransition::Replace(
                                ApplicationState::MainMenu,
                            ));
                        }
                        _ => {}
                    }
                }

                // F3 toggles debug overlay (works in any state)
                if state == ElementState::Pressed {
                    if let PhysicalKey::Code(KeyCode::F3) = physical_key {
                        self.debug_visible = !self.debug_visible;
                        info!("Debug overlay: {}", if self.debug_visible { "ON" } else { "OFF" });
                    }
                }

                // Pass to input handler for game controls
                if matches!(self.app_state, ApplicationState::Playing) {
                    self.input_handler.handle_keyboard(physical_key, state);

                    // Debug keys for weather/time
                    if state == ElementState::Pressed {
                        if let PhysicalKey::Code(key_code) = physical_key {
                            match key_code {
                                KeyCode::KeyT => {
                                    // Toggle time speed
                                    self.time_of_day.paused = !self.time_of_day.paused;
                                    info!("Time of day: {} ({})",
                                        self.time_of_day.formatted_time(),
                                        if self.time_of_day.paused { "paused" } else { "running" }
                                    );
                                }
                                KeyCode::KeyY => {
                                    // Cycle weather
                                    self.weather.cycle_next();
                                    info!("Weather: {}", self.weather.current.name());
                                }
                                KeyCode::KeyU => {
                                    // Fast forward time by 1 hour
                                    self.time_of_day.set_time(self.time_of_day.time_hours + 1.0);
                                    info!("Time: {} ({})",
                                        self.time_of_day.formatted_time(),
                                        self.time_of_day.period_name()
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if matches!(self.app_state, ApplicationState::Playing) {
                    self.input_handler.handle_mouse_button(button, state);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if matches!(self.app_state, ApplicationState::Playing) {
                    self.input_handler.handle_scroll(delta);
                }
            }
            WindowEvent::RedrawRequested => {
                // Update timing
                let now = Instant::now();
                let delta = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                // Update game logic
                self.update(delta);

                // Check for exit state
                if matches!(self.app_state, ApplicationState::Exiting) {
                    event_loop.exit();
                    return;
                }

                // Render
                self.render();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        // Handle raw mouse motion for camera (only when cursor is captured)
        if let DeviceEvent::MouseMotion { delta } = event {
            if matches!(self.app_state, ApplicationState::Playing) && self.cursor_captured {
                self.input_handler.handle_mouse_motion(delta);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    info!("Starting Infinite engine...");

    // Create event loop
    let event_loop = EventLoop::new().context("Failed to create event loop")?;
    event_loop.set_control_flow(ControlFlow::Poll);

    // Load Vulkan library
    let library = VulkanLibrary::new().context("Failed to load Vulkan library")?;

    // Get required extensions for windowing
    let mut required_extensions = Surface::required_extensions(&event_loop)
        .context("Failed to get required surface extensions")?;

    // Enable debug utils extension for validation layer messages
    required_extensions.ext_debug_utils = true;

    // Enable validation layers in debug builds
    let enabled_layers: Vec<String> = if cfg!(debug_assertions) {
        // Check if validation layer is available
        let available_layers: Vec<_> = library
            .layer_properties()
            .map(|iter| iter.map(|l| l.name().to_owned()).collect())
            .unwrap_or_default();

        if available_layers.iter().any(|l| l == "VK_LAYER_KHRONOS_validation") {
            info!("Enabling Vulkan validation layer");
            vec!["VK_LAYER_KHRONOS_validation".to_owned()]
        } else {
            info!("Vulkan validation layer not available - install vulkan-validation-layers package");
            vec![]
        }
    } else {
        vec![]
    };

    // Create Vulkan instance
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            enabled_extensions: required_extensions,
            enabled_layers,
            ..Default::default()
        },
    )
    .context("Failed to create Vulkan instance")?;

    // Set up debug messenger to receive validation layer messages
    let _debug_messenger = if cfg!(debug_assertions) {
        use vulkano::instance::debug::DebugUtilsMessengerCallback;
        unsafe {
            let callback = DebugUtilsMessengerCallback::new(
                |severity, ty, data| {
                    let severity_str = if severity.intersects(DebugUtilsMessageSeverity::ERROR) {
                        "ERROR"
                    } else if severity.intersects(DebugUtilsMessageSeverity::WARNING) {
                        "WARNING"
                    } else {
                        "INFO"
                    };
                    eprintln!(
                        "[Vulkan {}] {:?}: {}",
                        severity_str,
                        ty,
                        data.message
                    );
                },
            );
            DebugUtilsMessenger::new(
                instance.clone(),
                DebugUtilsMessengerCreateInfo::user_callback(callback),
            )
            .ok()
        }
    } else {
        None
    };

    // Create application
    let mut app = InfiniteApp::new(instance);

    // Run event loop
    event_loop
        .run_app(&mut app)
        .context("Event loop error")?;

    info!("Infinite engine shutting down");
    Ok(())
}

// === Helper Functions for 3D Rendering ===

/// Create the basic 3D rendering pipeline
fn create_basic_pipeline(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
) -> Option<Arc<GraphicsPipeline>> {
    // Compile shaders using vulkano_shaders
    mod basic_vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "assets/shaders/basic.vert",
        }
    }

    mod basic_fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "assets/shaders/basic.frag",
        }
    }

    let vs = basic_vs::load(device.clone()).ok()?;
    let fs = basic_fs::load(device.clone()).ok()?;

    let vs_entry = vs.entry_point("main")?;
    let fs_entry = fs.entry_point("main")?;

    let vertex_input_state = [Vertex3D::per_vertex()]
        .definition(&vs_entry)
        .ok()?;

    let stages = [
        PipelineShaderStageCreateInfo::new(vs_entry),
        PipelineShaderStageCreateInfo::new(fs_entry),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .ok()?,
    )
    .ok()?;

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::None, // Disable culling for debugging
                front_face: FrontFace::CounterClockwise,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                1,
                ColorBlendAttachmentState::default(),
            )),
            dynamic_state: [DynamicState::Viewport, DynamicState::Scissor].into_iter().collect(),
            subpass: Some(Subpass::from(render_pass, 0).unwrap().into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .ok()
}

/// Create a wireframe rendering pipeline (same as basic but with PolygonMode::Line)
fn create_wireframe_pipeline(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
) -> Option<Arc<GraphicsPipeline>> {
    mod wireframe_vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "assets/shaders/basic.vert",
        }
    }

    mod wireframe_fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "assets/shaders/basic.frag",
        }
    }

    let vs = wireframe_vs::load(device.clone()).ok()?;
    let fs = wireframe_fs::load(device.clone()).ok()?;

    let vs_entry = vs.entry_point("main")?;
    let fs_entry = fs.entry_point("main")?;

    let vertex_input_state = [Vertex3D::per_vertex()]
        .definition(&vs_entry)
        .ok()?;

    let stages = [
        PipelineShaderStageCreateInfo::new(vs_entry),
        PipelineShaderStageCreateInfo::new(fs_entry),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .ok()?,
    )
    .ok()?;

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::None,
                front_face: FrontFace::CounterClockwise,
                polygon_mode: PolygonMode::Line,
                line_width: 2.0,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                1,
                ColorBlendAttachmentState::default(),
            )),
            dynamic_state: [DynamicState::Viewport, DynamicState::Scissor].into_iter().collect(),
            subpass: Some(Subpass::from(render_pass, 0).unwrap().into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .ok()
}

/// Create the sky dome rendering pipeline
fn create_sky_pipeline(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
) -> Option<Arc<GraphicsPipeline>> {
    mod sky_vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "assets/shaders/sky.vert",
        }
    }

    mod sky_fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "assets/shaders/sky.frag",
        }
    }

    let vs = sky_vs::load(device.clone()).ok()?;
    let fs = sky_fs::load(device.clone()).ok()?;

    let vs_entry = vs.entry_point("main")?;
    let fs_entry = fs.entry_point("main")?;

    let vertex_input_state = [SkyVertex::per_vertex()]
        .definition(&vs_entry)
        .ok()?;

    let stages = [
        PipelineShaderStageCreateInfo::new(vs_entry),
        PipelineShaderStageCreateInfo::new(fs_entry),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .ok()?,
    )
    .ok()?;

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::Front, // Render inside of sphere
                front_face: FrontFace::CounterClockwise,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: false, // Don't write to depth for sky
                    compare_op: vulkano::pipeline::graphics::depth_stencil::CompareOp::LessOrEqual,
                }),
                ..Default::default()
            }),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                1,
                ColorBlendAttachmentState::default(),
            )),
            dynamic_state: [DynamicState::Viewport, DynamicState::Scissor].into_iter().collect(),
            subpass: Some(Subpass::from(render_pass, 0).unwrap().into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .ok()
}

/// Get tint color for time transitions based on how far from the present
fn time_tint_color(year: i64, present_year: i64) -> (u8, u8, u8) {
    let years_from_present = year - present_year;
    if years_from_present == 0 {
        return (0, 0, 0); // Present: neutral black
    }

    let abs_years = years_from_present.unsigned_abs() as f32;

    if years_from_present < 0 {
        // Past: warm amber tones, more intense the further back
        let t = (abs_years / 5000.0).min(1.0);
        let r = (120.0 + t * 60.0) as u8;   // 120..180
        let g = (80.0 + t * 40.0) as u8;    // 80..120
        let b = (40.0) as u8;                // stays warm
        (r, g, b)
    } else {
        // Future: cool blue tones, more intense the further forward
        let t = (abs_years / 3000.0).min(1.0);
        let r = (40.0 + t * 20.0) as u8;    // 40..60
        let g = (80.0 + t * 80.0) as u8;    // 80..160
        let b = (160.0 + t * 40.0) as u8;   // 160..200
        (r, g, b)
    }
}

/// Create GPU buffers for a mesh
fn create_mesh_buffers(
    memory_allocator: Arc<StandardMemoryAllocator>,
    vertices: &[Vertex3D],
    indices: &[u32],
) -> Result<MeshBuffers> {
    let vertex_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vertices.iter().copied(),
    )
    .context("Failed to create vertex buffer")?;

    let index_buffer = Buffer::from_iter(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        indices.iter().copied(),
    )
    .context("Failed to create index buffer")?;

    Ok(MeshBuffers {
        vertex_buffer,
        index_buffer,
        index_count: indices.len() as u32,
    })
}

/// Create GPU buffers for a sky mesh
fn create_sky_mesh_buffers(
    memory_allocator: Arc<StandardMemoryAllocator>,
    vertices: &[SkyVertex],
    indices: &[u32],
) -> Result<SkyMeshBuffers> {
    let vertex_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vertices.iter().copied(),
    )
    .context("Failed to create sky vertex buffer")?;

    let index_buffer = Buffer::from_iter(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        indices.iter().copied(),
    )
    .context("Failed to create sky index buffer")?;

    Ok(SkyMeshBuffers {
        vertex_buffer,
        index_buffer,
        index_count: indices.len() as u32,
    })
}

/// Return default view and projection matrices
fn default_matrices(aspect_ratio: f32) -> (Mat4, Mat4) {
    let view = Mat4::look_at_rh(
        Vec3::new(0.0, 5.0, 10.0),
        Vec3::ZERO,
        Vec3::Y,
    );
    let projection = Mat4::perspective_rh(45f32.to_radians(), aspect_ratio, 0.1, 1000.0);
    (view, projection)
}
