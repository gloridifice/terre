use std::ops::Range;
use wgpu::{VertexBufferLayout};
use crate::render::texture;

pub trait Vertex{
    fn desc() -> wgpu::VertexBufferLayout<'static>;
    // fn get_position(&self) -> Option<&[f32; 3]> ;
    // fn get_tex_coords(&self) -> Option<&[f32; 2]>;
    // fn get_normal(&self) -> Option<&[f32; 3]>;
    // fn get_color(&self) -> Option<&[f32; 3]>;
    // fn get_normal_tex_coords(&self) -> Option<&[f32; 3]>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelVertex{
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

unsafe  impl bytemuck::Zeroable for ModelVertex{}
unsafe  impl bytemuck::Pod for ModelVertex{}

impl Vertex for ModelVertex{
    fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout{
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}

pub struct Material{
    pub name: String,
    pub diffuse_texture: texture::Texture,
}

pub struct Mesh{
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct Model{
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}
