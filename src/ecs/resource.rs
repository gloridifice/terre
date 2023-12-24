use std::any::{Any, TypeId};
use std::collections::HashMap;
use downcast_rs::{Downcast, impl_downcast};

pub trait Res: Downcast {}
impl_downcast!(Res);

pub struct ResManager {
    resources: HashMap<TypeId, Box<dyn Res>>,
}

impl ResManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new()
        }
    }
    pub fn push_res<T>(&mut self, it: T) where T: Res {
        self.resources.entry(it.type_id()).or_insert(Box::new(it));
    }

    pub fn get_res_mut<T>(&mut self) -> Option<&mut T> where T: Res {
        let a = self.resources.get_mut(&TypeId::of::<T>())?;
        a.downcast_mut::<T>()
    }

    pub fn get_res<T>(&mut self) -> Option<&T> where T: Res {
        let a = self.resources.get(&TypeId::of::<T>())?;
        a.downcast_ref::<T>()
    }
}


