// TODO: hide camera classes outside of crate

/// Simple camera controller for interactive viewing.  Allows panning, rotating around look-at
/// point, and dollying.  All input operations in normalized device coordinate space ([-1,1]^2
/// over view-plane).  Uses acrball control.
struct CameraView {
    pan_translation: glam::Vec3,
    dolly_translation: glam::Vec3,
    rotation: glam::Quat,
}

impl Default for CameraView {
    fn default() -> Self {
        CameraView::new(glam::Vec3::Z, glam::Vec3::ZERO, glam::Vec3::Y)
    }
}

impl CameraView {
    fn new(eye: glam::Vec3, look_at: glam::Vec3, up: glam::Vec3) -> Self {
        let look_dir = look_at - eye;
        let z_axis = look_dir.normalize();
        let x_axis = z_axis.cross(up.normalize()).normalize();
        let y_axis = x_axis.cross(z_axis).normalize();
        let x_axis = z_axis.cross(y_axis);

        let pan_translation = -look_at;
        let dolly_translation = glam::Vec3::new(0.0, 0.0, -look_dir.length());
        let rotation = glam::Quat::from_mat4(&glam::Mat4::transpose(&glam::Mat4::from_cols(
            glam::Vec4::from((x_axis, 0.0)),
            glam::Vec4::from((y_axis, 0.0)),
            glam::Vec4::from((-z_axis, 0.0)),
            glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
        )))
        .normalize();

        Self {
            pan_translation,
            dolly_translation,
            rotation,
        }
    }

    fn rotate(&mut self, pos0: glam::Vec2, pos1: glam::Vec2) {
        // Clamp to NDC
        let pos0 = pos0.clamp(glam::Vec2::new(-1.0, -1.0), glam::Vec2::new(1.0, 1.0));
        let pos1 = pos1.clamp(glam::Vec2::new(-1.0, -1.0), glam::Vec2::new(1.0, 1.0));

        let arcball0 = pos_to_arcball(pos0);
        let arcball1 = pos_to_arcball(pos1);

        self.rotation = arcball1 * arcball0 * self.rotation;
    }

    fn dolly(&mut self, amount: f32) {
        self.dolly_translation += glam::Vec3::new(0.0, 0.0, amount);
    }

    fn pan(&mut self, amount: glam::Vec2) {
        let dolly_scale = self.dolly_translation.z;
        let pan = glam::Vec3::new(amount.x * dolly_scale, amount.y * dolly_scale, 0.0);

        // transform to world space
        let inv_view = self.matrix().inverse();
        let pan = inv_view * glam::Vec4::from((pan, 0.0));
        let pan = glam::Vec3::new(pan.x, pan.y, pan.z);

        self.pan_translation = pan + self.pan_translation;
    }

    fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(self.dolly_translation)
            * glam::Mat4::from_quat(self.rotation)
            * glam::Mat4::from_translation(self.pan_translation)
    }
}

fn pos_to_arcball(pos: glam::Vec2) -> glam::Quat {
    let dist = pos.dot(pos);

    if dist <= 1.0 {
        // Inside sphere
        glam::Quat::from_xyzw(pos.x, pos.y, (1.0 - dist).sqrt(), 0.0)
    } else {
        // Outside sphere -- clip to sphere
        let clip = pos.normalize();
        glam::Quat::from_xyzw(clip.x, clip.y, 0.0, 0.0)
    }
}

/// Camera projection controller.  Allows initial setup of projection transform.  No interactivity.
struct CameraProjection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Default for CameraProjection {
    fn default() -> Self {
        CameraProjection {
            aspect: 1.0,
            fovy: std::f32::consts::PI / 4.0,
            znear: 0.0001,
            zfar: 100.0,
        }
    }
}

impl CameraProjection {
    fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Default)]
pub struct Camera {
    camera_view: CameraView,
    camera_projection: CameraProjection,
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_view(mut self, eye: glam::Vec3, look_at: glam::Vec3, up: glam::Vec3) -> Camera {
        self.camera_view = CameraView::new(eye, look_at, up);
        self
    }

    pub fn with_projection(mut self, aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Camera {
        self.camera_projection = CameraProjection {
            aspect,
            fovy,
            znear,
            zfar,
        };
        self
    }

    pub fn rotate(&mut self, pos0: glam::Vec2, pos1: glam::Vec2) {
        self.camera_view.rotate(pos0, pos1);
    }

    pub fn dolly(&mut self, amount: f32) {
        self.camera_view.dolly(amount);
    }

    pub fn pan(&mut self, amount: glam::Vec2) {
        self.camera_view.pan(amount);
    }

    pub fn view_matrix(&self) -> glam::Mat4 {
        self.camera_view.matrix()
    }

    pub fn projection_matrix(&self) -> glam::Mat4 {
        self.camera_projection.matrix()
    }

    #[allow(unused)]
    pub fn view_projection_matrix(&self) -> glam::Mat4 {
        let view = self.camera_view.matrix();
        let proj = self.camera_projection.matrix();
        proj * view
    }
}
