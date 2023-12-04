use std::iter;

use cgmath::prelude::*;
use cgmath::Vector3;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    window::Window,
};
use instance::{Instance, InstanceRaw};

mod model;
mod resources;
mod texture;
pub mod ecs;
pub mod level;
pub mod camera;
pub mod instance;
pub mod world;
pub mod window;
pub mod graphics;
pub mod node;

use model::{DrawModel, Vertex};
use crate::graphics::GraphicsContext;
use crate::window::WindowEvents;
use crate::world::CameraBundle;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


pub struct State {
    pub context: GraphicsContext,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub depth_texture: texture::Texture,
    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,

    pub camera_bundle: CameraBundle,
    obj_model: model::Model,
}

impl State {
    async fn new(window: &window::Window) -> Self {
        let size = window.window.inner_size();

        let context = GraphicsContext::new(window).await;

        let device = &context.device;
        let config = &context.config;
        let queue = &context.queue;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let fbm = Fbm::<Perlin>::new(0).set_frequency(0.01f64);

        let mut instances: Vec<Instance> = vec![];
        let rotation = cgmath::Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Deg(0.0));
        for x in 0..128 {
            for z in 0..128 {
                let y = (fbm.get([x as f64, z as f64]) * 10.0).round() as f32;
                let position = Vector3::new(x as f32, y, z as f32) * 2f32;
                instances.push(Instance::new(position, rotation));
            }
        }

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let camera_bundle = CameraBundle::new(&config, &device);
        log::warn!("Load model");
        let obj_model =
            resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("graphics/pass/shader.wgsl").into()),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bundle.camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        Self {
            context,
            size,
            render_pipeline,
            camera_bundle,
            obj_model,
            instances,
            instance_buffer,
            depth_texture,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.camera_bundle.camera.aspect = self.context.config.width as f32 / self.context.config.height as f32;
            self.size = new_size;
            self.context.config.width = new_size.width;
            self.context.config.height = new_size.height;
            self.context.surface.configure(&self.context.device, &self.context.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.context.device, &self.context.config, "depth_texture");
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_bundle.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera_bundle.camera_controller.update_camera(&mut self.camera_bundle.camera);
        self.camera_bundle.camera_uniform.update_view_proj(&self.camera_bundle.camera);
        self.context.queue.write_buffer(
            &self.camera_bundle.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_bundle.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.camera_bundle.camera_bind_group,
            );
        }

        self.context.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run(){
    env_logger::init();

    let window = window::Window::new();
    let mut state = State::new(&window).await;
    window.run(move |event| match event {
        WindowEvents::Resized { width, height } => { state.resize(winit::dpi::PhysicalSize{width, height})}
        WindowEvents::Keyboard { state, virtual_keycode } => { }
        WindowEvents::Draw => {
            state.update();
            state.render().expect("TODO: render panic message");
        }
    })
}
