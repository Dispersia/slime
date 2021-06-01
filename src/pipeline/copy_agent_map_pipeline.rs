use std::borrow::Cow;

use crate::shader_pipeline::Agent;

use super::{SlimeSimSetup, TimeBuffer};

const AGENTS_PER_GROUP: f32 = 16.0;

pub struct CopyAgentMapPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    workgroup_count: u32,
}

impl super::Pipeline for CopyAgentMapPipeline {
    type Bind = SlimeSimSetup;
    type Update = TimeBuffer;

    fn new(device: &wgpu::Device, settings: &crate::app::AppSettings, bind: &Self::Bind) -> Self {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("slime::shader::copy"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/copy_agents.wgsl"
            ))),
            flags: wgpu::ShaderFlags::all(),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            (std::mem::size_of::<Agent>() * settings.num_agents)
                                as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: bind.binding.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&bind.display_texture_view),
                },
            ],
        });

        let diffuse_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let diffuse_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&diffuse_pipeline_layout),
            entry_point: "cs_main",
            module: &shader,
        });

        let workgroup_count = (bind.num_agents as f32 / AGENTS_PER_GROUP).ceil() as u32;

        Self {
            pipeline: diffuse_pipeline,
            bind_group,
            workgroup_count,
        }
    }

    fn update(&mut self, _queue: &wgpu::Queue, _update: &Self::Update) {}

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, _frame: &wgpu::SwapChainTexture) {
        encoder.push_debug_group("Render Pipeline");
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch(self.workgroup_count, 1, 1);
        }
        encoder.pop_debug_group();
    }
}
