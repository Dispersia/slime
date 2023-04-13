use wgpu::{Features, Limits};
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder};

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

pub struct App {
    pub settings: AppSettings,
    pub window: winit::window::Window,
    pub event_loop: EventLoop<()>,
    pub size: PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl App {
    pub async fn new(settings: AppSettings) -> Self {
        pretty_env_logger::init();

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Slime")
            .build(&event_loop)
            .expect("Could not create window");

        let instance = wgpu::Instance::default();

        let size = window.inner_size();

        let surface = unsafe { instance.create_surface(&window).unwrap() };

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(
            &instance,
            wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all),
            Some(&surface),
        )
        .await
        .expect("No suitable GPU adapters found");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("slime::device"),
                    features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: Limits::default(),
                },
                None,
            )
            .await
            .expect("Unable to get gpu device");

        App {
            settings,
            window,
            event_loop,
            size,
            surface,
            adapter,
            device,
            queue,
        }
    }
}
