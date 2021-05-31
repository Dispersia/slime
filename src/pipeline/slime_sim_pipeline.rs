use std::{borrow::Cow, f32::consts::PI};

use bytemuck::{Pod, Zeroable};
use rand::{distributions::Uniform, prelude::Distribution};
use wgpu::util::DeviceExt;

const PARTICLES_PER_GROUP: usize = 64;

pub struct SlimeSimPipeline {
    pipeline: wgpu::ComputePipeline,
    time_buffer: wgpu::Buffer,
    work_group_count: u32,
    bind_group: wgpu::BindGroup,
}

impl super::Pipeline for SlimeSimPipeline {
    type Bind = SlimeSimSetup;
    type Update = TimeBuffer;

    fn new(
        device: &wgpu::Device,
        settings: &crate::app::AppSettings,
        texture: &wgpu::Texture,
        bind: &Self::Bind,
    ) -> Self {
        let mut rng = rand::thread_rng();

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let slime_sim_compute_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("slime::shader::slime_sim_compute"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/slime_sim.wgsl"
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
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<TimeBuffer>() as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                SpeciesSetting,
                            >()
                                as wgpu::BufferAddress),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
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
                        binding: 4,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadWrite,
                            format: wgpu::TextureFormat::Rgba16Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let globals_data = Globals {
            trail_weight: settings.trail_weight,
            width: bind.width,
            height: bind.height,
        };

        let globals_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("slime::shader::simulation_parameter_buffer"),
            contents: bytemuck::bytes_of(&globals_data),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let time_data = TimeBuffer {
            time: 0,
            delta_time: 0.0,
        };

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("slime::shader::time_buffer"),
            contents: bytemuck::bytes_of(&time_data),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let species_settings = SpeciesSetting {
            move_speed: 30.0,
            turn_speed: -3.0,

            sensor_angle_degrees: 112.0,
            sensor_offset_dst: 20.0,
            sensor_size: 1,
        };

        let species_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("slime::shader::species_buffer"),
            contents: bytemuck::bytes_of(&species_settings),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let agent_uniform = Uniform::new_inclusive(0.0, 1.0);
        let agents = (0..settings.num_agents)
            .into_iter()
            .map(|_| {
                let start_pos = [bind.width as f32 / 2.0, bind.height as f32 / 2.0];

                let random_angle: f32 = agent_uniform.sample(&mut rng) * PI * 2.0;

                Agent {
                    position: start_pos,
                    angle: random_angle,
                    _padding: 0,
                }
            })
            .collect::<Vec<Agent>>();

        let agent_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("slime::shader::simulation::agents_buffer")),
            contents: bytemuck::cast_slice(&agents),
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
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
                    resource: time_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: species_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: agent_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
            label: Some("slime::shader::slime_sim::bind_group"),
        });

        let work_group_count =
            ((settings.num_agents as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

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
            time_buffer,
            work_group_count,
            bind_group,
        }
    }

    fn update(&mut self, queue: &wgpu::Queue, update: &Self::Update) {
        queue.write_buffer(&self.time_buffer, 0, bytemuck::bytes_of(update));
    }

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
    trail_weight: f32,
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct TimeBuffer {
    pub time: u32,
    pub delta_time: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SpeciesSetting {
    move_speed: f32,
    turn_speed: f32,
    sensor_angle_degrees: f32,
    sensor_offset_dst: f32,
    sensor_size: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Agent {
    position: [f32; 2],
    angle: f32,
    _padding: i32,
}

#[derive(Debug)]
pub struct SlimeSimSetup {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}
