    pub struct UniformData {
        _enabled: bool,
        _dir: glam::Vec3,
        _color: glam::Vec3,
        _scale: f32,
        _ambient: glam::Vec3,
        _ambient_scale: f32,
    }

    impl Default for UniformData {
        fn default() -> Self {
            Self {
                _enabled: true,
                _dir: glam::Vec3::new(-1.0, -1.0, -1.0).normalize(),
                _color: glam::Vec3::new(0.9, 0.6, 0.3),
                _scale: 0.7,
                _ambient: glam::Vec3::new(0.5, 0.5, 0.7),
                _ambient_scale: 0.3,
            }
        }
    }

    #[derive(Default)]
    pub struct Light {
        _uniform_data: UniformData,
    }
