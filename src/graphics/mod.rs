pub mod pass;

use pollster::block_on;
use wgpu::TextureFormat::Bgra8UnormSrgb;
use winit::window::Window;
use crate::app::{App, Plugin, Runtime};
use crate::camera::{Camera, CameraController};
use crate::ecs::resource::Res;
use crate::ecs::Stage;
use crate::egui_renderer::EguiRenderer;
use crate::graphics::pass::phong::{PhongConfig, PhongPass};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin{
    fn run(&self, app: App) -> App{
        app.add_system(Stage::Start, start_graphics)
    }
}

fn start_graphics(runtime: &mut Runtime){
    let camera = Camera::new(runtime.size.width as f32 / runtime.size.height as f32);
    let camera_controller = CameraController::new(0.5f32);
    let context = GraphicsContext::new(&runtime.window);
    let context = block_on(context);

    let phong_pass = PhongPass::new(
        &PhongConfig {
            max_lights: 0,
            ambient: [1, 1, 1, 1],
        },
        &context.device,
        &context.queue,
        &context.config,
        &camera,
    );
    let egui_renderer = EguiRenderer::new(&context.device, Bgra8UnormSrgb, None, 1, &runtime.window);

    runtime.res_manager.push_res(camera);
    runtime.res_manager.push_res(camera_controller);
    runtime.res_manager.push_res(context);
    runtime.res_manager.push_res(phong_pass);
    runtime.res_manager.push_res(egui_renderer);
}

impl Res for GraphicsContext{}
impl Res for PhongPass{}

pub struct GraphicsContext {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl GraphicsContext {
    pub async fn new(window: &Window) -> Self {
        let size = &window.inner_size();

        log::warn!("WGPU setup");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        log::warn!("device and queue");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        log::warn!("Surface");
        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        GraphicsContext {
            surface,
            device,
            queue,
            config,
        }
    }
}

