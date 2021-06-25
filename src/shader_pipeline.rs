use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use rand::{distributions::Uniform, prelude::Distribution};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::{app::AppSettings, pipeline::{BlitPipeline, BlitSettings, ClearPipeline, ClearSetup, CopyAgentMapPipeline, DiffusePipeline, DiffuseSettings, Pipeline, RenderPipeline, RenderSettings, SlimeSimPipeline, SlimeSimSetup, TimeBuffer}};

pub struct ShaderPipeline {
    clear_pipeline: ClearPipeline,
    slime_sim_pipeline: SlimeSimPipeline,
    diffuse_pipeline: DiffusePipeline,
    blit_diffuse_pipeline: BlitPipeline,
    blit_display_pipeline: BlitPipeline,
    blit_trail_map_pipeline: BlitPipeline,
    blit_trail_map_copy_pipeline: BlitPipeline,
    copy_agents_pipeline: CopyAgentMapPipeline,
    render_pipeline: RenderPipeline,
    frame_num: usize,
    settings: AppSettings,
}

impl ShaderPipeline {
    pub fn new(
        settings: AppSettings,
        size: &PhysicalSize<u32>,
        swapchain_descriptor: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let agent_uniform = Uniform::new_inclusive(0.0, 1.0);
        let agents = (0..settings.num_agents)
            .into_iter()
            .map(|_| {
                let start_pos = [size.width as f32 / 2.0, size.height as f32 / 2.0];

                let random_angle: f32 = agent_uniform.sample(&mut rng) * PI * 2.0;

                Agent {
                    position: start_pos,
                    angle: random_angle,
                    _padding: 0,
                }
            })
            .collect::<Vec<Agent>>();

        let agent_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("slime::shader::simulation::agents_buffer"),
            contents: bytemuck::cast_slice(&agents),
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
        });

        let trail_map = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("slime::shader::simulation::texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
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

        let trail_map_copy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("slime::shader::simulation::trail_map_copy"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
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

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
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

        let display_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
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

        let render_setup = RenderSettings {
            format: swapchain_descriptor.format,
            width: size.width,
            height: size.height,
            texture_view: display_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        };

        let diffuse_settings = DiffuseSettings {
            width: size.width,
            height: size.height,
            trail_map_texture: trail_map.create_view(&wgpu::TextureViewDescriptor::default()),
            diffuse_texture: diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        };

        let slime_sim_setup = SlimeSimSetup {
            width: size.width,
            height: size.height,
            format: swapchain_descriptor.format,
            binding: agent_buffer,
            trail_map_texture_view: trail_map.create_view(&wgpu::TextureViewDescriptor::default()),
            trail_map_write_texture_view: trail_map_copy.create_view(&wgpu::TextureViewDescriptor::default()),
            display_texture_view: display_texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            num_agents: agents.len() as u32,
        };

        let clear_setup = ClearSetup {
            width: size.width,
            height: size.height,
            texture_view: display_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        };

        let slime_sim_pipeline = SlimeSimPipeline::new(device, &settings, &slime_sim_setup);
        let diffuse_pipeline = DiffusePipeline::new(device, &settings, &diffuse_settings);
        let clear_pipeline = ClearPipeline::new(device, &settings, &clear_setup);
        let copy_agents_pipeline = CopyAgentMapPipeline::new(device, &settings, &slime_sim_setup);
        let render_pipeline = RenderPipeline::new(device, &settings, &render_setup);

        let blit_diffuse_settings = BlitSettings {
            width: size.width,
            height: size.height,
            input_texture: diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            output_texture: trail_map.create_view(&wgpu::TextureViewDescriptor::default())
        };

        let blit_diffuse_pipeline = BlitPipeline::new(device, &settings, &blit_diffuse_settings);

        let blit_display_settings = BlitSettings {
            width: size.width,
            height: size.height,
            input_texture: trail_map.create_view(&wgpu::TextureViewDescriptor::default()),
            output_texture: display_texture.create_view(&wgpu::TextureViewDescriptor::default())
        };

        let blit_display_pipeline = BlitPipeline::new(device, &settings, &blit_display_settings);

        let blip_trail_map_settings = BlitSettings {
            width: size.width,
            height: size.height,
            input_texture: trail_map.create_view(&wgpu::TextureViewDescriptor::default()),
            output_texture: trail_map_copy.create_view(&wgpu::TextureViewDescriptor::default())
        };

        let blit_trail_map_pipeline = BlitPipeline::new(device, &settings, &blip_trail_map_settings);
        
        let blip_trail_map_copy_settings = BlitSettings {
            width: size.width,
            height: size.height,
            input_texture: trail_map_copy.create_view(&wgpu::TextureViewDescriptor::default()),
            output_texture: trail_map.create_view(&wgpu::TextureViewDescriptor::default())
        };

        let blit_trail_map_copy_pipeline = BlitPipeline::new(device, &settings, &blip_trail_map_copy_settings);

        Self {
            clear_pipeline,
            slime_sim_pipeline,
            diffuse_pipeline,
            copy_agents_pipeline,
            render_pipeline,
            blit_diffuse_pipeline,
            blit_display_pipeline,
            blit_trail_map_pipeline,
            blit_trail_map_copy_pipeline,
            settings,
            frame_num: 0,
        }
    }

    pub fn swap_buffers(&mut self) {
        self.settings.agents_only = !self.settings.agents_only;
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

        self.slime_sim_pipeline.update(&queue, &time_buffer);
        self.diffuse_pipeline.update(&queue, &time_buffer);

        for _ in 0..self.settings.steps_per_frame {
            self.blit_trail_map_pipeline.execute(&mut command_encoder, &frame);
            self.slime_sim_pipeline
                .execute(&mut command_encoder, &frame);
            self.blit_trail_map_copy_pipeline.execute(&mut command_encoder, &frame);

            self.diffuse_pipeline.execute(&mut command_encoder, &frame);

            self.blit_diffuse_pipeline.execute(&mut command_encoder, &frame);
        }

        if self.settings.agents_only {
            self.clear_pipeline.execute(&mut command_encoder, &frame);

            self.copy_agents_pipeline
                .execute(&mut command_encoder, &frame);
        } else {
            self.blit_display_pipeline.execute(&mut command_encoder, &frame);
        }

        self.render_pipeline.execute(&mut command_encoder, &frame);

        self.frame_num += 1;

        queue.submit(Some(command_encoder.finish()));
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Agent {
    position: [f32; 2],
    angle: f32,
    _padding: i32,
}
