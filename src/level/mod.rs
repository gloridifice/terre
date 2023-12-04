use std::collections::HashMap;
use crate::level::world::{Block, World};

pub mod world;


pub struct Terre{
    registry_manager: RegistryManager,
    world: World,
}

impl Terre{
    pub fn new() -> Self{
        Self{
            registry_manager : RegistryManager::new(),
            world: World::new(),
        }
    }
}

pub struct RegistryManager{
    blocks: HashMap<String, Block>,
}
impl RegistryManager{
    pub fn new() -> Self{
        Self{
            blocks: HashMap::new(),
        }
    }

    pub fn registry_block(&mut self, block: Block){
        self.blocks.insert(block.registry_name.to_string(), block);
    }
}



