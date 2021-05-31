use wgpu::{Features, Limits};
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder};

pub struct AppSettings {
    pub trail_weight: f32,
    pub num_agents: usize,
    pub steps_per_frame: usize,
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
        #[cfg(not(target_arch = "wasm32"))]
        pretty_env_logger::init();

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Slime")
            .build(&event_loop)
            .expect("Could not create window");

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            console_log::init().expect("couldn't create logger");
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to body");
        }

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let size = window.inner_size();

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("No suitable GPU adapter found");

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
