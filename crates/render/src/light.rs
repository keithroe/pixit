#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UniformData {
    _dir: [f32; 3],
    _enabled: u32,
    _color: [f32; 3],
    _scale: f32,
    _ambient: [f32; 3],
    _ambient_scale: f32,
}

impl Default for UniformData {
    fn default() -> Self {
        Self {
            _dir: [-0.5, -1.0, -0.5],
            _enabled: 1,
            _color: [0.9, 0.8, 0.3],
            _scale: 0.7,
            _ambient: [0.5, 0.5, 0.7],
            _ambient_scale: 0.3,
        }
    }
}

#[derive(Default)]
pub struct Light {
    pub uniform_data: UniformData,
}
