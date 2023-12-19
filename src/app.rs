use winit::window::{Window, WindowBuilder};
use winit::event::{Event, KeyboardInput, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use cgmath::vec2;
use wgpu::TextureFormat::Bgra8UnormSrgb;
use crate::camera::{Camera, CameraController};
use crate::ecs::{KeyHandleSystem, Stage, System};
use crate::egui_renderer::EguiRenderer;
use crate::graphics::GraphicsContext;
use crate::graphics::pass::phong::{PhongConfig, PhongPass};
use crate::input::CursorInput;
use crate::node::Node;
use crate::texture;

pub struct Runtime {
    pub context: GraphicsContext,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub world: hecs::World,

    pub camera: Camera,
    pub camera_controller: CameraController,
    pub input: CursorInput,
    pub pass: PhongPass,
    pub nodes: Vec<Node>,
    pub window: Window,
    pub egui_renderer: EguiRenderer,
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

        let nodes = crate::create_nodes(device, queue).await;
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

        let world = hecs::World::new();

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
            world
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
}

pub struct App {
    updates: Vec<Box<dyn System>>,
    starts: Vec<Box<dyn System>>,
    key_board_handles: Vec<Box<dyn KeyHandleSystem>>,
}

impl App {
    pub fn new() -> Self {
        App {
            updates: vec![],
            starts: vec![],
            key_board_handles: vec![],
        }
    }

    pub fn add_key_handle(mut self, handle: impl Fn(&mut Runtime, &KeyboardInput) + 'static) -> Self{
        self.key_board_handles.push(Box::new(handle));
        self
    }
    pub fn add_system(mut self, stage: Stage, system: impl Fn(&mut Runtime) + 'static) -> Self {
        match stage {
            Stage::Start => self.starts.push(Box::new(system)),
            Stage::Update => self.updates.push(Box::new(system)),
        };
        self
    }
    pub fn add_plugin(mut self, plugin: impl Plugin + 'static) -> Self{
        plugin.run(&mut self);
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
                            input,
                            ..
                        } => {
                            self.key_board_handles.iter().for_each(|it| {
                                it.run(&mut runtime, input)
                            });
                        }
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

pub trait Plugin{
    fn run(&self, app: &mut App);
}
