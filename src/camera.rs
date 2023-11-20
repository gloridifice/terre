use bytemuck::{Pod, Zeroable};

pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}
impl Default for Camera{
    fn default() -> Self {
        Self{
            eye: (0., 1., 2.).into(),
            target: (0., 0., 0.).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: 1.,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.,
        }
    }
}
impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn new_default(aspect: f32) -> Self{
        Camera{
            aspect,
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform{
    view_proj: [[f32; 4]; 4],
}
unsafe impl Zeroable for CameraUniform{}
unsafe impl Pod for CameraUniform{}
impl CameraUniform{
    pub fn new() -> Self{
        use cgmath::SquareMatrix;
        Self{
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
    pub fn update_view_proj(&mut self, camera: &Camera){
        self.view_proj = camera.build_view_projection_matrix().into()
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
