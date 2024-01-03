use std::any::{Any, type_name, TypeId};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use anyhow::Error;
use downcast_rs::{Downcast, impl_downcast};
use terre_core_macros::Resource;

pub struct Res<'a, T> {
    value: &'a T,
}

impl<'a, T> Res<'a, T> {
    fn new(value: &'a T) -> Self {
        Self { value }
    }
}

pub struct ResMut<'a, T> {
    value: &'a mut T,
}

impl<'a, T> ResMut<'a, T> {
    fn new(content: &'a mut T) -> Self {
        Self { value: content }
    }
}

impl<'a, T> Deref for Res<'a, T> where T: Resource {
    type Target = &'a T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T> Deref for ResMut<'a, T> where T: Resource {
    type Target = &'a mut T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, T> DerefMut for ResMut<'a, T> where T: Resource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}


pub trait Resource: Downcast {}
impl_downcast!(Resource);

pub struct ResManager {
    resources: HashMap<TypeId, Box<dyn Resource>>,
}

impl ResManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new()
        }
    }
    pub fn push_res<T>(&mut self, it: T) -> anyhow::Result<()> where T: Resource {
        if !self.resources.contains_key(&it.type_id()) {
            self.resources.entry(it.type_id()).or_insert(Box::new(it));
            Ok(())
        } else {
            Err(Error::msg(format!("Resource 'type:[{}]' already exist!", type_name::<T>())))
        }
    }

    pub fn get_res_mut<T>(&mut self) -> Option<ResMut<T>> where T: Resource {
        let a = self.resources.get_mut(&TypeId::of::<T>())?;
        Some(ResMut::new(a.downcast_mut::<T>().unwrap()))
    }

    pub fn get_res<T>(&mut self) -> Option<Res<T>> where T: Resource {
        let a = self.resources.get(&TypeId::of::<T>())?;
        Some(Res::new(a.downcast_ref::<T>().unwrap()))
    }
}



#[cfg(test)]
mod test {
    use crate::ecs::resource::{ResManager, Resource};

    impl Resource for i32 {}

    #[test]
    fn test_res_manager_push_and_get() {
        let mut res = ResManager::new();
        res.push_res(42i32).unwrap();
        let twice = res.push_res(12i32);

        assert!(twice.is_err());
        assert_eq!(res.get_res::<i32>().unwrap().clone(), 42i32);
    }

    #[test]
    fn test_res_manager_get_mut() {
        let mut res = ResManager::new();

        res.push_res(12i32).unwrap();

        {
            let mut_i32 = res.get_res_mut::<i32>();
            **mut_i32.unwrap() = 42;
        }

        assert_eq!(res.get_res::<i32>().unwrap().clone(), 42);
    }
}
