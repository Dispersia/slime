use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::TimeBuffer;

const LENGTH_PER_GROUP: f32 = 8.0;

pub struct DiffusePipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    time_buffer: wgpu::Buffer,
    workgroup_count_x: u32,
    workgroup_count_y: u32,
}

impl super::Pipeline for DiffusePipeline {
    type Bind = DiffuseSettings;
    type Update = TimeBuffer;

    fn new(device: &wgpu::Device, settings: &crate::app::AppSettings, bind: &Self::Bind) -> Self {
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("slime::shader::diffuse"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/diffuse.wgsl"
            ))),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Globals>() as wgpu::BufferAddress
                        ),
                        ty: wgpu::BufferBindingType::Uniform,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<TimeBuffer>() as wgpu::BufferAddress
                        ),
                        ty: wgpu::BufferBindingType::Uniform,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let globals = Globals {
            width: bind.width,
            height: bind.height,
            diffuse_rate: settings.diffuse_rate,
            decay_rate: settings.decay_rate,
        };

        let time = TimeBuffer {
            time: 0,
            delta_time: 0.0,
        };

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&globals),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&time),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: time_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&bind.trail_map_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&bind.diffuse_texture),
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

        let workgroup_count_x = (bind.width as f32 / LENGTH_PER_GROUP).ceil() as u32;
        let workgroup_count_y = (bind.height as f32 / LENGTH_PER_GROUP).ceil() as u32;

        Self {
            pipeline: diffuse_pipeline,
            bind_group,
            time_buffer,
            workgroup_count_x,
            workgroup_count_y,
        }
    }

    fn update(&mut self, queue: &wgpu::Queue, update: &Self::Update) {
        queue.write_buffer(&self.time_buffer, 0, bytemuck::bytes_of(update));
    }

    fn execute(&self, encoder: &mut wgpu::CommandEncoder, _frame: &wgpu::TextureView) {
        encoder.push_debug_group("Render Pipeline");
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch(self.workgroup_count_x, self.workgroup_count_y, 1);
        }
        encoder.pop_debug_group();
    }
}

pub struct DiffuseSettings {
    pub width: u32,
    pub height: u32,
    pub trail_map_texture: wgpu::TextureView,
    pub diffuse_texture: wgpu::TextureView,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    width: u32,
    height: u32,
    diffuse_rate: f32,
    decay_rate: f32,
}
