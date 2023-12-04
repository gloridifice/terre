use wgpu::{Device, Queue, Surface};

use crate::node::Node;

pub mod phong;

pub trait Pass {
    fn draw(
        &mut self,
        surface: &Surface,
        device: &Device,
        queue: &Queue,
        node: &Node,
    ) -> Result<(), wgpu::SurfaceError>;
}