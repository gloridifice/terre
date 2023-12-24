use bytemuck::Zeroable;
use cgmath::{Matrix3, Matrix4, Quaternion, Vector3};
use hecs::{ComponentRefShared, Entity};
use crate::app::{App, Plugin, State};
use crate::schedule::Stage::Update;



pub struct TransformPlugin;
impl Plugin for TransformPlugin{
    fn build(&self, app: App) -> App {
        // app.add_system(Update, transform_update_system)
        app
    }
}

pub struct Transform {
    /// todo parent part
    /// set_parent()
    pub parent: Option<Entity>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>
}

pub struct GlobalTransform(
    pub Matrix4<f32>,
    pub Matrix3<f32>
);

impl GlobalTransform{
    pub fn new(transform: &Transform) -> GlobalTransform{
        let trans = Matrix4::from_translation(transform.position);
        let rot = Matrix4::from(transform.rotation);
        //scale todo
        GlobalTransform(trans * rot, Matrix3::from(transform.rotation))
    }
    
    pub fn update(&mut self, transform: &Transform){
        let trans = Matrix4::from_translation(transform.position);
        let rot = Matrix4::from(transform.rotation);
        
        self.0 = trans * rot;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GlobalTransformRaw {
    #[allow(dead_code)]
    world: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

unsafe impl Zeroable for GlobalTransformRaw {}
unsafe impl bytemuck::Pod for GlobalTransformRaw {}

impl Default for GlobalTransformRaw{
    fn default() -> Self {
        Self{
            ..Default::default()
        }
    }
}
impl GlobalTransformRaw {
    pub fn from_global_transform(global: &GlobalTransform) -> Self{
        GlobalTransformRaw {
            world: global.0.into(),
            normal: global.1.into()
        }
    }
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GlobalTransformRaw>() as wgpu::BufferAddress,

            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                //world---
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                //normal---
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
