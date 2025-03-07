use wgpu::util::DeviceExt;

use crate::camera;
use crate::light;

/// 3D scene for simple wgpu renderer
///
///  
pub struct Scene {
    pub bbox: model::BoundingBox,
    pub camera: WGPUCamera,
    pub _light: WGPULight,
    pub _materials: Vec<WGPUMaterial>,
    pub skins: Vec<WGPUSkin>,
    pub meshes: Vec<WGPUMesh>,
}

impl Scene {
    pub fn from_model(model: &model::Model, device: &wgpu::Device) -> Self {
        let mut scene = Self {
            bbox: model.bbox,
            camera: WGPUCamera::with_looking_at(&model.bbox, device),
            _light: WGPULight::new(device),
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
                let mut wgpu_mesh = WGPUMesh::from_model_primitive(primitive, device);
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
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl WGPUCamera {
    pub fn with_looking_at(bbox: &model::BoundingBox, device: &wgpu::Device) -> Self {
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
            label: Some("camera_bind_group_layout"),
        })
    }
}

/// Light controller and its WGPU state for device-side light data
pub struct WGPULight {
    pub _controller: light::Light,
    pub _buffer: wgpu::Buffer,
    pub _bind_group: wgpu::BindGroup,
}

impl WGPULight {
    pub fn new(device: &wgpu::Device) -> Self {
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
            _controller: controller,
            _buffer: buffer,
            _bind_group: bind_group,
        }
    }
}

pub struct WGPUMesh {
    pub skin_id: Option<u32>,
    pub material_id: Option<u32>,

    pub num_triangles: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub _index_buffer: Option<wgpu::Buffer>,
}

impl WGPUMesh {
    pub fn from_model_primitive(prim: &model::Primitive, device: &wgpu::Device) -> Self {
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
            _index_buffer: index_buffer,
        }
    }
}

pub struct WGPUSkin {}

impl WGPUSkin {
    pub fn new(_model_skin: &model::Skin, _device: &wgpu::Device) -> Self {
        Self {}
    }
}

pub struct WGPUMaterial {}
