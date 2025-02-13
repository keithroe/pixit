// TODO: can we pull all egui out of renderer?

use crate::model;

///
/// A texture to be used as a rendering target.  Hard codes texture format to
/// be Rgba8UnormSrgb as required by egui_wgpu.  Texture dims are fixed at
/// creation time.
///
struct RenderTexture {
    _desc: wgpu::TextureDescriptor<'static>,
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl RenderTexture {
    const VIEW_FORMATS: &[wgpu::TextureFormat] = &[wgpu::TextureFormat::Rgba8UnormSrgb];

    fn new(width: u32, height: u32, device: &wgpu::Device) -> Self {
        let desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            view_formats: RenderTexture::VIEW_FORMATS,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&Default::default());

        Self {
            _desc: desc,
            _texture: texture,
            view,
        }
    }
}

///
/// Renders single model to offscreen texture.
///
pub struct Renderer {
    queue: std::sync::Arc<wgpu::Queue>,
    device: std::sync::Arc<wgpu::Device>,
    render_texture: RenderTexture,
}

// TODO: create render_pipeline to store reusable part of render pass

impl Renderer {
    pub fn new(
        width: u32,
        height: u32,
        device: std::sync::Arc<wgpu::Device>,
        queue: std::sync::Arc<wgpu::Queue>,
        _model: &model::Model,
    ) -> Self {
        let render_texture = RenderTexture::new(width, height, &device);

        Self {
            render_texture,
            device,
            queue,
        }
    }

    pub fn render(&self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn get_render_texture_view(&self) -> &wgpu::TextureView {
        &self.render_texture.view
    }
}
