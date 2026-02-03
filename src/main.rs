//! Infinite - A Vulkan-based game engine with ray tracing
//!
//! This is the main entry point for the Infinite engine and game.

mod character;
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
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
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
use infinite_core::{GameTime, Timeline};
use infinite_game::{CameraController, InputHandler, PlayerController};
use infinite_physics::PhysicsWorld;
use infinite_render::{BasicPushConstants, Mesh, SkyMesh, SkyPushConstants, Vertex3D, SkyVertex};
use infinite_world::{Terrain, TerrainConfig, TimeOfDay, Weather, WeatherState};

use crate::character::CharacterData;
use crate::settings::GameSettings;
use crate::state::{ApplicationState, StateTransition};
use crate::ui::{CharacterCreator, LoadingScreen, MainMenu, PauseMenu, SettingsMenu};

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
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    // Depth buffer
    depth_buffer: Arc<ImageView>,

    // 3D pipelines
    basic_pipeline: Option<Arc<GraphicsPipeline>>,
    sky_pipeline: Option<Arc<GraphicsPipeline>>,

    // Mesh buffers
    capsule_mesh: Option<MeshBuffers>,
    terrain_mesh: Option<MeshBuffers>,
    sky_mesh: Option<SkyMeshBuffers>,
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
    /// Terrain data
    terrain: Option<Terrain>,
    /// Time of day system
    time_of_day: TimeOfDay,
    /// Weather system
    weather: Weather,
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
            character_creator: CharacterCreator::new(),
            current_character: None,
            loading_timer: 0.0,
            physics_world: None,
            player: None,
            camera: None,
            input_handler: InputHandler::new(),
            cursor_captured: false,

            terrain: None,
            time_of_day: TimeOfDay::default(),
            weather: Weather::default(),
        }
    }

    /// Initialize game systems when entering Playing state
    fn init_game_systems(&mut self) {
        // Create physics world
        let mut physics = PhysicsWorld::new();

        // Generate terrain
        let terrain_config = TerrainConfig {
            size: 100.0,
            subdivisions: 64,
            max_height: 5.0,
            noise_scale: 0.02,
            seed: 42,
            ..Default::default()
        };
        let terrain = Terrain::generate(terrain_config);

        // Create ground at base terrain level
        physics.create_ground(terrain.min_height - 0.5);

        // Create some test obstacles
        physics.create_static_box(Vec3::new(2.0, 1.0, 2.0), Vec3::new(10.0, 1.0 + terrain.height_at(10.0, 0.0), 0.0));
        physics.create_static_box(Vec3::new(1.0, 2.0, 1.0), Vec3::new(-5.0, 2.0 + terrain.height_at(-5.0, 5.0), 5.0));

        // Create terrain mesh buffers
        if let Some(render_ctx) = &mut self.render_ctx {
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
                render_ctx.terrain_mesh = Some(buffers);
            }
        }

        self.terrain = Some(terrain);

        // Create player - spawn above terrain center
        let spawn_height = self.terrain.as_ref().map(|t| t.height_at(0.0, 0.0)).unwrap_or(0.0);
        let mut player = PlayerController::new();
        player.spawn(&mut physics, Vec3::new(0.0, spawn_height + 2.0, 0.0));

        // Create camera
        let camera = CameraController::new();

        self.physics_world = Some(physics);
        self.player = Some(player);
        self.camera = Some(camera);

        info!("Game systems initialized with terrain");
    }

    /// Cleanup game systems when leaving Playing state
    fn cleanup_game_systems(&mut self) {
        self.physics_world = None;
        self.player = None;
        self.camera = None;
        self.terrain = None;

        // Clear terrain mesh
        if let Some(render_ctx) = &mut self.render_ctx {
            render_ctx.terrain_mesh = None;
        }

        info!("Game systems cleaned up");
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
                            "Loading complete - Era: {}",
                            self.timeline.current_era().name()
                        );
                    }
                }
            }
            ApplicationState::Playing => {
                // Update cursor capture
                self.update_cursor_capture(true);

                // Update world systems
                self.time_of_day.update(delta);
                self.weather.update(delta);

                // Fixed timestep physics update
                let fixed_dt = self.game_time.config.fixed_timestep;
                let steps = self.game_time.fixed_steps();

                for _ in 0..steps {
                    if let (Some(physics), Some(player), Some(camera)) =
                        (&mut self.physics_world, &mut self.player, &self.camera)
                    {
                        // Update player with input
                        player.fixed_update(
                            physics,
                            &self.input_handler.state,
                            camera.yaw,
                            fixed_dt,
                        );

                        // Step physics
                        physics.step();
                    }
                }

                // Variable timestep camera update
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
                if matches!(old_state, ApplicationState::Playing | ApplicationState::Paused) {
                    self.cleanup_game_systems();
                    self.current_character = None;
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

        // Collect data needed for UI rendering
        let era_name = self.timeline.current_era().name().to_string();
        let game_time = self.game_time.total_time;

        if let Some(gui) = &mut self.gui {
            gui.immediate_ui(|gui| {
                let ctx = gui.context();

                // Dark theme background
                let mut style = (*ctx.style()).clone();
                style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 40);
                style.visuals.panel_fill = egui::Color32::from_rgb(30, 30, 40);
                ctx.set_style(style);

                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(20, 20, 30)))
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
                                            egui::RichText::new("WASD: Move | Space: Jump | Shift: Sprint | Scroll: Zoom | ESC: Pause | T: Pause Time | Y: Weather | U: +1 Hour")
                                                .color(egui::Color32::from_rgba_unmultiplied(150, 150, 170, 200))
                                                .font(egui::FontId::proportional(12.0)),
                                        );
                                    });

                                StateTransition::None
                            }
                            ApplicationState::Exiting => StateTransition::None,
                        };

                        pending_transition = transition;
                    });
            });
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
        let (view_matrix, projection_matrix) = if matches!(self.app_state, ApplicationState::Playing) {
            if let Some(camera) = &self.camera {
                (camera.view_matrix(), camera.projection_matrix(aspect_ratio, 60.0))
            } else {
                default_matrices(aspect_ratio)
            }
        } else {
            default_matrices(aspect_ratio)
        };

        // Get lighting from time of day and weather
        let sun_direction = self.time_of_day.light_direction();
        let sun_intensity = self.time_of_day.light_intensity() * self.weather.sun_modifier();
        let ambient_intensity = 0.3 * self.weather.ambient_modifier();

        // Render 3D scene if playing
        if matches!(self.app_state, ApplicationState::Playing) {
            builder.set_viewport(0, [viewport.clone()].into_iter().collect()).unwrap();

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

            // Render terrain
            if let (Some(basic_pipeline), Some(terrain_mesh)) = (&render_ctx.basic_pipeline, &render_ctx.terrain_mesh) {
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
                        .bind_pipeline_graphics(basic_pipeline.clone())
                        .unwrap()
                        .push_constants(basic_pipeline.layout().clone(), 0, push)
                        .unwrap()
                        .bind_vertex_buffers(0, terrain_mesh.vertex_buffer.clone())
                        .unwrap()
                        .bind_index_buffer(terrain_mesh.index_buffer.clone())
                        .unwrap()
                        .draw_indexed(terrain_mesh.index_count, 1, 0, 0, 0)
                        .unwrap();
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
        }

        // Character creator 3D preview
        if matches!(self.app_state, ApplicationState::CharacterCreation) {
            // Check if we have a preview rect from egui
            let preview_rect: Option<[f32; 4]> = gui.egui_winit.egui_ctx().data(|data: &egui::util::IdTypeMap| {
                data.get_temp(egui::Id::new("character_preview_rect"))
            });

            if let Some([px, py, pw, ph]) = preview_rect {
                if pw > 10.0 && ph > 10.0 {
                    // Set viewport to preview area
                    let preview_viewport = Viewport {
                        offset: [px, py],
                        extent: [pw, ph],
                        depth_range: 0.0..=1.0,
                    };
                    builder.set_viewport(0, [preview_viewport].into_iter().collect()).unwrap();

                    // Calculate preview camera (orbit around capsule)
                    let rotation = self.character_creator.preview_rotation.to_radians();
                    let distance = 3.0 / self.character_creator.preview_zoom;
                    let cam_pos = Vec3::new(rotation.sin() * distance, 1.0, rotation.cos() * distance);
                    let target = Vec3::new(0.0, 0.9, 0.0);

                    let preview_view = Mat4::look_at_rh(cam_pos, target, Vec3::Y);
                    let preview_proj = Mat4::perspective_rh(45f32.to_radians(), pw / ph, 0.1, 100.0);

                    // Render capsule with fixed lighting
                    if let (Some(basic_pipeline), Some(capsule_mesh)) =
                        (&render_ctx.basic_pipeline, &render_ctx.capsule_mesh)
                    {
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

                    // Reset viewport for UI
                    builder.set_viewport(0, [viewport].into_iter().collect()).unwrap();
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

        // Create capsule mesh for player/preview
        let capsule_mesh_data = Mesh::capsule(1.8, 0.4, 16, 16, [0.6, 0.7, 0.8, 1.0]);
        let capsule_mesh = create_mesh_buffers(
            memory_allocator.clone(),
            &capsule_mesh_data.vertices,
            &capsule_mesh_data.indices,
        )
        .ok();

        // Create sky dome mesh
        let sky_mesh_data = SkyMesh::dome(32, 16);
        let sky_mesh = create_sky_mesh_buffers(
            memory_allocator.clone(),
            &sky_mesh_data.vertices,
            &sky_mesh_data.indices,
        )
        .ok();

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
            descriptor_set_allocator,
            recreate_swapchain: false,
            previous_frame_end: None,
            depth_buffer,
            basic_pipeline,
            sky_pipeline,
            capsule_mesh,
            terrain_mesh: None,
            sky_mesh,
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
                        ApplicationState::CharacterCreation => {
                            self.apply_transition(StateTransition::Replace(
                                ApplicationState::MainMenu,
                            ));
                        }
                        _ => {}
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
    let required_extensions = Surface::required_extensions(&event_loop)
        .context("Failed to get required surface extensions")?;

    // Create Vulkan instance
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .context("Failed to create Vulkan instance")?;

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
                cull_mode: CullMode::Back,
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
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
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
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(Subpass::from(render_pass, 0).unwrap().into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .ok()
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
