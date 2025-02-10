// TODO: can we pull all egui out of renderer?

use crate::model;
use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};
use std::num::NonZeroU64;

pub struct RenderCallback {
    pub resource_idx: usize,
    pub frame_state: FrameState,
}

impl egui_wgpu::CallbackTrait for RenderCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &RenderResources = resources.get().unwrap();
        resources.resources[self.resource_idx].prepare(device, queue, self.frame_state.angle);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &RenderResources = resources.get().unwrap();
        resources.resources[self.resource_idx].paint(render_pass);
    }
}

/// Represents data that may vary from frame to frame
#[derive(Copy, Clone, Default)]
pub struct FrameState {
    pub angle: f32,
}

pub struct RenderResources {
    resources: Vec<RenderState>,
}

pub struct RenderState {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

// TODO: make render-state POD.  fold render-state impl into RenderCallback?
impl RenderState {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, angle: f32) {
        // Update our uniform buffer with the angle from the UI
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[angle, 0.0, 0.0, 0.0]),
        );
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // Draw our triangle!
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

/// Initialize render pipeline and rendering state, returns render state list index
pub fn init(render_state: &egui_wgpu::RenderState, _model: model::Model) -> usize {
    let device = &render_state.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custom3d"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./custom3d_wgpu_shader.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("custom3d"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(16),
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("custom3d"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("custom3d"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: None,
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(render_state.target_format.into())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("custom3d"),
        contents: bytemuck::cast_slice(&[0.0_f32; 4]), // 16 bytes aligned!
        // Mapping at creation (as done by the create_buffer_init utility) doesn't require us to to add the MAP_WRITE usage
        // (this *happens* to workaround this bug )
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("custom3d"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    // Because the graphics pipeline must have the same lifetime as the egui render pass,
    // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
    // `paint_callback_resources` type map, which is stored alongside the render pass.
    let callback_resources = &mut render_state.renderer.write().callback_resources;
    match callback_resources.get_mut::<RenderResources>() {
        Some(rs) => {
            rs.resources.push(RenderState {
                pipeline,
                bind_group,
                uniform_buffer,
            });
            rs.resources.len() - 1
        }
        None => {
            callback_resources.insert(RenderResources {
                resources: vec![RenderState {
                    pipeline,
                    bind_group,
                    uniform_buffer,
                }],
            });
            0 // index in renderer state list
        }
    }
}

fn _texture_id() -> egui::TextureId {
    egui::TextureId::default()
}
