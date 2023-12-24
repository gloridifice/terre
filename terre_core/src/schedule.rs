use std::collections::HashMap;
use hecs::World;
use crate::ecs::resource::ResManager;
use crate::ecs::system::{IntoSystem, System};

/// Lifecycle of the game.
/// # Start
/// invoke when game start;
/// # Updates
/// invoke per frame;
#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum Stage {
    Start,
    PreUpdate,
    Update,
    PostUpdate,
}

pub struct GameSchedule {
    pub systems: HashMap<Stage, Vec<Box<dyn System>>>,
}


impl GameSchedule {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new()
        }
    }

    pub fn add_system<Params>(&mut self, stage: Stage, function: impl IntoSystem<Params>) {
        let vec = self.systems.get_mut(&stage);
        let to_add = Box::new(function.into_system());
        match vec {
            None => { self.systems.insert(stage, vec![to_add]); }
            Some(it) => { it.push(to_add); }
        };
    }

    fn run_stages(&mut self, world: &mut World, stages: Vec<Stage>, res_manager: &mut ResManager){
        stages.iter().for_each(|stage| {
            if let Some(it) = self.systems.get_mut(stage) {
                it.iter_mut().for_each(|sys| sys.run(world, res_manager));
            }
        });
    }
    pub fn run_updates(&mut self, world: &mut World, res_manager: &mut ResManager){
        self.run_stages(world, vec![Stage::PreUpdate, Stage::Update, Stage::PostUpdate], res_manager);
    }

    pub fn run_starts(&mut self, world: &mut World, res_manager: &mut ResManager) {
        self.run_stages(world, vec![Stage::Start], res_manager);
    }
}

#[cfg(test)]
mod test {
    use hecs::{QueryMut, World};
    use crate::ecs::resource::ResManager;
    use crate::schedule::{GameSchedule, Stage};

    #[test]
    fn test_app_add_system() {
        fn test_system_function(query: QueryMut<(&mut i32, )>) {
            for (_id, (num,)) in query{
                *num = 4i32;
            }
        }
        let mut cycle = GameSchedule::new();
        let mut world = World::new();
        let entity = world.spawn((2i32,));
        cycle.add_system(Stage::Start, test_system_function);
        cycle.systems.get_mut(&Stage::Start).unwrap().iter_mut().for_each(|it| {
            it.run(&mut world, &mut ResManager::new())
        });

        let mut query = world.query_one::<&i32>(entity).unwrap();
        let a = query.get().unwrap();

        assert_eq!(a.clone(), 4i32);
    }
}

