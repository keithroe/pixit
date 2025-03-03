// TODO: can we pull all egui out of renderer?

mod camera;

use wgpu::util::DeviceExt;

mod light {
    pub struct UniformData {
        enabled: bool,
        dir: glam::Vec3,
        color: glam::Vec3,
        scale: f32,
        ambient: glam::Vec3,
        ambient_scale: f32,
    }

    impl Default for UniformData {
        fn default() -> Self {
            Self {
                enabled: true,
                dir: glam::Vec3::new(-1.0, -1.0, -1.0).normalize(),
                color: glam::Vec3::new(0.9, 0.6, 0.3),
                scale: 0.7,
                ambient: glam::Vec3::new(0.5, 0.5, 0.7),
                ambient_scale: 0.3,
            }
        }
    }

    #[derive(Default)]
    pub struct Light {
        uniform_data: UniformData,
    }
}

/// Simple renderer for single 3D model
///
/// Handles user events (eg, mouse drag) and renders model to offscreen texture.
/// Assumes Y-up.
pub struct Renderer {
    queue: wgpu::Queue,
    device: wgpu::Device,
    render_view: RenderView,
    shader_module: wgpu::ShaderModule,
    scene: Scene,
}

impl Renderer {
    pub fn new(
        size: &[u32; 2],
        device: wgpu::Device,
        queue: wgpu::Queue,
        input_model: &model::Model,
    ) -> Self {
        let render_view = RenderView::new(size, &device);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/render.wgsl").into()),
        });

        let scene = Scene::from_model(input_model, &device);

        Self {
            queue,
            device,
            render_view,
            shader_module,
            scene,
        }
    }

    pub fn get_render_texture_view(&self) -> &wgpu::TextureView {
        &self.render_view.view
    }

    pub fn render(&self) {
        todo!()
    }

    pub fn handle_event(&mut self, event: event::Event) {
        if let event::Event::Drag {
            button,
            drag_begin,
            drag_end,
            modifiers: _,
        } = event
        {
            // TODO: api for accessing camera view
            let drag_begin = self.raster_to_ndc(drag_begin);
            let drag_end = self.raster_to_ndc(drag_end);
            match button {
                event::MouseButton::Primary => {
                    self.scene
                        .camera
                        .controller
                        .camera_view
                        .rotate(drag_begin, drag_end);
                }
                event::MouseButton::Secondary => {
                    self.scene
                        .camera
                        .controller
                        .camera_view
                        .pan(drag_begin - drag_end);
                }
                event::MouseButton::Middle => {
                    self.scene
                        .camera
                        .controller
                        .camera_view
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

/// 3D scene for simple wgpu renderer
///
///  
struct Scene {
    bbox: model::BoundingBox,
    camera: WGPUCamera,
    light: WGPULight,
    materials: Vec<WGPUMaterial>,
    skins: Vec<WGPUSkin>,
    meshes: Vec<WGPUMesh>,
}

impl Scene {
    fn from_model(model: &model::Model, device: &wgpu::Device) -> Self {
        let mut scene = Self {
            bbox: model.bbox,
            camera: WGPUCamera::with_looking_at(&model.bbox, device),
            light: WGPULight::new(device),
            materials: Vec::new(),
            skins: Vec::new(),
            meshes: Vec::new(),
        };

        for mesh in &model.meshes {
            let skin_id = if let Some(skin) = &mesh.skin {
                scene.skins.push(WGPUSkin::new(&skin, device));
                Some((scene.skins.len() - 1) as u32)
            } else {
                None
            };

            for primitive in &mesh.primitives {
                let mut wgpu_mesh = WGPUMesh::from_model_primitive(primitive, device);
                wgpu_mesh.skin_id = skin_id;
                wgpu_mesh.material_id = None;
                scene.meshes.push(wgpu_mesh);
            }
        }
        scene
    }
}

///
///
///
struct WGPUCamera {
    controller: camera::Camera,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl WGPUCamera {
    fn with_looking_at(bbox: &model::BoundingBox, device: &wgpu::Device) -> Self {
        // Generate initial viewing params, camera is looking in -Z direction at
        // center of model's bounding box
        let bbox_mid = bbox.mid();
        let scale = bbox.longest_axis();
        let controller = camera::Camera::new(
            bbox_mid + glam::Vec3::Z * scale * 1.5,
            bbox_mid,
            glam::Vec3::Y,
            1.0,
            std::f32::consts::PI / 4.0,
            0.001,
            scale * 4.0,
        );

        // Buffer to store the camera matrix as uniform data
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<glam::Mat4>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &WGPUCamera::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            controller,
            buffer,
            bind_group,
        }
    }

    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
        })
    }
}

///
///
///
struct WGPULight {
    controller: light::Light,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl WGPULight {
    fn new(device: &wgpu::Device) -> Self {
        let controller = light::Light::default();

        // Buffer to store the camera matrix as uniform data
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size: std::mem::size_of::<light::UniformData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &WGPUCamera::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            controller,
            buffer,
            bind_group,
        }
    }
}

struct WGPUMesh {
    skin_id: Option<u32>,
    material_id: Option<u32>,

    num_triangles: u32,
    vertex_buffer: wgpu::Buffer,
    index_buffer: Option<wgpu::Buffer>,
}

impl WGPUMesh {
    fn from_model_primitive(prim: &model::Primitive, device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(prim.positions.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = if !prim.indices.is_empty() {
            Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(prim.indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                }),
            )
        } else {
            None
        };

        Self {
            skin_id: None,
            material_id: None,
            num_triangles: prim.positions.len() as u32,
            vertex_buffer,
            index_buffer,
        }
    }
}

struct WGPUSkin {}

impl WGPUSkin {
    fn new(model_skin: &model::Skin, device: &wgpu::Device) -> Self {
        Self {}
    }
}

struct WGPUMaterial {}

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
    fn size(&self) -> glam::UVec2 {
        glam::UVec2::new(self.desc.size.width, self.desc.size.height)
    }

    /// return render texture dimensions as f32 (width, height)
    fn sizef(&self) -> glam::Vec2 {
        glam::Vec2::new(self.desc.size.width as f32, self.desc.size.height as f32)
    }
}

/*
pub struct Renderer {
    queue: wgpu::Queue,
    device: wgpu::Device,

    render_texture: RenderTexture,
    shader_module: wgpu::ShaderModule,

    scene: Scene,
}

impl Renderer {
    pub fn new(
        width: u32,
        height: u32,
        device: wgpu::Device,
        queue: wgpu::Queue,
        input_model: &model::Model,
    ) -> Self {
        let render_texture = RenderTexture::new(width, height, &device);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/render.wgsl").into()),
        });

        let mut scene = Scene::from_model(input_model, &device);

        Self {
            queue,
            device,
            render_texture,
            shader_module,
            scene,
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
                    self.camera.camera.camera_view.rotate(drag_begin, drag_end);
                }
                event::MouseButton::Secondary => {
                    self.camera.camera.camera_view.pan(drag_begin - drag_end);
                }
                event::MouseButton::Middle => {
                    self.camera
                        .camera
                        .camera_view
                        .dolly((drag_begin.y - drag_end.y) * self.scene.bbox.longest_axis());
                }
                _ => {}
            }
        }
    }

    pub fn render(&self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let camera_matrix = self.camera.camera.view_projection_matrix();
            self.queue.write_buffer(
                &self.camera.matrix_buffer,
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
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.set_vertex_buffer(0, self.model_state.vertex_buffer.slice(..));
            //render_pass.draw(0..3, 0..1);
            render_pass.draw(0..self.model_state.vertex_count, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn get_render_texture_view(&self) -> &wgpu::TextureView {
        &self.render_texture.view
    }

    pub fn size(&self) -> glam::UVec2 {
        self.render_texture.size()
    }

    fn raster_to_ndc(&self, r: glam::Vec2) -> glam::Vec2 {
        let size = self.render_texture.sizef();
        // invert y
        let r = glam::Vec2::new(r.x, size.y - r.y);

        // center around origin, then scale to [-1,1]^2
        let half_size = size * 0.5;
        (r - half_size) / half_size
    }
}

/// The 3D model to be rendered
///
/// Stores device side representation of the model and metadata to facilitate host-side
/// operations (eg, camera orientation toward model)
struct Scene {
    bbox: model::BoundingBox, // TODO: move to shared location?

    camera: Camera,
    lights: Vec<u32>, // TODO: placeholder

    materials: Vec<u32>, // TODO: placeholder
    textures: Vec<u32>,  // TODO: placeholder
    skins: Vec<u32>,     // TODO: placeholder
    meshes: Vec<Mesh>,
}

impl Scene {
    fn from_model(model: &model::Model, device: &wgpu::Device) -> Self {

        let mut scene = Self {
            bbox: model.bbox,
            camera: camera::Camera::
            lights: Vec::new(),
            materials: Vec::new(),
            textures: Vec::new(),
            skins: Vec::new(),
            meshes: Vec::new(),
        };
        for model_mesh in &model.meshes {
            let skin_id = if let Some(skin) = model_mesh.skin {
                scene.skins.push(skin);
                Some((scene.skins.len() - 1) as u32)
            } else {
                None
            };

            for primitive in &model_mesh.primitives {
                let mut mesh = Mesh::new(primitive, device);
                mesh.skin_id = skin_id;
                scene.meshes.push(mesh);
            }
        }
        scene
    }
}

struct Mesh {
    skin_id: Option<u32>,
    material_id: Option<u32>,

    num_triangles: u32,
    vertex_buffer: wgpu::Buffer,

    index_buffer: Option<wgpu::Buffer>,
}

impl Mesh {
    fn new(primitive: &model::Primitive, device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(primitive.positions.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera.matrix_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[model_state.vertex_buffer_layout()],
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
            vertex_count: mesh.primitives.first().unwrap().positions.len() as u32,
            vertex_buffer,
            model_scale: mesh.bbox.longest_axis(),
        }
    }

    fn vertex_buffer_layout(&self) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<glam::Vec3>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

///
/// Holds the Camera controller and the state needed to upload the camera matrix to the GPU.
///
pub struct Camera {
    pub camera: camera::Camera,
    matrix_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Camera {

    fn new() -> Self {
        Self::default()
    }

    /// Initialize the Camera to look at the provided model and prepare the WGPU uniform
    /// buffer for the on-device camera matrix
    fn look_at_model(bbox: &model::BoundingBox, device: &wgpu::Device) -> Self {
        // Generate initial viewing params
        let bbox_mid = bbox.mid();
        let longest_axis = bbox.longest_axis();
        let camera = camera::Camera::new(
            bbox_mid + glam::Vec3::new(0.0, 0.0, longest_axis * 1.5),
            bbox_mid,
            (0.0, 1.0, 0.0).into(),
            std::f32::consts::PI / 4.0,
            1.0,
            0.01,
            longest_axis * 10.0,
        );

        // Initialize matrix wgpu buffer and binding
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

        // Create camera state
        Self {
            camera,
            matrix_buffer,
            matrix_bind_group_layout,
            matrix_bind_group,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
    }
}


///
/// A WGPU texture to be used as a rendering output target.
///
/// Hard codes texture format to be Rgba8UnormSrgb as required by egui_wgpu.
/// Texture dims are fixed at creation time.
///
struct RenderTexture {
    desc: wgpu::TextureDescriptor<'static>,
    view: wgpu::TextureView,
}

impl RenderTexture {
    /// View format is always Rgba8UnormSrgb as required by egui
    const VIEW_FORMATS: &[wgpu::TextureFormat] = &[wgpu::TextureFormat::Rgba8UnormSrgb];

    /// Create WGPU texture for render target
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

        Self { desc, view }
    }

    /// return render texture dimensions (width, height)
    fn size(&self) -> glam::UVec2 {
        glam::UVec2::new(self.desc.size.width, self.desc.size.height)
    }

    /// return render texture dimensions as f32 (width, height)
    fn sizef(&self) -> glam::Vec2 {
        glam::Vec2::new(self.desc.size.width as f32, self.desc.size.height as f32)
    }
}
*/
