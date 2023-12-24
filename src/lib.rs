use cgmath::prelude::*;
use cgmath::{Deg, Quaternion, Vector3};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::pixels_per_point;
use instance::Instance;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use wgpu::{Device, Queue};
use winit::event::*;
use app::{App, Runtime};
use ecs::Stage;
use crate::camera::{Camera, CameraController};

pub mod camera;
pub mod ecs;
pub mod egui_renderer;
pub mod graphics;
pub mod input;
pub mod instance;
mod model;
pub mod node;
mod resources;
mod texture;
pub mod window;
pub mod app;
pub mod component;
pub mod asset;

use crate::graphics::pass::Pass;
use crate::graphics::pass::phong::PhongPass;
use crate::node::Node;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

async fn create_nodes(device: &Device, queue: &Queue) -> Vec<Node> {
    log::warn!("Load model");
    let model = resources::load_model("cube.obj", &device, &queue)
        .await
        .unwrap();

    let fbm = Fbm::<Perlin>::new(0).set_frequency(0.01f64);

    let mut instances: Vec<Instance> = vec![];
    let rotation = Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Deg(0.0));
    for x in 0..128 {
        for z in 0..128 {
            let y = (fbm.get([x as f64, z as f64]) * 10.0).round() as f32;
            let position = Vector3::new(x as f32, y, z as f32) * 2f32;
            instances.push(Instance::new(position, rotation));
        }
    }

    vec![Node::new(device, 0, model, instances)]
}
fn update(runtime: &mut Runtime) {
    let offset = runtime.input.cursor_position - runtime.input.last_cursor_position;
    
    let camera = runtime.res_manager.get_res_mut::<Camera>().unwrap();
    let camera_controller = runtime.res_manager.get_res_mut::<CameraController>().unwrap();
    let pass = runtime.res_manager.get_res_mut::<PhongPass>().unwrap();
    camera.rotate(
        &(
            Quaternion::from_axis_angle(runtime.camera.up, Deg(-offset.x as f32) / 4f32)
            // * Quaternion::from_axis_angle(runtime.camera.forward(), Deg(offset.y as f32))
        ),
    );

    camera_controller.update_camera(&mut camera);
    pass.camera_uniform.update(&runtime.camera);

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
                    ui.add(egui::Slider::new(&mut runtime.camera.fovy, 10f32..=120f32).text("fovy"))
                });
        },
    );

    queue.submit(Some(encoder.finish()));
    output.present();
}
fn handle_camera_key_controller(runtime:&mut Runtime, input: &KeyboardInput){
    runtime.camera_controller.process_events(&input.state, &input.virtual_keycode);
}
pub async fn run() {
    App::new()
        .add_system(Stage::Update, update)
        .add_system(Stage::Update, render)
        .add_key_handle(handle_camera_key_controller)
        .run()
        .await;
}
