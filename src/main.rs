//! Infinite - A Vulkan-based game engine with ray tracing
//!
//! This is the main entry point for the Infinite engine and game.

use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use vulkano::{
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    swapchain::Surface,
    VulkanLibrary,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use infinite_core::{GameTime, Timeline};

/// Application state
struct InfiniteApp {
    #[allow(dead_code)]
    instance: Arc<Instance>,
    #[allow(dead_code)]
    device: Arc<Device>,
    #[allow(dead_code)]
    surface: Arc<Surface>,
    game_time: GameTime,
    timeline: Timeline,
}

impl InfiniteApp {
    fn new(instance: Arc<Instance>, device: Arc<Device>, surface: Arc<Surface>) -> Self {
        Self {
            instance,
            device,
            surface,
            game_time: GameTime::default(),
            timeline: Timeline::default(),
        }
    }

    fn update(&mut self, delta: f32) {
        self.game_time.update(delta);
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
    let event_loop = EventLoop::new();

    // Load Vulkan library
    let library = VulkanLibrary::new().context("Failed to load Vulkan library")?;

    // Get required extensions for windowing
    let required_extensions = Surface::required_extensions(&event_loop);

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

    // Create window
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Infinite")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .context("Failed to create window")?,
    );

    // Create surface
    let surface =
        Surface::from_window(instance.clone(), window).context("Failed to create surface")?;

    // Select physical device
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .context("Failed to enumerate physical devices")?
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
        .context("No suitable GPU found")?;

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
    let (device, _queues) = Device::new(
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
    .context("Failed to create logical device")?;

    // Create application
    let mut app = InfiniteApp::new(instance, device, surface);
    let mut last_frame = std::time::Instant::now();

    info!(
        "Infinite engine started - Era: {}",
        app.timeline.current_era().name()
    );

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Window close requested");
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                info!("Window resized to {}x{}", size.width, size.height);
                // TODO: Recreate swapchain
            }
            Event::MainEventsCleared => {
                // Update game time
                let now = std::time::Instant::now();
                let delta = now.duration_since(last_frame).as_secs_f32();
                last_frame = now;
                app.update(delta);

                // TODO: Render frame
            }
            _ => {}
        }
    });
}
