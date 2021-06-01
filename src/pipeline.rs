use crate::app::AppSettings;

mod clear_pipeline;
mod copy_agent_map_pipeline;
mod diffuse_pipeline;
mod render_pipeline;
mod slime_sim_pipeline;

pub trait Pipeline {
    type Bind;
    type Update;

    fn new(device: &wgpu::Device, settings: &AppSettings, bind: &Self::Bind) -> Self;
    fn update(&mut self, queue: &wgpu::Queue, update: &Self::Update);
    fn execute(&self, encoder: &mut wgpu::CommandEncoder, frame: &wgpu::SwapChainTexture);
}

pub use self::{
    clear_pipeline::{ClearPipeline, ClearSetup},
    copy_agent_map_pipeline::CopyAgentMapPipeline,
    diffuse_pipeline::{DiffusePipeline, DiffuseSettings},
    render_pipeline::{RenderPipeline, RenderSettings},
    slime_sim_pipeline::{SlimeSimPipeline, SlimeSimSetup, TimeBuffer},
};
