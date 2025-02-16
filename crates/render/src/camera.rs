/// * operates in NDC
/// * arcball for view
pub struct CameraView {
    pan_translation: glam::Vec3,
    dolly_translation: glam::Vec3,
    rotation: glam::Quat,
    //camera: glam::Mat4,
    //inv_camera: glam::Mat4,
}

impl CameraView {
    pub fn new(eye: glam::Vec3, look_at: glam::Vec3, up: glam::Vec3) -> Self {
        println!("YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY");
        let look_dir = look_at - eye;
        let z_axis = look_dir.normalize();
        let x_axis = z_axis.cross(up.normalize()).normalize();
        let y_axis = x_axis.cross(z_axis).normalize();
        let x_axis = z_axis.cross(y_axis);

        println!("x: {}", x_axis);
        println!("y: {}", y_axis);
        println!("z: {}", z_axis);

        let pan_translation = -look_at;
        let dolly_translation = glam::Vec3::new(0.0, 0.0, -look_dir.length());
        let rotation = glam::Quat::from_mat4(&glam::Mat4::transpose(&glam::Mat4::from_cols(
            glam::Vec4::from((x_axis, 0.0)),
            glam::Vec4::from((y_axis, 0.0)),
            glam::Vec4::from((-z_axis, 0.0)),
            glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
        )))
        .normalize();

        println!("pan  : {}", pan_translation);
        println!("dolly: {}", dolly_translation);
        println!("rotation: {}", rotation);
        println!("rotation_matrix: {}", glam::Mat4::from_quat(rotation));

        Self {
            pan_translation,
            dolly_translation,
            rotation,
        }
    }

    pub fn rotate(&mut self, pos0: glam::Vec2, pos1: glam::Vec2) {
        // Clamp to NDC
        println!("p0: {}", pos0);
        println!("p1: {}", pos1);
        let pos0 = pos0.clamp(glam::Vec2::new(-1.0, -1.0), glam::Vec2::new(1.0, 1.0));
        let pos1 = pos1.clamp(glam::Vec2::new(-1.0, -1.0), glam::Vec2::new(1.0, 1.0));
        println!("p0: {}", pos0);
        println!("p1: {}", pos1);

        let arcball0 = pos_to_arcball(pos0);
        let arcball1 = pos_to_arcball(pos1);
        println!("ab0: {}", arcball0);
        println!("ab1: {}", arcball1);

        self.rotation = arcball1 * arcball0 * self.rotation;
        println!("self.rotation: {}", self.rotation);
    }

    pub fn dolly(&mut self, amount: f32) {
        self.dolly_translation += glam::Vec3::new(0.0, 0.0, amount);
    }

    pub fn pan(&mut self, amount: glam::Vec2) {
        let dolly_scale = self.dolly_translation.z;
        let pan = glam::Vec3::new(amount.x * dolly_scale, amount.y * dolly_scale, 0.0);

        // transform to world space
        let inv_view = self.matrix().inverse();
        let pan = inv_view * glam::Vec4::from((pan, 0.0));
        let pan = glam::Vec3::new(pan.x, pan.y, pan.z);

        self.pan_translation = pan + self.pan_translation;
    }

    pub fn matrix(&self) -> glam::Mat4 {
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

struct CameraProjection {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl CameraProjection {
    fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

// TODO: API here
pub struct Camera {
    eye: glam::Vec3,
    look_at: glam::Vec3,
    up: glam::Vec3,
    pub camera_view: CameraView,
    pub camera_projection: CameraProjection,
}

impl Camera {
    // TODO: builder
    pub fn new(
        eye: glam::Vec3,
        look_at: glam::Vec3,
        up: glam::Vec3,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            look_at,
            up,
            camera_view: CameraView::new(eye, look_at, up),
            camera_projection: CameraProjection {
                aspect,
                fovy,
                znear,
                zfar,
            },
        }
    }
    pub fn view_projection_matrix(&self) -> glam::Mat4 {
        let view = self.camera_view.matrix();
        println!("rotation: {}", self.camera_view.rotation);
        println!("view_0: {}", view);
        //let view = glam::Mat4::look_at_rh(self.eye, self.look_at, self.up);
        let proj = self.camera_projection.matrix();
        proj * view
    }
}
