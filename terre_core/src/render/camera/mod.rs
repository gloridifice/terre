pub mod bundle;

use std::ops::Add;
use cgmath::{InnerSpace, Quaternion, SquareMatrix, Vector3, Zero};
use bytemuck::Zeroable;
use winit::event::{ElementState, VirtualKeyCode};
use terre_core_macros::Resource;


#[derive(Resource)]
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            eye: (0.0, 5.0, -10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }

    pub fn forward(&self) -> Vector3<f32> {
        self.target - self.eye
    }
    pub fn move_position(&mut self, vec: Vector3<f32>) {
        self.eye += vec;
        self.target += vec;
    }

    pub fn rotate(&mut self, quaternion: &Quaternion<f32>) {
        let offset = self.target - self.eye;
        let a = quaternion * offset;
        self.target = self.eye.add(a);
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

unsafe impl Zeroable for CameraUniform {}

unsafe impl bytemuck::Pod for CameraUniform {}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: cgmath::Vector4::zero().into(),
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = (crate::render::OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
        self.view_position = [camera.eye.x, camera.eye.y, camera.eye.y, 1f32];
    }
}

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, state: &ElementState, keycode: &Option<VirtualKeyCode>) -> bool {
        match keycode {
            None => false,
            Some(keycode) => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let left = camera.up.cross(forward);

        let forward_norm = forward.normalize();
        let left_norm = left.normalize();
        let forward_mag = forward.magnitude();

        let mut forward_move = forward_norm;
        forward_move.y = 0.;


        if self.is_forward_pressed && forward_mag > self.speed {
            camera.move_position(forward_move * self.speed);
        }
        if self.is_backward_pressed {
            camera.move_position(-forward_move * self.speed);
        }
        if self.is_up_pressed {
            camera.move_position(Vector3::unit_y() * self.speed);
        }
        if self.is_down_pressed {
            camera.move_position(-Vector3::unit_y() * self.speed);
        }

        if self.is_right_pressed {
            camera.move_position(-left_norm * self.speed);
        }
        if self.is_left_pressed {
            camera.move_position(left_norm * self.speed);
        }
    }
}
