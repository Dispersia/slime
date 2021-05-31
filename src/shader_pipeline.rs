use winit::dpi::PhysicalSize;

use crate::{
    app::AppSettings,
    pipeline::{
        ClearPipeline, ClearSetup, Pipeline, RenderPipeline, RenderSettings, SlimeSimPipeline,
        SlimeSimSetup, TimeBuffer,
    },
};

pub struct ShaderPipeline {
    clear_pipeline: ClearPipeline,
    slime_sim_pipeline: SlimeSimPipeline,
    render_pipeline: RenderPipeline,
    steps_per_frame: usize,
    frame_num: usize,
}

impl ShaderPipeline {
    pub fn new(
        settings: &AppSettings,
        size: &PhysicalSize<u32>,
        swapchain_descriptor: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> Self {
        let slime_sim_setup = SlimeSimSetup {
            width: size.width,
            height: size.height,
            format: swapchain_descriptor.format,
        };

        let render_setup = RenderSettings {
            format: swapchain_descriptor.format,
            width: size.width,
            height: size.height,
        };

        let clear_setup = ClearSetup {
            width: size.width,
            height: size.height,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("slime::shader::simulation::texture"),
            size: wgpu::Extent3d {
                width: slime_sim_setup.width,
                height: slime_sim_setup.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT
                | wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::STORAGE,
        });

        let clear_pipeline = ClearPipeline::new(device, settings, &texture, &clear_setup);
        let slime_sim_pipeline =
            SlimeSimPipeline::new(device, settings, &texture, &slime_sim_setup);
        let render_pipeline = RenderPipeline::new(device, settings, &texture, &render_setup);

        Self {
            clear_pipeline,
            slime_sim_pipeline,
            render_pipeline,
            frame_num: 0,
            steps_per_frame: settings.steps_per_frame,
        }
    }

    pub fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        time_buffer: &TimeBuffer,
    ) {
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.clear_pipeline.execute(&mut command_encoder, &frame);

        self.slime_sim_pipeline.update(&queue, &time_buffer);

        for _ in 0..self.steps_per_frame {
            self.slime_sim_pipeline
                .execute(&mut command_encoder, &frame);
        }

        self.render_pipeline.execute(&mut command_encoder, &frame);

        self.frame_num += 1;

        queue.submit(Some(command_encoder.finish()));
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Agent {
    position: [f32; 2],
    angle: f32,
    species_index: i32,
    species_mask: [i32; 4],
}
