use std::collections::HashMap;
use hecs::World;
use uuid::Uuid;
use wgpu::CommandEncoder;
use winit::window::Window;
use crate::render::model::Model;
use crate::render::pass::{Pass, PassQueue};

pub mod texture;
pub mod model;
pub mod pass;
pub mod work;
pub mod material;
pub mod camera;



pub struct RenderState {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_context: RenderContext,
    pub pass_queue: PassQueue,
    pub window: Window,
}

impl RenderState {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let render_context = RenderContext::new(&window).await;
        let pass_queue = PassQueue::new();

        Self {
            size,
            render_context,
            window,
            pass_queue,
        }
    }
}



/// Package of render target.
/// Including *view, format, size*.
pub struct Target {
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub size: wgpu::Extent3d,
}

#[derive(PartialEq, Copy, Clone, Hash, Eq)]
pub struct ModelRef(Uuid);

/// The main context of Rendering,
/// just use `RenderContext::new()` to create a new context.
pub struct RenderContext {
    pub instance: wgpu::Instance,
    pub surface: Option<RenderSurface>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub models: HashMap<ModelRef, Model>,
}

/// Struct about surface
pub struct RenderSurface {
    pub raw: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

/// A wrapper including output view (see [`Target`]) and [`CommandEncoder`], which are used just one frame.
/// Will be created and dropped every frame.
pub struct FrameContext {
    pub output: Target,
    pub encoder: CommandEncoder,
}

impl Target {
    pub fn aspect(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }
}

impl RenderSurface {
    pub fn update_configure(&mut self, device: &wgpu::Device) {
        self.raw.configure(device, &self.config);
    }
}

impl RenderContext {
    pub async fn new(window: &Window) -> Self {
        let size = &window.inner_size();

        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await.unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await.unwrap();


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

        let mut sur = RenderSurface {
            raw: surface,
            config,
        };
        sur.update_configure(&device);

        RenderContext {
            instance,
            adapter,
            device,
            queue,
            surface: Some(sur),

            models: HashMap::new(),

        }
    }

    pub fn new_frame_context(&mut self) -> FrameContext {
        let output = self.surface.as_mut().unwrap().raw.get_current_texture().unwrap();
        let desc = wgpu::TextureViewDescriptor::default();
        let view = output.texture.create_view(&desc);
        let encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Render Encoder"),
        });

        FrameContext { output: Target { view, size: output.texture.size(), format: output.texture.format() }, encoder }
    }


    /// Render this frame and present
    pub fn render_and_present(&mut self, world: &mut World, pass_queue: &mut PassQueue) {
        let mut frame_context = self.new_frame_context();

        pass_queue.draw(world, self, &mut frame_context);

        self.queue.submit(Some(frame_context.encoder.finish()));
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let surface = match self.surface {
            None => return,
            Some(ref mut suf) => suf
        };

        if (surface.config.width, surface.config.height) == (width, height) { return; };

        surface.config.width = width;
        surface.config.height = height;
        surface.update_configure(&self.device);
    }

    pub fn add_model(&mut self, model: Model) -> ModelRef {
        let model_ref = ModelRef(Uuid::new_v4());
        self.models.entry(model_ref.clone()).or_insert(model);
        model_ref
    }
    pub fn remove_model(&mut self, model: &ModelRef) {
        self.models.remove(model);
    }
    pub fn get_model(&mut self, model: &ModelRef) -> Option<&Model> {
        self.models.get(model)
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

