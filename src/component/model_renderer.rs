use std::collections::HashMap;
use crate::app::{App, Plugin, Runtime};
use crate::asset::model_loader::ModelLoader;
use crate::component::transform::GlobalTransform;
use crate::ecs::resource::Res;
use crate::ecs::Stage::PostUpdate;
use crate::graphics::GraphicsContext;
use crate::graphics::pass::Pass;
use crate::graphics::pass::phong::PhongPass;
use crate::node::Node;

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn run(&self, app: App) -> App {
        app.add_system(PostUpdate, update_model_manager)
            .add_system(PostUpdate, render)
    }
}

pub struct ModelRenderer{
    pub model_path: String
}
pub struct ModelManager{
    pub nodes: Vec<Node>
}
impl Res for ModelManager{}

fn update_model_manager(runtime: &mut Runtime){
    let context = runtime.res_manager.get_res_mut::<GraphicsContext>().unwrap();
    let model_loader = runtime.res_manager.get_res_mut::<ModelLoader>().unwrap();
    let model_manager = runtime.res_manager.get_res_mut::<ModelManager>().unwrap();

    let query = runtime.world.query_mut::<(&ModelRenderer, &GlobalTransform)>();
    model_manager.nodes.clear();
    let map = HashMap::<String, Node>::new();
    for (id, (model_renderer, g_trans)) in query{
        let model = model_loader.load(&model_renderer.model_path, &context.device, &context.queue);
    }
}

fn render(runtime: &mut Runtime){
    let context = runtime.res_manager.get_res_mut::<GraphicsContext>().unwrap();
    let phong = runtime.res_manager.get_res_mut::<PhongPass>().unwrap();
    // let model_loader = runtime.res_manager.get_res_mut::<ModelLoader>().unwrap();
    let model_manager = runtime.res_manager.get_res_mut::<ModelManager>().unwrap();

    let output = context.surface.get_current_texture().unwrap();
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Egui Render Encoder"),
    });

    model_manager.nodes.iter().for_each(|it| {
        phong.draw(&context.device, &context.queue, &mut encoder, &view, it).expect("Render Failed!");
    });

    output.present();
}




