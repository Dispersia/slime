use std::sync::Arc;

use wgpu::{Features, Limits};
use winit::{
    dpi::PhysicalSize,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

pub struct AppSettings {
    pub width: u32,
    pub height: u32,
    pub trail_weight: f32,
    pub num_agents: usize,
    pub steps_per_frame: usize,
    pub move_speed: f32,
    pub turn_speed: f32,
    pub sensor_angle_degrees: f32,
    pub sensor_offset_dst: f32,
    pub sensor_size: i32,
    pub decay_rate: f32,
    pub diffuse_rate: f32,
    pub agents_only: bool,
}

pub struct AppEventLoop {
    pub settings: AppSettings,
    pub event_loop: EventLoop<()>,
}

pub struct App {
    pub settings: AppSettings,
    pub window: Arc<Window>,
    pub size: PhysicalSize<u32>,
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl AppEventLoop {
    pub fn new(settings: AppSettings) -> Self {
        pretty_env_logger::init();

        let event_loop = EventLoop::new().expect("Failed to create event loop");

        AppEventLoop {
            settings,
            event_loop,
        }
    }
}

impl App {
    pub async fn new(settings: AppSettings, event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Slime"))
                .expect("Could not create window"),
        );

        let instance = wgpu::Instance::default();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("No suitable GPU adapters found");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("slime::device"),
                required_features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits: Limits::default(),
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Unable to get gpu device");

        App {
            settings,
            window,
            size,
            surface,
            adapter,
            device,
            queue,
        }
    }
}
