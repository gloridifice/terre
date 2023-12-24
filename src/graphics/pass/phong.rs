use std::{collections::HashMap, mem};
use bytemuck::{Pod, Zeroable};

use wgpu::{util::DeviceExt, BindGroupLayout, Device, Queue, Surface, StoreOp, CommandEncoder, TextureView};

use crate::{camera::{Camera, CameraUniform}, instance::{InstanceRaw}, model::{self, Vertex}, render, texture};
use crate::node::Node;

use super::Pass;

#[repr(C)]
#[derive(Clone, Copy)]
struct Globals {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    ambient: [f32; 4],
}
unsafe impl Zeroable for Globals {}
unsafe impl Pod for Globals{}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    pub color: [f32; 3],
    _padding2: u32,
    pub direction: [f32; 3],
    _padding3: u32,
}
unsafe impl Zeroable for LightUniform {}
unsafe impl Pod for LightUniform{}

pub struct PhongConfig {
    pub max_lights: usize,
    pub ambient: [u32; 4],
}

pub struct PhongPass {
    // Uniforms
    pub global_bind_group_layout: BindGroupLayout,
    pub global_uniform_buffer: wgpu::Buffer,
    pub global_bind_group: wgpu::BindGroup,
    pub local_bind_group_layout: BindGroupLayout,
    local_bind_groups: HashMap<usize, wgpu::BindGroup>,
    // Textures
    pub depth_texture: texture::Texture,
    // Render pipeline
    pub render_pipeline: wgpu::RenderPipeline,
    // Lighting
    pub light_uniform: LightUniform,
    pub light_buffer: wgpu::Buffer,
    // Camera
    pub camera_uniform: CameraUniform,
}

impl PhongPass {
    pub fn new(
        phong_config: &PhongConfig,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        camera: &Camera,
    ) -> PhongPass {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Normal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Setup global uniforms
        // Global bind group layout
        let light_size = mem::size_of::<LightUniform>() as wgpu::BufferAddress;
        let global_size = mem::size_of::<Globals>() as wgpu::BufferAddress;
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("[Phong] Globals"),
                entries: &[
                    // Global uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(global_size),
                        },
                        count: None,
                    },
                    // Lights
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(light_size),
                        },
                        count: None,
                    },
                    // Sampler for textures
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Global uniform buffer
        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("[Phong] Globals"),
            size: global_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Create light uniforms and setup buffer for them
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
            direction: [1.0, 1.0, 0.0],
            _padding3: 0
        };
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("[Phong] Lights"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // We also need a sampler for our textures
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("[Phong] sampler"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("[Phong] Globals"),
            layout: &global_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("[Phong] Locals"),
                entries: &[
                    // Mesh texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Setup the render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("[Phong] Pipeline"),
            bind_group_layouts: &[&global_bind_group_layout, &local_bind_group_layout],
            push_constant_ranges: &[],
        });
        let vertex_buffers = [model::ModelVertex::desc(), InstanceRaw::desc()];
        let depth_stencil = Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        });
        let primitive = wgpu::PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        };
        let multisample = wgpu::MultisampleState {
            ..Default::default()
        };
        let color_format = texture::Texture::DEPTH_FORMAT;
        // let color_format = wgpu::TextureFormat::Depth24Plus;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("[Phong] Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            primitive,
            depth_stencil,
            multisample,
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        // Create depth texture
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        // Setup camera uniform
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);

        PhongPass {
            global_bind_group_layout,
            global_uniform_buffer,
            global_bind_group,
            local_bind_group_layout,
            local_bind_groups: Default::default(),
            depth_texture,
            render_pipeline,
            camera_uniform,
            light_uniform,
            light_buffer,
        }
    }
}

impl Pass for PhongPass {
    fn draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        node: &Node,
    ) -> Result<(), wgpu::SurfaceError> {
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Set the clear color during redraw
                        // This is basically a background color applied if an object isn't taking up space
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                // Create a depth stencil buffer using the depth texture
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.local_bind_groups.entry(0).or_insert_with(|| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("[Phong] Locals"),
                    layout: &self.local_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &node.model.materials[0].diffuse_texture.view,
                            ),
                        },
                    ],
                })
            });
            queue.write_buffer(&self.global_uniform_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

            render_pass.set_vertex_buffer(1, node.instance_buffer().slice(..));

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.global_bind_group, &[]);
            render_pass.set_bind_group(1, &self.local_bind_groups.get(&0).unwrap(), &[]);

            for mesh in &node.model.meshes{
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..*&node.instances_len() as u32);
            }

        }


        Ok(())
    }
}