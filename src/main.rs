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
    let requested_format = adapter.get_swap_chain_preferred_format(&surface).unwrap();

    let mut swapchain_descriptor = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: requested_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &swapchain_descriptor);

    let size = PhysicalSize::new(settings.width, settings.height);
    let mut shader_pipeline = ShaderPipeline::new(settings, &size, &swapchain_descriptor, &device);

    let start_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                let frame = match swap_chain.get_current_frame() {
                    Ok(frame) => frame,
                    Err(_) => {
                        swap_chain = device.create_swap_chain(&surface, &swapchain_descriptor);
                        swap_chain
                            .get_current_frame()
                            .expect("Failed to get swap chain")
                    }
                };

                let time_buffer = TimeBuffer {
                    time: start_time.elapsed().as_micros() as u32,
                    delta_time: 0.005,
                };

                shader_pipeline.render(&frame.output, &device, &queue, &time_buffer);
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                swapchain_descriptor.width = size.width.max(1);
                swapchain_descriptor.height = size.height.max(1);
                swap_chain = device.create_swap_chain(&surface, &swapchain_descriptor);
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
                    input: event::KeyboardInput {
                        virtual_keycode: Some(event::VirtualKeyCode::L),
                        state: event::ElementState::Pressed,
                        ..
                    },
                    ..
                } => { shader_pipeline.swap_buffers(); }
                _ => {}
            },
            _ => {}
        }
    });
}
