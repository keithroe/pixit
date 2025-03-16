mod camera;
mod light;
mod scene;
mod shader;
mod texture;

use scene::*;

//use wgpu::{util::DeviceExt, PrimitiveTopology};

/// Simple renderer for single 3D model
///
/// Handles user events (eg, mouse drag) and renders model to offscreen texture.
/// Assumes Y-up.
pub struct Renderer {
    queue: wgpu::Queue,
    device: wgpu::Device,
    render_view: RenderView,
    scene: Scene,
    _shader_cache: shader::Cache,

    render_pipelines: Vec<wgpu::RenderPipeline>,
    depth_texture: texture::Texture,
}

impl Renderer {
    pub fn new(
        size: &[u32; 2],
        device: wgpu::Device,
        queue: wgpu::Queue,
        input_model: &model::Model,
    ) -> Self {
        let render_view = RenderView::new(size, &device);

        let scene = Scene::from_model(input_model, &device);

        let mut shader_cache = shader::Cache::new(&device);
        let render_pipelines = generate_pipelines(&scene, &mut shader_cache, &render_view, &device);

        let depth_texture = texture::Texture::new_depth_texture(size, &device);

        Self {
            queue,
            device,
            render_view,
            _shader_cache: shader_cache,
            scene,
            render_pipelines,
            depth_texture,
        }
    }

    pub fn get_render_texture_view(&self) -> &wgpu::TextureView {
        &self.render_view.view
    }

    pub fn render(&self) {
        // Update camera uniforms
        let view_matrix = self.scene.camera.controller.view_matrix();
        let proj_matrix = self.scene.camera.controller.projection_matrix();
        let view_proj = proj_matrix * view_matrix;
        self.queue.write_buffer(
            &self.scene.camera.view_proj_buffer,
            0,
            bytemuck::cast_slice(&[view_proj]),
        );

        self.scene.light.update_uniform(&self.queue);

        for (mesh_idx, mesh) in self.scene.meshes.iter().enumerate() {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
            {
                self.queue.write_buffer(
                    &self.scene.camera.view_proj_buffer,
                    0,
                    bytemuck::cast_slice(&[view_proj]),
                );

                let model_matrix = mesh.transform;
                let normal_transform = generate_normal_transform(view_matrix, model_matrix);
                self.queue.write_buffer(
                    &mesh.normal_transform_buffer,
                    0,
                    bytemuck::cast_slice(&[normal_transform]),
                );

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        // This is what @location(0) in the fragment shader targets
                        Some(wgpu::RenderPassColorAttachment {
                            view: &self.render_view.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.02,
                                    g: 0.02,
                                    b: 0.02,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                render_pass.set_bind_group(0, &self.scene.camera.bind_group, &[]);
                render_pass.set_bind_group(1, &mesh.bind_group, &[]);
                render_pass.set_bind_group(2, &self.scene.light.bind_group, &[]);

                render_pass.set_pipeline(&self.render_pipelines[mesh_idx]); // 2.
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                if let Some(nbuff) = &mesh.normal_buffer {
                    render_pass.set_vertex_buffer(1, nbuff.slice(..));
                }
                if let Some(tcbuff) = &mesh.texcoord_buffer {
                    render_pass.set_vertex_buffer(2, tcbuff.slice(..));
                }
                if let Some(ibuff) = &mesh.index_buffer {
                    render_pass.set_index_buffer(ibuff.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.num_triangles * 3, 0, 0..1);
                } else {
                    render_pass.draw(0..mesh.num_triangles, 0..1);
                }
                //render_pass.draw(0..3, 0..1);
            }
            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    pub fn handle_event(&mut self, event: event::Event) {
        if let event::Event::Drag {
            button,
            drag_begin,
            drag_end,
            modifiers: _,
        } = event
        {
            let drag_begin = self.raster_to_ndc(drag_begin);
            let drag_end = self.raster_to_ndc(drag_end);
            match button {
                event::MouseButton::Primary => {
                    self.scene.camera.controller.rotate(drag_begin, drag_end);
                }
                event::MouseButton::Secondary => {
                    self.scene.camera.controller.pan(drag_begin - drag_end);
                }
                event::MouseButton::Middle => {
                    self.scene
                        .camera
                        .controller
                        .dolly((drag_begin.y - drag_end.y) * self.scene.bbox.longest_axis());
                }
                _ => {}
            }
        }
    }

    fn raster_to_ndc(&self, r: glam::Vec2) -> glam::Vec2 {
        let size = self.render_view.sizef();
        // invert y
        let r = glam::Vec2::new(r.x, size.y - r.y);

        // center around origin, then scale to [-1,1]^2
        let half_size = size * 0.5;
        (r - half_size) / half_size
    }
}

/// A WGPU texture to be used as a rendering output target.
///
/// Hard codes texture format to be Rgba8UnormSrgb as required by egui_wgpu.
/// Texture dims are fixed at creation time.
struct RenderView {
    desc: wgpu::TextureDescriptor<'static>,
    view: wgpu::TextureView,
}
impl RenderView {
    /// View format is always Rgba8UnormSrgb as required by egui
    const VIEW_FORMATS: &[wgpu::TextureFormat] = &[wgpu::TextureFormat::Rgba8UnormSrgb];

    /// Create WGPU texture for render target
    fn new(size: &[u32; 2], device: &wgpu::Device) -> Self {
        let desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            view_formats: RenderView::VIEW_FORMATS,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&Default::default());

        Self { desc, view }
    }

    /// return render texture dimensions (width, height)
    #[allow(unused)]
    fn size(&self) -> glam::UVec2 {
        glam::UVec2::new(self.desc.size.width, self.desc.size.height)
    }

    /// return render texture dimensions as f32 (width, height)
    fn sizef(&self) -> glam::Vec2 {
        glam::Vec2::new(self.desc.size.width as f32, self.desc.size.height as f32)
    }
}

fn generate_pipelines(
    scene: &Scene,
    shader_cache: &mut shader::Cache,
    render_view: &RenderView,
    device: &wgpu::Device,
) -> Vec<wgpu::RenderPipeline> {
    let mut pipelines = Vec::new();

    for mesh in &scene.meshes {
        // build render pipeline layout
        let bind_group_layouts = [
            &WGPUCamera::bind_group_layout(device),
            &WGPUMesh::bind_group_layout(device),
            &WGPULight::bind_group_layout(device),
        ];
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            });

        // build pipeline
        let mut shader_spec = shader::Specification::default();

        let mut vertex_buffer_layouts = vec![wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<glam::Vec3>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                // Positions
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }];

        // TODO:: create abstraction for VertexAttribute
        if mesh.normal_buffer.is_some() {
            println!("setting normal buffer");
            shader_spec.has_normals = true;
            vertex_buffer_layouts.push(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<glam::Vec3>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    // Normals
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }],
            });
        }

        if mesh.texcoord_buffer.is_some() {
            println!("setting texcoord buffer");
            shader_spec.has_texcoords = true;
            vertex_buffer_layouts.push(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<glam::Vec2>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    // Normals
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            });
        }

        let (vert_module, frag_module) = shader_cache.get_modules(shader_spec);

        let vertex_state = wgpu::VertexState {
            module: vert_module,
            entry_point: Some("main"),
            buffers: &vertex_buffer_layouts,
            compilation_options: Default::default(),
        };

        let fragment_state = wgpu::FragmentState {
            module: frag_module,
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: render_view.desc.format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        };

        let primitive_state = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        };

        let multisample_state = wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        let depth_stencil = Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: vertex_state,
            fragment: Some(fragment_state),
            primitive: primitive_state,
            depth_stencil,
            multisample: multisample_state,
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            // Useful for optimizing shader compilation on Android
            cache: None,
        });

        pipelines.push(render_pipeline);
    }

    pipelines
}

fn generate_normal_transform(view_matrix: glam::Mat4, model_matrix: glam::Mat4) -> glam::Mat4 {
    let model_view = view_matrix * model_matrix;
    model_view.inverse().transpose()
}
