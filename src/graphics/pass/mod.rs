use wgpu::{CommandEncoder, Device, Queue, Surface, TextureView};

use crate::node::Node;

pub mod phong;

pub trait Pass {
    fn draw(
        &mut self,
        surface: &Surface,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        node: &Node,
    ) -> Result<(), wgpu::SurfaceError>;
}