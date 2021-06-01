use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const BOUND_SIZE: f32 = 64.0;

pub struct ClearPipeline {
    pipeline: wgpu::ComputePipeline,
    work_group_count: u32,
    bind_group: wgpu::BindGroup,
}

impl super::Pipeline for ClearPipeline {
    type Bind = ClearSetup;
    type Update = ();

    fn new(
        device: &wgpu::Device,
        _settings: &crate::app::AppSettings,
        bind: &Self::Bind,
    ) -> Self {
        let slime_sim_compute_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("slime::shader::slime_sim_compute"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/clear.wgsl"
            ))),
            flags: wgpu::ShaderFlags::all(),
        });

        let slime_sim_compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("slime::shader::slime_sim_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<Globals>() as wgpu::BufferAddress
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

        let globals_data = Globals {
            width: bind.width,
            height: bind.height,
        };

        let globals_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("slime::shader::simulation_parameter_buffer"),
            contents: bytemuck::bytes_of(&globals_data),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &slime_sim_compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&bind.texture_view),
                },
            ],
            label: Some("slime::shader::slime_sim::bind_group"),
        });

        let work_group_count =
            ((bind.width * bind.height) as f32 / BOUND_SIZE).ceil() as u32;

        let slime_sim_compute_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("slime::shader::slime_sim_compute_pipeline_layout"),
                bind_group_layouts: &[&slime_sim_compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("slime::shader::compute_pipeline"),
            layout: Some(&slime_sim_compute_layout),
            module: &slime_sim_compute_shader,
            entry_point: "cs_main",
        });

        Self {
            pipeline,
            work_group_count,
            bind_group,
        }
    }

    fn update(&mut self, _queue: &wgpu::Queue, _update: &Self::Update) {}

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, _frame: &wgpu::SwapChainTexture) {
        encoder.push_debug_group("compute boid movement");
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch(self.work_group_count, 1, 1);
        }
        encoder.pop_debug_group();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    width: u32,
    height: u32,
}

pub struct ClearSetup {
    pub width: u32,
    pub height: u32,
    pub texture_view: wgpu::TextureView,
}
