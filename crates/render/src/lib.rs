// TODO: can we pull all egui out of renderer?

mod camera;

use wgpu::util::DeviceExt; // for Device::create_buffer_init
                           //
                           // lib.rs

pub struct CameraState {
    pub camera: camera::Camera,
    matrix_buffer: wgpu::Buffer,
    // TODO: move all wgpu state into WGPUState?
    matrix_bind_group_layout: wgpu::BindGroupLayout,
    matrix_bind_group: wgpu::BindGroup,
}

impl CameraState {
    fn init(bbox: &model::BoundingBox, device: &wgpu::Device) -> Self {
        let bbox_mid = bbox.mid();
        let longest_axis = bbox.longest_axis();
        let camera = camera::Camera::new(
            bbox_mid + glam::Vec3::new(0.0, 0.0, longest_axis * 1.5),
            bbox_mid,
            (0.0, 1.0, 0.0).into(),
            /*
            (0.0, 0.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            */
            std::f32::consts::PI / 4.0,
            1.0,
            0.01,
            1000.0,
        );
        let matrix = camera.view_projection_matrix();

        let matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let matrix_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let matrix_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &matrix_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: matrix_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            camera,
            matrix_buffer,
            matrix_bind_group_layout,
            matrix_bind_group,
        }
    }
}
///
/// A texture to be used as a rendering target.  Hard codes texture format to
/// be Rgba8UnormSrgb as required by egui_wgpu.  Texture dims are fixed at
/// creation time.
///
struct RenderTexture {
    desc: wgpu::TextureDescriptor<'static>,
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
            desc,
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
    render_pipeline: wgpu::RenderPipeline,
    vertex_count: u32,
    vertex_buffer: wgpu::Buffer,
    model_scale: f32,
    pub camera_state: CameraState, // TODO
}

// TODO: create render_pipeline to store reusable part of render pass

impl Renderer {
    pub fn new(
        width: u32,
        height: u32,
        device: std::sync::Arc<wgpu::Device>,
        queue: std::sync::Arc<wgpu::Queue>,
        model: &model::Model,
    ) -> Self {
        let render_texture = RenderTexture::new(width, height, &device);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            //contents: bytemuck::cast_slice(VERTICES),
            contents: bytemuck::cast_slice(model.verts.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vbuffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<glam::Vec3>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        };
        /*
        let vbuffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // 1.
            step_mode: wgpu::VertexStepMode::Vertex,                            // 2.
            attributes: &[
                // 3.
                wgpu::VertexAttribute {
                    offset: 0,                             // 4.
                    shader_location: 0,                    // 5.
                    format: wgpu::VertexFormat::Float32x3, // 6.
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        };
        */

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/render.wgsl").into()),
        });

        let camera_state = CameraState::init(&model.bbox, &device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_state.matrix_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vbuffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_texture.desc.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            // Useful for optimizing shader compilation on Android
            cache: None,
        });

        Self {
            render_texture,
            device,
            queue,
            render_pipeline,
            vertex_count: model.verts.len() as u32,
            vertex_buffer,
            model_scale: model.bbox.longest_axis(),
            camera_state,
        }
    }

    fn raster_to_ndc(&self, r: glam::Vec2) -> glam::Vec2 {
        let size = glam::Vec2::new(
            self.render_texture.desc.size.width as f32,
            self.render_texture.desc.size.height as f32,
        );
        // invert y
        let r = glam::Vec2::new(r.x, size.y as f32 - r.y);

        // center around origin, then scale to [-1,1]^2
        let half_size = size * 0.5;
        (r - half_size) / half_size
    }

    pub fn handle_event(&mut self, event: event::Event) {
        if let event::Event::Drag {
            button: button,
            drag_begin,
            drag_end,
            modifiers: _,
        } = event
        {
            let drag_begin = self.raster_to_ndc(drag_begin);
            let drag_end = self.raster_to_ndc(drag_end);
            match button {
                event::MouseButton::Primary => {
                    self.camera_state
                        .camera
                        .camera_view
                        .rotate(drag_begin, drag_end);
                }
                event::MouseButton::Secondary => {
                    self.camera_state
                        .camera
                        .camera_view
                        .pan(drag_begin - drag_end);
                }
                event::MouseButton::Middle => {
                    self.camera_state
                        .camera
                        .camera_view
                        .dolly((drag_begin.y - drag_end.y) * self.model_scale);
                }
                _ => {}
            }
        }
        /*
        match event {
            event::Event::Drag{event::MouseButton::Primary, drag_begin, drag_end, modifiers} => {
            },
            event::Event::Drag{event::MouseButton::Secondary, drag_begin, drag_end, modifiers} => {
            },
            event::Event::Drag{event::MouseButton::Middle, drag_begin, drag_end, modifiers} => {
            },
            _ => {}
        }
        */
    }

    pub fn render(&self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let camera_matrix = self.camera_state.camera.view_projection_matrix();
            self.queue.write_buffer(
                &self.camera_state.matrix_buffer,
                0,
                bytemuck::cast_slice(&[camera_matrix]),
            );
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            render_pass.set_bind_group(0, &self.camera_state.matrix_bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            //render_pass.draw(0..3, 0..1);
            render_pass.draw(0..self.vertex_count, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn get_render_texture_view(&self) -> &wgpu::TextureView {
        &self.render_texture.view
    }
}
