use std::sync::Arc;

use instant::Instant;

use app::{App, AppEventLoop, AppSettings};
use pipeline::TimeBuffer;
use shader_pipeline::ShaderPipeline;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod app;
mod pipeline;
mod shader_pipeline;

fn main() {
    let settings = AppSettings {
        width: 800,
        height: 600,

        num_agents: 750_000,
        steps_per_frame: 1,

        move_speed: 50.0,
        turn_speed: -3.0,

        sensor_angle_degrees: 112.0,
        sensor_offset_dst: 20.0,
        sensor_size: 1,

        trail_weight: 2.0,
        decay_rate: 0.75,
        diffuse_rate: 5.0,

        agents_only: false,
    };

    let app_event_loop = AppEventLoop::new(settings);

    let AppEventLoop {
        settings,
        event_loop,
    } = app_event_loop;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut handler = AppHandler::Uninitialized(settings);
    event_loop.run_app(&mut handler).unwrap();
}

struct AppState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    shader_pipeline: ShaderPipeline,
    start_time: Instant,
}

enum AppHandler {
    Uninitialized(AppSettings),
    Initialized(Box<AppState>),
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let AppHandler::Uninitialized(settings) = std::mem::replace(
            self,
            AppHandler::Uninitialized(AppSettings {
                width: 0,
                height: 0,
                trail_weight: 0.0,
                num_agents: 0,
                steps_per_frame: 0,
                move_speed: 0.0,
                turn_speed: 0.0,
                sensor_angle_degrees: 0.0,
                sensor_offset_dst: 0.0,
                sensor_size: 0,
                decay_rate: 0.0,
                diffuse_rate: 0.0,
                agents_only: false,
            }),
        ) {
            let app = pollster::block_on(App::new(settings, event_loop));

            let App {
                settings,
                window,
                size,
                surface,
                adapter,
                device,
                queue,
            } = app;

            let capabilities = surface.get_capabilities(&adapter);
            let surface_format = capabilities
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(capabilities.formats[0]);

            let surface_configuration = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            surface.configure(&device, &surface_configuration);

            let size = PhysicalSize::new(settings.width, settings.height);
            let shader_pipeline =
                ShaderPipeline::new(settings, &size, &surface_configuration, &device);

            let start_time = Instant::now();

            *self = AppHandler::Initialized(Box::new(AppState {
                window,
                surface,
                device,
                queue,
                surface_configuration,
                shader_pipeline,
                start_time,
            }));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let AppHandler::Initialized(state) = self else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                let frame = state
                    .surface
                    .get_current_texture()
                    .expect("Failed to get swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let time_buffer = TimeBuffer {
                    time: state.start_time.elapsed().as_micros() as u32,
                    delta_time: 0.005,
                };

                state
                    .shader_pipeline
                    .render(&view, &state.device, &state.queue, &time_buffer);
                frame.present();
            }
            WindowEvent::Resized(size) => {
                state.surface_configuration.width = size.width.max(1);
                state.surface_configuration.height = size.height.max(1);
                state
                    .surface
                    .configure(&state.device, &state.surface_configuration);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            }
            | WindowEvent::CloseRequested => {
                // Drop GPU resources before exiting to avoid SIGSEGV
                // The surface must be dropped before the window/display connection closes
                *self = AppHandler::Uninitialized(AppSettings {
                    width: 0,
                    height: 0,
                    trail_weight: 0.0,
                    num_agents: 0,
                    steps_per_frame: 0,
                    move_speed: 0.0,
                    turn_speed: 0.0,
                    sensor_angle_degrees: 0.0,
                    sensor_offset_dst: 0.0,
                    sensor_size: 0,
                    decay_rate: 0.0,
                    diffuse_rate: 0.0,
                    agents_only: false,
                });
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::KeyL),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                state.shader_pipeline.swap_buffers();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let AppHandler::Initialized(state) = self {
            state.window.request_redraw();
        }
    }
}
