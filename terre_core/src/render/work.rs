use crate::app::{App, Plugin};
use crate::render::ModelRef;
use crate::schedule::Stage;


pub struct Renderer3D{
    pub model: ModelRef,
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin{
    fn build(&self, app: App) -> App {
        app.add_system(Stage::Start, startup_render)
    }
}

fn startup_render(){
    
}

fn update_render(){
    
}