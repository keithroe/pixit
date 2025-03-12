/// For now assuming only a single set of a given attribute semantic per mesh (eg, single color
/// set).
#[derive(Default, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Specification {
    pub has_normals: bool,
    pub has_tangents: bool,
    pub has_texcoords: bool,
    pub has_colors: bool,
    pub has_joints: bool,
}

impl Specification {
    fn get_defines(self) -> wgpu::naga::FastHashMap<String, String> {
        let mut defines = wgpu::naga::FastHashMap::default();

        if self.has_normals {
            defines.insert("HAS_NORMALS".to_string(), "1".to_string());
        }
        if self.has_tangents {
            defines.insert("HAS_TANGENTS".to_string(), "1".to_string());
        }
        if self.has_texcoords {
            defines.insert("HAS_TEXCOORDS".to_string(), "1".to_string());
        }
        if self.has_colors {
            defines.insert("HAS_COLORS".to_string(), "1".to_string());
        }
        if self.has_joints {
            defines.insert("HAS_JOINTS".to_string(), "1".to_string());
        }

        defines
    }
}

pub struct Cache {
    device: wgpu::Device,
    cache: std::collections::HashMap<Specification, (wgpu::ShaderModule, wgpu::ShaderModule)>,
}

impl Cache {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            device: device.clone(),
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn get_modules(
        &mut self,
        spec: Specification,
    ) -> &(wgpu::ShaderModule, wgpu::ShaderModule) {
        if self.cache.contains_key(&spec) {
            return self.cache.get(&spec).unwrap();
        }

        let vert_shader_source = wgpu::ShaderSource::Glsl {
            shader: include_str!("../shader/vert.glsl").into(),
            stage: wgpu::naga::ShaderStage::Vertex,
            defines: spec.get_defines(),
        };

        let vert_shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("VertShader"),
                source: vert_shader_source,
            });

        let frag_shader_source = wgpu::ShaderSource::Glsl {
            shader: include_str!("../shader/frag.glsl").into(),
            stage: wgpu::naga::ShaderStage::Fragment,
            defines: spec.get_defines(),
        };

        let frag_shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("FragShader"),
                source: frag_shader_source,
            });

        self.cache
            .insert(spec, (vert_shader_module, frag_shader_module));
        self.cache.get(&spec).unwrap()
    }
}
