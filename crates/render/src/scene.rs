use wgpu::util::DeviceExt;

use crate::camera;
use crate::light;

/// 3D scene for simple wgpu renderer
///
///  
pub struct Scene {
    pub bbox: model::BoundingBox,
    pub camera: WGPUCamera,
    pub light: WGPULight,
    pub _materials: Vec<WGPUMaterial>,
    pub skins: Vec<WGPUSkin>,
    pub meshes: Vec<WGPUMesh>,
}

impl Scene {
    pub fn from_model(model: &model::Model, device: &wgpu::Device) -> Self {
        let mut scene = Self {
            bbox: model.bbox,
            camera: WGPUCamera::with_looking_at(&model.bbox, device),
            light: WGPULight::new(device),
            _materials: Vec::new(),
            skins: Vec::new(),
            meshes: Vec::new(),
        };

        for mesh in &model.meshes {
            let skin_id = if let Some(skin) = &mesh.skin {
                scene.skins.push(WGPUSkin::new(skin, device));
                Some((scene.skins.len() - 1) as u32)
            } else {
                None
            };

            for primitive in &mesh.primitives {
                let mut wgpu_mesh =
                    WGPUMesh::from_model_primitive(primitive, mesh.transform, device);
                wgpu_mesh.skin_id = skin_id;
                wgpu_mesh.material_id = None;
                scene.meshes.push(wgpu_mesh);
            }
        }
        scene
    }
}

/// Camera controller and WGPU state for device-side camera data
pub struct WGPUCamera {
    pub controller: camera::Camera,
    pub view_proj_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl WGPUCamera {
    pub fn with_looking_at(bbox: &model::BoundingBox, device: &wgpu::Device) -> Self {
        // Generate initial viewing params, camera is looking in -Z direction at
        // center of model's bounding box
        let bbox_mid = bbox.mid();
        let scale = bbox.longest_axis();
        let controller = camera::Camera::new()
            .with_view(
                bbox_mid + glam::Vec3::Z * scale * 1.5, // eye
                bbox_mid,                               // lookat
                glam::Vec3::Y,                          // up
            )
            .with_projection(
                1.0,                        // aspect
                std::f32::consts::PI / 4.0, // yfov
                0.001,                      // znear
                scale * 4.0,                // zfar
            );

        // Buffer to store the camera matrix as uniform data
        let view_proj_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<glam::Mat4>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &WGPUCamera::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_proj_buffer.as_entire_binding(),
            }],
            label: Some("Camera BindGroup"),
        });

        Self {
            controller,
            view_proj_buffer,
            bind_group,
        }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
            label: Some("Camera BindGroupLayout"),
        })
    }
}

/// Light controller and its WGPU state for device-side light data
pub struct WGPULight {
    pub controller: light::Light,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl WGPULight {
    pub fn new(device: &wgpu::Device) -> Self {
        let controller = light::Light::default();

        // Buffer to store light params as uniform data
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size: std::mem::size_of::<light::UniformData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        println!(
            "Host size of light uniformdata: {}",
            std::mem::size_of::<light::UniformData>()
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &WGPULight::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Light BindGroup"),
        });

        Self {
            controller,
            buffer,
            bind_group,
        }
    }

    pub fn update_uniform(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.controller.uniform_data]),
        );
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Light BindGroupLayout"),
        })
    }
}

pub struct WGPUMesh {
    pub transform: glam::Mat4,
    pub skin_id: Option<u32>,
    pub material_id: Option<u32>,

    pub num_triangles: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub normal_buffer: Option<wgpu::Buffer>,
    pub texcoord_buffer: Option<wgpu::Buffer>,

    //pub transform_buffer: wgpu::Buffer,
    pub normal_transform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl WGPUMesh {
    pub fn from_model_primitive(
        prim: &model::Primitive,
        transform: glam::Mat4,
        device: &wgpu::Device,
    ) -> Self {
        let mut num_triangles = prim.positions.len() as u32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(prim.positions.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = if !prim.indices.is_empty() {
            num_triangles = prim.indices.len() as u32;
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(prim.indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            });
            Some(buffer)
        } else {
            None
        };

        let normal_buffer = if !prim.normals.is_empty() {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Normal Buffer"),
                contents: bytemuck::cast_slice(prim.normals.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            });
            Some(buffer)
        } else {
            None
        };

        // TODO: right now only handling first texcoord set
        let texcoord_buffer = if !prim.texcoords.is_empty() && !prim.texcoords[0].is_empty() {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Texcoord Buffer"),
                contents: bytemuck::cast_slice(prim.texcoords[0].as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            });
            Some(buffer)
        } else {
            None
        };

        // Buffer to store the model tranform
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Transform Buffer"),
            contents: bytemuck::cast_slice(&[transform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Buffer to store the normal transform (inv-transpose of model-view)
        let normal_transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Normal Transform Buffer"),
            size: std::mem::size_of::<glam::Mat4>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &WGPUMesh::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: normal_transform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Camera BindGroup"),
        });

        Self {
            transform,
            skin_id: None,
            material_id: None,
            num_triangles,
            vertex_buffer,
            index_buffer,
            normal_buffer,
            texcoord_buffer,

            //transform_buffer,
            normal_transform_buffer,
            bind_group,
        }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("mesh_bind_group_layout"),
        })
    }
}

pub struct WGPUSkin {}

impl WGPUSkin {
    pub fn new(_model_skin: &model::Skin, _device: &wgpu::Device) -> Self {
        Self {}
    }
}

pub struct WGPUMaterial {}
