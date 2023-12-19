use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait Res{}
pub struct Resources{
    resources: HashMap<TypeId, Box<dyn Res>>
}

impl Resources{
    pub fn new() -> Self{
        Self{
            resources: HashMap::new()
        }
    }
    pub fn push_resources<T>(&mut self, it: T) -> &mut Box<T> where T: Res{
    }
}


