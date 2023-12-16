use egui_wgpu::renderer::ScreenDescriptor;
use egui_wgpu::Renderer;
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, StoreOp, Surface, TextureFormat, TextureView};
use winit::event::WindowEvent;
use winit::window::Window;

pub struct EguiRenderer {
    pub context: egui::Context,
    state: State,
    renderer: Renderer,
}

impl EguiRenderer {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_sample: u32,
        window: &Window,
    ) -> Self {
        let context = egui::Context::default();
        let mut state = State::new(
            context.viewport_id(),
            window,
            Some(window.scale_factor() as f32),
            None,
        );
        let renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_sample,
        );
        EguiRenderer {
            context,
            state,
            renderer,
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.state.on_window_event(&self.context, event);
    }

    pub fn draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        window: &Window,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        screen_descriptor: &ScreenDescriptor,
        run_ui: impl FnOnce(&egui::Context),
    ) {
        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.context.run(raw_input, |ui| {
            run_ui(ui);
        });
        {
            self.state
                .handle_platform_output(&window, &self.context, full_output.platform_output);

            let tris = self.context.tessellate(
                full_output.shapes,
                egui_winit::pixels_per_point(&self.context, &window),
            );
            for (id, image_delta) in &full_output.textures_delta.set {
                self.renderer
                    .update_texture(&device, &queue, *id, &image_delta);
            }
            self.renderer
                .update_buffers(&device, &queue, encoder, &tris, screen_descriptor);


            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("Egui Main Render Pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.renderer
                .render(&mut render_pass, &tris, &screen_descriptor);

        }

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x);
        }
    }
}
