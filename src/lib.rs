use std::iter;

use cgmath::prelude::*;
use cgmath::{Deg, Quaternion, Rad, vec2, Vector3};
use log::{info, log};
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use wgpu::{Device, Queue};
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
pub mod input;

use model::{DrawModel, Vertex};
use crate::camera::{Camera, CameraController, CameraUniform};
use crate::graphics::GraphicsContext;
use crate::graphics::pass::Pass;
use crate::graphics::pass::phong::{PhongConfig, PhongPass};
use crate::input::CursorInput;
use crate::node::Node;
use crate::window::WindowEvents;

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

    pub camera: Camera,
    pub camera_controller: CameraController,
    pub input: CursorInput,
    pub pass: PhongPass,
    pub nodes: Vec<Node>,
}

pub struct App;
impl App{
    pub fn new() -> Self{
        App
    }

    pub fn run(){

    }
}

async fn create_nodes(device: &Device, queue: &Queue) -> Vec<Node> {
    log::warn!("Load model");
    let model =
        resources::load_model("cube.obj", &device, &queue)
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

impl State {
    async fn new(window: &window::Window) -> Self {
        let size = window.window.inner_size();

        let context = GraphicsContext::new(window).await;
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
        Self {
            context,
            size,
            camera,
            camera_controller,
            pass: phong_pass,
            input,
            nodes,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.camera.aspect = self.context.config.width as f32 / self.context.config.height as f32;
            self.size = new_size;
            self.context.config.width = new_size.width;
            self.context.config.height = new_size.height;
            self.context.surface.configure(&self.context.device, &self.context.config);
            self.pass.depth_texture =
                texture::Texture::create_depth_texture(&self.context.device, &self.context.config, "depth_texture");
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        let offset = self.input.cursor_position - self.input.last_cursor_position;
        self.camera.rotate(
            &(
                Quaternion::from_axis_angle(self.camera.up, Deg(-offset.x as f32) / 4f32)
                // * Quaternion::from_axis_angle(self.camera.forward(), Deg(offset.y as f32))
            )
        );

        self.camera_controller.update_camera(&mut self.camera);
        self.pass.camera_uniform.update(&self.camera);

        self.input.last_cursor_position = self.input.cursor_position;
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.nodes.iter().for_each(|it| {
            self.pass.draw(&self.context.surface, &self.context.device, &self.context.queue, it).expect("Draw failed!");
        });

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();

    let window = window::Window::new();
    let mut state = State::new(&window).await;
    window.run(move |event| match event {
        WindowEvents::Resized { width, height } => { state.resize(winit::dpi::PhysicalSize { width, height }) }
        WindowEvents::Keyboard { state: element_state, virtual_keycode } => {
           state.camera_controller.process_events(element_state, virtual_keycode);
        }
        WindowEvents::Cursor { position } => {
            state.input.cursor_position = vec2(position.x, position.y)
        }
        WindowEvents::Draw => {
            state.update();
            state.render().expect("TODO: render panic message");
        }
    })
}
