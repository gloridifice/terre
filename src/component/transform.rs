use cgmath::{Matrix3, Matrix4, Quaternion, Vector3};
use crate::app::{App, Plugin, Runtime};
use crate::ecs::Stage::Update;


pub struct TransformPlugin;
impl Plugin for TransformPlugin{
    fn run(&self, app: App) -> App {
        app.add_system(Update, transform_update_system)
    }
}
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>
}

pub struct GlobalTransform(
    pub Matrix4<f32>,
    pub Matrix3<f32>
);

impl GlobalTransform{
    pub fn new(transform: &Transform) -> GlobalTransform{
        let trans = Matrix4::from_translation(transform.position);
        let rot = Matrix4::from(transform.rotation);
        //scale todo
        GlobalTransform(trans * rot, Matrix3::from(transform.rotation))
    }
    
    pub fn update(&mut self, transform: &Transform){
        let trans = Matrix4::from_translation(transform.position);
        let rot = Matrix4::from(transform.rotation);
        
        self.0 = trans * rot;
    }
}

pub fn transform_update_system(runtime: &mut Runtime){
    let query = runtime.world.query_mut::<(&Transform, &mut GlobalTransform)>();
    for (id, (trans, global_trans)) in query{
        global_trans.update(trans);
    }
}