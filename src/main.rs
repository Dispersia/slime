use instant::Instant;

use app::{App, AppSettings};
use pipeline::TimeBuffer;
use shader_pipeline::ShaderPipeline;
use winit::{
    dpi::PhysicalSize,
    event::{self, Event, WindowEvent},
    event_loop::ControlFlow,
};

mod app;
mod pipeline;
mod runner;
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

    runner::run_app(settings, start);
}

fn start(
    App {
        settings,
        window,
        event_loop,
        size,
        surface,
        adapter,
        device,
        queue,
    }: App,
) {
    let requested_format = surface.get_preferred_format(&adapter).unwrap();

    let mut surface_configuration = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: requested_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    surface.configure(&device, &surface_configuration);

    let size = PhysicalSize::new(settings.width, settings.height);
    let mut shader_pipeline = ShaderPipeline::new(settings, &size, &surface_configuration, &device);

    let start_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to get swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let time_buffer = TimeBuffer {
                    time: start_time.elapsed().as_micros() as u32,
                    delta_time: 0.005,
                };

                shader_pipeline.render(&view, &device, &queue, &time_buffer);
                frame.present();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                surface_configuration.width = size.width.max(1);
                surface_configuration.height = size.height.max(1);
                surface.configure(&device, &surface_configuration);
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::L),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    shader_pipeline.swap_buffers();
                }
                _ => {}
            },
            _ => {}
        }
    });
}
