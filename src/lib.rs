use cgmath::prelude::*;
use cgmath::{vec2, Deg, Quaternion, Vector3};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::pixels_per_point;
use instance::Instance;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use wgpu::util::DeviceExt;
use wgpu::TextureFormat::{Bgra8UnormSrgb, Rgba8Unorm, Rgba8UnormSrgb};
use wgpu::{Device, Queue, StoreOp};
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub mod camera;
pub mod ecs;
pub mod egui_renderer;
pub mod graphics;
pub mod input;
pub mod instance;
pub mod level;
mod model;
pub mod node;
mod resources;
mod texture;
pub mod window;
pub mod world;

use crate::camera::{Camera, CameraController, CameraUniform};
use crate::ecs::System;
use crate::egui_renderer::EguiRenderer;
use crate::graphics::pass::phong::{PhongConfig, PhongPass};
use crate::graphics::pass::Pass;
use crate::graphics::GraphicsContext;
use crate::input::CursorInput;
use crate::node::Node;
use crate::window::{AppWindow, WindowEvents};
use model::{DrawModel, Vertex};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Runtime {
    pub context: GraphicsContext,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub camera: Camera,
    pub camera_controller: CameraController,
    pub input: CursorInput,
    pub pass: PhongPass,
    pub nodes: Vec<Node>,
    pub window: Window,
    pub egui_renderer: EguiRenderer,
}
pub struct App {
    updates: Vec<Box<dyn System>>,
    starts: Vec<Box<dyn System>>,
}
pub enum Stage {
    Start,
    Update,
}
impl App {
    pub fn new() -> Self {
        App {
            updates: vec![],
            starts: vec![],
        }
    }

    pub fn add_system(mut self, stage: Stage, system: impl Fn(&mut Runtime) + 'static) -> Self {
        match stage {
            Stage::Start => self.starts.push(Box::new(system)),
            Stage::Update => self.updates.push(Box::new(system)),
        };
        self
    }

    pub async fn run(mut self) {
        env_logger::init();

        let event_loop = EventLoop::new();
        let mut runtime = Runtime::new(
            WindowBuilder::new()
                .with_title("Terre")
                .build(&event_loop)
                .unwrap(),
        )
        .await;

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == runtime.window.id() => {
                    runtime.egui_renderer.handle_event(event);
                    match event {
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state,
                                    virtual_keycode,
                                    ..
                                },
                            ..
                        } => {}
                        WindowEvent::CursorMoved { position, .. } => {
                            runtime.input.cursor_position = vec2(position.x, position.y);
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            runtime.resize(physical_size.clone())
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            runtime.resize((*new_inner_size).clone())
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == runtime.window.id() => {
                    self.updates.iter().for_each(|it| it.run(&mut runtime));
                }
                Event::RedrawEventsCleared => {
                    runtime.window.request_redraw();
                }
                _ => {}
            }
        });
    }
}

async fn create_nodes(device: &Device, queue: &Queue) -> Vec<Node> {
    log::warn!("Load model");
    let model = resources::load_model("cube.obj", &device, &queue)
        .await
        .unwrap();

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

    vec![Node::new(device, 0, model, instances)]
}

impl Runtime {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let context = GraphicsContext::new(&window).await;
        let device = &context.device;
        let config = &context.config;
        let queue = &context.queue;

        let camera = Camera::new(size.width as f32 / size.height as f32);
        let camera_controller = CameraController::new(0.5f32);

        let nodes = create_nodes(device, queue).await;
        let phong_pass = PhongPass::new(
            &PhongConfig {
                max_lights: 0,
                ambient: [1, 1, 1, 1],
            },
            device,
            queue,
            config,
            &camera,
        );
        let input = CursorInput::new();
        let egui_renderer = EguiRenderer::new(device, Bgra8UnormSrgb, None, 1, &window);
        Self {
            context,
            size,
            camera,
            camera_controller,
            pass: phong_pass,
            input,
            nodes,
            window,
            egui_renderer,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.camera.aspect =
                self.context.config.width as f32 / self.context.config.height as f32;
            self.size = new_size;
            self.context.config.width = new_size.width;
            self.context.config.height = new_size.height;
            self.context
                .surface
                .configure(&self.context.device, &self.context.config);
            self.pass.depth_texture = texture::Texture::create_depth_texture(
                &self.context.device,
                &self.context.config,
                "depth_texture",
            );
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }
}

fn update(runtime: &mut Runtime) {
    let offset = runtime.input.cursor_position - runtime.input.last_cursor_position;
    runtime.camera.rotate(
        &(
            Quaternion::from_axis_angle(runtime.camera.up, Deg(-offset.x as f32) / 4f32)
            // * Quaternion::from_axis_angle(runtime.camera.forward(), Deg(offset.y as f32))
        ),
    );

    runtime.camera_controller.update_camera(&mut runtime.camera);
    runtime.pass.camera_uniform.update(&runtime.camera);

    runtime.input.last_cursor_position = runtime.input.cursor_position;
}
fn render(runtime: &mut Runtime) {
    let surface = &runtime.context.surface;
    let device = &runtime.context.device;
    let queue = &runtime.context.queue;
    let output = surface.get_current_texture().unwrap();
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Egui Render Encoder"),
    });

    runtime.nodes.iter().for_each(|it| {
        runtime
            .pass
            .draw(
                surface,
                device,
                queue,
                &mut encoder,
                &view,
                it,
            )
            .expect("Draw failed!");
    });
    let size = runtime.window.inner_size();
    runtime.egui_renderer.draw(device, queue, &runtime.window, &mut encoder, &view,
        &ScreenDescriptor {
            size_in_pixels: [ size.width, size.height ],
            pixels_per_point: pixels_per_point(
                &runtime.egui_renderer.context,
                &runtime.window,
            ),
        },
        |mut ui| {
            egui::Window::new("Settings")
                .resizable(true)
                .vscroll(true)
                .default_open(false)
                .show(&ui, |mut ui| {
                    ui.label("Window!");
                    ui.label("Window!");
                    ui.label("Window!");
                    ui.label("Window!");
                });
        },
    );

    queue.submit(Some(encoder.finish()));
    output.present();
}
pub async fn run() {
    App::new()
        .add_system(Stage::Update, update)
        .add_system(Stage::Update, render)
        .run()
        .await;
}
