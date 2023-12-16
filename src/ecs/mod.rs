use std::any::{Any, TypeId};
use std::collections::{HashMap};
use std::rc::Rc;
use uuid::Uuid;
use crate::Runtime;

pub trait System{
    fn run(&self, runtime: &mut Runtime);
}

impl<F> System for F where F: Fn(&mut Runtime) -> (){
    fn run(&self, runtime: &mut Runtime) {
        self(runtime)
    }
}




#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Id(uuid::Uuid);
impl Id{
    fn new() -> Self{
        Id(Uuid::new_v4())
    }
}

#[derive(Clone)]
pub struct EntityInstance {
    pub id: Id,
    pub components: Vec<Component>,
    pub children: Vec<Id>,
}
impl EntityInstance{
    fn new() -> Self{
        Self{
            id: Id::new(),
            components: vec![],
            children: vec![]
        }
    }
}
pub struct ComponentLifeTime {
    pub id: Id,
    pub awake: Option<fn(command: &mut World, entity: EntityInstance)>,
    pub update: Option<fn(world: &mut World, entity: EntityInstance)>,
}

#[derive(Copy, Clone)]
pub struct Component {
    pub id: Id,
    pub entity: Id,
    pub com_type: TypeId
}

pub trait ComponentData {}
impl EntityInstance {
    fn get_component<T: 'static>(&self) -> Option<Component>{
        for it in self.components.iter(){
            if it.type_id() == TypeId::of::<T>() {return Some(it.clone())};
        };
        None
    }
    fn get_all_components<T: 'static>(&self) -> Option<Vec<Component>>{
        let mut ret: Vec<Component> = Vec::new();
        self.components.iter().for_each(|it| {
            if it.type_id() == TypeId::of::<T>() { ret.push(it.clone()) }
        });
        return if ret.is_empty() { None } else { Some(ret) }
    }
}

pub struct GameLifetime {
    pub update: Vec<Rc<fn(world: &mut World)>>,
}
impl GameLifetime {
    fn new() -> Self{
        GameLifetime {
            update: Vec::new()
        }
    }
}
pub struct ComponentsTable{
    typed: HashMap<TypeId, Vec<Box<dyn ComponentData>>>

}

pub struct World{
    entities: HashMap<Id, EntityInstance>,
    lifetime: GameLifetime
}
impl World{
    pub fn new() -> Self{
        World{
            entities: HashMap::new(),
            lifetime: GameLifetime::new()
        }
    }

    fn spawn(&mut self) -> Id{
        let entity = EntityInstance::new();
        entity.id.clone()
    }
    fn spawn_empty(&mut self) -> Id{
        let entity = EntityInstance::new();
        self.entities.insert(entity.id, entity).unwrap().id
    }

    fn entity(&self, id: Id) -> Option<&EntityInstance>{
        self.entities.get(&id)
    }

    fn add_component(&mut self, entity: Id, component: Id){

    }
}

pub fn run(){
    let mut world = World::new();
    let up :Vec<_> =world.lifetime.update.iter().map(|it| Rc::clone(it)).collect();
    up.iter().for_each(|it| it(&mut world))
}