use std::collections::HashMap;
use pollster::block_on;
use wgpu::{Device, Queue};
use crate::ecs::resource::Res;
use crate::model::Model;

pub struct ModelAssets {
    models: HashMap<String, Model>,
}
impl Res for ModelAssets {}

impl ModelAssets {
    pub fn load(&mut self, path: &str, device: &Device, queue: &Queue) -> Option<&Model> {
        match self.models.get(path) { 
            Some(it) => Some(it),
            None => {
                let res = block_on(crate::resources::load_model(path, device, queue));
                match res {
                    Ok(it) => { Some(self.models.entry(path.to_string()).or_insert(it)) }
                    Err(_) => { None }
                }
            }
        } 
    }
    pub fn get_model(&self, path: &String) -> Option<&Model> {
        self.models.get(path)
    }
}